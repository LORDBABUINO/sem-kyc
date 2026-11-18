//! HTTP and WebSocket transport for O Rastro.

pub mod bots;
pub mod error;
pub mod routes;
pub mod state;
pub mod views;
pub mod ws;

use std::path::PathBuf;
use std::time::Duration;

use axum::Router;
use tower_http::services::{ServeDir, ServeFile};

use crate::state::{now, Shared};

/// The API plus the built frontend, every unknown path falling back to the
/// app's index so the three screens can route by pathname.
pub fn app(state: Shared, static_dir: Option<PathBuf>) -> Router {
    let api = routes::api().with_state(state);
    match static_dir {
        Some(dir) => {
            let index = dir.join("index.html");
            api.fallback_service(ServeDir::new(dir).fallback(ServeFile::new(index)))
        }
        None => api,
    }
}

/// Blocks every `block_secs`, the CoinJoin window every second, bots every
/// two seconds.
pub fn spawn_clock(state: Shared, block_secs: u64) {
    let blocks = state.clone();
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(block_secs.max(1)));
        tick.tick().await;
        loop {
            tick.tick().await;
            let _ = blocks.with_game(|g| Ok(g.mine_block()));
        }
    });
    let rooms = state.clone();
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(1));
        loop {
            tick.tick().await;
            let _ = rooms.with_game(|g| {
                g.coinjoin_tick(now());
                Ok(())
            });
        }
    });
    tokio::spawn(bots::run(state));
}
