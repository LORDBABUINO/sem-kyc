use bitcoin::Amount;
use rastro_core::cluster::is_round;
use rastro_core::economy::{PriceStyle, Sector};
use rastro_core::game::{
    CoinSelection, GameError, InvoiceStatus, ItemStatus, Payment, ReceiveMode, Unlock,
};
use rastro_core::ledger::{Origin, Owner};

use crate::common::*;

#[test]
fn nothing_sells_outside_a_negotiation() {
    let mut g = game(8);
    let padaria = player_in(&g, Sector::Padaria);
    assert_eq!(
        g.create_invoice(padaria, "refeições do canteiro", NOW)
            .map(|_| ()),
        Err(GameError::WrongPhase)
    );
    g.advance().expect("newspaper");
    assert_eq!(
        g.create_invoice(padaria, "refeições do canteiro", NOW)
            .map(|_| ()),
        Err(GameError::WrongPhase)
    );
}

#[test]
fn an_invoice_prices_the_item_and_carries_the_sellers_address() {
    let mut g = game(8);
    start_round(&mut g, NOW);
    let fazenda = player_in(&g, Sector::Fazenda);
    let invoice = g
        .create_invoice(fazenda, "farinha e ovos", NOW)
        .expect("invoice");
    assert_eq!(invoice.seller, fazenda);
    assert_eq!(invoice.item.name, "farinha e ovos");
    assert_eq!(invoice.code.len(), 6);
    assert!(is_round(invoice.amount));
    // Round 1: Fazenda sells to 3 sectors, one seller each -> 120_000 / 3.
    assert_eq!(invoice.amount, Amount::from_sat(40_000));
    assert_eq!(
        invoice.display,
        invoice.address.clone().assume_checked().to_string()
    );
    assert!(!invoice.payjoin);
    assert_eq!(
        g.invoice(&invoice.code.to_lowercase())
            .expect("lookup")
            .status,
        InvoiceStatus::Open
    );
    assert_eq!(
        g.create_invoice(fazenda, "forno elétrico", NOW).map(|_| ()),
        Err(GameError::NotInCatalog("forno elétrico".into()))
    );
}

#[test]
fn fresh_addresses_are_the_default_once_unlocked_and_the_storefront_stays_optional() {
    let mut g = game(8);
    let fazenda = player_in(&g, Sector::Fazenda);
    assert_eq!(
        g.set_receive_mode(fazenda, ReceiveMode::Fresh),
        Err(GameError::Locked)
    );
    start_round(&mut g, NOW);
    assert!(g.player(fazenda).expect("p").has(Unlock::FreshAddress));
    assert_eq!(
        g.player(fazenda).expect("p").receive_mode,
        ReceiveMode::Fresh
    );
    let a = g.create_invoice(fazenda, "farinha e ovos", NOW).expect("a");
    let b = g.create_invoice(fazenda, "farinha e ovos", NOW).expect("b");
    assert_ne!(a.address, b.address);
    g.set_receive_mode(fazenda, ReceiveMode::Storefront)
        .expect("back to the storefront");
    let c = g.create_invoice(fazenda, "farinha e ovos", NOW).expect("c");
    assert_eq!(c.address, g.player(fazenda).expect("p").storefront);
}

#[test]
fn paying_moves_the_coins_marks_the_mission_and_pays_the_bonus() {
    let mut g = game(8);
    start_round(&mut g, NOW);
    let padaria = player_in(&g, Sector::Padaria);
    let before = balance(&g, padaria);
    let (seller, item) = supplier_for(&g, padaria, 0);
    let seller_before = balance(&g, seller);
    let txid = trade(&mut g, padaria, seller, &item, NOW);
    let invoice_amount = g.operations()[0].amount;
    assert_eq!(balance(&g, seller) - seller_before, invoice_amount);
    let mission = &g.player(padaria).expect("p").missions[&1];
    assert_eq!(mission.items[0].status, ItemStatus::Paid { txid });
    assert_eq!(mission.items[1].status, ItemStatus::Pending);
    assert!(mission.bonus.is_none());
    let (seller2, item2) = supplier_for(&g, padaria, 1);
    trade(&mut g, padaria, seller2, &item2, NOW);
    let mission = &g.player(padaria).expect("p").missions[&1];
    assert!(mission.complete());
    let bonus = mission.bonus.expect("bonus paid");
    let spent: Amount = g
        .operations()
        .iter()
        .filter(|o| o.payer == padaria)
        .map(|o| o.amount)
        .sum();
    let expected_bonus = Amount::from_sat(spent.to_sat() / 4);
    assert_eq!(balance(&g, padaria), before - spent + expected_bonus);
    assert!(g
        .ledger
        .coins_of(Owner::Player(padaria))
        .iter()
        .any(|c| c.origin == Origin::Bonus && c.outpoint.txid == bonus));
    let events = g.drain_events();
    assert!(events.iter().any(
        |e| matches!(e, rastro_core::game::Event::Bonus { player, .. } if *player == padaria)
    ));
}

