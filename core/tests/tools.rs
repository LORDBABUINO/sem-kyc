use bitcoin::Amount;
use dever_core::address::ScriptKind;
use dever_core::cluster::{is_coinjoin_shape, Identity};
use dever_core::economy::Sector;
use dever_core::game::{Event, GameError, ReceiveMode};
use dever_core::ledger::{Origin, Owner};

use crate::common::*;

#[test]
fn the_dust_attack_reaches_every_address_once_and_only_once() {
    let mut g = game(8);
    skip_rounds(&mut g, 3, NOW);
    start_round(&mut g, NOW);
    let addresses_before = g.ledger.addresses().len();
    let txid = g.dust_attack().expect("dust");
    let tx = g.ledger.tx(&txid).expect("tx");
    let dusted = tx
        .vout
        .iter()
        .filter(|o| o.value == Amount::from_sat(546))
        .count();
    assert!(dusted >= 8, "{dusted} outputs");
    assert!(dusted <= addresses_before);
    for id in g.players().map(|p| p.id).collect::<Vec<_>>() {
        assert!(g
            .ledger
            .coins_of(Owner::Player(id))
            .iter()
            .any(|c| c.origin == Origin::Dust));
    }
    assert_eq!(g.dust_attack().map(|_| ()), Err(GameError::WrongPhase));
    assert!(g.dust_sent());
}

#[test]
fn frozen_coins_are_skipped_by_every_selection() {
    let mut g = game(8);
    let padaria = player_in(&g, Sector::Padaria);
    let op = coins(&g, padaria)[0];
    assert_eq!(g.set_frozen(padaria, op, true), Err(GameError::Locked));
    skip_rounds(&mut g, 3, NOW);
    start_round(&mut g, NOW);
    g.dust_attack().expect("dust");
    let dust = g
        .ledger
        .coins_of(Owner::Player(padaria))
        .into_iter()
        .find(|c| c.origin == Origin::Dust)
        .cloned()
        .expect("dust coin");
    g.set_frozen(padaria, dust.outpoint, true).expect("freeze");
    assert!(!coins(&g, padaria).contains(&dust.outpoint));
    let fazenda = player_in(&g, Sector::Fazenda);
    let invoice = g
        .create_invoice(fazenda, "lote grande de farinha", NOW)
        .expect("invoice");
    assert_eq!(
        g.pay_invoice(padaria, &invoice.code, Some(&[dust.outpoint]), false),
        Err(GameError::Frozen)
    );
    let other = player_in(&g, Sector::Energia);
    assert_eq!(
        g.set_frozen(other, dust.outpoint, true),
        Err(GameError::NotYourCoin(dust.outpoint))
    );
}

#[test]
fn lightning_takes_only_exchange_coins_and_brings_back_a_coin_with_no_past() {
    let mut g = game(8);
    let loja = player_in(&g, Sector::Loja);
    let exchange_coin = g
        .ledger
        .coins_of(Owner::Player(loja))
        .into_iter()
        .find(|c| c.origin == Origin::ExchangeWithdrawal)
        .cloned()
        .expect("the withdrawal survives the registration fee");
    assert_eq!(
        g.withdraw_via_lightning(loja, exchange_coin.outpoint),
        Err(GameError::Locked)
    );
    skip_rounds(&mut g, 4, NOW);
    start_round(&mut g, NOW);
    g.apply_kyc();
    assert_eq!(g.identity(&exchange_coin.address), Identity::Known(loja));
    let padaria = player_in(&g, Sector::Padaria);
    let op = coins(&g, padaria)[0];
    assert_eq!(
        g.withdraw_via_lightning(padaria, op),
        Err(GameError::WrongCoin)
    );
    let fazenda = player_in(&g, Sector::Fazenda);
    let theirs = coins(&g, fazenda)[0];
    assert_eq!(
        g.withdraw_via_lightning(padaria, theirs),
        Err(GameError::NotYourCoin(theirs))
    );
    let before = balance(&g, loja);
    let (_, inbound) = g
        .withdraw_via_lightning(loja, exchange_coin.outpoint)
        .expect("lightning");
    assert_eq!(balance(&g, loja), before);
    let back = g
        .ledger
        .coins_of(Owner::Player(loja))
        .into_iter()
        .find(|c| c.outpoint.txid == inbound)
        .cloned()
        .expect("inbound coin");
    assert_eq!(g.identity(&back.address), Identity::Unknown);
}

