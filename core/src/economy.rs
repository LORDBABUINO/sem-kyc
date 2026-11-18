//! The eight sectors, what each one buys in each round, and how prices even out
//! the demand.

use std::collections::HashMap;

use bitcoin::Amount;
use serde::{Deserialize, Serialize};

use crate::address::ScriptKind;

pub const ROUNDS: u8 = 8;
pub const ITEMS_PER_ROUND: usize = 2;
const TRILHAS: &str = include_str!("../../economia/trilhas.json");
const MISSOES: &str = include_str!("../../economia/missoes.json");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sector {
    Padaria,
    Fazenda,
    Energia,
    Transportadora,
    Loja,
    Construtora,
    Consultoria,
    Grafica,
}

impl Sector {
    pub const ALL: [Sector; 8] = [
        Sector::Padaria,
        Sector::Fazenda,
        Sector::Energia,
        Sector::Transportadora,
        Sector::Loja,
        Sector::Construtora,
        Sector::Consultoria,
        Sector::Grafica,
    ];

    pub fn code(self) -> char {
        match self {
            Sector::Padaria => 'P',
            Sector::Fazenda => 'F',
            Sector::Energia => 'E',
            Sector::Transportadora => 'T',
            Sector::Loja => 'L',
            Sector::Construtora => 'C',
            Sector::Consultoria => 'K',
            Sector::Grafica => 'G',
        }
    }

    pub fn from_code(c: char) -> Option<Sector> {
        Sector::ALL.into_iter().find(|s| s.code() == c)
    }

    pub fn name(self) -> &'static str {
        match self {
            Sector::Padaria => "Padaria",
            Sector::Fazenda => "Fazenda",
            Sector::Energia => "Energia solar",
            Sector::Transportadora => "Transportadora",
            Sector::Loja => "Loja de eletrônicos",
            Sector::Construtora => "Construtora",
            Sector::Consultoria => "Consultoria",
            Sector::Grafica => "Gráfica",
        }
    }

    pub fn color(self) -> &'static str {
        match self {
            Sector::Padaria => "#e0a53a",
            Sector::Fazenda => "#5c9e4a",
            Sector::Energia => "#f2d13c",
            Sector::Transportadora => "#4a7fb5",
            Sector::Loja => "#8e5cc7",
            Sector::Construtora => "#c7613a",
            Sector::Consultoria => "#3aa6a6",
            Sector::Grafica => "#d14b8f",
        }
    }

    /// The starting coins of the sector: equal totals, different shapes.
    pub fn initial_coins(self) -> Vec<(ScriptKind, Amount)> {
        let sats = |n: u64| Amount::from_sat(n);
        match self {
            Sector::Padaria => vec![(ScriptKind::Segwit, sats(100_000)); 20],
            Sector::Fazenda => {
                vec![
                    (ScriptKind::Legacy, sats(1_000_000)),
                    (ScriptKind::Segwit, sats(1_000_000)),
                ]
            }
            Sector::Energia => vec![(ScriptKind::Segwit, sats(500_000)); 4],
            Sector::Transportadora => vec![(ScriptKind::Segwit, sats(250_000)); 8],
            Sector::Loja => vec![(ScriptKind::Segwit, sats(2_000_000))],
            Sector::Construtora => vec![
                (ScriptKind::Segwit, sats(666_667)),
                (ScriptKind::Segwit, sats(666_667)),
                (ScriptKind::Segwit, sats(666_666)),
            ],
            Sector::Consultoria => vec![(ScriptKind::Segwit, sats(2_000_000))],
            Sector::Grafica => {
                let mut coins = vec![(ScriptKind::Segwit, sats(125_000)); 14];
                coins.push((ScriptKind::Legacy, sats(125_000)));
                coins.push((ScriptKind::Legacy, sats(125_000)));
                coins
            }
        }
    }

    /// Sectors paid in many small receipts kept them all on the storefront.
    pub fn on_storefront(self) -> bool {
        matches!(
            self,
            Sector::Padaria | Sector::Grafica | Sector::Transportadora
        )
    }

    /// The Loja's coins come from an exchange withdrawal; everyone else's are
    /// unlabelled.
    pub fn funded_by_exchange(self) -> bool {
        self == Sector::Loja
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    pub supplier: Sector,
}

/// What every sector buys, round by round, loaded from `economia/trilhas.json`.
#[derive(Debug, Clone)]
pub struct Trails {
    paths: HashMap<Sector, Vec<Item>>,
    texts: HashMap<Sector, Vec<String>>,
}

