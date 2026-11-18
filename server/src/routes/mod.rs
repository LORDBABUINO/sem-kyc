pub mod admin;
pub mod player;
pub mod public;

use axum::routing::{get, post};
use axum::Router;

use crate::state::Shared;
use crate::ws;

pub fn api() -> Router<Shared> {
    Router::new()
        .route("/api/join", post(player::join))
        .route("/api/me", get(player::me))
        .route("/api/register", post(player::register))
        .route("/api/settings", post(player::settings))
        .route("/api/coins/freeze", post(player::freeze))
        .route("/api/invoices", post(player::create_invoice))
        .route("/api/invoices/{code}", get(player::get_invoice))
        .route("/api/invoices/{code}/pay", post(player::pay_invoice))
        .route(
            "/api/invoices/{code}/payjoin",
            post(player::respond_payjoin),
        )
        .route("/api/lightning", post(player::lightning))
        .route("/api/coinjoin", post(player::coinjoin))
        .route("/api/taproot", post(player::taproot))
        .route("/api/storefront/silent", post(player::silent_storefront))
        .route("/api/autos/{id}/pay", post(player::pay_auto))
        .route("/api/report", get(player::report))
        .route("/api/state", get(public::state))
        .route("/api/clusters", get(public::clusters))
        .route("/api/admin/advance", post(admin::advance))
        .route("/api/admin/open", post(admin::open))
        .route("/api/admin/close", post(admin::close))
        .route("/api/admin/dust", post(admin::dust))
        .route("/api/admin/kyc", post(admin::kyc))
        .route("/api/admin/mine", post(admin::mine))
        .route("/api/admin/autos/{id}/forgive", post(admin::forgive))
        .route("/api/admin/bots", post(admin::add_bots))
        .route("/api/admin/reset", post(admin::reset))
        .route("/api/admin/config", post(admin::config))
        .route("/api/admin/room", get(admin::room))
        .route("/ws", get(ws::player_socket))
        .route("/ws/projector", get(ws::projector_socket))
        .route("/ws/admin", get(ws::admin_socket))
}
