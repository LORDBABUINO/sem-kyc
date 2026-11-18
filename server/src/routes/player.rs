use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::Json;
use bitcoin::OutPoint;
use fiscal_core::economy::PriceStyle;
use fiscal_core::game::{CoinSelection, Payment, Phase, ReceiveMode};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::ApiError;
use crate::state::{now, Shared};
use crate::views::{self, Me};

#[derive(Debug, Deserialize)]
pub struct JoinBody {
    pub name: String,
}

pub async fn join(
    State(state): State<Shared>,
    Json(body): Json<JoinBody>,
) -> Result<Json<Value>, ApiError> {
    let id = state.with_game(|g| g.join(&body.name))?;
    let session = state.new_session(id);
    let me = views::me(&state.game(), id, &state.public_url)?;
    Ok(Json(json!({ "session": session, "me": me })))
}

pub async fn me(State(state): State<Shared>, headers: HeaderMap) -> Result<Json<Me>, ApiError> {
    let id = state.session(&headers)?;
    Ok(Json(views::me(&state.game(), id, &state.public_url)?))
}

pub async fn register(
    State(state): State<Shared>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    let id = state.session(&headers)?;
    let txid = state.with_game(|g| g.register(id))?;
    Ok(Json(json!({ "txid": txid })))
}

#[derive(Debug, Deserialize)]
pub struct SettingsBody {
    pub receive_mode: Option<ReceiveMode>,
    pub price_style: Option<PriceStyle>,
    pub coin_selection: Option<CoinSelection>,
}

pub async fn settings(
    State(state): State<Shared>,
    headers: HeaderMap,
    Json(body): Json<SettingsBody>,
) -> Result<Json<Me>, ApiError> {
    let id = state.session(&headers)?;
    state.with_game(|g| {
        if let Some(m) = body.receive_mode {
            g.set_receive_mode(id, m)?;
        }
        if let Some(s) = body.price_style {
            g.set_price_style(id, s)?;
        }
        if let Some(c) = body.coin_selection {
            g.set_coin_selection(id, c)?;
        }
        Ok(())
    })?;
    Ok(Json(views::me(&state.game(), id, &state.public_url)?))
}

#[derive(Debug, Deserialize)]
pub struct FreezeBody {
    pub outpoint: String,
    pub frozen: bool,
}

pub async fn freeze(
    State(state): State<Shared>,
    headers: HeaderMap,
    Json(body): Json<FreezeBody>,
) -> Result<Json<Me>, ApiError> {
    let id = state.session(&headers)?;
    let op = views::parse_outpoint(&body.outpoint)?;
    state.with_game(|g| g.set_frozen(id, op, body.frozen))?;
    Ok(Json(views::me(&state.game(), id, &state.public_url)?))
}

#[derive(Debug, Deserialize)]
pub struct InvoiceBody {
    pub item: String,
}

/// The negotiation timer is enforced here: the game only knows phases.
fn open_for_trade(state: &Shared) -> Result<(), ApiError> {
    match state.game().phase() {
        Phase::Negotiation { deadline, .. } if now() <= deadline => Ok(()),
        Phase::Negotiation { .. } => Err(ApiError::forbidden("a rodada fechou")),
        _ => Err(ApiError::forbidden("isso não é possível nesta fase")),
    }
}

pub async fn create_invoice(
    State(state): State<Shared>,
    headers: HeaderMap,
    Json(body): Json<InvoiceBody>,
) -> Result<Json<views::InvoiceView>, ApiError> {
    let id = state.session(&headers)?;
    open_for_trade(&state)?;
    let invoice = state.with_game(|g| g.create_invoice(id, &body.item, now()))?;
    Ok(Json(views::invoice_view(
        &state.game(),
        &invoice,
        &state.public_url,
    )))
}

pub async fn get_invoice(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(code): Path<String>,
) -> Result<Json<views::InvoiceView>, ApiError> {
    state.session(&headers)?;
    let game = state.game();
    let invoice = game
        .invoice(&code)
        .ok_or_else(|| ApiError::not_found("pedido não encontrado"))?;
    Ok(Json(views::invoice_view(&game, invoice, &state.public_url)))
}

#[derive(Debug, Default, Deserialize)]
pub struct PayBody {
    pub inputs: Option<Vec<String>>,
    pub payjoin: Option<bool>,
}

fn parse_inputs(list: &Option<Vec<String>>) -> Result<Option<Vec<OutPoint>>, ApiError> {
    match list {
        None => Ok(None),
        Some(items) => items
            .iter()
            .map(|s| views::parse_outpoint(s))
            .collect::<Result<_, _>>()
            .map(Some),
    }
}

