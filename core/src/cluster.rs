//! What the State deduces: addresses grouped into entities by the common
//! heuristics, labelled only where an identity leaked.

use std::collections::{BTreeSet, HashMap, HashSet};

use bitcoin::address::NetworkUnchecked;
use bitcoin::{Address, Amount, Txid};
use serde::{Deserialize, Serialize};

use stealth_model::gateway::DecodedTransaction;

use crate::address::ScriptKind;
use crate::ledger::{Coin, PlayerId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cause {
    CommonInput,
    Change,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "event")]
pub enum ClusterEvent {
    Merged {
        #[serde(with = "stealth_model::types::serde_addr")]
        absorbed: Address<NetworkUnchecked>,
        #[serde(with = "stealth_model::types::serde_addr")]
        into: Address<NetworkUnchecked>,
        cause: Cause,
        txid: Txid,
    },
    Labeled {
        #[serde(with = "stealth_model::types::serde_addr")]
        root: Address<NetworkUnchecked>,
        player: PlayerId,
    },
    Conflict {
        #[serde(with = "stealth_model::types::serde_addr")]
        root: Address<NetworkUnchecked>,
        players: BTreeSet<PlayerId>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Identity {
    Unknown,
    Known(PlayerId),
    Conflict(BTreeSet<PlayerId>),
}

/// `{"identity": "known", "players": [3]}`: always a list, as api.md says.
impl Serialize for Identity {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let (tag, players): (&str, Vec<PlayerId>) = match self {
            Identity::Unknown => ("unknown", Vec::new()),
            Identity::Known(p) => ("known", vec![*p]),
            Identity::Conflict(set) => ("conflict", set.iter().copied().collect()),
        };
        let mut st = s.serialize_struct("Identity", 2)?;
        st.serialize_field("identity", tag)?;
        st.serialize_field("players", &players)?;
        st.end()
    }
}

#[derive(Debug, Default, Clone)]
pub struct ClusterView {
    parent: HashMap<Address<NetworkUnchecked>, Address<NetworkUnchecked>>,
    size: HashMap<Address<NetworkUnchecked>, usize>,
    labels: HashMap<Address<NetworkUnchecked>, BTreeSet<PlayerId>>,
    excluded: HashSet<Address<NetworkUnchecked>>,
}

/// Humans pick round prices; nobody picks round change.
pub fn is_round(amount: Amount) -> bool {
    amount.to_sat().is_multiple_of(1_000)
}

/// Three or more outputs of one value, from three or more inputs.
pub fn is_coinjoin_shape(tx: &DecodedTransaction) -> bool {
    if tx.vin.len() < 3 {
        return false;
    }
    let mut counts: HashMap<u64, usize> = HashMap::new();
    for o in &tx.vout {
        *counts.entry(o.value.to_sat()).or_default() += 1;
    }
    counts.values().any(|c| *c >= 3)
}

impl ClusterView {
    pub fn new() -> Self {
        Self::default()
    }

    /// An address the State owns itself: it never joins a cluster, and a
    /// payment to it tells the State which output is the payer's change.
    pub fn exclude(&mut self, a: &Address<NetworkUnchecked>) {
        self.excluded.insert(a.clone());
    }

    pub fn is_excluded(&self, a: &Address<NetworkUnchecked>) -> bool {
        self.excluded.contains(a)
    }

    fn make_set(&mut self, a: &Address<NetworkUnchecked>) {
        if !self.excluded.contains(a) && !self.parent.contains_key(a) {
            self.parent.insert(a.clone(), a.clone());
            self.size.insert(a.clone(), 1);
        }
    }

    pub fn root(&self, a: &Address<NetworkUnchecked>) -> Address<NetworkUnchecked> {
        let mut cur = a.clone();
        while let Some(p) = self.parent.get(&cur) {
            if p == &cur {
                return cur;
            }
            cur = p.clone();
        }
        cur
    }

    pub fn knows(&self, a: &Address<NetworkUnchecked>) -> bool {
        self.parent.contains_key(a)
    }