impl Trails {
    pub fn load() -> Result<Self, String> {
        let raw: HashMap<String, Vec<(String, String)>> =
            serde_json::from_str(TRILHAS).map_err(|e| e.to_string())?;
        let mut paths = HashMap::new();
        for (code, items) in raw {
            let sector = code
                .chars()
                .next()
                .and_then(Sector::from_code)
                .ok_or_else(|| format!("unknown sector code {code}"))?;
            let mut list = Vec::with_capacity(items.len());
            for (name, sup) in items {
                let supplier = sup
                    .chars()
                    .next()
                    .and_then(Sector::from_code)
                    .ok_or_else(|| format!("unknown supplier code {sup}"))?;
                list.push(Item { name, supplier });
            }
            if list.len() != ROUNDS as usize * ITEMS_PER_ROUND {
                return Err(format!("{} has {} items", sector.name(), list.len()));
            }
            paths.insert(sector, list);
        }
        if paths.len() != Sector::ALL.len() {
            return Err(format!("{} sectors in trilhas", paths.len()));
        }
        let raw: HashMap<String, HashMap<String, String>> =
            serde_json::from_str(MISSOES).map_err(|e| e.to_string())?;
        let mut texts = HashMap::new();
        for (code, by_round) in raw {
            let sector = code
                .chars()
                .next()
                .and_then(Sector::from_code)
                .ok_or_else(|| format!("unknown sector code {code}"))?;
            let mut list = Vec::with_capacity(ROUNDS as usize);
            for r in 1..=ROUNDS {
                let t = by_round
                    .get(&r.to_string())
                    .ok_or_else(|| format!("{} has no text for round {r}", sector.name()))?;
                list.push(t.clone());
            }
            texts.insert(sector, list);
        }
        if texts.len() != Sector::ALL.len() {
            return Err(format!("{} sectors in missoes", texts.len()));
        }
        Ok(Self { paths, texts })
    }

    /// The cooperative's message for a sector's round (1-based).
    pub fn mission_text(&self, sector: Sector, round: u8) -> &str {
        self.texts
            .get(&sector)
            .and_then(|t| t.get(round.saturating_sub(1) as usize))
            .map(String::as_str)
            .unwrap_or("")
    }

    /// The two items a sector buys in a round (1-based).
    pub fn mission(&self, sector: Sector, round: u8) -> &[Item] {
        let start = (round.saturating_sub(1) as usize) * ITEMS_PER_ROUND;
        self.paths
            .get(&sector)
            .map(|p| &p[start..start + ITEMS_PER_ROUND])
            .unwrap_or(&[])
    }

    /// Everything a sector ever sells.
    pub fn catalog(&self, sector: Sector) -> Vec<&Item> {
        let mut out: Vec<&Item> = self
            .paths
            .values()
            .flatten()
            .filter(|i| i.supplier == sector)
            .collect();
        out.sort_by(|a, b| a.name.cmp(&b.name));
        out.dedup_by(|a, b| a.name == b.name);
        out
    }

