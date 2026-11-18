//! The game's chain: an in-memory UTXO set with the true owner of every coin.
//! The State never reads the owners; it only sees the transactions.

use std::collections::{BTreeMap, HashMap};

use bitcoin::address::NetworkUnchecked;
use bitcoin::hashes::{sha256d, Hash};
use bitcoin::{Address, Amount, OutPoint, Txid};
use serde::{Deserialize, Serialize};
use stealth_model::gateway::{DecodedTransaction, DescriptorType, TxInputRef, TxOutput};
use thiserror::Error;

use crate::address::ScriptKind;

pub type PlayerId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "id")]
pub enum Owner {
    Player(PlayerId),
    State,
    Cooperative,
    Exchange,
}

/// Where a coin came from, as the wallet of its owner would label it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "from")]
pub enum Origin {
    Initial,
    ExchangeWithdrawal,
    Sale(PlayerId),
    Change,
    Bonus,
    Dust,
    Lightning,
    CoinJoin,
    Migration,
    Tax,
    Fee,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Coin {
    pub outpoint: OutPoint,
    pub owner: Owner,
    #[serde(with = "stealth_model::types::serde_addr")]
    pub address: Address<NetworkUnchecked>,
    pub amount: Amount,
    pub kind: ScriptKind,
    pub origin: Origin,
    pub frozen: bool,
    pub round: u8,
}

#[derive(Debug, Clone)]
pub struct OutputSpec {
    pub address: Address<NetworkUnchecked>,
    pub amount: Amount,
    pub owner: Owner,
    pub origin: Origin,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LedgerError {
    #[error("a transaction needs at least one input")]
    NoInputs,
    #[error("a transaction needs at least one output")]
    NoOutputs,
    #[error("{0} is not an unspent output")]
    UnknownInput(OutPoint),
    #[error("inputs pay {inputs} but outputs need {outputs}")]
    Insufficient { inputs: Amount, outputs: Amount },
    #[error("{0} is not an address kind the game knows")]
    UnknownAddressKind(String),
}

#[derive(Debug, Default)]
pub struct Ledger {
    txs: HashMap<Txid, DecodedTransaction>,
    order: Vec<Txid>,
    heights: HashMap<Txid, Option<u32>>,
    utxos: BTreeMap<OutPoint, Coin>,
    spent: HashMap<OutPoint, Coin>,
    height: u32,
    nonce: u64,
    round: u8,
}

pub fn descriptor_type(kind: ScriptKind) -> DescriptorType {
    match kind {
        ScriptKind::Legacy => DescriptorType::P2pkh,
        ScriptKind::Segwit => DescriptorType::P2wpkh,
        ScriptKind::Taproot => DescriptorType::P2tr,
    }
}

impl Ledger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn set_round(&mut self, round: u8) {
        self.round = round;
    }

    /// Closes a block: every pending transaction gets this height.
    pub fn mine(&mut self) -> u32 {
        self.height += 1;
        for h in self.heights.values_mut() {
            if h.is_none() {
                *h = Some(self.height);
            }
        }
        self.height
    }

    pub fn confirmations(&self, txid: &Txid) -> u32 {
        match self.heights.get(txid) {
            Some(Some(h)) => self.height - h + 1,
            _ => 0,
        }
    }

    pub fn block_height(&self, txid: &Txid) -> Option<u32> {
        self.heights.get(txid).copied().flatten()
    }

    pub fn tx(&self, txid: &Txid) -> Option<&DecodedTransaction> {
        self.txs.get(txid)
    }

    pub fn transactions(&self) -> impl Iterator<Item = &DecodedTransaction> {
        self.order.iter().filter_map(|t| self.txs.get(t))
    }

    pub fn coin(&self, outpoint: &OutPoint) -> Option<&Coin> {
        self.utxos.get(outpoint)
    }

    /// A coin, spent or not.
    pub fn any_coin(&self, outpoint: &OutPoint) -> Option<&Coin> {
        self.utxos
            .get(outpoint)
            .or_else(|| self.spent.get(outpoint))
    }

