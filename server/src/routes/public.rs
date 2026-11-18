use axum::extract::State;
use axum::Json;
use rastro_core::view::ClusterSnapshot;

use crate::state::Shared;
use crate::views::{self, PublicState};

pub async fn state(State(state): State<Shared>) -> Json<PublicState> {
    Json(views::public_state(&state.game(), state.join_url()))
}

pub async fn clusters(State(state): State<Shared>) -> Json<ClusterSnapshot> {
    Json(state.game().cluster_snapshot())
}
