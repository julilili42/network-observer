extern crate pnet;

use axum::{
    Router,
    http::{Method, header},
    routing::{get, post},
};
use axum_server::tls_rustls::RustlsConfig;
use network_sniffer::api::{
    ApiEvent, AppState, Channels,
    observer::{
        get_hosts, get_packets, get_sessions, start_capture, start_scan, stop_capture, stop_scan,
    },
    transfer::{
        get_messages, get_peers, handle_incoming, handle_outgoing_file_accept,
        handle_outgoing_file_offer, handle_outgoing_file_reject, handle_outgoing_message,
    },
    ws::ws_handler,
};
use network_sniffer::observer::{
    interface::{find_pnet_interface, get_interface_ipv4},
    processing::spawn_observer_processing,
    types::{Flags, ObserverStore},
};
use network_sniffer::transfer::{
    mdns::start_mdns,
    tls,
    types::{Identity, TransferStore},
};
use rustls::crypto;
use std::{
    collections::{HashMap, VecDeque},
    net::Ipv4Addr,
    sync::{Arc, atomic::AtomicBool},
};
use tokio::sync::{RwLock, broadcast};
use tower_http::cors::{Any, CorsLayer};

fn build_app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE]);

    let capture_routes = Router::new().route("/", post(start_capture).delete(stop_capture));

    let peer_routes = Router::new()
        .route("/", get(get_peers))
        .route("/messages", get(get_messages))
        .route("/incoming", post(handle_incoming))
        .route("/outgoing_message", post(handle_outgoing_message))
        .route("/outgoing_file_offer", post(handle_outgoing_file_offer))
        .route("/outgoing_file_accept", post(handle_outgoing_file_accept))
        .route("/outgoing_file_reject", post(handle_outgoing_file_reject));

    Router::new()
        .nest("/capture", capture_routes)
        .route("/scan", post(start_scan).delete(stop_scan))
        .nest("/peers", peer_routes)
        .route("/packets", get(get_packets))
        .route("/sessions", get(get_sessions))
        .route("/hosts", get(get_hosts))
        .route("/ws", get(ws_handler))
        .layer(cors)
        .with_state(state)
}

fn build_observer_store() -> ObserverStore {
    ObserverStore {
        events: Arc::new(RwLock::new(VecDeque::new())),
        hosts: Arc::new(RwLock::new(HashMap::new())),
        sessions: Arc::new(RwLock::new(HashMap::new())),
    }
}

fn build_transfer_store() -> TransferStore {
    TransferStore {
        peers: Arc::new(RwLock::new(HashMap::new())),
        messages: Arc::new(RwLock::new(HashMap::new())),
        pending: Arc::new(RwLock::new(HashMap::new())),
    }
}

fn build_identity(port: u16, device_name: String) -> Option<Identity> {
    let interface_name = std::env::var("INTERFACE").unwrap_or_else(|_| "eth0".into());
    let interface = find_pnet_interface(&interface_name)?;

    Some(Identity {
        name: device_name,
        ip: get_interface_ipv4(&interface).unwrap_or(Ipv4Addr::UNSPECIFIED),
        port,
    })
}

async fn start_server(app: Router, port: u16, tls_identity: tls::TlsIdentity) {
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

#[tokio::main]
async fn main() {
    // initialize tracing and crypto provider
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install ring crypto provider");

    // external_tx is 1:n sender of last 100 captured events > used for websocket > client connected to ws receives external_rx
    let (api_tx, _) = broadcast::channel::<ApiEvent>(100);

    // currently bounded to 1000 captured events > when processing falls behind > capturing thread is blocked
    let (observer_tx, observer_rx) = tokio::sync::mpsc::channel(1000);

    // thread save variables
    let capture_running = Arc::new(AtomicBool::new(false));
    let scan_running = Arc::new(AtomicBool::new(false));

    let flags = Flags {
        capture_running,
        scan_running,
    };

    // port variable is set via environment variable
    let port: u16 = std::env::var("PORT")
        .unwrap_or("3000".into())
        .parse::<u16>()
        .unwrap_or(3000);
    let device_name = std::env::var("DEVICE_NAME").unwrap_or_else(|_| "Unknown".into());

    // generates self-signed tls certificate
    let tls_identity = tls::load_or_generate(&device_name);
    tracing::info!("TLS identity ready");

    // first contact trusted blindly > saved for later contact > SSH model
    let tofu_verifier = tls::TofuVerifier::new();

    // tls used to encrypt communication
    let http = tls::build_http_client(tofu_verifier);

    let identity = build_identity(port, device_name).expect("Failed to build identity");

    let observer_store = build_observer_store();
    let transfer_store = build_transfer_store();

    // start multicast dns discovery > name resolver in localnet
    let _ = start_mdns(
        identity.name.clone(),
        identity.ip,
        identity.port,
        transfer_store.peers.clone(),
    );
    tracing::info!("Started mdns");

    // start processing thread of captured events
    spawn_observer_processing(observer_store.clone(), observer_rx, api_tx.clone());

    tracing::info!("Started event processing");

    let channels = Channels {
        api_tx,
        observer_tx,
    };

    let app = build_app(AppState {
        observer: observer_store,
        transfer: transfer_store,
        channels,
        identity,
        flags,
        http,
    });

    start_server(app, port, tls_identity).await;
}