    pub fn coins_of(&self, owner: Owner) -> Vec<&Coin> {
        self.utxos.values().filter(|c| c.owner == owner).collect()
    }

    /// Every coin the owner ever held, spent ones included.
    pub fn all_coins_of(&self, owner: Owner) -> Vec<&Coin> {
        self.utxos
            .values()
            .chain(self.spent.values())
            .filter(|c| c.owner == owner)
            .collect()
    }

    pub fn balance(&self, owner: Owner) -> Amount {
        self.coins_of(owner).iter().map(|c| c.amount).sum()
    }

    /// What anyone can see: the unspent value sitting on an address.
    pub fn unspent_on(&self, address: &Address<NetworkUnchecked>) -> Amount {
        self.utxos
            .values()
            .filter(|c| &c.address == address)
            .map(|c| c.amount)
            .sum()
    }

    /// The true owner behind an address, spent coins included.
    pub fn owner_of(&self, address: &Address<NetworkUnchecked>) -> Option<Owner> {
        self.utxos
            .values()
            .chain(self.spent.values())
            .find(|c| &c.address == address)
            .map(|c| c.owner)
    }

    /// Every address that ever appeared in an output.
    pub fn addresses(&self) -> Vec<Address<NetworkUnchecked>> {
        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        for c in self.utxos.values().chain(self.spent.values()) {
            if seen.insert(c.address.clone()) {
                out.push(c.address.clone());
            }
        }
        out.sort_by_key(|a| a.assume_checked_ref().to_string());
        out
    }

    pub fn set_frozen(&mut self, outpoint: &OutPoint, frozen: bool) -> bool {
        match self.utxos.get_mut(outpoint) {
            Some(coin) => {
                coin.frozen = frozen;
                true
            }
            None => false,
        }
    }

    /// Funding out of nowhere, like a coinbase. Used for the initial balances
    /// and for whatever arrives from off-chain (the Lightning hop).
    pub fn mint(&mut self, outputs: Vec<OutputSpec>) -> Result<Txid, LedgerError> {
        if outputs.is_empty() {
            return Err(LedgerError::NoOutputs);
        }
        let vin = vec![TxInputRef {
            previous_txid: Txid::all_zeros(),
            previous_vout: u32::MAX,
            sequence: 0xffff_ffff,
            coinbase: true,
        }];
        self.record(vin, outputs)
    }

    pub fn spend(
        &mut self,
        inputs: &[OutPoint],
        outputs: Vec<OutputSpec>,
    ) -> Result<Txid, LedgerError> {
        if inputs.is_empty() {
            return Err(LedgerError::NoInputs);
        }
        if outputs.is_empty() {
            return Err(LedgerError::NoOutputs);
        }
        let mut paid = Amount::ZERO;
        for op in inputs {
            let coin = self.utxos.get(op).ok_or(LedgerError::UnknownInput(*op))?;
            paid += coin.amount;
        }
        let needed: Amount = outputs.iter().map(|o| o.amount).sum();
        if paid < needed {
            return Err(LedgerError::Insufficient {
                inputs: paid,
                outputs: needed,
            });
        }
        for o in &outputs {
            if ScriptKind::of(&o.address).is_none() {
                return Err(LedgerError::UnknownAddressKind(
                    o.address.assume_checked_ref().to_string(),
                ));
            }
        }
        let vin = inputs
            .iter()
            .map(|op| TxInputRef {
                previous_txid: op.txid,
                previous_vout: op.vout,
                sequence: 0xffff_fffd,
                coinbase: false,
            })
            .collect();
        let txid = self.record(vin, outputs)?;
        for op in inputs {
            if let Some(coin) = self.utxos.remove(op) {
                self.spent.insert(*op, coin);
            }
        }
        Ok(txid)
    }

