use crate::api::types::{AcceptRejectRequest, OfferTransferRequest, SendMessageRequest};
use crate::transfer::file::{accept_transfer, offer_transfer, reject_transfer};
use crate::transfer::message::send_message;
use crate::transfer::types::{Message, Transfer};
use crate::{
    api::types::TransferState,
    transfer::{
        event::handle_peer_event,
        types::{PeerEvent, PeerInfo},
    },
};
use axum::{Json, extract::State, response::Html};
use reqwest::StatusCode;

pub async fn get_peers(State(state): State<TransferState>) -> Json<Vec<PeerInfo>> {
    let map = state.store.peers.read().await;
    Json(map.values().cloned().collect())
}

pub async fn get_test_ui() -> Html<&'static str> {
    Html(include_str!("test_ui.html"))
}

pub async fn get_transfers(State(state): State<TransferState>) -> Json<Vec<Transfer>> {
    Json(
        state
            .store
            .transfers
            .read()
            .await
            .values()
            .cloned()
            .collect(),
    )
}

pub async fn get_messages(
    State(state): State<TransferState>,
) -> Json<Vec<(PeerInfo, Vec<Message>)>> {
    let map = state.store.messages.read().await;
    Json(map.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
}

pub async fn handle_accept_transfer(
    State(state): State<TransferState>,
    Json(req): Json<AcceptRejectRequest>,
) -> StatusCode {
    match accept_transfer(req.transfer_id, &state.store, &state.identity, &state.http).await {
        Ok(_) => StatusCode::OK,
        Err(e) => e.into(),
    }
}

pub async fn handle_reject_transfer(
    State(state): State<TransferState>,
    Json(req): Json<AcceptRejectRequest>,
) -> StatusCode {
    match reject_transfer(req.transfer_id, &state.store, &state.identity, &state.http).await {
        Ok(_) => StatusCode::OK,
        Err(e) => e.into(),
    }
}

pub async fn handle_send_message(
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

pub async fn handle_peer_events(
    State(state): State<TransferState>,
    Json(event): Json<PeerEvent>,
) -> StatusCode {
    tracing::debug!(sender = %event.from, "Incoming peer event");

    if let Err(e) = handle_peer_event(&state.store, &event, &state.http, state.identity).await {
        return e.into();
    }

    let _ = state.api_tx.send(event.into());

    StatusCode::OK
}

pub async fn handle_offer_transfer(
    State(state): State<TransferState>,
    Json(req): Json<OfferTransferRequest>,
) -> StatusCode {
    let peer = {
        let peers = state.store.peers.read().await;
        peers
            .values()
            .find(|peer| peer.name == req.recipient_name)
            .cloned()
    };

    let Some(peer) = peer else {
        return StatusCode::NOT_FOUND;
    };

    match offer_transfer(peer, req.path, &state.store, &state.identity, &state.http).await {
        Ok(_) => StatusCode::OK,
        Err(e) => e.into(),
    }
}
