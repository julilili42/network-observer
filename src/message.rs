use std::net::Ipv4Addr;

use axum::{Json, extract::State};
use reqwest::StatusCode;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    api::AppState,
    types::{CapturedEvent, FilePayload, MessagePayload, PeerEvent, PeerInfo, PeerPayload},
};

#[derive(Deserialize)]
pub struct SendMessageRequest {
    pub name: String,
    pub content: String,
}

pub async fn handle_outgoing_message(
    State(state): State<AppState>,
    Json(req): Json<SendMessageRequest>,
) -> StatusCode {
    let state_clone = state.clone();

    // lookup recipient info from name
    let recipient: Option<PeerInfo> = {
        let peers = state.store.peers.read().await;
        peers.values().find(|p| p.name == req.name).cloned()
    };

    let Some(recipient) = recipient else {
        return StatusCode::NOT_FOUND;
    };

    let sender = PeerInfo {
        name: state.identity.name,
        ip: state.identity.ip,
        port: state.identity.port,
    };

    // save outgoing message event
    let mut messages = state.store.messages.write().await;
    let from = sender.clone();
    let payload_store = PeerPayload::Message(MessagePayload {
        content: req.content.clone(),
        outgoing: true,
    });

    messages
        .entry(recipient.clone())
        .or_default()
        .push(PeerEvent {
            from: from.clone(),
            payload: payload_store,
        });

    // send message to peer
    // false from receiver perspective
    let payload_send = PeerPayload::Message(MessagePayload {
        content: req.content,
        outgoing: false,
    });

    let event = &PeerEvent {
        from,
        payload: payload_send,
    };

    send_event(state_clone, recipient.ip, recipient.port, event).await;

    StatusCode::OK
}

pub async fn send_event(state: AppState, ip: Ipv4Addr, port: u16, event: &PeerEvent) {
    let res = state
        .http
        .post(format!("https://{}:{}/peers/incoming", ip, port))
        .json(event)
        .send()
        .await;

    if let Err(e) = res {
        tracing::error!(error = %e, "event send failed");
        return;
    }
}

pub async fn handle_incoming(
    State(state): State<AppState>,
    Json(event): Json<PeerEvent>,
) -> StatusCode {
    tracing::debug!(sender = %event.from, "Incoming peer event");

    // side effect: peer accepted file offer -> send file
    if let PeerPayload::File(FilePayload::Accept { transfer_id }) = event.payload {
        let transfer_id = transfer_id.clone();
        let recipient = event.from.clone();
        let state_clone = state.clone();

        tokio::spawn(async move { send_pending_file(state_clone, transfer_id, recipient).await });
    }

    // send message to event_processing -> sends it to ws
    let _ = state
        .channels
        .internal_tx
        .send(CapturedEvent::Peer(event))
        .await;

    StatusCode::OK
}

async fn send_pending_file(state: AppState, transfer_id: Uuid, recipient: PeerInfo) {
    let state_clone = state.clone();

    let transfer = {
        let mut transfers = state.store.pending_transfer.write().await;
        transfers.remove(&transfer_id)
    };

    let Some(transfer) = transfer else {
        tracing::warn!(transfer_id=%transfer_id, "No pending transfer found");
        return;
    };

    let sender = PeerInfo {
        name: state.identity.name,
        ip: state.identity.ip,
        port: state.identity.port,
    };

    let event = PeerEvent {
        from: sender,
        payload: PeerPayload::File(FilePayload::Data {
            transfer_id: transfer_id.clone(),
            filename: transfer.filename,
            data: transfer.data,
        }),
    };

    send_event(state_clone, recipient.ip, recipient.port, &event).await;

    state
        .store
        .pending_transfer
        .write()
        .await
        .remove(&transfer_id);

    tracing::info!(transfer_id=%transfer_id, "file sent successfully");
}
