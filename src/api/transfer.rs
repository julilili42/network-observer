use crate::api::types::{AcceptRejectRequest, SendMessageRequest};
use crate::transfer::file::{accept_transfer, reject_transfer};
use crate::transfer::message::send_message;
use crate::{
    api::types::TransferState,
    transfer::{
        processing::handle_peer_event,
        types::{PeerEvent, PeerInfo},
    },
};
use axum::{Json, extract::State};
use reqwest::StatusCode;

pub async fn get_peers(State(state): State<TransferState>) -> Json<Vec<PeerInfo>> {
    let map = state.store.peers.read().await;
    Json(map.values().cloned().collect())
}

pub async fn get_messages(
    State(state): State<TransferState>,
) -> Json<Vec<(PeerInfo, Vec<PeerEvent>)>> {
    let map = state.store.messages.read().await;
    Json(map.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
}

pub async fn handle_outgoing_file_accept(
    State(state): State<TransferState>,
    Json(req): Json<AcceptRejectRequest>,
) -> StatusCode {
    match accept_transfer(req.transfer_id, &state.store, &state.identity, &state.http).await {
        Ok(_) => StatusCode::OK,
        Err(e) => e.into(),
    }
}

pub async fn handle_outgoing_file_reject(
    State(state): State<TransferState>,
    Json(req): Json<AcceptRejectRequest>,
) -> StatusCode {
    match reject_transfer(req.transfer_id, &state.store, &state.identity, &state.http).await {
        Ok(_) => StatusCode::OK,
        Err(e) => e.into(),
    }
}

pub async fn handle_outgoing_message(
    State(state): State<TransferState>,
    Json(req): Json<SendMessageRequest>,
) -> StatusCode {
    match send_message(
        &state.identity,
        &state.store,
        &state.http,
        &req.name,
        &req.content,
    )
    .await
    {
        Ok(_) => StatusCode::OK,
        Err(e) => e.into(),
    }
}

pub async fn handle_incoming(
    State(state): State<TransferState>,
    Json(event): Json<PeerEvent>,
) -> StatusCode {
    tracing::debug!(sender = %event.from, "Incoming peer event");

    if let Err(e) = handle_peer_event(&state.store, &event, &state.http, state.identity).await {
        return e.into();
    }

    // send message to event_processing -> sends it to ws
    let _ = state.api_tx.send(event.into());

    StatusCode::OK
}