    fn union(
        &mut self,
        a: &Address<NetworkUnchecked>,
        b: &Address<NetworkUnchecked>,
        cause: Cause,
        txid: Txid,
        events: &mut Vec<ClusterEvent>,
    ) {
        if self.excluded.contains(a) || self.excluded.contains(b) {
            return;
        }
        self.make_set(a);
        self.make_set(b);
        let ra = self.root(a);
        let rb = self.root(b);
        if ra == rb {
            return;
        }
        let (big, small) = if self.size[&ra] >= self.size[&rb] {
            (ra, rb)
        } else {
            (rb, ra)
        };
        self.parent.insert(small.clone(), big.clone());
        let added = self.size.remove(&small).unwrap_or(1);
        *self.size.entry(big.clone()).or_insert(1) += added;
        let before = self.labels.get(&big).cloned().unwrap_or_default();
        if let Some(moved) = self.labels.remove(&small) {
            let merged = self.labels.entry(big.clone()).or_default();
            merged.extend(moved);
        }
        events.push(ClusterEvent::Merged {
            absorbed: small,
            into: big.clone(),
            cause,
            txid,
        });
        if let Some(after) = self.labels.get(&big) {
            if after.len() > 1 && *after != before {
                events.push(ClusterEvent::Conflict {
                    root: big,
                    players: after.clone(),
                });
            }
        }
    }

    /// An identity leaked for this address (storefront registration, KYC).
    pub fn label(&mut self, a: &Address<NetworkUnchecked>, player: PlayerId) -> Vec<ClusterEvent> {
        self.make_set(a);
        let root = self.root(a);
        let set = self.labels.entry(root.clone()).or_default();
        if !set.insert(player) {
            return Vec::new();
        }
        let mut events = vec![ClusterEvent::Labeled {
            root: root.clone(),
            player,
        }];
        if set.len() > 1 {
            events.push(ClusterEvent::Conflict {
                root,
                players: set.clone(),
            });
        }
        events
    }

    pub fn identity(&self, a: &Address<NetworkUnchecked>) -> Identity {
        match self.labels.get(&self.root(a)) {
            None => Identity::Unknown,
            Some(set) if set.is_empty() => Identity::Unknown,
            Some(set) if set.len() == 1 => {
                Identity::Known(*set.iter().next().unwrap_or(&PlayerId::MAX))
            }
            Some(set) => Identity::Conflict(set.clone()),
        }
    }

    /// Every address grouped by its root.
    pub fn clusters(&self) -> HashMap<Address<NetworkUnchecked>, Vec<Address<NetworkUnchecked>>> {
        let mut out: HashMap<_, Vec<_>> = HashMap::new();
        for a in self.parent.keys() {
            out.entry(self.root(a)).or_default().push(a.clone());
        }
        out
    }

    /// The output the State would call change, if any.
    pub fn change_output(tx: &DecodedTransaction, inputs: &[&Coin]) -> Option<usize> {
        if tx.vout.len() != 2 {
            return None;
        }
        let rounds: Vec<bool> = tx.vout.iter().map(|o| is_round(o.value)).collect();
        match (rounds[0], rounds[1]) {
            (true, false) => return Some(1),
            (false, true) => return Some(0),
            _ => {}
        }
        let kinds: Vec<Option<ScriptKind>> = tx
            .vout
            .iter()
            .map(|o| o.address.as_ref().and_then(ScriptKind::of))
            .collect();
        let input_kind = inputs.first().map(|c| c.kind)?;
        if inputs.iter().any(|c| c.kind != input_kind) {
            return None;
        }
        match (kinds[0] == Some(input_kind), kinds[1] == Some(input_kind)) {
            (true, false) => Some(0),
            (false, true) => Some(1),
            _ => None,
        }
    }