#[test]
fn lightning_round_trip_on_a_fresh_exchange_coin() {
    let mut g = game(8);
    let loja = player_in(&g, Sector::Loja);
    skip_rounds(&mut g, 4, NOW);
    start_round(&mut g, NOW);
    // Simulate a withdrawal landing during the game.
    let addr = g.player(loja).expect("p").storefront.clone();
    let txid = g
        .ledger
        .mint(vec![dever_core::ledger::OutputSpec {
            address: addr.clone(),
            amount: Amount::from_sat(300_000),
            owner: Owner::Player(loja),
            origin: Origin::ExchangeWithdrawal,
        }])
        .expect("mint");
    let op = bitcoin::OutPoint { txid, vout: 0 };
    let before = balance(&g, loja);
    let (out, inbound) = g.withdraw_via_lightning(loja, op).expect("lightning");
    assert_eq!(balance(&g, loja), before);
    assert!(g.ledger.coin(&op).is_none());
    let back = g
        .ledger
        .coins_of(Owner::Player(loja))
        .into_iter()
        .find(|c| c.origin == Origin::Lightning)
        .cloned()
        .expect("inbound");
    assert_eq!(back.outpoint.txid, inbound);
    assert_eq!(g.identity(&back.address), Identity::Unknown);
    assert_eq!(g.identity(&addr), Identity::Known(loja));
    assert!(g.ledger.tx(&out).expect("out").vout.len() == 1);
}

#[test]
fn coinjoin_needs_five_and_then_mixes_into_equal_outputs() {
    let mut g = game(8);
    skip_rounds(&mut g, 5, NOW);
    for a in g.autos().to_vec() {
        if a.paid.is_none() {
            g.pay_auto(a.player, a.id, None).expect("settle");
        }
    }
    start_round(&mut g, NOW);
    let ids: Vec<_> = g.players().map(|p| p.id).collect();
    let padaria = player_in(&g, Sector::Padaria);
    let small = coins(&g, padaria)[0];
    assert_eq!(
        g.join_coinjoin(padaria, small, NOW).map(|_| ()),
        Err(GameError::WrongCoin)
    );
    // Four at the table: the window closes, nobody mixes.
    for id in &ids[1..5] {
        let op = *coins(&g, *id).last().expect("a big coin");
        g.join_coinjoin(*id, op, NOW).expect("join");
    }
    assert_eq!(g.coinjoin_room().expect("room").entries.len(), 4);
    g.drain_events();
    g.coinjoin_tick(NOW + 31);
    assert!(g.coinjoin_room().is_none());
    assert!(g
        .drain_events()
        .iter()
        .any(|e| matches!(e, Event::CoinJoin { txid: None, .. })));
    // Five at the table: one transaction, five outputs of the denomination.
    let mut joined = Vec::new();
    for id in &ids[1..6] {
        let op = *coins(&g, *id).last().expect("a big coin");
        g.join_coinjoin(*id, op, NOW + 40).expect("join");
        joined.push(*id);
    }
    assert_eq!(
        g.join_coinjoin(ids[1], coins(&g, ids[1])[0], NOW + 40),
        Err(GameError::RoomClosed)
    );
    g.coinjoin_tick(NOW + 71);
    let events = g.drain_events();
    let txid = events
        .iter()
        .find_map(|e| match e {
            Event::CoinJoin { txid: Some(t), .. } => Some(*t),
            _ => None,
        })
        .expect("coinjoin tx");
    let tx = g.ledger.tx(&txid).expect("tx");
    assert!(is_coinjoin_shape(tx));
    assert_eq!(
        tx.vout
            .iter()
            .filter(|o| o.value == Amount::from_sat(100_000))
            .count(),
        5
    );
    for id in joined {
        let mixed = g
            .ledger
            .coins_of(Owner::Player(id))
            .into_iter()
            .find(|c| c.origin == Origin::CoinJoin)
            .cloned()
            .expect("mixed coin");
        assert_eq!(g.identity(&mixed.address), Identity::Unknown);
    }
}

