use fiscal_core::economy::Sector;
use fiscal_core::game::{GameError, Phase, Unlock};
use stealth_engine::types::VulnerabilityType;

use crate::common::*;

#[test]
fn the_phases_run_lobby_newspaper_negotiation_reveal_and_end_in_final() {
    let mut g = game(8);
    assert_eq!(g.open_negotiation(NOW), Err(GameError::WrongPhase));
    assert_eq!(g.close_round(NOW), Err(GameError::WrongPhase));
    assert_eq!(g.advance().expect("1"), Phase::Newspaper { round: 1 });
    assert_eq!(g.advance(), Err(GameError::WrongPhase));
    assert_eq!(
        g.open_negotiation(NOW).expect("open"),
        Phase::Negotiation {
            round: 1,
            deadline: NOW + 180
        }
    );
    assert_eq!(
        g.close_round(NOW + 200).expect("close"),
        Phase::Reveal { round: 1 }
    );
    for r in 2..=8 {
        assert_eq!(g.advance().expect("next"), Phase::Newspaper { round: r });
        g.open_negotiation(NOW).expect("open");
        g.close_round(NOW).expect("close");
    }
    assert_eq!(g.advance().expect("final"), Phase::Final);
    assert!(g.players().all(|p| p.has(Unlock::Report)));
    assert_eq!(g.advance(), Err(GameError::WrongPhase));
}

#[test]
fn each_round_unlocks_its_tool_and_hands_out_two_items() {
    let mut g = game(8);
    let expected = [
        (1, Unlock::FreshAddress),
        (2, Unlock::CoinControl),
        (3, Unlock::OddPrice),
        (4, Unlock::Freeze),
        (5, Unlock::Lightning),
        (6, Unlock::CoinJoin),
        (7, Unlock::Taproot),
        (8, Unlock::Payjoin),
    ];
    for (round, unlock) in expected {
        start_round(&mut g, NOW);
        for p in g.players() {
            assert!(
                p.has(unlock),
                "{} lacks {unlock:?} in round {round}",
                p.name
            );
            let m = &p.missions[&round];
            assert_eq!(m.items.len(), 2);
            assert!(m.items.iter().all(|i| i.item.supplier != p.sector));
        }
        g.close_round(NOW).expect("close");
    }
    assert!(g.players().all(|p| p.has(Unlock::SilentPayments)));
}

#[test]
fn round_one_headline_is_the_address_reuse_of_a_storefront_sector() {
    let mut g = game(8);
    start_round(&mut g, NOW);
    let padaria = player_in(&g, Sector::Padaria);
    complete_mission(&mut g, padaria, NOW);
    g.close_round(NOW + 200).expect("close");
    let headline = g.headline(1).expect("headline");
    assert_eq!(
        headline.finding.vulnerability_type,
        VulnerabilityType::AddressReuse
    );
    let sector = g.player(headline.player).expect("p").sector;
    assert!(sector.on_storefront(), "{sector:?}");
    let reports = g.reports(padaria).expect("reports");
    assert!(reports[&1]
        .findings
        .iter()
        .any(|f| f.vulnerability_type == VulnerabilityType::AddressReuse));
}

#[test]
fn round_two_headline_is_about_common_inputs() {
    let mut g = game(8);
    start_round(&mut g, NOW);
    g.close_round(NOW).expect("close 1");
    start_round(&mut g, NOW);
    // In round 2 the Fazenda sells to one sector, so its item costs 120_000:
    // the automatic wallet joins two of the Padaria's coins to pay it.
    let padaria = player_in(&g, Sector::Padaria);
    g.set_coin_selection(padaria, fiscal_core::game::CoinSelection::Automatic)
        .expect("auto");
    let fazenda = player_in(&g, Sector::Fazenda);
    let txid = trade(&mut g, padaria, fazenda, "lote grande de farinha", NOW);
    assert_eq!(g.ledger.tx(&txid).expect("tx").vin.len(), 2);
    g.close_round(NOW + 200).expect("close 2");
    let headline = g.headline(2).expect("headline");
    assert!(matches!(
        headline.finding.vulnerability_type,
        VulnerabilityType::Cioh | VulnerabilityType::UnnecessaryInput
    ));
}

#[test]
fn the_final_report_covers_the_whole_game() {
    let mut g = game(8);
    let padaria = player_in(&g, Sector::Padaria);
    start_round(&mut g, NOW);
    complete_mission(&mut g, padaria, NOW);
    g.close_round(NOW + 200).expect("close");
    let report = g.report(padaria);
    assert!(report.stats.transactions_analyzed >= 20);
    assert!(!report.findings.is_empty());
}
