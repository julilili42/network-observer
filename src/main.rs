mod api;
mod capture;
mod helper;
mod mdns;
mod message;
mod parser;
mod processing;
mod scanner;
mod tls;
mod types;

extern crate pnet;
use crate::api::{
    AppState, Identity, get_hosts, get_messages, get_packets, get_sessions, start_capture,
    start_scan, stop_capture, stop_scan, ws_handler,
};
use crate::api::{Channels, Flags, Store, get_peers};
use crate::helper::{find_pnet_interface, get_interface_ipv4};
use crate::mdns::start_mdns;
use crate::message::{handle_incoming_message, handle_outgoing_message};
use crate::processing::spawn_event_processing;
use axum::http::{Method, header};
use axum::{
    Router,
    routing::{get, post},
};
use axum_server::tls_rustls::RustlsConfig;
use rustls;
use std::net::Ipv4Addr;
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, atomic::AtomicBool},
};
use tokio::sync::{RwLock, broadcast};
use tower_http::cors::{Any, CorsLayer};
use types::CapturedEvent;

fn build_app(
    store: Store,
    channels: Channels,
    flags: Flags,
    identity: Identity,
    http: reqwest::Client,
) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE]);

    let capture_routes = Router::new().route("/", post(start_capture).delete(stop_capture));

    let peer_routes = Router::new()
        .route("/", get(get_peers))
        .route("/messages", get(get_messages))
        .route("/incoming_message", post(handle_incoming_message))
        .route("/outgoing_message", post(handle_outgoing_message));

    Router::new()
        .nest("/capture", capture_routes)
        .route("/scan", post(start_scan).delete(stop_scan))
        .nest("/peers", peer_routes)
        .route("/packets", get(get_packets))
        .route("/sessions", get(get_sessions))
        .route("/hosts", get(get_hosts))
        .route("/ws", get(ws_handler))
        .layer(cors)
        .with_state(AppState {
            store,
            channels,
            flags,
            identity,
            http,
        })
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install ring crypto provider");

    let (external_tx, _) = broadcast::channel::<CapturedEvent>(100);
    let (internal_tx, internal_rx) = tokio::sync::mpsc::channel(1000);

    let events = Arc::new(RwLock::new(VecDeque::new()));
    let hosts = Arc::new(RwLock::new(HashMap::new()));
    let sessions = Arc::new(RwLock::new(HashMap::new()));
    let peers = Arc::new(RwLock::new(HashMap::new()));
    let messages = Arc::new(RwLock::new(HashMap::new()));

    let capture = Arc::new(AtomicBool::new(false));
    let scan = Arc::new(AtomicBool::new(false));

    let external_tx_clone = external_tx.clone();

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".into())
        .parse::<u16>()
        .unwrap_or_else(|_| 3000);
    let interface_name = std::env::var("INTERFACE").unwrap_or_else(|_| "eth0".into());
    let device_name = std::env::var("DEVICE_NAME").unwrap_or_else(|_| "Unknown".into());
    let interface = find_pnet_interface(&interface_name).unwrap();

    let tls_identity = tls::load_or_generate(&device_name);
    tracing::info!("TLS identity ready");

    let identity = Identity {
        name: device_name,
        ip: get_interface_ipv4(&interface).unwrap_or_else(|| Ipv4Addr::UNSPECIFIED),
        port,
    };

    let store = Store {
        events,
        hosts,
        sessions,
        peers,
        messages,
    };
    let store_clone = store.clone();
    let channels = Channels {
        internal_tx,
        external_tx,
    };
    let flags = Flags { capture, scan };

    let tofu_verifier = tls::TofuVerifier::new();

    let http = tls::build_http_client(tofu_verifier);

    // start mdns discovery
    let _ = start_mdns(
        identity.name.clone(),
        identity.ip,
        identity.port,
        store.peers.clone(),
    );

    // processes received packets
    spawn_event_processing(store_clone, internal_rx, external_tx_clone);

    let app = build_app(store, channels, flags, identity, http);

    let rustls_config = RustlsConfig::from_pem(
        tls_identity.cert.into_bytes(),
        tls_identity.key.into_bytes(),
    )
    .await
    .expect("Failed to build TLS config");

    axum_server::bind_rustls(format!("0.0.0.0:{}", port).parse().unwrap(), rustls_config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
