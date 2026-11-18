//! Read models for the screens: the player's history, the State's cluster map
//! and the reveal of a round.

use std::collections::{BTreeMap, BTreeSet};

use bitcoin::address::NetworkUnchecked;
use bitcoin::{Address, Amount, Txid};
use serde::Serialize;

use crate::cluster::Identity;
use crate::game::{Game, Headline};
use crate::ledger::{Origin, Owner, PlayerId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    In,
    Out,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HistoryEntry {
    pub txid: Txid,
    pub direction: Direction,
    pub amount: Amount,
    pub counterparty: Option<String>,
    pub memo: String,
    pub height: Option<u32>,
    pub round: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClusterInfo {
    #[serde(with = "stealth_model::types::serde_addr")]
    pub root: Address<NetworkUnchecked>,
    pub addresses: Vec<String>,
    pub identity: Identity,
    pub names: Vec<String>,
    pub color: Option<String>,
    pub balance: Amount,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Edge {
    pub txid: Txid,
    pub from: String,
    pub to: String,
    pub amount: Amount,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClusterSnapshot {
    pub clusters: Vec<ClusterInfo>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AutoView {
    pub id: u32,
    pub player: PlayerId,
    pub name: String,
    pub round: u8,
    pub amount: Amount,
    pub paid: bool,
    pub late_rounds: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct Reveal {
    pub round: u8,
    pub headline: Option<Headline>,
    pub headline_name: Option<String>,
    pub autos: Vec<AutoView>,
    pub assessed: Amount,
    pub collected: Amount,
    pub payments: usize,
    pub known: KnownView,
}

/// What the State knows about the headline's player.
#[derive(Debug, Clone, Default, Serialize)]
pub struct KnownView {
    pub attributed_operations: usize,
    pub real_operations: usize,
    pub linked_addresses: usize,
    pub counterparties: Vec<String>,
}

impl Game {
    fn name_of(&self, owner: Owner) -> Option<String> {
        match owner {
            Owner::Player(p) => self.player(p).ok().map(|p| p.name.clone()),
            Owner::State => Some("Estado".to_owned()),
            Owner::Cooperative => Some("Cooperativa".to_owned()),
            Owner::Exchange => Some("Exchange".to_owned()),
        }
    }

    /// The player's own history, as a wallet would show it: no verdicts.
    pub fn history(&self, player: PlayerId) -> Vec<HistoryEntry> {
        let owner = Owner::Player(player);
        let mut out = Vec::new();
        for tx in self.ledger.transactions() {
            let inputs = self.ledger.resolved_inputs(tx);
            let spent: Amount = inputs
                .iter()
                .filter(|c| c.owner == owner)
                .map(|c| c.amount)
                .sum();
            let received: Vec<_> = tx
                .vout
                .iter()
                .filter_map(|o| {
                    let op = bitcoin::OutPoint {
                        txid: tx.txid,
                        vout: o.n,
                    };
                    self.ledger.any_coin(&op).filter(|c| c.owner == owner)
                })
                .collect();
            let got: Amount = received.iter().map(|c| c.amount).sum();
            if spent == Amount::ZERO && got == Amount::ZERO {
                continue;
            }
            let round = received.first().map(|c| c.round).unwrap_or(0);
            let height = self.ledger.block_height(&tx.txid);
            let entry = if spent > Amount::ZERO {
                let others: Vec<_> = tx
                    .vout
                    .iter()
                    .filter_map(|o| {
                        let op = bitcoin::OutPoint {
                            txid: tx.txid,
                            vout: o.n,
                        };
                        self.ledger.any_coin(&op).filter(|c| c.owner != owner)
                    })
                    .collect();
                if others.is_empty() {
                    let memo = match received.first().map(|c| c.origin) {
                        Some(Origin::Migration) => "Migração pra Taproot",
                        Some(Origin::CoinJoin) => "CoinJoin",
                        _ => "Transferência interna",
                    };
                    HistoryEntry {
                        txid: tx.txid,
                        direction: Direction::Internal,
                        amount: spent,
                        counterparty: None,
                        memo: memo.to_owned(),
                        height,
                        round,
                    }
                } else {
                    let paid: Amount = others.iter().map(|c| c.amount).sum();
                    let first = others[0];
                    let memo = match first.origin {
                        Origin::Tax => self
                            .autos()
                            .iter()
                            .find(|a| a.paid == Some(tx.txid))
                            .map(|a| format!("ICMS, Auto de Infração nº {}", a.id))
                            .unwrap_or_else(|| "Taxa de cadastro da vitrine".to_owned()),
                        Origin::Lightning => "Saque pela Lightning".to_owned(),
                        Origin::CoinJoin | Origin::Change if others.len() > 1 => {
                            "CoinJoin".to_owned()
                        }
                        _ => self
                            .operations()
                            .iter()
                            .find(|o| o.txid == tx.txid)
                            .and_then(|o| self.invoice_item(o.txid))
                            .unwrap_or_default(),
                    };
                    let is_coinjoin = crate::cluster::is_coinjoin_shape(tx);
                    HistoryEntry {
                        txid: tx.txid,
                        direction: Direction::Out,
                        amount: if is_coinjoin { spent } else { paid },
                        counterparty: if is_coinjoin {
                            None
                        } else {
                            self.name_of(first.owner)
                        },
                        memo: if is_coinjoin {
                            "CoinJoin".to_owned()
                        } else {
                            memo
                        },
                        height,
                        round,
                    }
                }
            } else {
                let first = received[0];
                let (counterparty, memo) = match first.origin {
                    Origin::Initial => (None, "Saldo inicial".to_owned()),
                    Origin::ExchangeWithdrawal => {
                        (Some("Exchange".to_owned()), "Saque da exchange".to_owned())
                    }
                    Origin::Sale(buyer) => (
                        self.name_of(Owner::Player(buyer)),
                        self.invoice_item(tx.txid).unwrap_or_default(),
                    ),
                    Origin::Bonus => (Some("Cooperativa".to_owned()), "Bônus da missão".to_owned()),
                    Origin::Dust => (None, "Desconhecido".to_owned()),
                    Origin::Lightning => (None, "Recebido pela Lightning".to_owned()),
                    _ => (None, String::new()),
                };
                HistoryEntry {
                    txid: tx.txid,
                    direction: Direction::In,
                    amount: got,
                    counterparty,
                    memo,
                    height,
                    round,
                }
            };
            out.push(entry);
        }
        out.reverse();
        out
    }

    fn invoice_item(&self, txid: Txid) -> Option<String> {
        self.invoices_paid_by_txid(txid)
            .map(|i| i.item.name.clone())
    }

    /// The whole map the State has: one node per cluster, one edge per
    /// payment between clusters.
    pub fn cluster_snapshot(&self) -> ClusterSnapshot {
        let groups = self.clusters.clusters();
        let mut clusters: Vec<ClusterInfo> = groups
            .iter()
            .map(|(root, members)| {
                let identity = self.clusters.identity(root);
                let ids: Vec<PlayerId> = match &identity {
                    Identity::Unknown => Vec::new(),
                    Identity::Known(p) => vec![*p],
                    Identity::Conflict(set) => set.iter().copied().collect(),
                };
                let names = ids
                    .iter()
                    .filter_map(|p| self.player(*p).ok())
                    .map(|p| p.name.clone())
                    .collect();
                let color = match ids.as_slice() {
                    [one] => self.player(*one).ok().map(|p| p.sector.color().to_owned()),
                    _ => None,
                };
                let balance = members.iter().map(|a| self.ledger.unspent_on(a)).sum();
                let mut addresses: Vec<String> = members
                    .iter()
                    .map(|a| a.assume_checked_ref().to_string())
                    .collect();
                addresses.sort();
                ClusterInfo {
                    root: root.clone(),
                    addresses,
                    identity,
                    names,
                    color,
                    balance,
                }
            })
            .collect();
        clusters.sort_by(|a, b| {
            b.balance.cmp(&a.balance).then(
                a.root
                    .assume_checked_ref()
                    .to_string()
                    .cmp(&b.root.assume_checked_ref().to_string()),
            )
        });

        let mut edges = Vec::new();
        for tx in self.ledger.transactions() {
            let inputs = self.ledger.resolved_inputs(tx);
            let Some(first) = inputs.first() else {
                continue;
            };
            if self.clusters.is_excluded(&first.address) {
                continue;
            }
            let from = self
                .clusters
                .root(&first.address)
                .assume_checked_ref()
                .to_string();
            let mut seen: BTreeSet<String> = BTreeSet::new();
            for o in &tx.vout {
                let Some(a) = &o.address else { continue };
                if self.clusters.is_excluded(a) {
                    continue;
                }
                let to = self.clusters.root(a).assume_checked_ref().to_string();
                if to == from || !seen.insert(to.clone()) {
                    continue;
                }
                edges.push(Edge {
                    txid: tx.txid,
                    from: from.clone(),
                    to,
                    amount: o.value,
                });
            }
        }
        ClusterSnapshot { clusters, edges }
    }

    pub fn auto_views(&self) -> Vec<AutoView> {
        self.autos()
            .iter()
            .map(|a| AutoView {
                id: a.id,
                player: a.player,
                name: self
                    .player(a.player)
                    .map(|p| p.name.clone())
                    .unwrap_or_default(),
                round: a.round,
                amount: a.amount,
                paid: a.paid.is_some(),
                late_rounds: a.late_rounds,
            })
            .collect()
    }

    /// Everything the projector shows when a round closes.
    pub fn reveal(&self, round: u8) -> Reveal {
        let headline = self.headline(round).cloned();
        let headline_name = headline
            .as_ref()
            .and_then(|h| self.player(h.player).ok())
            .map(|p| p.name.clone());
        let autos: Vec<AutoView> = self
            .auto_views()
            .into_iter()
            .filter(|a| a.round == round)
            .collect();
        let assessed = autos.iter().map(|a| a.amount).sum();
        let collected = autos.iter().filter(|a| a.paid).map(|a| a.amount).sum();
        let payments = self
            .operations()
            .iter()
            .filter(|o| o.round == round)
            .count();
        let known = headline
            .as_ref()
            .map(|h| self.known_about(h.player, round))
            .unwrap_or_default();
        Reveal {
            round,
            headline,
            headline_name,
            autos,
            assessed,
            collected,
            payments,
            known,
        }
    }

    fn known_about(&self, player: PlayerId, round: u8) -> KnownView {
        let real: Vec<_> = self
            .operations()
            .iter()
            .filter(|o| o.round == round && (o.payer == player || o.payee == player))
            .collect();
        let mut counterparties: BTreeMap<PlayerId, ()> = BTreeMap::new();
        let mut attributed = 0;
        for op in &real {
            let Some(tx) = self.ledger.tx(&op.txid) else {
                continue;
            };
            let payer_known = self
                .ledger
                .resolved_inputs(tx)
                .first()
                .map(|c| self.clusters.identity(&c.address) == Identity::Known(op.payer))
                .unwrap_or(false);
            let payee_known = tx
                .vout
                .iter()
                .find(|o| o.n == op.vout)
                .and_then(|o| o.address.as_ref())
                .map(|a| self.clusters.identity(a) == Identity::Known(op.payee))
                .unwrap_or(false);
            let mine_known = if op.payer == player {
                payer_known
            } else {
                payee_known
            };
            if mine_known {
                attributed += 1;
                let other = if op.payer == player {
                    op.payee
                } else {
                    op.payer
                };
                let other_known = if op.payer == player {
                    payee_known
                } else {
                    payer_known
                };
                if other_known {
                    counterparties.insert(other, ());
                }
            }
        }
        let linked = self
            .clusters
            .clusters()
            .into_iter()
            .filter(|(root, _)| self.clusters.identity(root) == Identity::Known(player))
            .map(|(_, members)| members.len())
            .sum();
        KnownView {
            attributed_operations: attributed,
            real_operations: real.len(),
            linked_addresses: linked,
            counterparties: counterparties
                .keys()
                .filter_map(|p| self.player(*p).ok())
                .map(|p| p.name.clone())
                .collect(),
        }
    }

    /// Final standings: balance minus open debt, best first.
    pub fn standings(&self) -> Vec<(PlayerId, String, Amount)> {
        let mut rows: Vec<_> = self
            .players()
            .map(|p| {
                let balance = self.ledger.balance(Owner::Player(p.id));
                let debt = self.debt(p.id);
                (
                    p.id,
                    p.name.clone(),
                    balance.checked_sub(debt).unwrap_or(Amount::ZERO),
                )
            })
            .collect();
        rows.sort_by(|a, b| b.2.cmp(&a.2).then(a.0.cmp(&b.0)));
        rows
    }
}
