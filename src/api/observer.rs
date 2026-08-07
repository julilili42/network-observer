use crate::api::AppState;
use crate::observer::types::{HostEntry, SessionKey, SessionStats};
use crate::observer::{capture::capture_packets, scanner::arp_scan};
use crate::{
    observer::interface::{
        change_flag, find_pcap_interface, find_pnet_interface, get_interface_ipv4,
    },
    observer::types::ObserverEvent,
};
use axum::{
    http::StatusCode,
    {Json, extract::State},
};
use serde::Deserialize;
use std::{collections::VecDeque, sync::atomic::Ordering};

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

pub async fn start_capture(
    State(state): State<AppState>,
    Json(req): Json<CaptureRequest>,
) -> Result<StatusCode, StatusCode> {
    let running = state.flags.capture_running.clone();

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
    let running = state.flags.scan_running.clone();

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
    state.flags.capture_running.store(false, Ordering::Relaxed);
}

pub async fn stop_scan(State(state): State<AppState>) {
    state.flags.scan_running.store(false, Ordering::Relaxed);
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
