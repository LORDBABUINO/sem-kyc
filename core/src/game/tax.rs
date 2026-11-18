//! The ICMS: assessed on every operation the State can attribute, paid at the
//! counter by the player, with interest while it waits.

use std::collections::BTreeMap;

use bitcoin::hashes::Hash;
use bitcoin::{Amount, OutPoint, Txid};
use serde::Serialize;

use super::{Event, Game, GameError};
use crate::cluster::Identity;
use crate::ledger::{Origin, PlayerId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Auto {
    pub id: u32,
    pub player: PlayerId,
    pub round: u8,
    /// Value of the operations attributed to the player.
    pub base: Amount,
    pub amount: Amount,
    pub operations: Vec<Txid>,
    pub late_rounds: u8,
    pub paid: Option<Txid>,
}

impl Game {
    pub fn autos(&self) -> &[Auto] {
        &self.autos
    }

    pub fn autos_of(&self, player: PlayerId) -> Vec<&Auto> {
        self.autos.iter().filter(|a| a.player == player).collect()
    }

    pub fn debt(&self, player: PlayerId) -> Amount {
        self.autos
            .iter()
            .filter(|a| a.player == player && a.paid.is_none())
            .map(|a| a.amount)
            .sum()
    }

    /// Who the State would bill for each side of a payment.
    fn attributed(&self, txid: &Txid, vout: u32) -> (Option<PlayerId>, Option<PlayerId>) {
        let Some(tx) = self.ledger.tx(txid) else {
            return (None, None);
        };
        let payer = self
            .ledger
            .resolved_inputs(tx)
            .first()
            .map(|c| self.clusters.identity(&c.address))
            .and_then(known);
        let payee = tx
            .vout
            .iter()
            .find(|o| o.n == vout)
            .and_then(|o| o.address.as_ref())
            .map(|a| self.clusters.identity(a))
            .and_then(known);
        (payer, payee)
    }

    /// Interest on what is still open, then the round's new assessments.
    pub(super) fn assess(&mut self, round: u8) {
        let late = self.config.late_percent;
        for a in self
            .autos
            .iter_mut()
            .filter(|a| a.paid.is_none() && a.round < round)
        {
            a.late_rounds += 1;
            a.amount += Amount::from_sat(a.amount.to_sat() * late / 100);
        }
        let mut charges: BTreeMap<PlayerId, (Amount, Vec<Txid>)> = BTreeMap::new();
        let ops: Vec<_> = self
            .operations
            .iter()
            .filter(|o| o.round == round)
            .cloned()
            .collect();
        for op in ops {
            let (payer, payee) = self.attributed(&op.txid, op.vout);
            // One person on both ends (the State's own confusion) is one operation.
            let sides = if payer == payee {
                vec![payer]
            } else {
                vec![payer, payee]
            };
            for who in sides.into_iter().flatten() {
                let e = charges.entry(who).or_insert((Amount::ZERO, Vec::new()));
                e.0 += op.amount;
                if !e.1.contains(&op.txid) {
                    e.1.push(op.txid);
                }
            }
        }
        for (player, (base, operations)) in charges {
            let tax = base.to_sat() * self.config.icms_percent / 100;
            let penalty = tax * self.config.penalty_percent / 100;
            let amount = Amount::from_sat(tax + penalty);
            if amount == Amount::ZERO {
                continue;
            }
            let id = self.autos.len() as u32 + 1;
            self.autos.push(Auto {
                id,
                player,
                round,
                base,
                amount,
                operations,
                late_rounds: 0,
                paid: None,
            });
            self.events.push(Event::AutoIssued {
                player,
                auto_id: id,
                amount,
            });
        }
    }

    /// The player scans the counter's QR and pays. Every coin used tells the
    /// State who it belongs to.
    pub fn pay_auto(
        &mut self,
        player: PlayerId,
        auto_id: u32,
        chosen: Option<&[OutPoint]>,
    ) -> Result<Txid, GameError> {
        let auto = self
            .autos
            .iter()
            .find(|a| a.id == auto_id && a.player == player)
            .cloned()
            .ok_or(GameError::UnknownAuto)?;
        if auto.paid.is_some() {
            return Err(GameError::AutoPaid);
        }
        let inputs = self.select_coins(player, auto.amount, chosen)?;
        let txid = self.pay_state(player, &inputs, auto.amount, Origin::Tax)?;
        if let Some(a) = self.autos.iter_mut().find(|a| a.id == auto_id) {
            a.paid = Some(txid);
        }
        self.events.push(Event::AutoPaid {
            player,
            auto_id,
            txid,
        });
        Ok(txid)
    }
}

impl Game {
    /// The presenter cancels an assessment made by mistake.
    pub fn forgive_auto(&mut self, auto_id: u32) -> Result<(), GameError> {
        let auto = self
            .autos
            .iter_mut()
            .find(|a| a.id == auto_id)
            .ok_or(GameError::UnknownAuto)?;
        if auto.paid.is_some() {
            return Err(GameError::AutoPaid);
        }
        auto.amount = Amount::ZERO;
        auto.paid = Some(Txid::all_zeros());
        let player = auto.player;
        self.events.push(Event::AutoPaid {
            player,
            auto_id,
            txid: Txid::all_zeros(),
        });
        Ok(())
    }
}

fn known(identity: Identity) -> Option<PlayerId> {
    match identity {
        Identity::Known(p) => Some(p),
        _ => None,
    }
}
