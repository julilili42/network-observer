use axum::{Json, extract::State};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    api::AppState,
    types::{CapturedEvent, PeerInfo},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PeerMessage {
    pub from: PeerInfo,
    pub content: String,
    pub outgoing: bool,
}

#[derive(Deserialize)]
pub struct SendRequest {
    pub name: String,
    pub content: String,
}

pub async fn handle_outgoing_message(
    State(state): State<AppState>,
    Json(req): Json<SendRequest>,
) -> StatusCode {
    // lookup recipient info from name
    let recipient_info: Option<PeerInfo> = {
        let peers = state.store.peers.read().await;
        peers.values().find(|p| p.name == req.name).cloned()
    };

    let Some(recipient_info) = recipient_info else {
        return StatusCode::NOT_FOUND;
    };

    let sender_info = PeerInfo {
        name: state.identity.name,
        ip: state.identity.ip,
        port: state.identity.port,
    };

    // save outgoing message
    let mut messages = state.store.messages.write().await;
    messages
        .entry(recipient_info.clone())
        .or_default()
        .push(PeerMessage {
            from: sender_info.clone(),
            content: req.content.clone(),
            outgoing: true,
        });

    send_message(state.http, sender_info, recipient_info, req.content).await;

    StatusCode::OK
}

pub async fn send_message(
    client: reqwest::Client,
    sender: PeerInfo,
    receiver: PeerInfo,
    content: String,
) {
    // outgoing false since receivers perspective is incoming
    // on wire not important if out- or incoming
    let res = client
        .post(format!(
            "https://{}:{}/peers/incoming_message",
            receiver.ip, receiver.port
        ))
        .json(&PeerMessage {
            from: sender,
            content,
            outgoing: false,
        })
        .send()
        .await;

    if let Err(e) = res {
        tracing::error!(error = %e, "send failed");
    }
}

pub async fn handle_incoming_message(
    State(state): State<AppState>,
    Json(msg): Json<PeerMessage>,
) -> Result<StatusCode, StatusCode> {
    tracing::debug!(content = %msg.content, "Peer message");

    // send message to event_processing -> sends it to ws
    let res = state
        .channels
        .internal_tx
        .send(CapturedEvent::IncomingMessage(msg))
        .await
        .map(|_| StatusCode::OK)
        .map_err(|_| StatusCode::BAD_REQUEST);
    res
}