#[test]
fn the_automatic_wallet_joins_small_coins_and_the_smallest_covering_one_does_not() {
    let mut g = game(8);
    start_round(&mut g, NOW);
    let padaria = player_in(&g, Sector::Padaria);
    let fazenda = player_in(&g, Sector::Fazenda);
    // 40_000 from twenty coins of 100_000: one coin covers, but "automatic"
    // takes the smallest first and here the smallest is already enough.
    let txid = trade(&mut g, padaria, fazenda, "farinha e ovos", NOW);
    assert_eq!(g.ledger.tx(&txid).expect("tx").vin.len(), 1);
    // The Construtora's round-1 price is 120_000 (one buyer sector); the
    // Padaria pays it with two coins when the wallet decides.
    let construtora = player_in(&g, Sector::Construtora);
    let txid = trade(&mut g, padaria, construtora, "balcão e reforma", NOW);
    let tx = g.ledger.tx(&txid).expect("tx");
    assert_eq!(tx.vin.len(), 2);
    assert_eq!(tx.vout.len(), 2, "payment and change");
}

#[test]
fn coin_control_needs_its_unlock_and_only_takes_the_payers_coins() {
    let mut g = game(8);
    start_round(&mut g, NOW);
    let padaria = player_in(&g, Sector::Padaria);
    let fazenda = player_in(&g, Sector::Fazenda);
    let mine = coins(&g, padaria);
    let invoice = g
        .create_invoice(fazenda, "farinha e ovos", NOW)
        .expect("invoice");
    assert_eq!(
        g.pay_invoice(padaria, &invoice.code, Some(&mine[..1]), false),
        Err(GameError::Locked)
    );
    assert_eq!(
        g.set_coin_selection(padaria, CoinSelection::Manual),
        Err(GameError::Locked)
    );
    g.close_round(NOW + 1).expect("close");
    start_round(&mut g, NOW);
    assert_eq!(
        g.player(padaria).expect("p").coin_selection,
        CoinSelection::SmallestCovering
    );
    let theirs = coins(&g, fazenda);
    let invoice = g
        .create_invoice(fazenda, "lote grande de farinha", NOW)
        .expect("invoice");
    assert_eq!(
        g.pay_invoice(padaria, &invoice.code, Some(&theirs[..1]), false),
        Err(GameError::NotYourCoin(theirs[0]))
    );
    let mine = coins(&g, padaria);
    let two = &mine[mine.len() - 2..];
    let paid = g
        .pay_invoice(padaria, &invoice.code, Some(two), false)
        .expect("paid");
    let Payment::Settled { txid } = paid else {
        panic!("settled")
    };
    assert_eq!(
        g.ledger.tx(&txid).expect("tx").vin[0].previous_txid,
        two[0].txid
    );
}

#[test]
fn a_paid_or_expired_invoice_cannot_be_paid_again() {
    let mut g = game(8);
    start_round(&mut g, NOW);
    let padaria = player_in(&g, Sector::Padaria);
    let fazenda = player_in(&g, Sector::Fazenda);
    let invoice = g
        .create_invoice(fazenda, "farinha e ovos", NOW)
        .expect("invoice");
    assert_eq!(
        g.pay_invoice(fazenda, &invoice.code, None, false),
        Err(GameError::SelfPayment)
    );
    g.pay_invoice(padaria, &invoice.code, None, false)
        .expect("first");
    assert_eq!(
        g.pay_invoice(padaria, &invoice.code, None, false),
        Err(GameError::InvoiceClosed)
    );
    let open = g
        .create_invoice(fazenda, "farinha e ovos", NOW)
        .expect("open");
    g.close_round(NOW + 200).expect("close");
    assert_eq!(
        g.invoice(&open.code).expect("i").status,
        InvoiceStatus::Expired
    );
    assert_eq!(
        g.pay_invoice(padaria, &open.code, None, false),
        Err(GameError::WrongPhase)
    );
    assert_eq!(
        g.pay_invoice(padaria, "NOPE", None, false),
        Err(GameError::WrongPhase)
    );
}

#[test]
fn odd_prices_arrive_with_round_three() {
    let mut g = game(8);
    let fazenda = player_in(&g, Sector::Fazenda);
    assert_eq!(
        g.set_price_style(fazenda, PriceStyle::Odd),
        Err(GameError::Locked)
    );
    skip_rounds(&mut g, 2, NOW);
    start_round(&mut g, NOW);
    assert_eq!(g.player(fazenda).expect("p").price_style, PriceStyle::Odd);
    let invoice = g
        .create_invoice(fazenda, "farinha e ovos", NOW)
        .expect("invoice");
    assert!(!is_round(invoice.amount));
    g.set_price_style(fazenda, PriceStyle::Round)
        .expect("back to round");
    let invoice = g
        .create_invoice(fazenda, "farinha e ovos", NOW)
        .expect("invoice");
    assert!(is_round(invoice.amount));
}

