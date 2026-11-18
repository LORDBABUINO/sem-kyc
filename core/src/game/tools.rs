//! The State's moves (dust, KYC) and the players' tools (Lightning, CoinJoin,
//! Taproot migration).

use bitcoin::{Amount, OutPoint, Txid};
use serde::Serialize;

use super::{Event, Game, GameError, Unlock};
use crate::address::ScriptKind;
use crate::ledger::{Origin, OutputSpec, Owner, PlayerId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CoinJoinRoom {
    pub deadline: u64,
    pub entries: Vec<(PlayerId, OutPoint)>,
}

impl Game {
    /// Round 4: 546 sats to every address the chain has shown so far.
    pub fn dust_attack(&mut self) -> Result<Txid, GameError> {
        if self.dust_sent {
            return Err(GameError::WrongPhase);
        }
        let targets: Vec<_> = self
            .ledger
            .addresses()
            .into_iter()
            .filter(|a| matches!(self.ledger.owner_of(a), Some(Owner::Player(_))))
            .collect();
        let mut outputs = Vec::with_capacity(targets.len() + 1);
        let mut players = Vec::new();
        for a in targets {
            let Some(owner) = self.ledger.owner_of(&a) else {
                continue;
            };
            if let Owner::Player(p) = owner {
                if !players.contains(&p) {
                    players.push(p);
                }
            }
            outputs.push(OutputSpec {
                address: a,
                amount: self.config.dust,
                owner,
                origin: Origin::Dust,
            });
        }
        let needed: Amount = outputs.iter().map(|o| o.amount).sum();
        let coin = self
            .ledger
            .coins_of(Owner::State)
            .into_iter()
            .find(|c| c.amount >= needed)
            .ok_or(GameError::InsufficientFunds)?;
        let (op, total) = (coin.outpoint, coin.amount);
        outputs.push(OutputSpec {
            address: self.state_address.clone(),
            amount: total - needed,
            owner: Owner::State,
            origin: Origin::Change,
        });
        let txid = self.ledger.spend(&[op], outputs)?;
        self.dust_sent = true;
        self.observe(txid, players);
        Ok(txid)
    }

    /// Round 5: the exchange hands over its customer list.
    pub fn apply_kyc(&mut self) -> Vec<PlayerId> {
        let mut named = Vec::new();
        let coins: Vec<_> = self
            .players
            .keys()
            .flat_map(|id| {
                self.ledger
                    .all_coins_of(Owner::Player(*id))
                    .into_iter()
                    .filter(|c| c.origin == Origin::ExchangeWithdrawal)
                    .map(|c| (*id, c.address.clone()))
                    .collect::<Vec<_>>()
            })
            .collect();
        let mut events = Vec::new();
        for (id, address) in coins {
            events.extend(self.clusters.label(&address, id));
            if !named.contains(&id) {
                named.push(id);
            }
        }
        self.kyc_applied = true;
        if !events.is_empty() {
            self.events.push(Event::Kyc {
                players: named.clone(),
                clusters: events,
            });
        }
        named
    }

    pub fn kyc_applied(&self) -> bool {
        self.kyc_applied
    }

    pub fn dust_sent(&self) -> bool {
        self.dust_sent
    }

    /// An exchange coin goes out through a channel and comes back with no past.
    pub fn withdraw_via_lightning(
        &mut self,
        player: PlayerId,
        op: OutPoint,
    ) -> Result<(Txid, Txid), GameError> {
        if !self.player(player)?.has(Unlock::Lightning) {
            return Err(GameError::Locked);
        }
        let coin = self
            .ledger
            .coin(&op)
            .ok_or(GameError::NotYourCoin(op))?
            .clone();
        if coin.owner != Owner::Player(player) {
            return Err(GameError::NotYourCoin(op));
        }
        if coin.frozen {
            return Err(GameError::Frozen);
        }
        if coin.origin != Origin::ExchangeWithdrawal {
            return Err(GameError::WrongCoin);
        }
        let channel = self.book.fresh(ScriptKind::Segwit);
        let out = self.ledger.spend(
            &[op],
            vec![OutputSpec {
                address: channel,
                amount: coin.amount,
                owner: Owner::Exchange,
                origin: Origin::Lightning,
            }],
        )?;
        self.observe(out, vec![player]);
        let back = self.fresh_for(player)?;
        let inbound = self.ledger.mint(vec![OutputSpec {
            address: back,
            amount: coin.amount,
            owner: Owner::Player(player),
            origin: Origin::Lightning,
        }])?;
        self.observe(inbound, vec![player]);
        Ok((out, inbound))
    }

    pub fn coinjoin_room(&self) -> Option<&CoinJoinRoom> {
        self.room.as_ref()
    }

    /// Walks up to the table with one coin. The round closes when the window
    /// ends; with fewer than the minimum, nobody mixes.
    pub fn join_coinjoin(
        &mut self,
        player: PlayerId,
        op: OutPoint,
        now: u64,
    ) -> Result<usize, GameError> {
        self.negotiation_round()?;
        self.require_active(player)?;
        if !self.player(player)?.has(Unlock::CoinJoin) {
            return Err(GameError::Locked);
        }
        let coin = self.ledger.coin(&op).ok_or(GameError::NotYourCoin(op))?;
        if coin.owner != Owner::Player(player) {
            return Err(GameError::NotYourCoin(op));
        }
        if coin.frozen {
            return Err(GameError::Frozen);
        }
        if coin.amount < self.config.coinjoin_denomination {
            return Err(GameError::WrongCoin);
        }
        if self.room.as_ref().is_some_and(|r| now >= r.deadline) {
            self.settle_room(now);
        }
        let room = self.room.get_or_insert_with(|| CoinJoinRoom {
            deadline: now + self.config.coinjoin_window_secs,
            entries: Vec::new(),
        });
        if room.entries.iter().any(|(p, _)| *p == player) {
            return Err(GameError::RoomClosed);
        }
        room.entries.push((player, op));
        Ok(room.entries.len())
    }

    /// Called by the clock: closes the room when its window has passed.
    pub fn coinjoin_tick(&mut self, now: u64) {
        if self.room.as_ref().is_some_and(|r| now >= r.deadline) {
            self.settle_room(now);
        }
    }

    pub(super) fn settle_room(&mut self, _now: u64) {
        let Some(room) = self.room.take() else { return };
        let players: Vec<PlayerId> = room.entries.iter().map(|(p, _)| *p).collect();
        if room.entries.len() < self.config.coinjoin_min {
            self.events.push(Event::CoinJoin {
                txid: None,
                players,
            });
            return;
        }
        let denom = self.config.coinjoin_denomination;
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        for (player, op) in &room.entries {
            let Some(coin) = self.ledger.coin(op).cloned() else {
                continue;
            };
            inputs.push(*op);
            let kind = self
                .players
                .get(player)
                .map_or(ScriptKind::Segwit, |p| p.kind());
            outputs.push(OutputSpec {
                address: self.book.fresh(kind),
                amount: denom,
                owner: Owner::Player(*player),
                origin: Origin::CoinJoin,
            });
            if coin.amount > denom {
                outputs.push(OutputSpec {
                    address: self.book.fresh(kind),
                    amount: coin.amount - denom,
                    owner: Owner::Player(*player),
                    origin: Origin::Change,
                });
            }
        }
        // Outputs sorted by address so the order says nothing about who is who.
        outputs.sort_by_key(|o| o.address.assume_checked_ref().to_string());
        match self.ledger.spend(&inputs, outputs) {
            Ok(txid) => {
                self.observe(txid, players.clone());
                self.events.push(Event::CoinJoin {
                    txid: Some(txid),
                    players,
                });
            }
            Err(_) => self.events.push(Event::CoinJoin {
                txid: None,
                players,
            }),
        }
    }

    /// Moves the chosen coins to Taproot addresses, one transaction per coin
    /// or all together into one coin, as the player decides. Fresh addresses
    /// are bc1p from then on.
    pub fn migrate_taproot(
        &mut self,
        player: PlayerId,
        ops: &[OutPoint],
        together: bool,
    ) -> Result<Vec<Txid>, GameError> {
        if !self.player(player)?.has(Unlock::Taproot) {
            return Err(GameError::Locked);
        }
        let mut coins = Vec::new();
        for op in ops {
            let coin = self.ledger.coin(op).ok_or(GameError::NotYourCoin(*op))?;
            if coin.owner != Owner::Player(player) {
                return Err(GameError::NotYourCoin(*op));
            }
            if coin.frozen {
                return Err(GameError::Frozen);
            }
            coins.push(coin.clone());
        }
        let mut txids = Vec::new();
        if together && !coins.is_empty() {
            // All together is a consolidation: one output, every history joined.
            let total: Amount = coins.iter().map(|c| c.amount).sum();
            let outputs = vec![OutputSpec {
                address: self.book.fresh(ScriptKind::Taproot),
                amount: total,
                owner: Owner::Player(player),
                origin: Origin::Migration,
            }];
            let txid = self.ledger.spend(ops, outputs)?;
            self.observe(txid, vec![player]);
            txids.push(txid);
        } else {
            for c in &coins {
                let txid = self.ledger.spend(
                    &[c.outpoint],
                    vec![OutputSpec {
                        address: self.book.fresh(ScriptKind::Taproot),
                        amount: c.amount,
                        owner: Owner::Player(player),
                        origin: Origin::Migration,
                    }],
                )?;
                self.observe(txid, vec![player]);
                txids.push(txid);
            }
        }
        self.player_mut(player)?.taproot = true;
        Ok(txids)
    }
}
