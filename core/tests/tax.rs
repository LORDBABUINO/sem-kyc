use bitcoin::Amount;
use semkyc_core::cluster::Identity;
use semkyc_core::economy::Sector;
use semkyc_core::game::{GameError, ReceiveMode};
use semkyc_core::ledger::Owner;

use crate::common::*;

/// A sale paid into the storefront: the State knows the seller, so the seller
/// owes 18% plus the 100% penalty on the operation.
#[test]
fn a_known_payee_is_assessed_and_suspended_until_the_counter_is_paid() {
    let mut g = game(8);
    start_round(&mut g, NOW);
    let padaria = player_in(&g, Sector::Padaria);
    let fazenda = player_in(&g, Sector::Fazenda);
    g.set_receive_mode(fazenda, ReceiveMode::Storefront)
        .expect("storefront");
    trade(&mut g, padaria, fazenda, "farinha e ovos", NOW);
    let amount = g.operations()[0].amount;
    g.close_round(NOW + 200).expect("close");
    let autos = g.autos_of(fazenda);
    assert_eq!(autos.len(), 1);
    assert_eq!(autos[0].base, amount);
    assert_eq!(
        autos[0].amount,
        Amount::from_sat(amount.to_sat() * 18 / 100 * 2)
    );
    assert_eq!(autos[0].round, 1);
    assert!(g.is_suspended(fazenda));
    assert_eq!(g.debt(fazenda), autos[0].amount);
    let auto_id = autos[0].id;

    g.advance().expect("newspaper 2");
    g.open_negotiation(NOW + 300).expect("open");
    assert_eq!(
        g.create_invoice(fazenda, "farinha e ovos", NOW + 300)
            .map(|_| ()),
        Err(GameError::Suspended)
    );
    let state_before = g.ledger.balance(Owner::State);
    let txid = g.pay_auto(fazenda, auto_id, None).expect("paid");
    assert_eq!(
        g.ledger.balance(Owner::State) - state_before,
        g.autos_of(fazenda)[0].amount
    );
    assert_eq!(g.autos_of(fazenda)[0].paid, Some(txid));
    assert!(!g.is_suspended(fazenda));
    assert_eq!(g.debt(fazenda), Amount::ZERO);
    assert_eq!(g.pay_auto(fazenda, auto_id, None), Err(GameError::AutoPaid));
    assert_eq!(
        g.pay_auto(padaria, auto_id, None),
        Err(GameError::UnknownAuto)
    );
    g.create_invoice(fazenda, "farinha e ovos", NOW + 300)
        .expect("back in business");
}

/// The Padaria's coins sit on its registered storefront, so whatever it pays
/// with is already its name: the buyer side is assessed.
#[test]
fn a_known_payer_is_assessed_too_and_an_unknown_one_is_not() {
    let mut g = game(8);
    start_round(&mut g, NOW);
    let padaria = player_in(&g, Sector::Padaria);
    let fazenda = player_in(&g, Sector::Fazenda);
    // Fazenda receives on a fresh address (round 1 default): payee unknown.
    trade(&mut g, padaria, fazenda, "farinha e ovos", NOW);
    g.close_round(NOW + 200).expect("close");
    assert_eq!(g.autos_of(padaria).len(), 1, "the payer was known");
    let amount = g.operations()[0].amount;
    assert_eq!(
        g.autos_of(padaria)[0].amount,
        Amount::from_sat(amount.to_sat() * 36 / 100)
    );
    assert!(g.autos_of(fazenda).is_empty(), "a fresh address is nobody");
}

#[test]
fn a_cluster_with_two_names_is_not_assessed() {
    let mut g = game(8);
    skip_rounds(&mut g, 7, NOW);
    for a in g.autos().to_vec() {
        if a.paid.is_none() {
            g.pay_auto(a.player, a.id, None).expect("settle debts");
        }
    }
    start_round(&mut g, NOW);
    let padaria = player_in(&g, Sector::Padaria);
    let fazenda = player_in(&g, Sector::Fazenda);
    g.set_receive_mode(fazenda, ReceiveMode::Payjoin)
        .expect("payjoin");
    let invoice = g
        .create_invoice(fazenda, "farinha e ovos", NOW)
        .expect("invoice");
    g.pay_invoice(padaria, &invoice.code, None, true)
        .expect("proposed");
    let mine = coins(&g, fazenda);
    // Make sure the seller's coin is already named, so the merge conflicts.
    let known: Vec<_> = mine
        .iter()
        .filter(|op| g.identity(&g.ledger.coin(op).expect("c").address) == Identity::Known(fazenda))
        .copied()
        .collect();
    let contributed = *known.first().expect("a coin the State already named");
    let txid = g
        .respond_payjoin(fazenda, &invoice.code, Some(contributed))
        .expect("settled");
    let tx = g.ledger.tx(&txid).expect("tx").clone();
    let first_input = g.ledger.resolved_inputs(&tx)[0].address.clone();
    assert!(matches!(g.identity(&first_input), Identity::Conflict(_)));
    g.close_round(NOW + 200).expect("close");
    assert!(
        g.autos().iter().all(|a| a.round != 8),
        "no assessment on a conflicted operation"
    );
}

#[test]
fn an_unpaid_assessment_grows_every_round_and_blocks_the_mission() {
    let mut g = game(8);
    start_round(&mut g, NOW);
    let padaria = player_in(&g, Sector::Padaria);
    complete_mission(&mut g, padaria, NOW);
    g.close_round(NOW + 200).expect("close 1");
    let first = g.autos_of(padaria)[0].amount;
    start_round(&mut g, NOW);
    assert_eq!(
        g.create_invoice(padaria, "refeições do canteiro", NOW)
            .map(|_| ()),
        Err(GameError::Suspended)
    );
    let fazenda = player_in(&g, Sector::Fazenda);
    let invoice = g
        .create_invoice(fazenda, "lote grande de farinha", NOW)
        .expect("invoice");
    assert_eq!(
        g.pay_invoice(padaria, &invoice.code, None, false),
        Err(GameError::Suspended)
    );
    g.close_round(NOW + 200).expect("close 2");
    let auto = g.autos_of(padaria)[0];
    assert_eq!(auto.late_rounds, 1);
    assert_eq!(auto.amount, Amount::from_sat(first.to_sat() * 120 / 100));
}

