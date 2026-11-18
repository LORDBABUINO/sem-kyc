//! The game: players, rounds, trades, the State's moves and the ICMS.

mod tax;
mod tools;
mod trade;

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use bitcoin::address::NetworkUnchecked;
use bitcoin::{Address, Amount, OutPoint, Txid};
use serde::{Deserialize, Serialize};
use stealth_engine::types::Report;
use thiserror::Error;

use crate::address::{AddressBook, ScriptKind};
use crate::cluster::{ClusterEvent, ClusterView, Identity};
use crate::economy::{self, Item, PriceStyle, Sector, Trails, ROUNDS};
use crate::ledger::{Coin, Ledger, LedgerError, Origin, OutputSpec, Owner, PlayerId};
use crate::stealth;

pub use tax::Auto;
pub use tools::CoinJoinRoom;
pub use trade::{Invoice, InvoiceStatus, Payment};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub icms_percent: u64,
    pub penalty_percent: u64,
    pub late_percent: u64,
    pub target_revenue: Amount,
    pub bonus_percent: u64,
    pub negotiation_secs: u64,
    pub coinjoin_min: usize,
    pub coinjoin_denomination: Amount,
    pub coinjoin_window_secs: u64,
    pub registration_fee: Amount,
    pub dust: Amount,
    pub seed: [u8; 32],
}

