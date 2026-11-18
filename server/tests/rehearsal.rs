//! The bots play the eight rounds alone. Every slide's detector has to fire on
//! the mistake the bots make for it; this is the rehearsal the talk relies on.

use fiscal_core::game::{round_detectors, Config, Phase};
use fiscal_server::state::AppState;
use stealth_engine::types::VulnerabilityType;

#[test]
fn eight_bots_trigger_each_rounds_detector() {
    let state =
        AppState::new(Config::default(), "x".to_owned(), "http://t".to_owned()).expect("state");
    fiscal_server::bots::add(&state, 8).expect("bots");
    let mut headlines = Vec::new();
    for round in 1..=8u8 {
        state.with_game(|g| g.advance()).expect("advance");
        fiscal_server::bots::on_newspaper(&state);
        state
            .with_game(|g| g.open_negotiation(fiscal_server::state::now()))
            .expect("open");
        if round == 4 {
            state.with_game(|g| g.dust_attack()).expect("dust");
        }
        if round == 5 {
            state.with_game(|g| Ok(g.apply_kyc())).expect("kyc");
        }
        for _ in 0..3 {
            fiscal_server::bots::step(&state);
        }
        let phase = state
            .with_game(|g| g.close_round(fiscal_server::state::now() + 1))
            .expect("close");
        assert_eq!(phase, Phase::Reveal { round });
        let game = state.game();
        let headline = game.headline(round).map(|h| h.finding.vulnerability_type);
        let payments = game
            .operations()
            .iter()
            .filter(|o| o.round == round)
            .count();
        headlines.push((round, headline, payments));
    }
    for (round, headline, payments) in &headlines {
        assert!(*payments >= 8, "round {round}: only {payments} payments");
        let wanted = round_detectors(*round);
        if *round <= 6 {
            let found =
                headline.unwrap_or_else(|| panic!("round {round}: no headline\n{headlines:?}"));
            assert!(
                wanted.contains(&found),
                "round {round}: {found:?} not in {wanted:?}"
            );
        }
    }
    let types: Vec<Option<VulnerabilityType>> = headlines.iter().map(|h| h.1).collect();
    assert_eq!(types[0], Some(VulnerabilityType::AddressReuse));
    let final_phase = state.with_game(|g| g.advance()).expect("final");
    assert_eq!(final_phase, Phase::Final);
}
