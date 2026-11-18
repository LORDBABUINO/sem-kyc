//! What the screens receive, shaped as `api.md` describes.

use bitcoin::{Amount, OutPoint};
use dever_core::address::ScriptKind;
use dever_core::economy::{PriceStyle, Sector};
use dever_core::game::{
    CoinSelection, Game, Invoice, InvoiceStatus, Mission, Phase, Player, ReceiveMode, Unlock,
};
use dever_core::ledger::{Coin, Origin, Owner, PlayerId};
use dever_core::view::{AutoView, HistoryEntry, Reveal};
use serde::Serialize;

use crate::error::ApiError;

#[derive(Debug, Serialize)]
pub struct SectorInfo {
    pub sector: Sector,
    pub name: &'static str,
    pub color: &'static str,
}

impl From<Sector> for SectorInfo {
    fn from(s: Sector) -> Self {
        Self {
            sector: s,
            name: s.name(),
            color: s.color(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CoinView {
    pub outpoint: String,
    pub address: String,
    pub amount: u64,
    pub kind: ScriptKind,
    pub frozen: bool,
    pub origin: Origin,
    pub round: u8,
}

pub fn outpoint_string(op: &OutPoint) -> String {
    format!("{}:{}", op.txid, op.vout)
}

pub fn parse_outpoint(s: &str) -> Result<OutPoint, ApiError> {
    let (txid, vout) = s
        .split_once(':')
        .ok_or_else(|| ApiError::bad("outpoint inválido"))?;
    let txid = txid
        .parse()
        .map_err(|_| ApiError::bad("outpoint inválido"))?;
    let vout = vout
        .parse()
        .map_err(|_| ApiError::bad("outpoint inválido"))?;
    Ok(OutPoint { txid, vout })
}

impl From<&Coin> for CoinView {
    fn from(c: &Coin) -> Self {
        Self {
            outpoint: outpoint_string(&c.outpoint),
            address: c.address.clone().assume_checked().to_string(),
            amount: c.amount.to_sat(),
            kind: c.kind,
            frozen: c.frozen,
            origin: c.origin,
            round: c.round,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PlayerView {
    pub id: PlayerId,
    pub name: String,
    pub sector: SectorInfo,
    pub storefront: String,
    pub registered: bool,
    pub storefront_is_silent: bool,
    pub receive_mode: ReceiveMode,
    pub price_style: PriceStyle,
    pub coin_selection: CoinSelection,
    pub unlocked: Vec<Unlock>,
    pub taproot: bool,
    pub silent_address: Option<String>,
}

impl From<&Player> for PlayerView {
    fn from(p: &Player) -> Self {
        Self {
            id: p.id,
            name: p.name.clone(),
            sector: p.sector.into(),
            storefront: p.storefront.clone().assume_checked().to_string(),
            registered: p.registered,
            storefront_is_silent: p.storefront_is_silent,
            receive_mode: p.receive_mode,
            price_style: p.price_style,
            coin_selection: p.coin_selection,
            unlocked: p.unlocked.iter().copied().collect(),
            taproot: p.taproot,
            silent_address: p.silent_address.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MissionItemView {
    pub item: dever_core::economy::Item,
    pub status: dever_core::game::ItemStatus,
    pub price: u64,
    pub supplier: SectorInfo,
}

#[derive(Debug, Serialize)]
pub struct MissionView {
    pub round: u8,
    pub text: String,
    pub items: Vec<MissionItemView>,
    pub bonus: Option<bitcoin::Txid>,
    pub complete: bool,
}

/// The sector's price this round, as the first seller in it would charge.
pub fn sector_price(game: &Game, sector: Sector, round: u8) -> u64 {
    if round == 0 {
        return 0;
    }
    game.players()
        .find(|p| p.sector == sector)
        .and_then(|p| game.unit_price(p.id, round).ok())
        .map(|a| a.to_sat())
        .unwrap_or(0)
}

pub fn mission_view(game: &Game, sector: Sector, m: &Mission) -> MissionView {
    MissionView {
        round: m.round,
        text: game.trails.mission_text(sector, m.round).to_owned(),
        items: m
            .items
            .iter()
            .map(|mi| MissionItemView {
                item: mi.item.clone(),
                status: mi.status.clone(),
                price: sector_price(game, mi.item.supplier, m.round),
                supplier: mi.item.supplier.into(),
            })
            .collect(),
        bonus: m.bonus,
        complete: m.complete(),
    }
}

#[derive(Debug, Serialize)]
pub struct InvoiceView {
    pub code: String,
    pub seller: PlayerId,
    pub seller_name: String,
    pub seller_sector: SectorInfo,
    pub item: dever_core::economy::Item,
    pub amount: u64,
    pub display: String,
    pub uri: String,
    pub payjoin: bool,
    pub round: u8,
    pub status: serde_json::Value,
}

pub fn invoice_view(game: &Game, i: &Invoice, public_url: &str) -> InvoiceView {
    let seller = game.player(i.seller).ok();
    let btc = format!("{:.8}", i.amount.to_btc());
    let mut uri = format!(
        "bitcoin:{}?amount={}&label={}",
        i.display,
        btc.trim_end_matches('0').trim_end_matches('.'),
        urlencode(&i.item.name)
    );
    // The code rides along so a scanned QR needs no typing.
    uri.push_str(&format!("&code={}", i.code));
    if i.payjoin {
        uri.push_str(&format!(
            "&pj={}/api/invoices/{}/payjoin",
            public_url.trim_end_matches('/'),
            i.code
        ));
    }
    InvoiceView {
        code: i.code.clone(),
        seller: i.seller,
        seller_name: seller.map(|p| p.name.clone()).unwrap_or_default(),
        seller_sector: seller.map(|p| p.sector).unwrap_or(Sector::Padaria).into(),
        item: i.item.clone(),
        amount: i.amount.to_sat(),
        display: i.display.clone(),
        uri,
        payjoin: i.payjoin,
        round: i.round,
        status: status_with_name(game, &i.status),
    }
}

/// The status as the enum serializes it, plus the buyer's name where there
/// is a buyer, so the seller's screen can say who is paying.
fn status_with_name(game: &Game, status: &InvoiceStatus) -> serde_json::Value {
    let mut value = serde_json::to_value(status).unwrap_or(serde_json::Value::Null);
    let buyer = match status {
        InvoiceStatus::PayjoinPending { buyer, .. } | InvoiceStatus::Paid { buyer, .. } => {
            Some(*buyer)
        }
        _ => None,
    };
    if let (Some(b), Some(obj)) = (buyer, value.as_object_mut()) {
        let name = game.player(b).map(|p| p.name.clone()).unwrap_or_default();
        obj.insert("buyer_name".to_owned(), serde_json::Value::String(name));
    }
    value
}

fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[derive(Debug, Serialize)]
pub struct CatalogEntry {
    pub name: String,
    pub price: u64,
}

#[derive(Debug, Serialize)]
pub struct RoomView {
    pub entries: usize,
    pub deadline: u64,
}

#[derive(Debug, Serialize)]
pub struct Me {
    pub player: PlayerView,
    pub phase: Phase,
    pub round: u8,
    pub height: u32,
    pub balance: u64,
    pub coins: Vec<CoinView>,
    pub mission: Option<MissionView>,
    pub missions: Vec<MissionView>,
    pub catalog: Vec<CatalogEntry>,
    pub invoices: Vec<InvoiceView>,
    pub payjoin_requests: Vec<InvoiceView>,
    pub history: Vec<HistoryEntry>,
    pub autos: Vec<AutoView>,
    pub suspended: bool,
    pub debt: u64,
    pub counter_address: String,
    pub coinjoin: Option<RoomView>,
    pub report_available: bool,
}

pub fn me(game: &Game, id: PlayerId, public_url: &str) -> Result<Me, ApiError> {
    let p = game.player(id)?;
    let round = game.round();
    let price = if round >= 1 {
        game.unit_price(id, round).map(|a| a.to_sat()).unwrap_or(0)
    } else {
        0
    };
    let catalog = game
        .trails
        .catalog(p.sector)
        .into_iter()
        .map(|i| CatalogEntry {
            name: i.name.clone(),
            price,
        })
        .collect();
    let mine: Vec<&Invoice> = game.invoices_of(id);
    let invoices = mine
        .iter()
        .filter(|i| i.round == round)
        .map(|i| invoice_view(game, i, public_url))
        .collect();
    let payjoin_requests = mine
        .iter()
        .filter(|i| matches!(i.status, InvoiceStatus::PayjoinPending { .. }))
        .map(|i| invoice_view(game, i, public_url))
        .collect();
    let autos = game
        .auto_views()
        .into_iter()
        .filter(|a| a.player == id)
        .collect();
    let mut coins: Vec<&Coin> = game.ledger.coins_of(Owner::Player(id));
    coins.sort_by_key(|c| {
        (
            std::cmp::Reverse(c.amount),
            c.outpoint.txid,
            c.outpoint.vout,
        )
    });
    Ok(Me {
        player: p.into(),
        phase: game.phase(),
        round,
        height: game.ledger.height(),
        balance: game.ledger.balance(Owner::Player(id)).to_sat(),
        coins: coins.into_iter().map(CoinView::from).collect(),
        mission: p
            .missions
            .get(&round)
            .map(|m| mission_view(game, p.sector, m)),
        missions: p
            .missions
            .values()
            .map(|m| mission_view(game, p.sector, m))
            .collect(),
        catalog,
        invoices,
        payjoin_requests,
        history: game.history(id),
        autos,
        suspended: game.is_suspended(id),
        debt: game.debt(id).to_sat(),
        counter_address: game.state_address().clone().assume_checked().to_string(),
        coinjoin: game.coinjoin_room().map(|r| RoomView {
            entries: r.entries.len(),
            deadline: r.deadline,
        }),
        report_available: game.phase() == Phase::Final,
    })
}

#[derive(Debug, Serialize)]
pub struct Standing {
    pub player: PlayerId,
    pub name: String,
    pub balance: u64,
}

#[derive(Debug, Serialize)]
pub struct Totals {
    pub assessed: u64,
    pub collected: u64,
}

#[derive(Debug, Serialize)]
pub struct PublicState {
    pub phase: Phase,
    pub round: u8,
    pub height: u32,
    pub players: usize,
    pub registered: usize,
    pub counter_address: String,
    pub join_url: String,
    pub reveal: Option<Reveal>,
    pub autos_open: Vec<AutoView>,
    pub totals: Totals,
    pub payments_this_round: usize,
    pub standings: Vec<Standing>,
}

pub fn public_state(game: &Game, join_url: String) -> PublicState {
    let round = game.round();
    let revealed = match game.phase() {
        Phase::Reveal { round } => Some(round),
        Phase::Newspaper { round } if round > 1 => Some(round - 1),
        Phase::Negotiation { round, .. } if round > 1 => Some(round - 1),
        Phase::Final => Some(round),
        _ => None,
    };
    let autos = game.auto_views();
    let standings = if game.phase() == Phase::Final {
        game.standings()
            .into_iter()
            .map(|(player, name, balance)| Standing {
                player,
                name,
                balance: balance.to_sat(),
            })
            .collect()
    } else {
        Vec::new()
    };
    PublicState {
        phase: game.phase(),
        round,
        height: game.ledger.height(),
        players: game.players().count(),
        registered: game.players().filter(|p| p.registered).count(),
        counter_address: game.state_address().clone().assume_checked().to_string(),
        join_url,
        reveal: revealed.map(|r| game.reveal(r)),
        autos_open: autos.iter().filter(|a| !a.paid).cloned().collect(),
        totals: Totals {
            assessed: autos.iter().map(|a| a.amount.to_sat()).sum(),
            collected: autos
                .iter()
                .filter(|a| a.paid)
                .map(|a| a.amount.to_sat())
                .sum(),
        },
        payments_this_round: game
            .operations()
            .iter()
            .filter(|o| o.round == round)
            .count(),
        standings,
    }
}

pub fn amount_sats(a: Amount) -> u64 {
    a.to_sat()
}
