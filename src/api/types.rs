use crate::observer::types::{Flags, ObserverEvent};
use crate::transfer::message::send_event;
use crate::transfer::types::{Identity, PeerEvent, PeerInfo, PeerPayload, TransferError};
use crate::{
    observer::types::{ArpPacket, ObserverStore, TransportPacket},
    transfer::types::TransferStore,
};
use axum::http::StatusCode;
use reqwest::Client;
use serde::Serialize;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub observer: ObserverStore,
    pub transfer: TransferStore,
    pub channels: Channels,
    pub identity: Identity,
    pub flags: Flags,
    pub http: Client,
}

impl AppState {
    pub async fn find_peer_by_name(&self, name: &str) -> Option<PeerInfo> {
        let peers = self.transfer.peers.read().await;
        peers.values().find(|p| p.name == name).cloned()
    }

    pub async fn send_to(&self, peer: &PeerInfo, payload: PeerPayload) -> StatusCode {
        let event = PeerEvent {
            from: self.identity.as_peer(),
            payload,
        };

        if let Err(error) = send_event(&self.http, peer.ip, peer.port, &event).await {
            return TransferError::SendFail(error).into();
        }
        StatusCode::OK
    }
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