    /// Reads one transaction the way a chain-analysis desk would.
    pub fn observe(&mut self, tx: &DecodedTransaction, inputs: &[&Coin]) -> Vec<ClusterEvent> {
        let mut events = Vec::new();
        for o in &tx.vout {
            if let Some(a) = &o.address {
                self.make_set(a);
            }
        }
        if inputs.is_empty() {
            return events;
        }
        for c in inputs {
            self.make_set(&c.address);
        }
        let first = &inputs[0].address;
        if !is_coinjoin_shape(tx) {
            for c in &inputs[1..] {
                self.union(first, &c.address, Cause::CommonInput, tx.txid, &mut events);
            }
            let to_excluded = tx.vout.iter().position(|o| {
                o.address
                    .as_ref()
                    .is_some_and(|a| self.excluded.contains(a))
            });
            // One output proves nothing: an exact payment and a sweep look alike.
            if let (2, Some(idx)) = (tx.vout.len(), to_excluded) {
                if let Some(a) = &tx.vout[1 - idx].address {
                    self.union(first, a, Cause::Change, tx.txid, &mut events);
                }
            } else if let Some(idx) = Self::change_output(tx, inputs) {
                if let Some(a) = &tx.vout[idx].address {
                    self.union(first, a, Cause::Change, tx.txid, &mut events);
                }
            }
        }
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::AddressBook;
    use crate::ledger::tests::{book, out};
    use crate::ledger::{Ledger, Origin, Owner};

    fn fund(ledger: &mut Ledger, book: &mut AddressBook, kind: ScriptKind, sats: &[u64]) {
        let outs = sats
            .iter()
            .map(|s| out(book, kind, *s, Owner::Player(1), Origin::Initial))
            .collect();
        ledger.mint(outs).expect("mint");
    }

    fn observe_last(view: &mut ClusterView, ledger: &Ledger) -> Vec<ClusterEvent> {
        let tx = ledger.transactions().last().expect("tx");
        let inputs = ledger.resolved_inputs(tx);
        view.observe(tx, &inputs)
    }

    #[test]
    fn common_inputs_merge_and_carry_the_label() {
        let mut book = book();
        let mut ledger = Ledger::new();
        let mut view = ClusterView::new();
        fund(
            &mut ledger,
            &mut book,
            ScriptKind::Segwit,
            &[30_000, 30_000],
        );
        let coins: Vec<_> = ledger
            .coins_of(Owner::Player(1))
            .into_iter()
            .cloned()
            .collect();
        view.label(&coins[0].address, 1);
        assert_eq!(view.identity(&coins[1].address), Identity::Unknown);
        ledger
            .spend(
                &[coins[0].outpoint, coins[1].outpoint],
                vec![out(
                    &mut book,
                    ScriptKind::Segwit,
                    50_000,
                    Owner::Player(2),
                    Origin::Sale(1),
                )],
            )
            .expect("spend");
        let events = observe_last(&mut view, &ledger);
        assert!(events.iter().any(|e| matches!(
            e,
            ClusterEvent::Merged {
                cause: Cause::CommonInput,
                ..
            }
        )));
        assert_eq!(view.identity(&coins[1].address), Identity::Known(1));
        // A single output could be a payment: the State does not follow it.
        let paid = &ledger.transactions().last().expect("tx").vout[0].address;
        assert_eq!(
            view.identity(paid.as_ref().expect("addr")),
            Identity::Unknown
        );
    }

    #[test]
    fn round_payment_exposes_the_change() {
        let mut book = book();
        let mut ledger = Ledger::new();
        let mut view = ClusterView::new();
        fund(&mut ledger, &mut book, ScriptKind::Segwit, &[100_000]);
        let coin = ledger.coins_of(Owner::Player(1))[0].clone();
        view.label(&coin.address, 1);
        let change = book.fresh(ScriptKind::Segwit);
        let payee = book.fresh(ScriptKind::Segwit);
        ledger
            .spend(
                &[coin.outpoint],
                vec![
                    OutputSpecBuilder::spec(&payee, 50_000, Owner::Player(2), Origin::Sale(1)),
                    OutputSpecBuilder::spec(
                        &change,
                        49_000 + 731,
                        Owner::Player(1),
                        Origin::Change,
                    ),
                ],
            )
            .expect("spend");
        let events = observe_last(&mut view, &ledger);
        assert!(events.iter().any(|e| matches!(
            e,
            ClusterEvent::Merged {
                cause: Cause::Change,
                ..
            }
        )));
        assert_eq!(view.identity(&change), Identity::Known(1));
        assert_eq!(view.identity(&payee), Identity::Unknown);
    }

    #[test]
    fn two_odd_amounts_of_the_same_kind_leave_the_change_ambiguous() {
        let mut book = book();
        let mut ledger = Ledger::new();
        let mut view = ClusterView::new();
        fund(&mut ledger, &mut book, ScriptKind::Segwit, &[100_000]);
        let coin = ledger.coins_of(Owner::Player(1))[0].clone();
        view.label(&coin.address, 1);
        let change = book.fresh(ScriptKind::Segwit);
        let payee = book.fresh(ScriptKind::Segwit);
        ledger
            .spend(
                &[coin.outpoint],
                vec![
                    OutputSpecBuilder::spec(&payee, 49_731, Owner::Player(2), Origin::Sale(1)),
                    OutputSpecBuilder::spec(&change, 50_123, Owner::Player(1), Origin::Change),
                ],
            )
            .expect("spend");
        let events = observe_last(&mut view, &ledger);
        assert!(events.is_empty());
        assert_eq!(view.identity(&change), Identity::Unknown);
    }

    #[test]
    fn script_kind_betrays_the_change_when_amounts_do_not() {
        let mut book = book();
        let mut ledger = Ledger::new();
        let mut view = ClusterView::new();
        fund(&mut ledger, &mut book, ScriptKind::Legacy, &[100_000]);
        let coin = ledger.coins_of(Owner::Player(1))[0].clone();
        let change = book.fresh(ScriptKind::Legacy);
        let payee = book.fresh(ScriptKind::Taproot);
        ledger
            .spend(
                &[coin.outpoint],
                vec![
                    OutputSpecBuilder::spec(&payee, 49_731, Owner::Player(2), Origin::Sale(1)),
                    OutputSpecBuilder::spec(&change, 50_123, Owner::Player(1), Origin::Change),
                ],
            )
            .expect("spend");
        observe_last(&mut view, &ledger);
        assert_eq!(view.root(&change), view.root(&coin.address));
        assert_ne!(view.root(&payee), view.root(&coin.address));
    }

    #[test]
    fn coinjoin_shape_links_nothing() {
        let mut book = book();
        let mut ledger = Ledger::new();
        let mut view = ClusterView::new();
        for p in 1..=3 {
            ledger
                .mint(vec![out(
                    &mut book,
                    ScriptKind::Segwit,
                    120_000,
                    Owner::Player(p),
                    Origin::Initial,
                )])
                .expect("mint");
        }
        let coins: Vec<_> = (1..=3)
            .flat_map(|p| {
                ledger
                    .coins_of(Owner::Player(p))
                    .into_iter()
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .collect();
        for c in &coins {
            view.label(&c.address, c.owner_id());
        }
        let ops: Vec<_> = coins.iter().map(|c| c.outpoint).collect();
        let mut outs = Vec::new();
        for p in 1..=3 {
            outs.push(out(
                &mut book,
                ScriptKind::Segwit,
                100_000,
                Owner::Player(p),
                Origin::CoinJoin,
            ));
            outs.push(out(
                &mut book,
                ScriptKind::Segwit,
                19_000,
                Owner::Player(p),
                Origin::Change,
            ));
        }
        ledger.spend(&ops, outs).expect("coinjoin");
        let events = observe_last(&mut view, &ledger);
        assert!(events.is_empty());
        for c in &coins {
            assert_eq!(view.identity(&c.address), Identity::Known(c.owner_id()));
        }
    }

    #[test]
    fn payjoin_merges_two_identities_into_a_conflict() {
        let mut book = book();
        let mut ledger = Ledger::new();
        let mut view = ClusterView::new();
        for p in 1..=2 {
            ledger
                .mint(vec![out(
                    &mut book,
                    ScriptKind::Taproot,
                    100_000,
                    Owner::Player(p),
                    Origin::Initial,
                )])
                .expect("mint");
        }
        let a = ledger.coins_of(Owner::Player(1))[0].clone();
        let b = ledger.coins_of(Owner::Player(2))[0].clone();
        view.label(&a.address, 1);
        view.label(&b.address, 2);
        ledger
            .spend(
                &[a.outpoint, b.outpoint],
                vec![
                    out(
                        &mut book,
                        ScriptKind::Taproot,
                        150_000,
                        Owner::Player(2),
                        Origin::Sale(1),
                    ),
                    out(
                        &mut book,
                        ScriptKind::Taproot,
                        49_500,
                        Owner::Player(1),
                        Origin::Change,
                    ),
                ],
            )
            .expect("payjoin");
        let events = observe_last(&mut view, &ledger);
        assert!(events
            .iter()
            .any(|e| matches!(e, ClusterEvent::Conflict { .. })));
        assert_eq!(
            view.identity(&a.address),
            Identity::Conflict(BTreeSet::from([1, 2]))
        );
    }

    #[test]
    fn a_payment_to_the_state_marks_the_other_output_as_change() {
        let mut book = book();
        let mut ledger = Ledger::new();
        let mut view = ClusterView::new();
        let state = book.fresh(ScriptKind::Segwit);
        view.exclude(&state);
        fund(&mut ledger, &mut book, ScriptKind::Segwit, &[100_000]);
        let coin = ledger.coins_of(Owner::Player(1))[0].clone();
        let change = book.fresh(ScriptKind::Segwit);
        ledger
            .spend(
                &[coin.outpoint],
                vec![
                    OutputSpecBuilder::spec(&state, 43_217, Owner::State, Origin::Tax),
                    OutputSpecBuilder::spec(&change, 56_783, Owner::Player(1), Origin::Change),
                ],
            )
            .expect("spend");
        observe_last(&mut view, &ledger);
        assert_eq!(view.root(&change), view.root(&coin.address));
        assert!(!view.knows(&state));
        assert!(view
            .clusters()
            .values()
            .all(|members| !members.contains(&state)));
    }

    #[test]
    fn identity_serializes_with_a_player_list() {
        let json = |i: &Identity| serde_json::to_string(i).expect("json");
        assert_eq!(
            json(&Identity::Unknown),
            r#"{"identity":"unknown","players":[]}"#
        );
        assert_eq!(
            json(&Identity::Known(3)),
            r#"{"identity":"known","players":[3]}"#
        );
        assert_eq!(
            json(&Identity::Conflict(BTreeSet::from([1, 2]))),
            r#"{"identity":"conflict","players":[1,2]}"#
        );
    }

    #[test]
    fn labelling_a_cluster_twice_with_different_players_is_a_conflict() {
        let mut book = book();
        let mut view = ClusterView::new();
        let a = book.fresh(ScriptKind::Segwit);
        assert_eq!(view.label(&a, 1).len(), 1);
        assert!(view.label(&a, 1).is_empty());
        let events = view.label(&a, 2);
        assert!(events
            .iter()
            .any(|e| matches!(e, ClusterEvent::Conflict { .. })));
        assert_eq!(view.clusters().len(), 1);
    }

    struct OutputSpecBuilder;
    impl OutputSpecBuilder {
        fn spec(
            address: &Address<NetworkUnchecked>,
            sats: u64,
            owner: Owner,
            origin: Origin,
        ) -> crate::ledger::OutputSpec {
            crate::ledger::OutputSpec {
                address: address.clone(),
                amount: Amount::from_sat(sats),
                owner,
                origin,
            }
        }
    }

    impl Coin {
        fn owner_id(&self) -> PlayerId {
            match self.owner {
                Owner::Player(p) => p,
                _ => panic!("not a player coin"),
            }
        }
    }
}
