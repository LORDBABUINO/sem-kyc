//! Game events become WebSocket messages, each aimed at an audience.

use std::collections::HashMap;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::Response;
use futures_util::{SinkExt, StreamExt};
use semkyc_core::game::{Event, Game, Phase};
use semkyc_core::ledger::PlayerId;
use serde_json::{json, Value};

use crate::state::Shared;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Audience {
    Player(PlayerId),
    Projector,
    Admin,
    Everyone,
}

#[derive(Debug, Clone)]
pub struct Outgoing {
    pub audience: Audience,
    pub text: String,
}

fn to(audience: Audience, value: Value) -> Outgoing {
    Outgoing {
        audience,
        text: value.to_string(),
    }
}

fn name(game: &Game, id: PlayerId) -> String {
    game.player(id).map(|p| p.name.clone()).unwrap_or_default()
}

pub fn fan_out(events: Vec<Event>, game: &Game) -> Vec<Outgoing> {
    let mut out = Vec::new();
    for e in events {
        match e {
            Event::PlayerJoined { player } => {
                out.push(to(
                    Audience::Admin,
                    json!({"event": "player_joined", "player": player, "name": name(game, player)}),
                ));
                out.push(to(
                    Audience::Projector,
                    json!({"event": "players", "count": game.players().count()}),
                ));
            }
            Event::Phase { phase } => {
                out.push(to(
                    Audience::Everyone,
                    json!({"event": "phase", "phase": phase}),
                ));
                if let Phase::Newspaper { round } = phase {
                    for p in game.players() {
                        out.push(to(
                            Audience::Player(p.id),
                            json!({"event": "mission", "round": round}),
                        ));
                    }
                }
                if phase == Phase::Final {
                    for p in game.players() {
                        out.push(to(
                            Audience::Player(p.id),
                            json!({"event": "report_available"}),
                        ));
                    }
                }
            }
            Event::Tx {
                txid,
                players,
                clusters,
            } => {
                for p in players {
                    out.push(to(
                        Audience::Player(p),
                        json!({"event": "tx", "txid": txid}),
                    ));
                }
                out.push(to(
                    Audience::Projector,
                    json!({"event": "tx", "txid": txid, "clusters": clusters}),
                ));
                let round = game.round();
                let count = game
                    .operations()
                    .iter()
                    .filter(|o| o.round == round)
                    .count();
                out.push(to(
                    Audience::Projector,
                    json!({"event": "payments", "round": round, "count": count}),
                ));
            }
            Event::Kyc { players, clusters } => {
                out.push(to(
                    Audience::Projector,
                    json!({"event": "kyc", "players": players, "clusters": clusters}),
                ));
            }
            Event::Unlock { player, unlock } => {
                out.push(to(
                    Audience::Player(player),
                    json!({"event": "unlock", "unlock": unlock}),
                ));
            }
            Event::InvoicePaid {
                code,
                buyer,
                seller,
                txid,
            } => {
                for p in [buyer, seller] {
                    out.push(to(
                        Audience::Player(p),
                        json!({"event": "invoice_paid", "code": code, "txid": txid}),
                    ));
                }
                out.push(to(
                    Audience::Admin,
                    json!({"event": "invoice_paid", "code": code, "txid": txid,
                           "buyer": name(game, buyer), "seller": name(game, seller)}),
                ));
            }
            Event::PayjoinRequested {
                code,
                buyer,
                seller,
            } => {
                out.push(to(
                    Audience::Player(seller),
                    json!({"event": "payjoin_request", "code": code, "buyer": name(game, buyer)}),
                ));
            }
            Event::Bonus {
                player,
                txid,
                amount,
            } => {
                out.push(to(
                    Audience::Player(player),
                    json!({"event": "bonus", "txid": txid, "amount": amount.to_sat()}),
                ));
            }
            Event::AutoIssued {
                player,
                auto_id,
                amount,
            } => {
                out.push(to(
                    Audience::Player(player),
                    json!({"event": "auto", "auto_id": auto_id, "amount": amount.to_sat(), "paid": false}),
                ));
                out.push(to(
                    Audience::Projector,
                    json!({"event": "auto", "auto_id": auto_id, "player": player,
                           "name": name(game, player), "amount": amount.to_sat(), "paid": false}),
                ));
            }
            Event::AutoPaid {
                player,
                auto_id,
                txid,
            } => {
                let amount = game
                    .autos()
                    .iter()
                    .find(|a| a.id == auto_id)
                    .map(|a| a.amount.to_sat())
                    .unwrap_or(0);
                out.push(to(
                    Audience::Player(player),
                    json!({"event": "auto", "auto_id": auto_id, "amount": amount, "paid": true, "txid": txid}),
                ));
                out.push(to(
                    Audience::Projector,
                    json!({"event": "auto", "auto_id": auto_id, "player": player,
                           "name": name(game, player), "amount": amount, "paid": true}),
                ));
            }
            Event::CoinJoin { txid, players } => {
                for p in &players {
                    out.push(to(
                        Audience::Player(*p),
                        json!({"event": "coinjoin", "txid": txid}),
                    ));
                }
                out.push(to(
                    Audience::Projector,
                    json!({"event": "coinjoin", "txid": txid, "players": players}),
                ));
            }
            Event::Reveal { round } => {
                out.push(to(
                    Audience::Projector,
                    json!({"event": "reveal", "round": round}),
                ));
            }
            Event::Block { height } => {
                out.push(to(
                    Audience::Everyone,
                    json!({"event": "block", "height": height}),
                ));
            }
        }
    }
    out
}

