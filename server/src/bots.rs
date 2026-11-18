//! Rehearsal players. They fill the empty sectors, trade among themselves and
//! make the mistake each round is about, so every slide's detector fires.

use fiscal_core::economy::{PriceStyle, Sector};
use fiscal_core::game::{CoinSelection, Game, ItemStatus, Phase, ReceiveMode};
use fiscal_core::ledger::{Origin, Owner, PlayerId};

use crate::error::ApiError;
use crate::state::{now, Shared};

/// Adds `count` bots, each registered. Sectors follow the join order like
/// everyone else's, so a full set of eight covers every sector.
pub fn add(state: &Shared, count: usize) -> Result<Vec<PlayerId>, ApiError> {
    let mut ids = Vec::new();
    for _ in 0..count {
        let id = state.with_game(|g| {
            let n = g.players().count() + 1;
            let sector = Sector::ALL[g.players().count() % Sector::ALL.len()];
            let id = g.join(&format!("{} robô {n}", sector.name()))?;
            g.register(id)?;
            Ok(id)
        })?;
        state
            .bots
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .push(id);
        ids.push(id);
    }
    Ok(ids)
}

fn bots_of(state: &Shared) -> Vec<PlayerId> {
    state.bots.lock().unwrap_or_else(|p| p.into_inner()).clone()
}

/// At every newspaper the bots settle their debts (with whatever coins the
/// wallet picks) and set themselves up for the round's lesson.
pub fn on_newspaper(state: &Shared) {
    let bots = bots_of(state);
    if bots.is_empty() {
        return;
    }
    let _ = state.with_game(|g| {
        let round = g.round();
        for id in &bots {
            for a in g
                .autos_of(*id)
                .into_iter()
                .map(|a| (a.id, a.paid.is_some()))
                .collect::<Vec<_>>()
            {
                if !a.1 {
                    let _ = g.pay_auto(*id, a.0, None);
                }
            }
            // Rounds 1 and 2: still on the storefront. Then fresh addresses.
            let mode = if round <= 2 {
                ReceiveMode::Storefront
            } else {
                ReceiveMode::Fresh
            };
            let _ = g.set_receive_mode(*id, mode);
            // Automatic selection and round prices until those lessons pass.
            if round <= 3 {
                let _ = g.set_coin_selection(*id, CoinSelection::Automatic);
                let _ = g.set_price_style(*id, PriceStyle::Round);
            }
            if round == 7 {
                let ops: Vec<_> = g.spendable(*id).iter().map(|c| c.outpoint).collect();
                let _ = g.migrate_taproot(*id, &ops, true);
            }
        }
        Ok(())
    });
}

fn bot_seller(g: &Game, bots: &[PlayerId], sector: Sector, buyer: PlayerId) -> Option<PlayerId> {
    bots.iter().copied().find(|id| {
        *id != buyer && g.player(*id).is_ok_and(|p| p.sector == sector) && !g.is_suspended(*id)
    })
}

/// One step of the bots' negotiation: each bot buys one pending item from a
/// bot in the right sector. Returns how many payments it made.
pub fn step(state: &Shared) -> usize {
    let bots = bots_of(state);
    if bots.is_empty() {
        return 0;
    }
    let mut made = 0;
    let _ = state.with_game(|g| {
        let Phase::Negotiation { round, deadline } = g.phase() else {
            return Ok(());
        };
        if now() > deadline {
            return Ok(());
        }
        for buyer in &bots {
            if g.is_suspended(*buyer) {
                continue;
            }
            let Some(item) = g.player(*buyer).ok().and_then(|p| {
                p.missions.get(&round).and_then(|m| {
                    m.items
                        .iter()
                        .find(|i| i.status == ItemStatus::Pending)
                        .map(|i| i.item.clone())
                })
            }) else {
                continue;
            };
            let Some(seller) = bot_seller(g, &bots, item.supplier, *buyer) else {
                continue;
            };
            let Ok(invoice) = g.create_invoice(seller, &item.name, now()) else {
                continue;
            };
            if g.pay_invoice(*buyer, &invoice.code, None, false).is_ok() {
                made += 1;
            }
        }
        // Round 5: a bot with an exchange coin never bothers with Lightning;
        // round 6: the bots with many coins consolidate in the open.
        if round == 6 {
            for id in &bots {
                let coins: Vec<_> = g
                    .ledger
                    .coins_of(Owner::Player(*id))
                    .into_iter()
                    .filter(|c| !c.frozen && c.origin != Origin::Dust)
                    .map(|c| c.outpoint)
                    .collect();
                if coins.len() >= 4 {
                    let _ = g.migrate_taproot(*id, &coins[..4], true);
                }
            }
        }
        Ok(())
    });
    made
}

/// Background loop: bots trade every two seconds while a negotiation is open.
pub async fn run(state: Shared) {
    let mut tick = tokio::time::interval(std::time::Duration::from_secs(2));
    loop {
        tick.tick().await;
        step(&state);
    }
}
