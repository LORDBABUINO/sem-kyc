use bitcoin::Amount;
use semkyc_core::cluster::Identity;
use semkyc_core::economy::Sector;
use semkyc_core::game::ReceiveMode;
use semkyc_core::view::Direction;

use crate::common::*;

#[test]
fn the_history_reads_like_a_wallet_and_hides_the_verdicts() {
    let mut g = game(8);
    start_round(&mut g, NOW);
    let padaria = player_in(&g, Sector::Padaria);
    let (seller, item) = supplier_for(&g, padaria, 0);
    trade(&mut g, padaria, seller, &item, NOW);
    let history = g.history(padaria);
    assert_eq!(history[0].direction, Direction::Out);
    assert_eq!(history[0].memo, item);
    assert_eq!(
        history[0].counterparty.as_deref(),
        Some(g.player(seller).expect("s").name.as_str())
    );
    let fee = history
        .iter()
        .find(|h| h.memo == "Taxa de cadastro da vitrine")
        .expect("fee entry");
    assert_eq!(fee.counterparty.as_deref(), Some("DOE"));
    assert_eq!(fee.amount, Amount::from_sat(10_000));
    assert!(history.iter().filter(|h| h.memo == "Saldo inicial").count() >= 1);
    let seller_history = g.history(seller);
    assert_eq!(seller_history[0].direction, Direction::In);
    assert_eq!(seller_history[0].memo, item);
    assert_eq!(seller_history[0].counterparty.as_deref(), Some("P1"));
    let text = serde_json::to_string(&history).expect("json");
    assert!(!text.contains("severity"));
}

#[test]
fn paying_the_counter_shows_the_assessment_number_only() {
    let mut g = game(8);
    start_round(&mut g, NOW);
    let padaria = player_in(&g, Sector::Padaria);
    complete_mission(&mut g, padaria, NOW);
    g.close_round(NOW + 200).expect("close");
    g.advance().expect("newspaper 2");
    let auto_id = g.autos_of(padaria)[0].id;
    g.pay_auto(padaria, auto_id, None).expect("paid");
    let history = g.history(padaria);
    assert_eq!(
        history[0].memo,
        format!("ICMS, Auto de Infração DOE nº {auto_id}")
    );
    assert!(history
        .iter()
        .any(|h| h.memo == "Bônus da missão" && h.direction == Direction::In));
}

#[test]
fn the_snapshot_names_known_clusters_and_links_payments() {
    let mut g = game(8);
    start_round(&mut g, NOW);
    let padaria = player_in(&g, Sector::Padaria);
    let fazenda = player_in(&g, Sector::Fazenda);
    g.set_receive_mode(fazenda, ReceiveMode::Storefront)
        .expect("storefront");
    let txid = trade(&mut g, padaria, fazenda, "farinha e ovos", NOW);
    let snap = g.cluster_snapshot();
    let padaria_cluster = snap
        .clusters
        .iter()
        .find(|c| c.identity == Identity::Known(padaria))
        .expect("padaria cluster");
    assert_eq!(padaria_cluster.names, vec!["P1".to_owned()]);
    assert_eq!(
        padaria_cluster.color.as_deref(),
        Some(Sector::Padaria.color())
    );
    assert!(padaria_cluster.addresses.len() >= 2);
    assert!(padaria_cluster.balance > Amount::ZERO);
    let edge = snap.edges.iter().find(|e| e.txid == txid).expect("edge");
    assert_eq!(
        edge.from,
        padaria_cluster.root.clone().assume_checked().to_string()
    );
    // The State may hold several clusters for one name; the payment landed
    // in the one that carries the Fazenda's storefront.
    let target = snap
        .clusters
        .iter()
        .find(|c| c.root.clone().assume_checked().to_string() == edge.to)
        .expect("target cluster");
    assert_eq!(target.identity, Identity::Known(fazenda));
    let state = g.state_address().clone().assume_checked().to_string();
    assert!(snap.clusters.iter().all(|c| !c.addresses.contains(&state)));
    assert!(snap.edges.iter().all(|e| e.to != state && e.from != state));
}

#[test]
fn the_reveal_sums_the_round_and_describes_the_headline_player() {
    let mut g = game(8);
    start_round(&mut g, NOW);
    let padaria = player_in(&g, Sector::Padaria);
    complete_mission(&mut g, padaria, NOW);
    g.close_round(NOW + 200).expect("close");
    let reveal = g.reveal(1);
    assert_eq!(reveal.round, 1);
    assert_eq!(reveal.payments, 2);
    assert!(reveal.headline.is_some());
    assert!(reveal.headline_name.is_some());
    assert!(!reveal.autos.is_empty());
    assert_eq!(reveal.assessed, reveal.autos.iter().map(|a| a.amount).sum());
    assert_eq!(reveal.collected, Amount::ZERO);
    assert!(reveal.known.linked_addresses >= 1);
    assert!(reveal.known.real_operations >= reveal.known.attributed_operations);
}

#[test]
fn standings_subtract_open_debt() {
    let mut g = game(8);
    start_round(&mut g, NOW);
    let padaria = player_in(&g, Sector::Padaria);
    complete_mission(&mut g, padaria, NOW);
    g.close_round(NOW + 200).expect("close");
    let rows = g.standings();
    assert_eq!(rows.len(), 8);
    let mine = rows.iter().find(|r| r.0 == padaria).expect("row");
    assert_eq!(mine.2, balance(&g, padaria) - g.debt(padaria));
    assert!(rows.windows(2).all(|w| w[0].2 >= w[1].2));
}

#[test]
fn a_payjoin_receipt_reads_as_income_for_the_seller() {
    let mut g = game(8);
    skip_rounds(&mut g, 7, NOW);
    for a in g.autos().to_vec() {
        if a.paid.is_none() {
            g.pay_auto(a.player, a.id, None).expect("settle");
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
    g.respond_payjoin(fazenda, &invoice.code, Some(mine[0]))
        .expect("settled");
    let entry = &g.history(fazenda)[0];
    assert_eq!(entry.direction, Direction::In);
    assert_eq!(entry.amount, invoice.amount);
    assert_eq!(entry.memo, "farinha e ovos");
    assert_eq!(entry.counterparty.as_deref(), Some("P1"));
}
