use crate::observer::{capture::capture_packets, scanner::arp_scan};
use crate::transfer::message::send_event;
use crate::transfer::types::{PeerEvent, PeerInfo, PeerPayload};
use crate::{
    helper::{change_flag, find_pcap_interface, find_pnet_interface, get_interface_ipv4},
    observer::types::ObserverEvent,
};
use crate::{
    observer::types::{
        ArpPacket, HostEntry, ObserverStore, SessionKey, SessionStats, TransportPacket,
    },
    transfer::types::TransferStore,
};
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    {Json, extract::State},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    net::Ipv4Addr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub observer: ObserverStore,
    pub transfer: TransferStore,
    pub channels: Channels,
    pub flags: Flags,
    pub identity: Identity,
    pub http: reqwest::Client,
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

        send_event(self.clone(), peer.ip, peer.port, &event).await;
        StatusCode::OK
    }
}

#[derive(Clone)]
pub struct Identity {
    pub name: String,
    pub ip: Ipv4Addr,
    pub port: u16,
}

impl Identity {
    pub fn as_peer(&self) -> PeerInfo {
        PeerInfo {
            name: self.name.clone(),
            ip: self.ip,
            port: self.port,
        }
    }
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

#[derive(Clone)]
pub struct Channels {
    pub observer_tx: tokio::sync::mpsc::Sender<ObserverEvent>,
    pub api_tx: broadcast::Sender<ApiEvent>,
}

#[derive(Clone)]
pub struct Flags {
    pub capture: Arc<AtomicBool>,
    pub scan: Arc<AtomicBool>,
}

#[derive(Deserialize)]
pub struct CaptureRequest {
    pub interface: String,
    pub filter: String,
}

#[derive(Deserialize)]
pub struct ScanRequest {
    pub interface: String,
    pub host_limit: u32,
}

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    let rx = state.channels.api_tx.subscribe();
    ws.on_upgrade(move |socket| handle_socket(socket, rx))
}

async fn handle_socket(mut socket: WebSocket, mut rx: broadcast::Receiver<ApiEvent>) {
    loop {
        match rx.recv().await {
            Ok(event) => {
                let json = match serde_json::to_string(&event) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                if socket.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                tracing::error!(skipped = %n, "ws client lagged");
            }
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }
}

pub async fn start_capture(
    State(state): State<AppState>,
    Json(req): Json<CaptureRequest>,
) -> Result<StatusCode, StatusCode> {
    let running = state.flags.capture.clone();

    let packet_tx = state.channels.observer_tx.clone();
    let device = find_pcap_interface(&req.interface).map_err(|_| StatusCode::BAD_REQUEST)?;

    change_flag(&running)?;

    tokio::task::spawn_blocking(move || {
        if let Err(e) = capture_packets(device, req.filter.as_str(), packet_tx, running.clone()) {
            tracing::error!(error = %e, "capture failed");
            running.store(false, Ordering::Relaxed);
        }
    });

    Ok(StatusCode::OK)
}

pub async fn start_scan(
    State(state): State<AppState>,
    Json(req): Json<ScanRequest>,
) -> Result<StatusCode, StatusCode> {
    let running = state.flags.scan.clone();

    let interface = find_pnet_interface(&req.interface).ok_or(StatusCode::BAD_REQUEST)?;
    let host_limit = req.host_limit;
    let sender_mac = interface.mac.ok_or(StatusCode::BAD_REQUEST)?;
    let sender_ip = get_interface_ipv4(&interface).ok_or(StatusCode::BAD_REQUEST)?;

    change_flag(&running)?;

    tokio::task::spawn_blocking(move || {
        arp_scan(interface, sender_mac, sender_ip, running, host_limit).ok();
    });

    Ok(StatusCode::OK)
}

pub async fn stop_capture(State(state): State<AppState>) {
    state.flags.capture.store(false, Ordering::Relaxed);
}

pub async fn stop_scan(State(state): State<AppState>) {
    state.flags.scan.store(false, Ordering::Relaxed);
}

pub async fn get_packets(State(state): State<AppState>) -> Json<VecDeque<ObserverEvent>> {
    let buf = state.observer.events.read().await;
    Json(buf.clone())
}

pub async fn get_hosts(State(state): State<AppState>) -> Json<Vec<HostEntry>> {
    let table = state.observer.hosts.read().await;
    Json(table.values().cloned().collect())
}

pub async fn get_sessions(State(state): State<AppState>) -> Json<Vec<(SessionKey, SessionStats)>> {
    let map = state.observer.sessions.read().await;
    let mut sessions: Vec<_> = map.iter().map(|(k, v)| (*k, *v)).collect();
    sessions.sort_by_key(|(_, v)| std::cmp::Reverse(v.bytes_total));
    Json(sessions)
}

pub async fn get_peers(State(state): State<AppState>) -> Json<Vec<PeerInfo>> {
    let map = state.transfer.peers.read().await;
    Json(map.values().cloned().collect())
}

pub async fn get_messages(State(state): State<AppState>) -> Json<Vec<(PeerInfo, Vec<PeerEvent>)>> {
    let map = state.transfer.messages.read().await;
    Json(map.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
}
