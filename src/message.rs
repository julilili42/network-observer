use std::time::SystemTime;

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
}

#[derive(Deserialize)]
pub struct SendRequest {
    pub ip: String,
    pub port: u16,
    pub content: String,
}

pub async fn handle_outgoing_message(
    State(state): State<AppState>,
    Json(req): Json<SendRequest>,
) -> StatusCode {
    let sender_info = PeerInfo {
        name: state.identity.name,
        ip: state.identity.ip,
        port: state.identity.port,
        last_seen: SystemTime::now(),
    };
    send_message(sender_info, req.ip, req.port, req.content).await;

    StatusCode::OK
}

pub async fn send_message(sender: PeerInfo, ip: String, port: u16, content: String) {
    let client = reqwest::Client::new();

    let res = client
        .post(format!("http://{}:{}/peers/incoming_message", ip, port))
        .json(&PeerMessage {
            from: sender,
            content,
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