#[test]
fn paying_the_counter_with_a_clean_coin_names_it() {
    let mut g = game(8);
    start_round(&mut g, NOW);
    let padaria = player_in(&g, Sector::Padaria);
    complete_mission(&mut g, padaria, NOW);
    g.close_round(NOW + 200).expect("close");
    g.advance().expect("newspaper 2");
    // The bonus arrived on a fresh address: unknown to the State until it is
    // used at the counter.
    let bonus = g
        .ledger
        .coins_of(Owner::Player(padaria))
        .into_iter()
        .find(|c| c.origin == semkyc_core::ledger::Origin::Bonus)
        .cloned()
        .expect("bonus coin");
    assert_eq!(g.identity(&bonus.address), Identity::Unknown);
    let auto_id = g.autos_of(padaria)[0].id;
    let big = *coins(&g, padaria).last().expect("a coin");
    g.pay_auto(padaria, auto_id, Some(&[bonus.outpoint, big]))
        .expect("paid with the bonus");
    assert_eq!(g.identity(&bonus.address), Identity::Known(padaria));
}

/// Paying with the exact coin leaves one output; the State must not follow
/// it, or the buyer would be billed as payer and payee.
#[test]
fn an_exact_payment_is_billed_once_and_the_seller_stays_unknown() {
    let mut g = game(8);
    skip_rounds(&mut g, 1, NOW);
    start_round(&mut g, NOW);
    let padaria = player_in(&g, Sector::Padaria);
    let energia = player_in(&g, Sector::Energia);
    let invoice = g
        .create_invoice(energia, "painel solar do forno", NOW)
        .expect("invoice");
    // The Padaria has no coin of that exact value: mint one on its storefront.
    let store = g.player(padaria).expect("p").storefront.clone();
    let txid = g
        .ledger
        .mint(vec![semkyc_core::ledger::OutputSpec {
            address: store,
            amount: invoice.amount,
            owner: Owner::Player(padaria),
            origin: semkyc_core::ledger::Origin::Initial,
        }])
        .expect("mint");
    let exact = bitcoin::OutPoint { txid, vout: 0 };
    g.pay_invoice(padaria, &invoice.code, Some(&[exact]), false)
        .expect("paid");
    let tx = g.ledger.transactions().last().expect("tx").clone();
    assert_eq!(tx.vout.len(), 1);
    assert_eq!(g.identity(&invoice.address), Identity::Unknown);
    g.close_round(NOW + 200).expect("close");
    let autos = g.autos_of(padaria);
    assert_eq!(autos.len(), 1);
    assert_eq!(autos[0].base, invoice.amount);
    assert_eq!(
        autos[0].amount,
        Amount::from_sat(invoice.amount.to_sat() * 36 / 100)
    );
    assert!(g.autos_of(energia).is_empty());
}

/// A buyer whose change goes to a Taproot address while paying with a Segwit
/// coin confuses the State into calling the seller's output the change. The
/// State then sees one person on both ends: one operation, billed once.
#[test]
fn the_same_player_on_both_ends_is_billed_once() {
    let mut g = game(8);
    skip_rounds(&mut g, 6, NOW);
    for a in g.autos().to_vec() {
        if a.paid.is_none() {
            g.pay_auto(a.player, a.id, None).expect("settle");
        }
    }
    start_round(&mut g, NOW);
    let padaria = player_in(&g, Sector::Padaria);
    let fazenda = player_in(&g, Sector::Fazenda);
    g.set_price_style(fazenda, semkyc_core::economy::PriceStyle::Odd)
        .expect("odd");
    let mine = coins(&g, padaria);
    g.migrate_taproot(padaria, &mine[..1], false)
        .expect("one coin");
    let segwit: Vec<_> = g
        .spendable(padaria)
        .iter()
        .filter(|c| c.kind == semkyc_core::address::ScriptKind::Segwit)
        .map(|c| c.outpoint)
        .collect();
    let invoice = g
        .create_invoice(fazenda, "farinha e ovos", NOW)
        .expect("invoice");
    g.pay_invoice(padaria, &invoice.code, Some(&segwit[..2]), false)
        .expect("paid");
    assert_eq!(g.identity(&invoice.address), Identity::Known(padaria));
    g.close_round(NOW + 200).expect("close");
    let autos = g.autos_of(padaria);
    assert_eq!(autos.len(), 1);
    assert_eq!(autos[0].base, invoice.amount);
    assert!(g.autos_of(fazenda).is_empty());
}

#[test]
fn the_presenter_can_forgive_an_assessment() {
    let mut g = game(8);
    start_round(&mut g, NOW);
    let padaria = player_in(&g, Sector::Padaria);
    complete_mission(&mut g, padaria, NOW);
    g.close_round(NOW + 200).expect("close");
    let auto_id = g.autos_of(padaria)[0].id;
    assert!(g.is_suspended(padaria));
    g.forgive_auto(auto_id).expect("forgiven");
    assert!(!g.is_suspended(padaria));
    assert_eq!(g.debt(padaria), Amount::ZERO);
    assert_eq!(g.forgive_auto(auto_id), Err(GameError::AutoPaid));
    assert_eq!(g.forgive_auto(999), Err(GameError::UnknownAuto));
}
