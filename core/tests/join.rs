use bitcoin::Amount;
use fiscal_core::cluster::Identity;
use fiscal_core::economy::Sector;
use fiscal_core::game::{Config, Game, GameError, Phase};
use fiscal_core::ledger::Owner;

use crate::common::*;

#[test]
fn players_get_sectors_in_join_order_and_equal_balances() {
    let mut g = Game::new(Config::default()).expect("game");
    let ids: Vec<_> = (1..=9)
        .map(|i| g.join(&format!("Loja {i}")).expect("join"))
        .collect();
    let sectors: Vec<_> = ids
        .iter()
        .map(|id| g.player(*id).expect("p").sector)
        .collect();
    assert_eq!(&sectors[..8], &Sector::ALL);
    assert_eq!(sectors[8], Sector::Padaria);
    for id in ids {
        assert_eq!(
            g.ledger.balance(Owner::Player(id)),
            Amount::from_sat(2_000_000)
        );
    }
    assert_eq!(g.phase(), Phase::Lobby);
}

#[test]
fn names_must_be_unique_and_non_empty() {
    let mut g = Game::new(Config::default()).expect("game");
    g.join("Padaria da Lu").expect("join");
    assert_eq!(g.join("padaria da lu"), Err(GameError::NameTaken));
    assert_eq!(g.join("   "), Err(GameError::EmptyName));
}

#[test]
fn small_receipt_sectors_start_with_everything_on_the_storefront() {
    let g = game(8);
    let padaria = player_in(&g, Sector::Padaria);
    let store = g.player(padaria).expect("p").storefront.clone();
    // Nineteen coins of 100_000 still there; the twentieth paid the fee.
    assert_eq!(g.ledger.unspent_on(&store), Amount::from_sat(1_900_000));
    let fazenda = player_in(&g, Sector::Fazenda);
    let store = g.player(fazenda).expect("p").storefront.clone();
    assert_eq!(g.ledger.unspent_on(&store), Amount::ZERO);
}

#[test]
fn registration_pays_the_state_and_names_the_payer() {
    let mut g = Game::new(Config::default()).expect("game");
    let id = g.join("Fazenda do Caio").expect("join");
    let before = g.ledger.balance(Owner::State);
    g.register(id).expect("register");
    assert_eq!(
        g.ledger.balance(Owner::State) - before,
        Amount::from_sat(10_000)
    );
    assert_eq!(
        g.ledger.balance(Owner::Player(id)),
        Amount::from_sat(1_990_000)
    );
    let p = g.player(id).expect("p");
    assert!(p.registered);
    assert_eq!(g.identity(&p.storefront), Identity::Known(id));
    // The coin that paid the fee, and its change, carry the name too.
    for coin in g.ledger.coins_of(Owner::Player(id)) {
        if coin.origin == fiscal_core::ledger::Origin::Change {
            assert_eq!(g.identity(&coin.address), Identity::Known(id));
        }
    }
    assert_eq!(g.register(id), Err(GameError::AlreadyRegistered));
}

#[test]
fn the_exchange_funded_sector_is_unknown_until_kyc() {
    let mut g = game(8);
    let loja = player_in(&g, Sector::Loja);
    let coin = g
        .ledger
        .all_coins_of(Owner::Player(loja))
        .into_iter()
        .find(|c| c.origin == fiscal_core::ledger::Origin::ExchangeWithdrawal)
        .cloned()
        .expect("exchange coin");
    assert_eq!(g.exchange_txids().len(), 1);
    // Registration spent part of it, so its change is already named; the
    // withdrawal address itself is only named by the exchange.
    assert_eq!(g.identity(&coin.address), Identity::Known(loja));
    // Stealth sees the batch shape of the withdrawal from day one; the
    // exchange's list only adds certainty when the exchange talks.
    let is_exchange = |r: &stealth_engine::types::Report| {
        r.findings.iter().any(|f| {
            f.vulnerability_type == stealth_engine::types::VulnerabilityType::ExchangeOrigin
        })
    };
    assert!(is_exchange(&g.report(loja)));
    let named = g.apply_kyc();
    assert_eq!(named, vec![loja]);
    assert!(g.kyc_applied());
    assert!(is_exchange(&g.report(loja)));
}