    fn record(
        &mut self,
        vin: Vec<TxInputRef>,
        outputs: Vec<OutputSpec>,
    ) -> Result<Txid, LedgerError> {
        let mut vout = Vec::with_capacity(outputs.len());
        let mut kinds = Vec::with_capacity(outputs.len());
        for (n, o) in outputs.iter().enumerate() {
            let kind = ScriptKind::of(&o.address).ok_or_else(|| {
                LedgerError::UnknownAddressKind(o.address.assume_checked_ref().to_string())
            })?;
            kinds.push(kind);
            vout.push(TxOutput {
                n: n as u32,
                address: Some(o.address.clone()),
                value: o.amount,
                script_type: descriptor_type(kind),
            });
        }
        let txid = self.next_txid(&vin, &vout);
        let vsize = 10 + 68 * vin.len() as u32 + 31 * vout.len() as u32;
        self.txs.insert(
            txid,
            DecodedTransaction {
                txid,
                vin,
                vout,
                version: 2,
                locktime: 0,
                vsize,
                confirmations: 0,
            },
        );
        self.order.push(txid);
        self.heights.insert(txid, None);
        for (n, o) in outputs.into_iter().enumerate() {
            let outpoint = OutPoint {
                txid,
                vout: n as u32,
            };
            self.utxos.insert(
                outpoint,
                Coin {
                    outpoint,
                    owner: o.owner,
                    address: o.address,
                    amount: o.amount,
                    kind: kinds[n],
                    origin: o.origin,
                    frozen: false,
                    round: self.round,
                },
            );
        }
        Ok(txid)
    }

    fn next_txid(&mut self, vin: &[TxInputRef], vout: &[TxOutput]) -> Txid {
        let mut data = self.nonce.to_le_bytes().to_vec();
        self.nonce += 1;
        for i in vin {
            data.extend_from_slice(i.previous_txid.as_byte_array());
            data.extend_from_slice(&i.previous_vout.to_le_bytes());
        }
        for o in vout {
            data.extend_from_slice(&o.value.to_sat().to_le_bytes());
            if let Some(a) = &o.address {
                data.extend_from_slice(a.assume_checked_ref().script_pubkey().as_bytes());
            }
        }
        Txid::from_raw_hash(sha256d::Hash::hash(&data))
    }

