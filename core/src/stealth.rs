//! The bridge to stealth-engine: a player's side of the ledger expressed as the
//! wallet history the engine scans.

use std::collections::{HashMap, HashSet};

use bitcoin::{Amount, Txid};
use stealth_engine::config::DetectorThresholds;
use stealth_engine::gateway::{Utxo, WalletHistory, WalletTxCategory, WalletTxEntry};
use stealth_engine::types::{Report, Severity, VulnerabilityType};
use stealth_engine::TxGraph;

use crate::ledger::{descriptor_type, Ledger, Origin, Owner, PlayerId};

pub fn wallet_history(ledger: &Ledger, player: PlayerId) -> WalletHistory {
    let owner = Owner::Player(player);
    let coins = ledger.all_coins_of(owner);
    let ours: HashSet<_> = coins.iter().map(|c| c.address.clone()).collect();
    let internal: HashSet<_> = coins
        .iter()
        .filter(|c| c.origin == Origin::Change)
        .map(|c| c.address.clone())
        .collect();

    let mut wallet_txs = Vec::new();
    let mut transactions = HashMap::new();
    for tx in ledger.transactions() {
        let inputs = ledger.resolved_inputs(tx);
        let spends_ours = inputs.iter().any(|c| c.owner == owner);
        let mut receives_ours = false;
        for out in &tx.vout {
            let Some(addr) = &out.address else { continue };
            if ours.contains(addr) {
                receives_ours = true;
                wallet_txs.push(WalletTxEntry {
                    txid: tx.txid,
                    address: Some(addr.clone()),
                    category: WalletTxCategory::Receive,
                    amount: out.value,
                    confirmations: ledger.confirmations(&tx.txid),
                    blockheight: ledger.block_height(&tx.txid).unwrap_or(0),
                });
            }
        }
        if spends_ours {
            let sent: Amount = tx
                .vout
                .iter()
                .filter(|o| o.address.as_ref().is_some_and(|a| !ours.contains(a)))
                .map(|o| o.value)
                .sum();
            let to = tx
                .vout
                .iter()
                .find(|o| o.address.as_ref().is_some_and(|a| !ours.contains(a)))
                .and_then(|o| o.address.clone());
            wallet_txs.push(WalletTxEntry {
                txid: tx.txid,
                address: to,
                category: WalletTxCategory::Send,
                amount: sent,
                confirmations: ledger.confirmations(&tx.txid),
                blockheight: ledger.block_height(&tx.txid).unwrap_or(0),
            });
        }
        if spends_ours || receives_ours {
            let mut decoded = tx.clone();
            decoded.confirmations = ledger.confirmations(&tx.txid);
            transactions.insert(tx.txid, decoded);
            for input in &tx.vin {
                if input.coinbase {
                    continue;
                }
                if let Some(parent) = ledger.tx(&input.previous_txid) {
                    let mut decoded = parent.clone();
                    decoded.confirmations = ledger.confirmations(&parent.txid);
                    transactions.entry(parent.txid).or_insert(decoded);
                }
            }
        }
    }

    let utxos = ledger
        .coins_of(owner)
        .into_iter()
        .map(|c| Utxo {
            txid: c.outpoint.txid,
            vout: c.outpoint.vout,
            address: Some(c.address.clone()),
            amount: c.amount,
            confirmations: ledger.confirmations(&c.outpoint.txid),
            script_type: descriptor_type(c.kind),
        })
        .collect();

    WalletHistory {
        wallet_txs,
        utxos,
        transactions,
        internal_addresses: internal,
        derived_addresses: ours,
    }
}

pub fn analyze(ledger: &Ledger, player: PlayerId, exchange_txids: &HashSet<Txid>) -> Report {
    let graph = TxGraph::from_wallet_history(wallet_history(ledger, player));
    let known_exchange = (!exchange_txids.is_empty()).then_some(exchange_txids);
    graph.detect_all(&DetectorThresholds::default(), None, known_exchange)
}

/// The findings a later report has that an earlier one did not.
pub fn new_findings<'a>(
    before: Option<&Report>,
    after: &'a Report,
) -> Vec<&'a stealth_engine::types::Finding> {
    let seen: HashSet<(VulnerabilityType, &str)> = before
        .map(|r| {
            r.findings
                .iter()
                .map(|f| (f.vulnerability_type, f.description.as_str()))
                .collect()
        })
        .unwrap_or_default();
    after
        .findings
        .iter()
        .filter(|f| !seen.contains(&(f.vulnerability_type, f.description.as_str())))
        .collect()
}

pub fn worst(report: &Report) -> Option<Severity> {
    report.findings.iter().map(|f| f.severity).max()
}
