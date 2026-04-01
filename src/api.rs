use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, extract::State};
use serde::Deserialize;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::{RwLock, broadcast};

use crate::capture::capture_packets;
use crate::discovery::discovery_sender;
use crate::helper::{change_flag, find_pcap_interface, find_pnet_interface, get_interface_ipv4};
use crate::message::PeerMessage;
use crate::scanner::arp_scan;
use crate::types::{CapturedEvent, HostEntry, PeerInfo, SessionKey, SessionStats};

#[derive(Clone)]
pub struct AppState {
    pub store: Store,
    pub channels: Channels,
    pub flags: Flags,
    pub identity: Identity,
}

#[derive(Clone)]
pub struct Identity {
    pub name: String,
    pub ip: Ipv4Addr,
    pub port: u16,
}

#[derive(Clone)]
pub struct Store {
    pub events: Arc<RwLock<VecDeque<CapturedEvent>>>,
    pub hosts: Arc<RwLock<HashMap<Ipv4Addr, HostEntry>>>,
    pub sessions: Arc<RwLock<HashMap<SessionKey, SessionStats>>>,
    pub peers: Arc<RwLock<HashMap<Ipv4Addr, PeerInfo>>>,
    pub messages: Arc<RwLock<HashMap<PeerInfo, Vec<PeerMessage>>>>,
}

#[derive(Clone)]
pub struct Channels {
    pub internal_tx: tokio::sync::mpsc::Sender<CapturedEvent>,
    pub external_tx: broadcast::Sender<CapturedEvent>,
}

#[derive(Clone)]
pub struct Flags {
    pub capture: Arc<AtomicBool>,
    pub scan: Arc<AtomicBool>,
    pub discovery: Arc<AtomicBool>,
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

#[derive(Deserialize)]
pub struct DiscoveryRequest {
    pub interface: String,
    pub name: String,
    pub port: u16,
}

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    let rx = state.channels.external_tx.subscribe();
    ws.on_upgrade(move |socket| handle_socket(socket, rx))
}

async fn handle_socket(mut socket: WebSocket, mut rx: broadcast::Receiver<CapturedEvent>) {
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

    let packet_tx = state.channels.internal_tx.clone();
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

pub async fn get_packets(State(state): State<AppState>) -> Json<VecDeque<CapturedEvent>> {
    let buf = state.store.events.read().await;
    Json(buf.clone())
}

pub async fn get_hosts(State(state): State<AppState>) -> Json<Vec<HostEntry>> {
    let table = state.store.hosts.read().await;
    Json(table.values().cloned().collect())
}

pub async fn get_sessions(State(state): State<AppState>) -> Json<Vec<(SessionKey, SessionStats)>> {
    let map = state.store.sessions.read().await;
    let mut sessions: Vec<_> = map.iter().map(|(k, v)| (*k, *v)).collect();
    sessions.sort_by_key(|(_, v)| std::cmp::Reverse(v.bytes_total));
    Json(sessions)
}

pub async fn start_discovery(
    State(state): State<AppState>,
    Json(req): Json<DiscoveryRequest>,
) -> Result<StatusCode, StatusCode> {
    let running = state.flags.discovery;

    let interface = find_pnet_interface(&req.interface).ok_or(StatusCode::BAD_REQUEST)?;

    let (name, port) = (req.name, req.port);

    let ip = get_interface_ipv4(&interface).ok_or(StatusCode::BAD_REQUEST)?;

    change_flag(&running)?;

    tokio::task::spawn(async move {
        discovery_sender(name, ip, port, running).await.ok();
    });

    Ok(StatusCode::OK)
}

pub async fn stop_discovery(State(state): State<AppState>) {
    let running = state.flags.discovery;
    running.store(false, Ordering::Relaxed);
}

pub async fn get_peers(State(state): State<AppState>) -> Json<Vec<PeerInfo>> {
    let map = state.store.peers.read().await;
    Json(map.values().cloned().collect())
}

pub async fn get_messages(
    State(state): State<AppState>,
) -> Json<Vec<(PeerInfo, Vec<PeerMessage>)>> {
    let map = state.store.messages.read().await;
    let messages: Vec<_> = map.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    Json(messages)
}
