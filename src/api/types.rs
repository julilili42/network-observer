use crate::observer::types::{Flags, ObserverEvent};
use crate::transfer::types::{Identity, PeerEvent, TransferError};
use crate::{
    observer::types::{ArpPacket, ObserverStore, TransportPacket},
    transfer::types::TransferStore,
};
use axum::http::StatusCode;
use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;
use tokio::sync::broadcast;
use uuid::Uuid;

#[derive(Clone)]
pub struct ObserverState {
    pub store: ObserverStore,
    pub channels: Channels,
    pub flags: Flags,
}

#[derive(Clone)]
pub struct TransferState {
    pub store: TransferStore,
    pub api_tx: broadcast::Sender<ApiEvent>,
    pub identity: Identity,
    pub http: Client,
}

#[derive(Clone)]
pub struct Channels {
    pub observer_tx: tokio::sync::mpsc::Sender<ObserverEvent>,
    pub api_tx: broadcast::Sender<ApiEvent>,
}

#[derive(Clone, Serialize)]
pub enum ApiEvent {
    Transport(TransportPacket),
    Arp(ArpPacket),
    Peer(PeerEvent),
}

impl From<ObserverEvent> for ApiEvent {
    fn from(event: ObserverEvent) -> Self {
        match event {
            ObserverEvent::Arp(packet) => Self::Arp(packet),
            ObserverEvent::Transport(packet) => Self::Transport(packet),
        }
    }
}
impl From<PeerEvent> for ApiEvent {
    fn from(event: PeerEvent) -> Self {
        ApiEvent::Peer(event)
    }
}

impl From<TransferError> for StatusCode {
    fn from(error: TransferError) -> Self {
        match error {
            TransferError::NoPending | TransferError::PeerNotFound => StatusCode::NOT_FOUND,
            TransferError::SendFail(error) => {
                tracing::error!(%error, "transfer failed");
                StatusCode::BAD_GATEWAY
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct OutgoingFileOffer {
    pub recipient_name: String,
    pub file_name: String,
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct AcceptRejectRequest {
    pub transfer_id: Uuid,
    pub from_name: String,
}

#[derive(Deserialize)]
pub struct SendMessageRequest {
    pub name: String,
    pub content: String,
}