#[test]
fn payjoin_needs_both_sides_and_puts_both_coins_in_one_transaction() {
    let mut g = game(8);
    skip_rounds(&mut g, 7, NOW);
    start_round(&mut g, NOW);
    let padaria = player_in(&g, Sector::Padaria);
    let fazenda = player_in(&g, Sector::Fazenda);
    g.set_receive_mode(fazenda, ReceiveMode::Payjoin)
        .expect("payjoin mode");
    let invoice = g
        .create_invoice(fazenda, "farinha e ovos", NOW)
        .expect("invoice");
    assert!(invoice.payjoin);
    let result = g
        .pay_invoice(padaria, &invoice.code, None, true)
        .expect("proposed");
    assert_eq!(result, Payment::AwaitingSeller);
    assert!(matches!(
        g.invoice(&invoice.code).expect("i").status,
        InvoiceStatus::PayjoinPending { .. }
    ));
    let mine = coins(&g, fazenda);
    let txid = g
        .respond_payjoin(fazenda, &invoice.code, Some(mine[0]))
        .expect("settled");
    let tx = g.ledger.tx(&txid).expect("tx");
    assert!(tx.vin.len() >= 2);
    let paid_to_seller = tx.vout[0].value;
    let seller_coin = g.ledger.any_coin(&mine[0]).expect("coin").amount;
    assert_eq!(paid_to_seller, invoice.amount + seller_coin);
    assert_eq!(
        g.invoice(&invoice.code).expect("i").status,
        InvoiceStatus::Paid {
            txid,
            buyer: padaria
        }
    );
}

#[test]
fn a_payjoin_invoice_paid_plainly_is_a_plain_payment() {
    let mut g = game(8);
    skip_rounds(&mut g, 7, NOW);
    start_round(&mut g, NOW);
    let padaria = player_in(&g, Sector::Padaria);
    let fazenda = player_in(&g, Sector::Fazenda);
    g.set_receive_mode(fazenda, ReceiveMode::Payjoin)
        .expect("payjoin mode");
    let invoice = g
        .create_invoice(fazenda, "farinha e ovos", NOW)
        .expect("invoice");
    let Payment::Settled { txid } = g
        .pay_invoice(padaria, &invoice.code, None, false)
        .expect("paid")
    else {
        panic!("settled")
    };
    assert!(g.ledger.tx(&txid).is_some());
    let other = g
        .create_invoice(fazenda, "farinha e ovos", NOW)
        .expect("invoice");
    g.pay_invoice(padaria, &other.code, None, true)
        .expect("proposed");
    let txid = g
        .respond_payjoin(fazenda, &other.code, None)
        .expect("declined, settled plainly");
    let tx = g.ledger.tx(&txid).expect("tx");
    assert!(tx.vin.iter().all(|i| g
        .ledger
        .any_coin(&bitcoin::OutPoint {
            txid: i.previous_txid,
            vout: i.previous_vout
        })
        .expect("coin")
        .owner
        == Owner::Player(padaria)));
}

#[test]
fn a_payjoin_proposal_with_coins_spent_meanwhile_is_refused_and_reopened() {
    let mut g = game(8);
    skip_rounds(&mut g, 7, NOW);
    start_round(&mut g, NOW);
    let padaria = player_in(&g, Sector::Padaria);
    let fazenda = player_in(&g, Sector::Fazenda);
    let energia = player_in(&g, Sector::Energia);
    g.set_receive_mode(fazenda, ReceiveMode::Payjoin)
        .expect("payjoin mode");
    let invoice = g
        .create_invoice(fazenda, "farinha e ovos", NOW)
        .expect("invoice");
    let mine = coins(&g, padaria);
    let big = *mine.last().expect("coin");
    g.pay_invoice(padaria, &invoice.code, Some(&[big]), true)
        .expect("proposed");
    // The buyer spends that very coin elsewhere before the seller answers.
    let other = g
        .create_invoice(energia, "painel solar do forno", NOW)
        .expect("other");
    g.pay_invoice(padaria, &other.code, Some(&[big]), false)
        .expect("paid elsewhere");
    let theirs = coins(&g, fazenda);
    assert_eq!(
        g.respond_payjoin(fazenda, &invoice.code, Some(theirs[0])),
        Err(GameError::NotYourCoin(big))
    );
    assert_eq!(
        g.invoice(&invoice.code).expect("i").status,
        InvoiceStatus::Open
    );
}

#[test]
fn duplicate_inputs_never_pay_twice() {
    let mut g = game(8);
    skip_rounds(&mut g, 1, NOW);
    start_round(&mut g, NOW);
    let padaria = player_in(&g, Sector::Padaria);
    let fazenda = player_in(&g, Sector::Fazenda);
    let before = balance(&g, padaria);
    let invoice = g
        .create_invoice(fazenda, "lote grande de farinha", NOW)
        .expect("invoice");
    let small = coins(&g, padaria)[0];
    assert_eq!(
        g.pay_invoice(padaria, &invoice.code, Some(&[small, small]), false),
        Err(GameError::NotYourCoin(small))
    );
    assert_eq!(balance(&g, padaria), before);
}

#[test]
fn a_sector_nobody_buys_from_cannot_invoice() {
    let mut g = game(1);
    start_round(&mut g, NOW);
    let padaria = player_in(&g, Sector::Padaria);
    assert_eq!(
        g.create_invoice(padaria, "refeições do canteiro", NOW)
            .map(|_| ()),
        Err(GameError::NoDemand)
    );
}