#[test]
fn taproot_migration_changes_the_kind_and_links_what_it_joins() {
    let mut g = game(8);
    let fazenda = player_in(&g, Sector::Fazenda);
    let mine = coins(&g, fazenda);
    assert_eq!(
        g.migrate_taproot(fazenda, &mine[..1], false),
        Err(GameError::Locked)
    );
    skip_rounds(&mut g, 6, NOW);
    start_round(&mut g, NOW);
    let mine = coins(&g, fazenda);
    assert_eq!(
        g.migrate_taproot(fazenda, &[mine[0], mine[0]], true),
        Err(GameError::NotYourCoin(mine[0]))
    );
    let txids = g
        .migrate_taproot(fazenda, &mine, false)
        .expect("one tx per coin");
    assert_eq!(txids.len(), mine.len());
    assert!(g.player(fazenda).expect("p").taproot);
    assert!(g
        .ledger
        .coins_of(Owner::Player(fazenda))
        .iter()
        .all(|c| c.kind == ScriptKind::Taproot));
    assert_eq!(
        g.set_receive_mode(fazenda, ReceiveMode::SilentPayments),
        Err(GameError::Locked)
    );
    let padaria = player_in(&g, Sector::Padaria);
    let theirs = coins(&g, padaria);
    let txids = g
        .migrate_taproot(padaria, &theirs[..3], true)
        .expect("one tx");
    assert_eq!(txids.len(), 1);
    let tx = g.ledger.tx(&txids[0]).expect("tx");
    assert_eq!(tx.vin.len(), 3);
    assert_eq!(tx.vout.len(), 1, "together means consolidated");
}

#[test]
fn silent_payments_need_taproot_and_hide_the_storefront() {
    let mut g = game(8);
    skip_rounds(&mut g, 7, NOW);
    for a in g.autos().to_vec() {
        if a.paid.is_none() {
            g.pay_auto(a.player, a.id, None).expect("settle");
        }
    }
    start_round(&mut g, NOW);
    let fazenda = player_in(&g, Sector::Fazenda);
    assert_eq!(
        g.set_receive_mode(fazenda, ReceiveMode::SilentPayments),
        Err(GameError::NeedsTaproot)
    );
    assert_eq!(
        g.register_silent_storefront(fazenda),
        Err(GameError::Locked)
    );
    let mine = coins(&g, fazenda);
    g.migrate_taproot(fazenda, &mine, false).expect("migrate");
    g.set_receive_mode(fazenda, ReceiveMode::SilentPayments)
        .expect("sp");
    let sp = g.register_silent_storefront(fazenda).expect("registered");
    assert!(sp.starts_with("sp1q"));
    assert!(g.player(fazenda).expect("p").storefront_is_silent);
    let a = g.create_invoice(fazenda, "farinha e ovos", NOW).expect("a");
    let b = g.create_invoice(fazenda, "farinha e ovos", NOW).expect("b");
    assert_eq!(a.display, sp);
    assert_eq!(b.display, sp);
    assert_ne!(a.address, b.address);
    assert_eq!(ScriptKind::of(&a.address), Some(ScriptKind::Taproot));
    let padaria = player_in(&g, Sector::Padaria);
    g.pay_invoice(padaria, &a.code, None, false).expect("paid");
    assert_eq!(g.identity(&a.address), Identity::Unknown);
}

#[test]
fn a_coin_spent_after_sitting_down_leaves_the_table() {
    let mut g = game(8);
    skip_rounds(&mut g, 5, NOW);
    for a in g.autos().to_vec() {
        if a.paid.is_none() {
            g.pay_auto(a.player, a.id, None).expect("settle");
        }
    }
    start_round(&mut g, NOW);
    let ids: Vec<_> = g.players().map(|p| p.id).collect();
    let mut seats = Vec::new();
    for id in &ids[..5] {
        let op = *coins(&g, *id).last().expect("a big coin");
        g.join_coinjoin(*id, op, NOW).expect("join");
        seats.push((*id, op));
    }
    // The first player pays an invoice with the coin at the table.
    let (buyer, op) = seats[0];
    let seller = ids.iter().copied().find(|s| *s != buyer).expect("seller");
    let item = g.trails.catalog(g.player(seller).expect("s").sector)[0]
        .name
        .clone();
    let invoice = g.create_invoice(seller, &item, NOW).expect("invoice");
    g.pay_invoice(buyer, &invoice.code, Some(&[op]), false)
        .expect("paid");
    g.drain_events();
    g.coinjoin_tick(NOW + 31);
    let events = g.drain_events();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::CoinJoin { txid: None, .. })),
        "four live coins do not mix"
    );
}

#[test]
fn migrating_nothing_does_not_count_as_migrating() {
    let mut g = game(8);
    skip_rounds(&mut g, 6, NOW);
    start_round(&mut g, NOW);
    let fazenda = player_in(&g, Sector::Fazenda);
    assert_eq!(
        g.migrate_taproot(fazenda, &[], true),
        Err(GameError::WrongCoin)
    );
    assert!(!g.player(fazenda).expect("p").taproot);
}
