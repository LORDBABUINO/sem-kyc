use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use semkyc_core::game::Config;
use semkyc_server::state::{AppState, Shared};
use semkyc_server::ws::{fan_out, listens, Audience};
use serde_json::{json, Value};
use tower::ServiceExt;

const ADMIN: &str = "segredo";

fn setup() -> (Router, Shared) {
    let state = AppState::new(
        Config::default(),
        ADMIN.to_owned(),
        "http://semkyc.test".to_owned(),
    )
    .expect("state");
    (semkyc_server::app(state.clone(), None), state)
}

async fn call(
    app: &Router,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut req = Request::builder().method(method).uri(path);
    for (k, v) in headers {
        req = req.header(*k, *v);
    }
    let req = match body {
        Some(b) => req
            .header("content-type", "application/json")
            .body(Body::from(b.to_string())),
        None => req.body(Body::empty()),
    }
    .expect("request");
    let res = app.clone().oneshot(req).await.expect("response");
    let status = res.status();
    let bytes = res.into_body().collect().await.expect("body").to_bytes();
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes)
            .unwrap_or(Value::String(String::from_utf8_lossy(&bytes).into()))
    };
    (status, value)
}

async fn join(app: &Router, name: &str) -> String {
    let (status, body) = call(app, "POST", "/api/join", &[], Some(json!({"name": name}))).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body["session"].as_str().expect("session").to_owned()
}

