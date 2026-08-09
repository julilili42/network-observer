extern crate pnet;

use axum::{
    Router,
    http::{Method, header},
    routing::{get, post},
};
use network_sniffer::api::{
    observer::{
        get_hosts, get_packets, get_sessions, start_capture, start_scan, stop_capture, stop_scan,
    },
    types::{ApiEvent, Channels, ObserverState},
    ws::ws_handler_observer,
};
use network_sniffer::observer::{
    processing::spawn_observer_processing,
    types::{Flags, ObserverStore},
};
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, atomic::AtomicBool},
};
use tokio::sync::{RwLock, broadcast};
use tower_http::cors::{Any, CorsLayer};

fn build_observer_app(state: ObserverState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE]);

    let capture_routes = Router::new().route("/", post(start_capture).delete(stop_capture));

    Router::new()
        .nest("/capture", capture_routes)
        .route("/scan", post(start_scan).delete(stop_scan))
        .route("/packets", get(get_packets))
        .route("/sessions", get(get_sessions))
        .route("/hosts", get(get_hosts))
        .route("/ws", get(ws_handler_observer))
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

async fn start_observer_server(app: Router, port: u16) {
    axum_server::bind(format!("0.0.0.0:{}", port).parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

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
    let observer_port: u16 = std::env::var("OBSERVER_PORT")
        .unwrap_or("3000".into())
        .parse::<u16>()
        .unwrap_or(3000);

    let observer_store = build_observer_store();

    let api_tx_clone = api_tx.clone();
    // start processing thread of captured events
    spawn_observer_processing(observer_store.clone(), observer_rx, move |event| {
        let _ = api_tx_clone.send(event.into());
    });

    tracing::info!("Started event processing");

    let api_tx_clone = api_tx.clone();
    let channels = Channels {
        api_tx: api_tx_clone,
        observer_tx,
    };

    let observer = build_observer_app(ObserverState {
        store: observer_store,
        channels,
        flags,
    });

    start_observer_server(observer, observer_port).await;
}
