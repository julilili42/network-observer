use std::{collections::HashMap, net::Ipv4Addr, sync::Arc};

use axum::{
    Router,
    routing::{get, post},
};
use axum_server::tls_rustls::RustlsConfig;
use network_sniffer::{
    api::{
        transfer::{
            get_messages, get_peers, handle_accept_transfer, handle_peer_events,
            handle_reject_transfer, handle_send_message,
        },
        types::{ApiEvent, TransferState},
        ws::ws_handler_transfer,
    },
    observer::interface::{find_pnet_interface, get_interface_ipv4},
    transfer::{
        cert::load_or_generate,
        mdns::start_mdns,
        tls,
        types::{PeerInfo, Store},
    },
};
use reqwest::{Method, header};
use rustls::crypto;
use tokio::sync::{RwLock, broadcast};
use tower_http::cors::{Any, CorsLayer};

fn build_transfer_app(state: TransferState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE]);

    Router::new()
        .route("/", get(get_peers))
        .route("/ws", get(ws_handler_transfer))
        .route("/messages", get(get_messages))
        .route("/events", post(handle_peer_events))
        .route("/send_message", post(handle_send_message))
        .route("/accept_transfer", post(handle_accept_transfer))
        .route("/reject_transfer", post(handle_reject_transfer))
        .layer(cors)
        .with_state(state)
}

fn build_identity(port: u16, device_name: String) -> Option<PeerInfo> {
    let interface_name = std::env::var("INTERFACE").unwrap_or_else(|_| "en0".into());
    let interface = find_pnet_interface(&interface_name)?;

    Some(PeerInfo {
        name: device_name,
        ip: get_interface_ipv4(&interface).unwrap_or(Ipv4Addr::UNSPECIFIED),
        port,
    })
}

fn build_transfer_store() -> Store {
    Store {
        peers: Arc::new(RwLock::new(HashMap::new())),
        messages: Arc::new(RwLock::new(HashMap::new())),
        transfers: Arc::new(RwLock::new(HashMap::new())),
    }
}

async fn start_transfer_server(app: Router, port: u16, tls_identity: tls::TlsIdentity) {
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

    let (api_tx, _) = broadcast::channel::<ApiEvent>(100);

    let transfer_port: u16 = std::env::var("TRANSFER_PORT")
        .unwrap_or("3001".into())
        .parse::<u16>()
        .unwrap_or(3001);

    let device_name = std::env::var("DEVICE_NAME").unwrap_or_else(|_| "Unknown".into());

    let cert_path: &str = "cert.pem";
    let key_path: &str = "key.pem";

    let tls_identity =
        load_or_generate(&device_name, cert_path, key_path).expect("Certificate failure");
    tracing::info!("TLS identity ready");

    // first contact trusted blindly > saved for later contact > SSH model
    let tofu_verifier = tls::TofuVerifier::new();

    // tls used to encrypt communication
    let http = tls::build_http_client(tofu_verifier);

    let identity = build_identity(transfer_port, device_name).expect("Failed to build identity");

    let transfer_store = build_transfer_store();

    // start multicast dns discovery > name resolver in localnet
    let _mdns = start_mdns(
        identity.name.clone(),
        identity.ip,
        identity.port,
        transfer_store.peers.clone(),
    )
    .expect("Failed to start mdns");
    tracing::info!("Started mdns");

    let transfer = build_transfer_app(TransferState {
        store: transfer_store,
        api_tx,
        identity,
        http,
    });

    start_transfer_server(transfer, transfer_port, tls_identity).await;
}