impl Default for Config {
    fn default() -> Self {
        Self {
            icms_percent: 18,
            penalty_percent: 100,
            late_percent: 20,
            target_revenue: economy::TARGET_REVENUE,
            bonus_percent: economy::BONUS_PERCENT,
            negotiation_secs: 180,
            coinjoin_min: 5,
            coinjoin_denomination: Amount::from_sat(100_000),
            coinjoin_window_secs: 30,
            registration_fee: Amount::from_sat(10_000),
            dust: Amount::from_sat(546),
            seed: [42u8; 32],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "phase")]
pub enum Phase {
    Lobby,
    Newspaper { round: u8 },
    Negotiation { round: u8, deadline: u64 },
    Reveal { round: u8 },
    Final,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Unlock {
    FreshAddress,
    CoinControl,
    OddPrice,
    Freeze,
    Lightning,
    CoinJoin,
    Taproot,
    Payjoin,
    SilentPayments,
    Report,
}

impl Unlock {
    pub fn for_round(round: u8) -> &'static [Unlock] {
        match round {
            1 => &[Unlock::FreshAddress],
            2 => &[Unlock::CoinControl],
            3 => &[Unlock::OddPrice],
            4 => &[Unlock::Freeze],
            5 => &[Unlock::Lightning],
            6 => &[Unlock::CoinJoin],
            7 => &[Unlock::Taproot],
            8 => &[Unlock::Payjoin, Unlock::SilentPayments],
            _ => &[],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMode {
    Storefront,
    Fresh,
    Payjoin,
    SilentPayments,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoinSelection {
    Automatic,
    SmallestCovering,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "status")]
pub enum ItemStatus {
    Pending,
    Paid { txid: Txid },
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissionItem {
    pub item: Item,
    pub status: ItemStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mission {
    pub round: u8,
    pub items: Vec<MissionItem>,
    pub bonus: Option<Txid>,
}

impl Mission {
    pub fn complete(&self) -> bool {
        self.items
            .iter()
            .all(|i| matches!(i.status, ItemStatus::Paid { .. }))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub sector: Sector,
    #[serde(with = "stealth_model::types::serde_addr")]
    pub storefront: Address<NetworkUnchecked>,
    pub registered: bool,
    pub storefront_is_silent: bool,
    pub receive_mode: ReceiveMode,
    pub price_style: PriceStyle,
    pub coin_selection: CoinSelection,
    pub unlocked: BTreeSet<Unlock>,
    pub taproot: bool,
    pub silent_address: Option<String>,
    pub missions: BTreeMap<u8, Mission>,
}

impl Player {
    pub fn kind(&self) -> ScriptKind {
        if self.taproot {
            ScriptKind::Taproot
        } else {
            ScriptKind::Segwit
        }
    }

    pub fn has(&self, unlock: Unlock) -> bool {
        self.unlocked.contains(&unlock)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Operation {
    pub round: u8,
    pub txid: Txid,
    pub payer: PlayerId,
    pub payee: PlayerId,
    pub amount: Amount,
    pub vout: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Headline {
    pub round: u8,
    pub player: PlayerId,
    pub finding: stealth_engine::types::Finding,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case", tag = "event")]
pub enum Event {
    PlayerJoined {
        player: PlayerId,
    },
    Phase {
        phase: Phase,
    },
    Tx {
        txid: Txid,
        players: Vec<PlayerId>,
        clusters: Vec<ClusterEvent>,
    },
    Kyc {
        players: Vec<PlayerId>,
        clusters: Vec<ClusterEvent>,
    },
    Unlock {
        player: PlayerId,
        unlock: Unlock,
    },
    InvoicePaid {
        code: String,
        buyer: PlayerId,
        seller: PlayerId,
        txid: Txid,
    },
    PayjoinRequested {
        code: String,
        buyer: PlayerId,
        seller: PlayerId,
    },
    Bonus {
        player: PlayerId,
        txid: Txid,
        amount: Amount,
    },
    AutoIssued {
        player: PlayerId,
        auto_id: u32,
        amount: Amount,
    },
    AutoPaid {
        player: PlayerId,
        auto_id: u32,
        txid: Txid,
    },
    CoinJoin {
        txid: Option<Txid>,
        players: Vec<PlayerId>,
    },
    Reveal {
        round: u8,
    },
    Block {
        height: u32,
    },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GameError {
    #[error("unknown player")]
    UnknownPlayer,
    #[error("that is not possible in this phase")]
    WrongPhase,
    #[error("the name is taken")]
    NameTaken,
    #[error("the name is empty")]
    EmptyName,
    #[error("register the storefront first")]
    NotRegistered,
    #[error("already registered")]
    AlreadyRegistered,
    #[error("your state registration is suspended until the assessment is paid")]
    Suspended,
    #[error("that is not unlocked yet")]
    Locked,
    #[error("Silent Payments need a Taproot wallet")]
    NeedsTaproot,
    #[error("unknown invoice")]
    UnknownInvoice,
    #[error("the invoice is no longer open")]
    InvoiceClosed,
    #[error("you cannot pay yourself")]
    SelfPayment,
    #[error("{0} is not in your catalog")]
    NotInCatalog(String),
    #[error("not enough unfrozen coins")]
    InsufficientFunds,
    #[error("{0} is not one of your spendable coins")]
    NotYourCoin(OutPoint),
    #[error("that coin is frozen")]
    Frozen,
    #[error("that coin cannot take this action")]
    WrongCoin,
    #[error("unknown assessment")]
    UnknownAuto,
    #[error("the assessment is already paid")]
    AutoPaid,
    #[error("the CoinJoin room is closed")]
    RoomClosed,
    #[error("ledger: {0}")]
    Ledger(#[from] LedgerError),
    #[error("economy: {0}")]
    Economy(String),
}

#[derive(Debug)]
pub struct Game {
    pub config: Config,
    pub ledger: Ledger,
    pub clusters: ClusterView,
    pub trails: Trails,
    book: AddressBook,
    players: BTreeMap<PlayerId, Player>,
    phase: Phase,
    round: u8,
    invoices: HashMap<String, Invoice>,
    operations: Vec<Operation>,
    autos: Vec<Auto>,
    room: Option<CoinJoinRoom>,
    state_address: Address<NetworkUnchecked>,
    exchange_txids: HashSet<Txid>,
    kyc_applied: bool,
    dust_sent: bool,
    reports: HashMap<PlayerId, BTreeMap<u8, Report>>,
    headlines: BTreeMap<u8, Headline>,
    events: Vec<Event>,
    next_code: u64,
}

impl Game {
    pub fn new(config: Config) -> Result<Self, GameError> {
        let trails = Trails::load().map_err(GameError::Economy)?;
        let mut book = AddressBook::new(config.seed);
        let mut ledger = Ledger::new();
        let state_address = book.fresh(ScriptKind::Segwit);
        let mut clusters = ClusterView::new();
        clusters.exclude(&state_address);
        // The State and the cooperative start with deep pockets.
        ledger.mint(vec![
            OutputSpec {
                address: state_address.clone(),
                amount: Amount::from_sat(100_000_000),
                owner: Owner::State,
                origin: Origin::Initial,
            },
            OutputSpec {
                address: book.fresh(ScriptKind::Segwit),
                amount: Amount::from_sat(100_000_000),
                owner: Owner::Cooperative,
                origin: Origin::Initial,
            },
        ])?;
        ledger.mine();
        Ok(Self {
            config,
            ledger,
            clusters,
            trails,
            book,
            players: BTreeMap::new(),
            phase: Phase::Lobby,
            round: 0,
            invoices: HashMap::new(),
            operations: Vec::new(),
            autos: Vec::new(),
            room: None,
            state_address,
            exchange_txids: HashSet::new(),
            kyc_applied: false,
            dust_sent: false,
            reports: HashMap::new(),
            headlines: BTreeMap::new(),
            events: Vec::new(),
            next_code: 1,
        })
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn round(&self) -> u8 {
        self.round
    }

    pub fn state_address(&self) -> &Address<NetworkUnchecked> {
        &self.state_address
    }

    pub fn players(&self) -> impl Iterator<Item = &Player> {
        self.players.values()
    }

    pub fn player(&self, id: PlayerId) -> Result<&Player, GameError> {
        self.players.get(&id).ok_or(GameError::UnknownPlayer)
    }

    fn player_mut(&mut self, id: PlayerId) -> Result<&mut Player, GameError> {
        self.players.get_mut(&id).ok_or(GameError::UnknownPlayer)
    }

    pub fn drain_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    pub fn exchange_txids(&self) -> &HashSet<Txid> {
        &self.exchange_txids
    }

    pub fn operations(&self) -> &[Operation] {
        &self.operations
    }

    pub fn headline(&self, round: u8) -> Option<&Headline> {
        self.headlines.get(&round)
    }

    /// The detectors over the player's whole history. The exchange's list
    /// only counts once the exchange handed it over.
    pub fn report(&self, player: PlayerId) -> Report {
        let none = HashSet::new();
        let known = if self.kyc_applied {
            &self.exchange_txids
        } else {
            &none
        };
        stealth::analyze(&self.ledger, player, known)
    }

    pub fn reports(&self, player: PlayerId) -> Option<&BTreeMap<u8, Report>> {
        self.reports.get(&player)
    }

    /// Who the State thinks owns an address.
    pub fn identity(&self, address: &Address<NetworkUnchecked>) -> Identity {
        self.clusters.identity(address)
    }

    pub fn mine_block(&mut self) -> u32 {
        let height = self.ledger.mine();
        self.events.push(Event::Block { height });
        height
    }

    /// A new player: sector by join order, starting coins by sector.
    pub fn join(&mut self, name: &str) -> Result<PlayerId, GameError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(GameError::EmptyName);
        }
        if self
            .players
            .values()
            .any(|p| p.name.eq_ignore_ascii_case(name))
        {
            return Err(GameError::NameTaken);
        }
        let id = self.players.keys().next_back().map_or(1, |k| k + 1);
        let sector = Sector::ALL[self.players.len() % Sector::ALL.len()];
        let coins = sector.initial_coins();
        let largest = coins.iter().map(|(_, a)| *a).max().unwrap_or(Amount::ZERO);
        let storefront = self.book.fresh(ScriptKind::Segwit);
        // Sectors that live off many small receipts kept them all on the
        // storefront address: the reuse is there before the first round.
        let mut batch = Vec::new();
        for (kind, amount) in coins {
            // The exchange-funded sector's big coin is the withdrawal; its small
            // one is pocket money for the registration fee.
            let origin = if sector.funded_by_exchange() && amount == largest {
                Origin::ExchangeWithdrawal
            } else {
                Origin::Initial
            };
            let on_storefront = sector.on_storefront() && kind == ScriptKind::Segwit;
            let spec = OutputSpec {
                address: if on_storefront {
                    storefront.clone()
                } else {
                    self.book.fresh(kind)
                },
                amount,
                owner: Owner::Player(id),
                origin,
            };
            if on_storefront {
                let txid = self.ledger.mint(vec![spec])?;
                self.observe(txid, vec![id]);
            } else {
                batch.push(spec);
            }
        }
        if sector.funded_by_exchange() {
            // An exchange pays its customers in batches; the other outputs
            // are strangers who withdrew at the same time.
            for sats in EXCHANGE_BATCH {
                batch.push(OutputSpec {
                    address: self.book.fresh(ScriptKind::Segwit),
                    amount: Amount::from_sat(sats),
                    owner: Owner::Exchange,
                    origin: Origin::ExchangeWithdrawal,
                });
            }
        }
        if !batch.is_empty() {
            let txid = self.ledger.mint(batch)?;
            if sector.funded_by_exchange() {
                self.exchange_txids.insert(txid);
            }
            self.observe(txid, vec![id]);
        }
        self.players.insert(
            id,
            Player {
                id,
                name: name.to_owned(),
                sector,
                storefront,
                registered: false,
                storefront_is_silent: false,
                receive_mode: ReceiveMode::Storefront,
                price_style: PriceStyle::Round,
                coin_selection: CoinSelection::Automatic,
                unlocked: BTreeSet::new(),
                taproot: false,
                silent_address: None,
                missions: BTreeMap::new(),
            },
        );
        self.events.push(Event::PlayerJoined { player: id });
        // Whoever arrives late gets what the room already has.
        for r in 1..=self.round {
            for u in Unlock::for_round(r) {
                self.grant(id, *u);
            }
        }
        if self.round >= 1 {
            self.assign_mission(id, self.round);
        }
        Ok(id)
    }

    /// Pays the registration fee to the State. The State learns the storefront
    /// address and whoever paid the fee.
    pub fn register(&mut self, player: PlayerId) -> Result<Txid, GameError> {
        let p = self.player(player)?;
        if p.registered {
            return Err(GameError::AlreadyRegistered);
        }
        let storefront = p.storefront.clone();
        let fee = self.config.registration_fee;
        let inputs = self.select_coins(player, fee, None)?;
        let txid = self.pay_state(player, &inputs, fee, Origin::Tax)?;
        self.clusters.label(&storefront, player);
        self.player_mut(player)?.registered = true;
        Ok(txid)
    }

    /// A payment to the State's address: the State knows who is paying, so
    /// every input gets the payer's name.
    fn pay_state(
        &mut self,
        player: PlayerId,
        inputs: &[OutPoint],
        amount: Amount,
        origin: Origin,
    ) -> Result<Txid, GameError> {
        let total: Amount = inputs
            .iter()
            .filter_map(|op| self.ledger.coin(op))
            .map(|c| c.amount)
            .sum();
        let mut outputs = vec![OutputSpec {
            address: self.state_address.clone(),
            amount,
            owner: Owner::State,
            origin,
        }];
        let change = total - amount;
        let kind = self.player(player)?.kind();
        if change > Amount::ZERO {
            outputs.push(OutputSpec {
                address: self.book.fresh(kind),
                amount: change,
                owner: Owner::Player(player),
                origin: Origin::Change,
            });
        }
        let input_addrs: Vec<_> = inputs
            .iter()
            .filter_map(|op| self.ledger.coin(op))
            .map(|c| c.address.clone())
            .collect();
        let txid = self.ledger.spend(inputs, outputs)?;
        let mut events = Vec::new();
        for a in &input_addrs {
            events.extend(self.clusters.label(a, player));
        }
        self.observe_with(txid, vec![player], events);
        Ok(txid)
    }

    fn observe(&mut self, txid: Txid, players: Vec<PlayerId>) {
        self.observe_with(txid, players, Vec::new());
    }

    fn observe_with(&mut self, txid: Txid, players: Vec<PlayerId>, mut extra: Vec<ClusterEvent>) {
        let Some(tx) = self.ledger.tx(&txid).cloned() else {
            return;
        };
        let inputs: Vec<Coin> = self
            .ledger
            .resolved_inputs(&tx)
            .into_iter()
            .cloned()
            .collect();
        let refs: Vec<&Coin> = inputs.iter().collect();
        let mut events = self.clusters.observe(&tx, &refs);
        events.append(&mut extra);
        self.events.push(Event::Tx {
            txid,
            players,
            clusters: events,
        });
    }

    /// The coins a player may spend right now, in the order the wallet would
    /// pick them automatically: smallest first, until the amount is covered.
    pub fn spendable(&self, player: PlayerId) -> Vec<&Coin> {
        let mut coins: Vec<&Coin> = self
            .ledger
            .coins_of(Owner::Player(player))
            .into_iter()
            .filter(|c| !c.frozen)
            .collect();
        coins.sort_by_key(|c| (c.amount, c.outpoint.txid, c.outpoint.vout));
        coins
    }

    /// Picks inputs for `amount` by the player's selection setting, or by the
    /// explicit list when the player chose the coins by hand.
    pub fn select_coins(
        &self,
        player: PlayerId,
        amount: Amount,
        chosen: Option<&[OutPoint]>,
    ) -> Result<Vec<OutPoint>, GameError> {
        let p = self.player(player)?;
        if let Some(list) = chosen {
            if !p.has(Unlock::CoinControl) {
                return Err(GameError::Locked);
            }
            let mut total = Amount::ZERO;
            for (i, op) in list.iter().enumerate() {
                if list[..i].contains(op) {
                    return Err(GameError::NotYourCoin(*op));
                }
                let coin = self.ledger.coin(op).ok_or(GameError::NotYourCoin(*op))?;
                if coin.owner != Owner::Player(player) {
                    return Err(GameError::NotYourCoin(*op));
                }
                if coin.frozen {
                    return Err(GameError::Frozen);
                }
                total += coin.amount;
            }
            if total < amount {
                return Err(GameError::InsufficientFunds);
            }
            return Ok(list.to_vec());
        }
        let coins = self.spendable(player);
        if p.coin_selection == CoinSelection::SmallestCovering {
            if let Some(c) = coins.iter().find(|c| c.amount >= amount) {
                return Ok(vec![c.outpoint]);
            }
        }
        let mut picked = Vec::new();
        let mut total = Amount::ZERO;
        for c in coins {
            picked.push(c.outpoint);
            total += c.amount;
            if total >= amount {
                return Ok(picked);
            }
        }
        Err(GameError::InsufficientFunds)
    }

    fn fresh_for(&mut self, player: PlayerId) -> Result<Address<NetworkUnchecked>, GameError> {
        let kind = self.player(player)?.kind();
        Ok(self.book.fresh(kind))
    }

    // ----- rounds -----

    /// Lobby -> newspaper 1; reveal n -> newspaper n+1; reveal 8 -> final.
    pub fn advance(&mut self) -> Result<Phase, GameError> {
        let next = match self.phase {
            Phase::Lobby => 1,
            Phase::Reveal { round } if round >= ROUNDS => {
                self.phase = Phase::Final;
                for id in self.players.keys().copied().collect::<Vec<_>>() {
                    self.grant(id, Unlock::Report);
                }
                self.events.push(Event::Phase { phase: self.phase });
                return Ok(self.phase);
            }
            Phase::Reveal { round } => round + 1,
            _ => return Err(GameError::WrongPhase),
        };
        self.round = next;
        self.ledger.set_round(next);
        let ids: Vec<PlayerId> = self.players.keys().copied().collect();
        for id in ids {
            for u in Unlock::for_round(next) {
                self.grant(id, *u);
            }
            self.assign_mission(id, next);
        }
        self.phase = Phase::Newspaper { round: next };
        self.events.push(Event::Phase { phase: self.phase });
        Ok(self.phase)
    }

    fn assign_mission(&mut self, player: PlayerId, round: u8) {
        let Some(sector) = self.players.get(&player).map(|p| p.sector) else {
            return;
        };
        let items = self
            .trails
            .mission(sector, round)
            .iter()
            .map(|i| MissionItem {
                item: i.clone(),
                status: ItemStatus::Pending,
            })
            .collect();
        if let Some(p) = self.players.get_mut(&player) {
            p.missions.insert(
                round,
                Mission {
                    round,
                    items,
                    bonus: None,
                },
            );
        }
    }

    fn grant(&mut self, player: PlayerId, unlock: Unlock) {
        if let Some(p) = self.players.get_mut(&player) {
            if p.unlocked.insert(unlock) {
                match unlock {
                    Unlock::FreshAddress if p.receive_mode == ReceiveMode::Storefront => {
                        p.receive_mode = ReceiveMode::Fresh
                    }
                    Unlock::CoinControl if p.coin_selection == CoinSelection::Automatic => {
                        p.coin_selection = CoinSelection::SmallestCovering
                    }
                    Unlock::OddPrice => p.price_style = PriceStyle::Odd,
                    _ => {}
                }
                self.events.push(Event::Unlock { player, unlock });
            }
        }
    }

    pub fn open_negotiation(&mut self, now: u64) -> Result<Phase, GameError> {
        let Phase::Newspaper { round } = self.phase else {
            return Err(GameError::WrongPhase);
        };
        self.phase = Phase::Negotiation {
            round,
            deadline: now + self.config.negotiation_secs,
        };
        self.events.push(Event::Phase { phase: self.phase });
        Ok(self.phase)
    }

    /// Closes the round: expires open invoices, assesses the ICMS, runs the
    /// detectors for everyone and picks the headline.
    pub fn close_round(&mut self, now: u64) -> Result<Phase, GameError> {
        let Phase::Negotiation { round, .. } = self.phase else {
            return Err(GameError::WrongPhase);
        };
        self.expire_invoices(round);
        self.settle_room(now);
        for p in self.players.values_mut() {
            if let Some(m) = p.missions.get_mut(&round) {
                for item in &mut m.items {
                    if item.status == ItemStatus::Pending {
                        item.status = ItemStatus::Expired;
                    }
                }
            }
        }
        self.assess(round);
        let ids: Vec<PlayerId> = self.players.keys().copied().collect();
        for id in ids {
            let report = self.report(id);
            self.reports.entry(id).or_default().insert(round, report);
        }
        self.pick_headline(round);
        self.phase = Phase::Reveal { round };
        self.events.push(Event::Reveal { round });
        self.events.push(Event::Phase { phase: self.phase });
        Ok(self.phase)
    }

    fn pick_headline(&mut self, round: u8) {
        let wanted = round_detectors(round);
        let mut best: Option<(
            stealth_engine::types::Severity,
            PlayerId,
            stealth_engine::types::Finding,
        )> = None;
        for (id, reports) in &self.reports {
            let Some(after) = reports.get(&round) else {
                continue;
            };
            let before = reports.range(..round).next_back().map(|(_, r)| r);
            for f in stealth::new_findings(before, after) {
                if !wanted.is_empty() && !wanted.contains(&f.vulnerability_type) {
                    continue;
                }
                let better = match &best {
                    None => true,
                    Some((sev, pid, _)) => {
                        f.severity > *sev
                            || (f.severity == *sev && salt(*id, round) < salt(*pid, round))
                    }
                };
                if better {
                    best = Some((f.severity, *id, f.clone()));
                }
            }
        }
        if let Some((_, player, finding)) = best {
            self.headlines.insert(
                round,
                Headline {
                    round,
                    player,
                    finding,
                },
            );
        }
    }

    pub fn is_suspended(&self, player: PlayerId) -> bool {
        self.autos
            .iter()
            .any(|a| a.player == player && a.paid.is_none())
    }

    fn require_active(&self, player: PlayerId) -> Result<&Player, GameError> {
        let p = self.player(player)?;
        if !p.registered {
            return Err(GameError::NotRegistered);
        }
        if self.is_suspended(player) {
            return Err(GameError::Suspended);
        }
        Ok(p)
    }

    fn negotiation_round(&self) -> Result<u8, GameError> {
        match self.phase {
            Phase::Negotiation { round, .. } => Ok(round),
            _ => Err(GameError::WrongPhase),
        }
    }

    // ----- settings -----

    pub fn set_receive_mode(
        &mut self,
        player: PlayerId,
        mode: ReceiveMode,
    ) -> Result<(), GameError> {
        let p = self.player(player)?;
        let needed = match mode {
            ReceiveMode::Storefront => None,
            ReceiveMode::Fresh => Some(Unlock::FreshAddress),
            ReceiveMode::Payjoin => Some(Unlock::Payjoin),
            ReceiveMode::SilentPayments => Some(Unlock::SilentPayments),
        };
        if let Some(u) = needed {
            if !p.has(u) {
                return Err(GameError::Locked);
            }
        }
        if mode == ReceiveMode::SilentPayments && !p.taproot {
            return Err(GameError::NeedsTaproot);
        }
        if mode == ReceiveMode::SilentPayments && p.silent_address.is_none() {
            let sp = self.book.silent_payment();
            self.player_mut(player)?.silent_address = Some(sp);
        }
        self.player_mut(player)?.receive_mode = mode;
        Ok(())
    }

    pub fn set_price_style(
        &mut self,
        player: PlayerId,
        style: PriceStyle,
    ) -> Result<(), GameError> {
        let p = self.player(player)?;
        if style == PriceStyle::Odd && !p.has(Unlock::OddPrice) {
            return Err(GameError::Locked);
        }
        self.player_mut(player)?.price_style = style;
        Ok(())
    }

    pub fn set_coin_selection(
        &mut self,
        player: PlayerId,
        selection: CoinSelection,
    ) -> Result<(), GameError> {
        let p = self.player(player)?;
        if selection != CoinSelection::Automatic && !p.has(Unlock::CoinControl) {
            return Err(GameError::Locked);
        }
        self.player_mut(player)?.coin_selection = selection;
        Ok(())
    }

    pub fn set_frozen(
        &mut self,
        player: PlayerId,
        op: OutPoint,
        frozen: bool,
    ) -> Result<(), GameError> {
        if !self.player(player)?.has(Unlock::Freeze) {
            return Err(GameError::Locked);
        }
        match self.ledger.coin(&op) {
            Some(c) if c.owner == Owner::Player(player) => {}
            _ => return Err(GameError::NotYourCoin(op)),
        }
        self.ledger.set_frozen(&op, frozen);
        Ok(())
    }

    /// Round 8: the storefront is re-registered, this time as an sp1 string.
    pub fn register_silent_storefront(&mut self, player: PlayerId) -> Result<String, GameError> {
        let p = self.player(player)?;
        if p.receive_mode != ReceiveMode::SilentPayments {
            return Err(GameError::Locked);
        }
        let sp = p.silent_address.clone().ok_or(GameError::Locked)?;
        self.player_mut(player)?.storefront_is_silent = true;
        Ok(sp)
    }
}

const EXCHANGE_BATCH: [u64; 7] = [
    312_500, 1_250_000, 487_000, 2_750_000, 95_000, 640_000, 1_100_000,
];

pub fn round_detectors(round: u8) -> &'static [stealth_engine::types::VulnerabilityType] {
    use stealth_engine::types::VulnerabilityType as V;
    match round {
        1 => &[V::AddressReuse],
        2 => &[V::Cioh, V::UnnecessaryInput],
        3 => &[V::ChangeDetection, V::PeelChain],
        4 => &[V::Dust, V::DustAttack, V::DustSpending],
        5 => &[
            V::ExchangeOrigin,
            V::TaintedUtxoMerge,
            V::ClusterMerge,
            V::DustSpending,
        ],
        6 => &[V::Consolidation, V::ClusterMerge, V::ToxicChange],
        7 => &[
            V::ScriptTypeMixing,
            V::BehavioralFingerprint,
            V::UtxoAgeSpread,
        ],
        _ => &[],
    }
}

fn salt(player: PlayerId, round: u8) -> u64 {
    (player as u64 ^ (round as u64) << 32).wrapping_mul(0x9E37_79B9_7F4A_7C15)
}