    /// The addresses, values and kinds behind a transaction's inputs.
    pub fn resolved_inputs(&self, tx: &DecodedTransaction) -> Vec<&Coin> {
        tx.vin
            .iter()
            .filter(|i| !i.coinbase)
            .filter_map(|i| {
                self.any_coin(&OutPoint {
                    txid: i.previous_txid,
                    vout: i.previous_vout,
                })
            })
            .collect()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::address::AddressBook;

    pub(crate) fn book() -> AddressBook {
        AddressBook::new([1u8; 32])
    }

    pub(crate) fn out(
        book: &mut AddressBook,
        kind: ScriptKind,
        sats: u64,
        owner: Owner,
        origin: Origin,
    ) -> OutputSpec {
        OutputSpec {
            address: book.fresh(kind),
            amount: Amount::from_sat(sats),
            owner,
            origin,
        }
    }

    #[test]
    fn minting_creates_spendable_coins_with_their_owner() {
        let mut book = book();
        let mut ledger = Ledger::new();
        let txid = ledger
            .mint(vec![
                out(
                    &mut book,
                    ScriptKind::Segwit,
                    1_000_000,
                    Owner::Player(1),
                    Origin::Initial,
                ),
                out(
                    &mut book,
                    ScriptKind::Legacy,
                    1_000_000,
                    Owner::Player(1),
                    Origin::Initial,
                ),
            ])
            .expect("mint");
        assert_eq!(
            ledger.balance(Owner::Player(1)),
            Amount::from_sat(2_000_000)
        );
        assert_eq!(ledger.coins_of(Owner::Player(1)).len(), 2);
        assert_eq!(ledger.confirmations(&txid), 0);
        assert_eq!(ledger.mine(), 1);
        assert_eq!(ledger.confirmations(&txid), 1);
        ledger.mine();
        assert_eq!(ledger.confirmations(&txid), 2);
        let tx = ledger.tx(&txid).expect("tx");
        assert!(tx.vin[0].coinbase);
        assert_eq!(tx.vout[1].script_type, DescriptorType::P2pkh);
    }

    #[test]
    fn spending_moves_coins_and_keeps_the_fee_implicit() {
        let mut book = book();
        let mut ledger = Ledger::new();
        ledger
            .mint(vec![out(
                &mut book,
                ScriptKind::Segwit,
                100_000,
                Owner::Player(1),
                Origin::Initial,
            )])
            .expect("mint");
        let coin = ledger.coins_of(Owner::Player(1))[0].outpoint;
        let txid = ledger
            .spend(
                &[coin],
                vec![
                    out(
                        &mut book,
                        ScriptKind::Segwit,
                        60_000,
                        Owner::Player(2),
                        Origin::Sale(1),
                    ),
                    out(
                        &mut book,
                        ScriptKind::Segwit,
                        39_000,
                        Owner::Player(1),
                        Origin::Change,
                    ),
                ],
            )
            .expect("spend");
        assert_eq!(ledger.balance(Owner::Player(1)), Amount::from_sat(39_000));
        assert_eq!(ledger.balance(Owner::Player(2)), Amount::from_sat(60_000));
        assert!(ledger.coin(&coin).is_none());
        assert!(ledger.any_coin(&coin).is_some());
        let tx = ledger.tx(&txid).expect("tx");
        assert_eq!(tx.vin[0].previous_txid, coin.txid);
        assert_eq!(
            ledger.resolved_inputs(tx)[0].amount,
            Amount::from_sat(100_000)
        );
        assert_eq!(ledger.all_coins_of(Owner::Player(1)).len(), 2);
    }

    #[test]
    fn spending_rejects_bad_inputs() {
        let mut book = book();
        let mut ledger = Ledger::new();
        ledger
            .mint(vec![out(
                &mut book,
                ScriptKind::Segwit,
                10_000,
                Owner::Player(1),
                Origin::Initial,
            )])
            .expect("mint");
        let coin = ledger.coins_of(Owner::Player(1))[0].outpoint;
        let ghost = OutPoint {
            txid: Txid::all_zeros(),
            vout: 3,
        };
        let pay = |b: &mut AddressBook| {
            vec![out(
                b,
                ScriptKind::Segwit,
                10_001,
                Owner::Player(2),
                Origin::Sale(1),
            )]
        };
        assert_eq!(
            ledger.spend(&[], pay(&mut book)),
            Err(LedgerError::NoInputs)
        );
        assert_eq!(
            ledger.spend(&[ghost], pay(&mut book)),
            Err(LedgerError::UnknownInput(ghost))
        );
        assert_eq!(
            ledger.spend(&[coin], pay(&mut book)),
            Err(LedgerError::Insufficient {
                inputs: Amount::from_sat(10_000),
                outputs: Amount::from_sat(10_001)
            })
        );
        assert_eq!(ledger.spend(&[coin], vec![]), Err(LedgerError::NoOutputs));
        assert_eq!(ledger.balance(Owner::Player(1)), Amount::from_sat(10_000));
    }

    #[test]
    fn txids_are_unique_even_for_identical_shapes() {
        let mut book = book();
        let mut ledger = Ledger::new();
        let addr = book.fresh(ScriptKind::Segwit);
        let spec = || OutputSpec {
            address: addr.clone(),
            amount: Amount::from_sat(546),
            owner: Owner::State,
            origin: Origin::Dust,
        };
        let a = ledger.mint(vec![spec()]).expect("a");
        let b = ledger.mint(vec![spec()]).expect("b");
        assert_ne!(a, b);
        assert_eq!(ledger.unspent_on(&addr), Amount::from_sat(1_092));
        assert_eq!(ledger.transactions().count(), 2);
    }

    #[test]
    fn frozen_flag_is_stored_on_the_coin() {
        let mut book = book();
        let mut ledger = Ledger::new();
        ledger
            .mint(vec![out(
                &mut book,
                ScriptKind::Segwit,
                546,
                Owner::Player(1),
                Origin::Dust,
            )])
            .expect("mint");
        let op = ledger.coins_of(Owner::Player(1))[0].outpoint;
        assert!(ledger.set_frozen(&op, true));
        assert!(ledger.coin(&op).expect("coin").frozen);
        assert!(!ledger.set_frozen(
            &OutPoint {
                txid: Txid::all_zeros(),
                vout: 0
            },
            true
        ));
    }
}