/// Which audiences a socket of this kind listens to.
pub fn listens(kind: Audience, message: Audience) -> bool {
    match (kind, message) {
        (_, Audience::Everyone) => true,
        (Audience::Player(a), Audience::Player(b)) => a == b,
        (Audience::Projector, Audience::Projector) => true,
        (Audience::Admin, Audience::Admin | Audience::Projector) => true,
        _ => false,
    }
}

async fn pump(socket: WebSocket, state: Shared, kind: Audience) {
    let (mut sink, mut stream) = socket.split();
    let mut rx = state.events.subscribe();
    let phase = state.game().phase();
    let hello = json!({"event": "phase", "phase": phase}).to_string();
    if sink.send(Message::Text(hello.into())).await.is_err() {
        return;
    }
    loop {
        tokio::select! {
            incoming = stream.next() => {
                match incoming {
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                    Some(Ok(Message::Ping(data))) => {
                        if sink.send(Message::Pong(data)).await.is_err() { break; }
                    }
                    Some(Ok(_)) => {}
                }
            }
            outgoing = rx.recv() => {
                match outgoing {
                    Ok(msg) => {
                        if listens(kind, msg.audience)
                            && sink.send(Message::Text(msg.text.clone().into())).await.is_err()
                        {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(_) => break,
                }
            }
        }
    }
}

pub async fn player_socket(
    ws: WebSocketUpgrade,
    State(state): State<Shared>,
    Query(q): Query<HashMap<String, String>>,
) -> Result<Response, crate::error::ApiError> {
    let player = q
        .get("session")
        .and_then(|t| state.player_of(t))
        .ok_or_else(|| crate::error::ApiError::unauthorized("sessão inválida"))?;
    Ok(ws.on_upgrade(move |socket| pump(socket, state, Audience::Player(player))))
}

pub async fn projector_socket(ws: WebSocketUpgrade, State(state): State<Shared>) -> Response {
    ws.on_upgrade(move |socket| pump(socket, state, Audience::Projector))
}

pub async fn admin_socket(
    ws: WebSocketUpgrade,
    State(state): State<Shared>,
    Query(q): Query<HashMap<String, String>>,
) -> Result<Response, crate::error::ApiError> {
    if !q.get("token").is_some_and(|t| state.is_admin_token(t)) {
        return Err(crate::error::ApiError::unauthorized(
            "token do apresentador inválido",
        ));
    }
    Ok(ws.on_upgrade(move |socket| pump(socket, state, Audience::Admin)))
}
