#![allow(dead_code)]

use bitcoin::{Amount, OutPoint, Txid};
use fiscal_core::economy::Sector;
use fiscal_core::game::{Config, Game, Phase};
use fiscal_core::ledger::PlayerId;

pub const NOW: u64 = 1_700_000_000;

/// A game with `n` registered players named P1..Pn, sectors in join order.
pub fn game(n: usize) -> Game {
    let mut g = Game::new(Config::default()).expect("game");
    for i in 1..=n {
        let id = g.join(&format!("P{i}")).expect("join");
        g.register(id).expect("register");
    }
    g.drain_events();
    g
}

/// Advances to the next newspaper and opens its negotiation.
pub fn start_round(g: &mut Game, now: u64) -> u8 {
    g.advance().expect("advance");
    let Phase::Negotiation { round, .. } = g.open_negotiation(now).expect("open") else {
        panic!("not negotiating")
    };
    round
}

/// Closes the round and advances to the next newspaper, `rounds` times.
pub fn skip_rounds(g: &mut Game, rounds: u8, now: u64) {
    for _ in 0..rounds {
        start_round(g, now);
        g.close_round(now + 1).expect("close");
    }
}

pub fn player_in(g: &Game, sector: Sector) -> PlayerId {
    g.players()
        .find(|p| p.sector == sector)
        .map(|p| p.id)
        .expect("a player in the sector")
}

pub fn players_in(g: &Game, sector: Sector) -> Vec<PlayerId> {
    g.players()
        .filter(|p| p.sector == sector)
        .map(|p| p.id)
        .collect()
}

/// A seller for the buyer's `idx`-th mission item of the current round.
pub fn supplier_for(g: &Game, buyer: PlayerId, idx: usize) -> (PlayerId, String) {
    let round = g.round();
    let p = g.player(buyer).expect("buyer");
    let item = &p.missions[&round].items[idx].item;
    let seller = g
        .players()
        .find(|q| q.sector == item.supplier && q.id != buyer)
        .map(|q| q.id)
        .expect("a seller");
    (seller, item.name.clone())
}

/// Seller invoices the item, buyer pays it with the wallet's own selection.
pub fn trade(g: &mut Game, buyer: PlayerId, seller: PlayerId, item: &str, now: u64) -> Txid {
    let invoice = g.create_invoice(seller, item, now).expect("invoice");
    match g
        .pay_invoice(buyer, &invoice.code, None, false)
        .expect("pay")
    {
        fiscal_core::game::Payment::Settled { txid } => txid,
        other => panic!("unexpected {other:?}"),
    }
}

/// Buys both mission items of the current round.
pub fn complete_mission(g: &mut Game, buyer: PlayerId, now: u64) -> Vec<Txid> {
    (0..2)
        .map(|idx| {
            let (seller, item) = supplier_for(g, buyer, idx);
            trade(g, buyer, seller, &item, now)
        })
        .collect()
}

pub fn coins(g: &Game, player: PlayerId) -> Vec<OutPoint> {
    g.spendable(player).iter().map(|c| c.outpoint).collect()
}

pub fn balance(g: &Game, player: PlayerId) -> Amount {
    g.ledger.balance(fiscal_core::ledger::Owner::Player(player))
}
