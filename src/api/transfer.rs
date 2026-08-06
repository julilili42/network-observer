use crate::{
    api::AppState,
    transfer::types::{PeerEvent, PeerInfo},
};
use axum::{Json, extract::State};

pub async fn get_peers(State(state): State<AppState>) -> Json<Vec<PeerInfo>> {
    let map = state.transfer.peers.read().await;
    Json(map.values().cloned().collect())
}

pub async fn get_messages(State(state): State<AppState>) -> Json<Vec<(PeerInfo, Vec<PeerEvent>)>> {
    let map = state.transfer.messages.read().await;
    Json(map.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
}