async fn admin(app: &Router, action: &str) -> Value {
    let (status, body) = call(
        app,
        "POST",
        &format!("/api/admin/{action}"),
        &[("x-admin", ADMIN)],
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{action}: {body}");
    body
}

#[tokio::test]
async fn joining_registering_and_reading_me() {
    let (app, _) = setup();
    let (status, body) = call(&app, "GET", "/api/me", &[], None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"], "entre no jogo primeiro");

    let (status, body) = call(&app, "POST", "/api/join", &[], Some(json!({"name": "  "}))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "escolha um nome");

    let session = join(&app, "Padaria da Lu").await;
    let (status, me) = call(&app, "GET", "/api/me", &[("x-session", &session)], None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(me["player"]["name"], "Padaria da Lu");
    assert_eq!(me["player"]["sector"]["sector"], "padaria");
    assert_eq!(me["player"]["registered"], false);
    assert_eq!(me["balance"], 2_000_000);
    assert_eq!(me["phase"]["phase"], "lobby");
    assert!(me["coins"].as_array().expect("coins").len() == 20);
    assert!(me["coins"][0]["outpoint"]
        .as_str()
        .expect("op")
        .contains(':'));
    assert_eq!(me["report_available"], false);

    let (status, body) = call(
        &app,
        "POST",
        "/api/register",
        &[("x-session", &session)],
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body["txid"].is_string());
    let (_, me) = call(&app, "GET", "/api/me", &[("x-session", &session)], None).await;
    assert_eq!(me["player"]["registered"], true);
    assert_eq!(me["balance"], 1_990_000);
    assert_eq!(me["history"][0]["memo"], "Taxa de cadastro da vitrine");
    assert_eq!(me["history"][0]["counterparty"], "DOE");

    let (status, body) = call(
        &app,
        "POST",
        "/api/register",
        &[("x-session", &session)],
        None,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "a vitrine já está cadastrada");
}

#[tokio::test]
async fn a_full_round_through_the_api() {
    let (app, _) = setup();
    let mut sessions = Vec::new();
    for i in 1..=8 {
        let s = join(&app, &format!("Loja {i}")).await;
        call(&app, "POST", "/api/register", &[("x-session", &s)], None).await;
        sessions.push(s);
    }
    let (status, body) = call(&app, "POST", "/api/admin/advance", &[], None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"], "token do apresentador inválido");

    let body = admin(&app, "advance").await;
    assert_eq!(body["phase"]["phase"], "newspaper");
    let padaria = &sessions[0];
    let fazenda = &sessions[1];
    // Prices are known as soon as the newspaper is out.
    let (_, me) = call(&app, "GET", "/api/me", &[("x-session", padaria)], None).await;
    assert_eq!(me["mission"]["items"][0]["price"], 40_000);
    let (_, seller) = call(&app, "GET", "/api/me", &[("x-session", fazenda)], None).await;
    assert_eq!(seller["catalog"][0]["price"], 40_000);
    let (status, body) = call(
        &app,
        "POST",
        "/api/invoices",
        &[("x-session", fazenda)],
        Some(json!({"item": "farinha e ovos"})),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");

    let body = admin(&app, "open").await;
    assert_eq!(body["phase"]["phase"], "negotiation");
    let (_, me) = call(&app, "GET", "/api/me", &[("x-session", padaria)], None).await;
    assert_eq!(me["mission"]["round"], 1);
    assert_eq!(me["mission"]["items"][0]["item"]["name"], "farinha e ovos");
    assert_eq!(me["mission"]["items"][0]["price"], 40_000);
    assert!(me["mission"]["text"]
        .as_str()
        .expect("text")
        .contains("farinha"));
    assert!(me["player"]["unlocked"]
        .as_array()
        .expect("u")
        .iter()
        .any(|u| u == "fresh_address"));

    let (status, invoice) = call(
        &app,
        "POST",
        "/api/invoices",
        &[("x-session", fazenda)],
        Some(json!({"item": "farinha e ovos"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{invoice}");
    let code = invoice["code"].as_str().expect("code").to_owned();
    assert_eq!(invoice["amount"], 40_000);
    assert!(invoice["uri"]
        .as_str()
        .expect("uri")
        .starts_with("bitcoin:bc1q"));
    assert!(invoice["uri"]
        .as_str()
        .expect("uri")
        .contains("amount=0.0004&label=farinha%20e%20ovos"));
    assert!(invoice["uri"]
        .as_str()
        .expect("uri")
        .ends_with(&format!("&code={code}")));
    assert_eq!(invoice["seller_name"], "Loja 2");

    let (status, seen) = call(
        &app,
        "GET",
        &format!("/api/invoices/{code}"),
        &[("x-session", padaria)],
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(seen["status"]["status"], "open");
    assert!(seen["status"].get("buyer_name").is_none());

    let (status, paid) = call(
        &app,
        "POST",
        &format!("/api/invoices/{}/pay", code.to_lowercase()),
        &[("x-session", padaria)],
        Some(json!({})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{paid}");
    assert_eq!(paid["result"], "settled");

    let (status, again) = call(
        &app,
        "POST",
        &format!("/api/invoices/{code}/pay"),
        &[("x-session", padaria)],
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(again["error"], "esse pedido não está mais aberto");

    let (_, state) = call(&app, "GET", "/api/state", &[], None).await;
    assert_eq!(state["payments_this_round"], 1);
    assert_eq!(state["players"], 8);
    assert_eq!(state["join_url"], "http://semkyc.test/");
    let (_, clusters) = call(&app, "GET", "/api/clusters", &[], None).await;
    assert!(clusters["clusters"].as_array().expect("clusters").len() >= 8);
    assert!(clusters["edges"]
        .as_array()
        .expect("edges")
        .iter()
        .any(|e| e["amount"] == 40_000));

    let body = admin(&app, "close").await;
    assert_eq!(body["phase"]["phase"], "reveal");
    assert_eq!(body["reveal"]["round"], 1);
    assert_eq!(body["reveal"]["payments"], 1);
    let (_, me) = call(&app, "GET", "/api/me", &[("x-session", padaria)], None).await;
    assert_eq!(
        me["suspended"], true,
        "the Padaria paid from its storefront coins"
    );
    assert!(me["debt"].as_u64().expect("debt") > 0);
    let auto_id = me["autos"][0]["id"].as_u64().expect("auto id");

    let (status, body) = call(&app, "GET", "/api/report", &[("x-session", padaria)], None).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");

    admin(&app, "advance").await;
    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/autos/{auto_id}/pay"),
        &[("x-session", padaria)],
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (_, me) = call(&app, "GET", "/api/me", &[("x-session", padaria)], None).await;
    assert_eq!(me["suspended"], false);
    assert_eq!(
        me["history"][0]["memo"],
        format!("ICMS, Auto de Infração DOE nº {auto_id}")
    );

    let (status, room) = call(&app, "GET", "/api/admin/room", &[("x-admin", ADMIN)], None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(room["players"].as_array().expect("players").len(), 8);
    assert_eq!(room["log"][0]["name"], "Loja 1");
    assert_eq!(room["log"][0]["counterparty"], "Loja 2");
    assert_eq!(room["log"][0]["memo"], "farinha e ovos");
    assert_eq!(room["log"][0]["direction"], "out");
    let (_, seen) = call(
        &app,
        "GET",
        &format!("/api/invoices/{code}"),
        &[("x-session", padaria)],
        None,
    )
    .await;
    assert_eq!(seen["status"]["status"], "paid");
    assert_eq!(seen["status"]["buyer_name"], "Loja 1");
    assert!(seen["status"].get("inputs").is_none());
}

#[tokio::test]
async fn bots_fill_the_room_and_trade_among_themselves() {
    let (app, state) = setup();
    let (status, body) = call(
        &app,
        "POST",
        "/api/admin/bots",
        &[("x-admin", ADMIN)],
        Some(json!({"count": 8})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["players"].as_array().expect("players").len(), 8);
    admin(&app, "advance").await;
    admin(&app, "open").await;
    let made = semkyc_server::bots::step(&state);
    assert!(made >= 6, "{made} payments");
    let (_, st) = call(&app, "GET", "/api/state", &[], None).await;
    assert_eq!(st["payments_this_round"], made);
    let (_, room) = call(&app, "GET", "/api/admin/room", &[("x-admin", ADMIN)], None).await;
    assert!(room["players"]
        .as_array()
        .expect("p")
        .iter()
        .all(|p| p["bot"] == true));
    admin(&app, "close").await;
    let (_, st) = call(&app, "GET", "/api/state", &[], None).await;
    assert!(st["reveal"]["headline"]["finding"]["type"].is_string());
    assert!(!st["autos_open"].as_array().expect("autos").is_empty());
    // The newspaper settles the bots' debts.
    admin(&app, "advance").await;
    let (_, st) = call(&app, "GET", "/api/state", &[], None).await;
    assert!(st["autos_open"].as_array().expect("autos").is_empty());
}

#[tokio::test]
async fn config_and_reset_are_for_the_presenter() {
    let (app, _) = setup();
    let (status, cfg) = call(
        &app,
        "POST",
        "/api/admin/config",
        &[("x-admin", ADMIN)],
        Some(json!({"negotiation_secs": 240, "icms_percent": 20})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{cfg}");
    assert_eq!(cfg["negotiation_secs"], 240);
    assert_eq!(cfg["icms_percent"], 20);
    join(&app, "alguém").await;
    admin(&app, "reset").await;
    let (_, st) = call(&app, "GET", "/api/state", &[], None).await;
    assert_eq!(st["players"], 0);
}

#[test]
fn events_reach_their_audiences() {
    let mut game = semkyc_core::game::Game::new(Config::default()).expect("game");
    let a = game.join("A").expect("a");
    let b = game.join("B").expect("b");
    game.register(a).expect("ra");
    game.register(b).expect("rb");
    game.drain_events();
    game.advance().expect("advance");
    let events = game.drain_events();
    let out = fan_out(events, &game);
    let missions: Vec<_> = out
        .iter()
        .filter(|o| o.text.contains("\"mission\""))
        .collect();
    assert_eq!(missions.len(), 2);
    assert!(missions.iter().any(|o| o.audience == Audience::Player(a)));
    assert!(out
        .iter()
        .any(|o| o.audience == Audience::Everyone && o.text.contains("\"phase\"")));
    assert!(out
        .iter()
        .filter(|o| o.text.contains("\"unlock\""))
        .all(|o| matches!(o.audience, Audience::Player(_))));

    assert!(listens(Audience::Player(a), Audience::Player(a)));
    assert!(!listens(Audience::Player(a), Audience::Player(b)));
    assert!(listens(Audience::Player(a), Audience::Everyone));
    assert!(!listens(Audience::Player(a), Audience::Projector));
    assert!(listens(Audience::Admin, Audience::Projector));
    assert!(!listens(Audience::Projector, Audience::Admin));
}
