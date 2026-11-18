use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::http::HeaderMap;
use fiscal_core::game::{Config, Game, GameError};
use fiscal_core::ledger::PlayerId;
use rand::RngCore;
use tokio::sync::broadcast;

use crate::error::ApiError;
use crate::ws::{fan_out, Outgoing};

#[derive(Debug)]
pub struct AppState {
    game: Mutex<Game>,
    sessions: Mutex<HashMap<String, PlayerId>>,
    pub admin_token: String,
    pub public_url: String,
    pub events: broadcast::Sender<Arc<Outgoing>>,
    pub bots: Mutex<Vec<PlayerId>>,
}

pub type Shared = Arc<AppState>;

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

impl AppState {
    pub fn new(
        config: Config,
        admin_token: String,
        public_url: String,
    ) -> Result<Shared, GameError> {
        let (events, _) = broadcast::channel(4096);
        Ok(Arc::new(Self {
            game: Mutex::new(Game::new(config)?),
            sessions: Mutex::new(HashMap::new()),
            admin_token,
            public_url,
            events,
            bots: Mutex::new(Vec::new()),
        }))
    }

    pub fn join_url(&self) -> String {
        format!("{}/", self.public_url.trim_end_matches('/'))
    }

    pub fn game(&self) -> MutexGuard<'_, Game> {
        self.game.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// Runs one mutation under the lock and broadcasts what it produced.
    pub fn with_game<R>(
        &self,
        f: impl FnOnce(&mut Game) -> Result<R, GameError>,
    ) -> Result<R, ApiError> {
        let mut game = self.game();
        let result = f(&mut game);
        let events = game.drain_events();
        for out in fan_out(events, &game) {
            // No receivers is fine: nobody is watching yet.
            let _ = self.events.send(Arc::new(out));
        }
        result.map_err(ApiError::from)
    }

    pub fn new_session(&self, player: PlayerId) -> String {
        let mut bytes = [0u8; 24];
        rand::thread_rng().fill_bytes(&mut bytes);
        let token: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        self.sessions
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(token.clone(), player);
        token
    }

    pub fn player_of(&self, token: &str) -> Option<PlayerId> {
        self.sessions
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get(token)
            .copied()
    }

    pub fn session(&self, headers: &HeaderMap) -> Result<PlayerId, ApiError> {
        headers
            .get("x-session")
            .and_then(|v| v.to_str().ok())
            .and_then(|t| self.player_of(t))
            .ok_or_else(|| ApiError::unauthorized("entre no jogo primeiro"))
    }

    pub fn admin(&self, headers: &HeaderMap) -> Result<(), ApiError> {
        let ok = headers
            .get("x-admin")
            .and_then(|v| v.to_str().ok())
            .is_some_and(|t| t == self.admin_token);
        if ok {
            Ok(())
        } else {
            Err(ApiError::unauthorized("token do apresentador inválido"))
        }
    }

    pub fn is_admin_token(&self, token: &str) -> bool {
        token == self.admin_token
    }

    pub fn reset(&self, config: Config) -> Result<(), GameError> {
        let fresh = Game::new(config)?;
        *self.game() = fresh;
        self.sessions
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clear();
        self.bots.lock().unwrap_or_else(|p| p.into_inner()).clear();
        Ok(())
    }
}
