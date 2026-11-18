use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::Json;
use fiscal_core::game::{Config, Phase};
use fiscal_core::ledger::Owner;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::bots;
use crate::error::ApiError;
use crate::state::{now, Shared};
use crate::views::{self, PublicState};

pub async fn advance(
    State(state): State<Shared>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    state.admin(&headers)?;
    let phase = state.with_game(|g| g.advance())?;
    bots::on_newspaper(&state);
    Ok(Json(json!({ "phase": phase })))
}

pub async fn open(
    State(state): State<Shared>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    state.admin(&headers)?;
    let phase = state.with_game(|g| g.open_negotiation(now()))?;
    Ok(Json(json!({ "phase": phase })))
}

pub async fn close(
    State(state): State<Shared>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    state.admin(&headers)?;
    let phase = state.with_game(|g| g.close_round(now()))?;
    let reveal = match phase {
        Phase::Reveal { round } => Some(state.game().reveal(round)),
        _ => None,
    };
    Ok(Json(json!({ "phase": phase, "reveal": reveal })))
}

pub async fn dust(
    State(state): State<Shared>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    state.admin(&headers)?;
    let txid = state.with_game(|g| g.dust_attack())?;
    Ok(Json(json!({ "txid": txid })))
}

pub async fn kyc(State(state): State<Shared>, headers: HeaderMap) -> Result<Json<Value>, ApiError> {
    state.admin(&headers)?;
    let players = state.with_game(|g| Ok(g.apply_kyc()))?;
    Ok(Json(json!({ "players": players })))
}

pub async fn mine(
    State(state): State<Shared>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    state.admin(&headers)?;
    let height = state.with_game(|g| Ok(g.mine_block()))?;
    Ok(Json(json!({ "height": height })))
}

pub async fn forgive(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(auto_id): Path<u32>,
) -> Result<Json<Value>, ApiError> {
    state.admin(&headers)?;
    state.with_game(|g| g.forgive_auto(auto_id))?;
    Ok(Json(json!({})))
}

#[derive(Debug, Deserialize)]
pub struct BotsBody {
    pub count: usize,
}

pub async fn add_bots(
    State(state): State<Shared>,
    headers: HeaderMap,
    Json(body): Json<BotsBody>,
) -> Result<Json<Value>, ApiError> {
    state.admin(&headers)?;
    let players = bots::add(&state, body.count)?;
    Ok(Json(json!({ "players": players })))
}

pub async fn reset(
    State(state): State<Shared>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    state.admin(&headers)?;
    let config = state.game().config.clone();
    state.reset(config)?;
    state.with_game(|_| Ok(()))?;
    Ok(Json(json!({})))
}

#[derive(Debug, Deserialize)]
pub struct ConfigBody {
    pub negotiation_secs: Option<u64>,
    pub icms_percent: Option<u64>,
    pub penalty_percent: Option<u64>,
    pub late_percent: Option<u64>,
    pub target_revenue: Option<u64>,
}

pub async fn config(
    State(state): State<Shared>,
    headers: HeaderMap,
    Json(body): Json<ConfigBody>,
) -> Result<Json<Config>, ApiError> {
    state.admin(&headers)?;
    let config = state.with_game(|g| {
        if let Some(v) = body.negotiation_secs {
            g.config.negotiation_secs = v;
        }
        if let Some(v) = body.icms_percent {
            g.config.icms_percent = v;
        }
        if let Some(v) = body.penalty_percent {
            g.config.penalty_percent = v;
        }
        if let Some(v) = body.late_percent {
            g.config.late_percent = v;
        }
        if let Some(v) = body.target_revenue {
            g.config.target_revenue = bitcoin::Amount::from_sat(v);
        }
        Ok(g.config.clone())
    })?;
    Ok(Json(config))
}

#[derive(Debug, Serialize)]
pub struct RoomPlayer {
    pub id: u32,
    pub name: String,
    pub sector: views::SectorInfo,
    pub registered: bool,
    pub balance: u64,
    pub suspended: bool,
    pub mission_complete: bool,
    pub bot: bool,
}

#[derive(Debug, Serialize)]
pub struct Room {
    pub state: PublicState,
    pub players: Vec<RoomPlayer>,
    pub invoices_open: usize,
    pub coinjoin: Option<views::RoomView>,
    pub log: Vec<Value>,
    pub config: Config,
}

pub async fn room(State(state): State<Shared>, headers: HeaderMap) -> Result<Json<Room>, ApiError> {
    state.admin(&headers)?;
    let game = state.game();
    let bots = state.bots.lock().unwrap_or_else(|p| p.into_inner()).clone();
    let round = game.round();
    let players = game
        .players()
        .map(|p| RoomPlayer {
            id: p.id,
            name: p.name.clone(),
            sector: p.sector.into(),
            registered: p.registered,
            balance: game.ledger.balance(Owner::Player(p.id)).to_sat(),
            suspended: game.is_suspended(p.id),
            mission_complete: p.missions.get(&round).is_some_and(|m| m.complete()),
            bot: bots.contains(&p.id),
        })
        .collect();
    let invoices_open = game
        .players()
        .flat_map(|p| game.invoices_of(p.id))
        .filter(|i| i.round == round && matches!(i.status, fiscal_core::game::InvoiceStatus::Open))
        .count();
    let log = game
        .operations()
        .iter()
        .rev()
        .take(30)
        .map(|o| {
            json!({
                "round": o.round,
                "txid": o.txid,
                "payer": game.player(o.payer).map(|p| p.name.clone()).unwrap_or_default(),
                "payee": game.player(o.payee).map(|p| p.name.clone()).unwrap_or_default(),
                "amount": o.amount.to_sat(),
                "item": game.invoices_paid_by_txid(o.txid).map(|i| i.item.name.clone()),
            })
        })
        .collect();
    Ok(Json(Room {
        state: views::public_state(&game, state.join_url()),
        players,
        invoices_open,
        coinjoin: game.coinjoin_room().map(|r| views::RoomView {
            entries: r.entries.len(),
            deadline: r.deadline,
        }),
        log,
        config: game.config.clone(),
    }))
}