pub async fn pay_invoice(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(code): Path<String>,
    body: Option<Json<PayBody>>,
) -> Result<Json<Value>, ApiError> {
    let id = state.session(&headers)?;
    open_for_trade(&state)?;
    let body = body.map(|b| b.0).unwrap_or_default();
    let inputs = parse_inputs(&body.inputs)?;
    let result = state.with_game(|g| {
        g.pay_invoice(id, &code, inputs.as_deref(), body.payjoin.unwrap_or(false))
    })?;
    Ok(Json(match result {
        Payment::Settled { txid } => json!({ "result": "settled", "txid": txid }),
        Payment::AwaitingSeller => json!({ "result": "awaiting_seller" }),
    }))
}

#[derive(Debug, Default, Deserialize)]
pub struct PayjoinBody {
    pub contribute: Option<String>,
}

pub async fn respond_payjoin(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(code): Path<String>,
    body: Option<Json<PayjoinBody>>,
) -> Result<Json<Value>, ApiError> {
    let id = state.session(&headers)?;
    open_for_trade(&state)?;
    let body = body.map(|b| b.0).unwrap_or_default();
    let contribute = body
        .contribute
        .as_deref()
        .map(views::parse_outpoint)
        .transpose()?;
    let txid = state.with_game(|g| g.respond_payjoin(id, &code, contribute))?;
    Ok(Json(json!({ "txid": txid })))
}

#[derive(Debug, Deserialize)]
pub struct CoinBody {
    pub outpoint: String,
}

pub async fn lightning(
    State(state): State<Shared>,
    headers: HeaderMap,
    Json(body): Json<CoinBody>,
) -> Result<Json<Value>, ApiError> {
    let id = state.session(&headers)?;
    let op = views::parse_outpoint(&body.outpoint)?;
    let (out, inbound) = state.with_game(|g| g.withdraw_via_lightning(id, op))?;
    Ok(Json(json!({ "out": out, "inbound": inbound })))
}

pub async fn coinjoin(
    State(state): State<Shared>,
    headers: HeaderMap,
    Json(body): Json<CoinBody>,
) -> Result<Json<Value>, ApiError> {
    let id = state.session(&headers)?;
    let op = views::parse_outpoint(&body.outpoint)?;
    let (entries, deadline) = state.with_game(|g| {
        let entries = g.join_coinjoin(id, op, now())?;
        let deadline = g.coinjoin_room().map(|r| r.deadline).unwrap_or(0);
        Ok((entries, deadline))
    })?;
    Ok(Json(json!({ "entries": entries, "deadline": deadline })))
}

#[derive(Debug, Deserialize)]
pub struct TaprootBody {
    pub outpoints: Vec<String>,
    #[serde(default)]
    pub together: bool,
}

pub async fn taproot(
    State(state): State<Shared>,
    headers: HeaderMap,
    Json(body): Json<TaprootBody>,
) -> Result<Json<Value>, ApiError> {
    let id = state.session(&headers)?;
    let ops: Vec<OutPoint> = body
        .outpoints
        .iter()
        .map(|s| views::parse_outpoint(s))
        .collect::<Result<_, _>>()?;
    let txids = state.with_game(|g| g.migrate_taproot(id, &ops, body.together))?;
    Ok(Json(json!({ "txids": txids })))
}

pub async fn silent_storefront(
    State(state): State<Shared>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    let id = state.session(&headers)?;
    let sp = state.with_game(|g| g.register_silent_storefront(id))?;
    Ok(Json(json!({ "sp": sp })))
}

pub async fn pay_auto(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(auto_id): Path<u32>,
    body: Option<Json<PayBody>>,
) -> Result<Json<Value>, ApiError> {
    let id = state.session(&headers)?;
    let body = body.map(|b| b.0).unwrap_or_default();
    let inputs = parse_inputs(&body.inputs)?;
    let txid = state.with_game(|g| g.pay_auto(id, auto_id, inputs.as_deref()))?;
    Ok(Json(json!({ "txid": txid })))
}

pub async fn report(
    State(state): State<Shared>,
    headers: HeaderMap,
) -> Result<Json<stealth_engine::types::Report>, ApiError> {
    let id = state.session(&headers)?;
    let game = state.game();
    if game.phase() != Phase::Final {
        return Err(ApiError::forbidden(
            "o relatório é liberado no fim da palestra",
        ));
    }
    Ok(Json(game.report(id)))
}