    /// How many sectors buy from `seller` in `round`.
    pub fn buyer_sectors(&self, seller: Sector, round: u8) -> usize {
        Sector::ALL
            .iter()
            .filter(|s| {
                self.mission(**s, round)
                    .iter()
                    .any(|i| i.supplier == seller)
            })
            .count()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PriceStyle {
    Round,
    Odd,
}

pub const TARGET_REVENUE: Amount = Amount::from_sat(120_000);
pub const BONUS_PERCENT: u64 = 25;

/// Equal revenue for every seller: the price falls with the number of buyers
/// and rises with the number of sellers. `odd` prices keep their last digits.
pub fn unit_price(
    target: Amount,
    buyers: usize,
    sellers: usize,
    style: PriceStyle,
    salt: u64,
) -> Amount {
    if buyers == 0 || sellers == 0 {
        return Amount::ZERO;
    }
    let base = target.to_sat() * sellers as u64 / buyers as u64;
    let sats = match style {
        PriceStyle::Round => (base / 1_000).max(1) * 1_000,
        PriceStyle::Odd => {
            let offset = 1 + (salt.wrapping_mul(2_654_435_761) % 900);
            base.saturating_sub(offset).max(1_001)
        }
    };
    Amount::from_sat(sats)
}

pub fn bonus_for(spent: Amount) -> Amount {
    Amount::from_sat(spent.to_sat() * BONUS_PERCENT / 100)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cluster::is_round;

    #[test]
    fn trails_load_eight_sectors_with_sixteen_items_each() {
        let trails = Trails::load().expect("trilhas");
        for s in Sector::ALL {
            for r in 1..=ROUNDS {
                let m = trails.mission(s, r);
                assert_eq!(m.len(), 2, "{} round {r}", s.name());
                assert!(
                    m.iter().all(|i| i.supplier != s),
                    "{} buys from itself",
                    s.name()
                );
            }
        }
    }

    #[test]
    fn every_sector_sells_something_every_round() {
        let trails = Trails::load().expect("trilhas");
        for r in 1..=ROUNDS {
            for s in Sector::ALL {
                assert!(
                    trails.buyer_sectors(s, r) >= 1,
                    "{} sells nothing in round {r}",
                    s.name()
                );
            }
        }
    }

    #[test]
    fn every_mission_has_its_text_naming_both_items() {
        let trails = Trails::load().expect("trilhas");
        for s in Sector::ALL {
            for r in 1..=ROUNDS {
                let text = trails.mission_text(s, r);
                assert!(!text.is_empty(), "{} round {r}", s.name());
                for item in trails.mission(s, r) {
                    assert!(
                        text.to_lowercase().contains(&item.name.to_lowercase()),
                        "{} round {r}: {text} lacks {}",
                        s.name(),
                        item.name
                    );
                }
            }
        }
    }

    #[test]
    fn padaria_opens_with_flour() {
        let trails = Trails::load().expect("trilhas");
        let m = trails.mission(Sector::Padaria, 1);
        assert_eq!(m[0].name, "farinha e ovos");
        assert_eq!(m[0].supplier, Sector::Fazenda);
        assert!(trails
            .catalog(Sector::Fazenda)
            .iter()
            .any(|i| i.name == "farinha e ovos"));
    }

    #[test]
    fn initial_balances_are_equal_but_shaped_differently() {
        for s in Sector::ALL {
            let total: Amount = s.initial_coins().iter().map(|(_, a)| *a).sum();
            assert_eq!(total, Amount::from_sat(2_000_000), "{}", s.name());
        }
        assert_eq!(Sector::Padaria.initial_coins().len(), 20);
        assert!(Sector::Fazenda
            .initial_coins()
            .iter()
            .any(|(k, _)| *k == ScriptKind::Legacy));
        assert!(Sector::Loja.funded_by_exchange());
        assert!(!Sector::Padaria.funded_by_exchange());
        assert!(Sector::Padaria.on_storefront());
        assert!(!Sector::Fazenda.on_storefront());
    }

    #[test]
    fn prices_fall_with_buyers_and_rise_with_sellers() {
        let one = unit_price(TARGET_REVENUE, 1, 1, PriceStyle::Round, 0);
        let three = unit_price(TARGET_REVENUE, 3, 1, PriceStyle::Round, 0);
        let crowded = unit_price(TARGET_REVENUE, 1, 2, PriceStyle::Round, 0);
        assert_eq!(one, Amount::from_sat(120_000));
        assert_eq!(three, Amount::from_sat(40_000));
        assert_eq!(crowded, Amount::from_sat(240_000));
        assert_eq!(
            unit_price(TARGET_REVENUE, 0, 1, PriceStyle::Round, 0),
            Amount::ZERO
        );
    }

    #[test]
    fn odd_prices_are_never_round_and_stay_close_to_the_base() {
        for salt in 0..200 {
            let p = unit_price(TARGET_REVENUE, 2, 1, PriceStyle::Odd, salt);
            assert!(!is_round(p), "{p} is round");
            assert!(p.to_sat() > 59_000 && p.to_sat() < 60_000, "{p}");
        }
        assert!(is_round(unit_price(
            TARGET_REVENUE,
            2,
            1,
            PriceStyle::Round,
            5
        )));
    }

    #[test]
    fn bonus_is_a_quarter_of_the_spend() {
        assert_eq!(
            bonus_for(Amount::from_sat(100_000)),
            Amount::from_sat(25_000)
        );
    }

    #[test]
    fn sector_codes_round_trip() {
        for s in Sector::ALL {
            assert_eq!(Sector::from_code(s.code()), Some(s));
        }
        assert_eq!(Sector::from_code('Z'), None);
    }
}
