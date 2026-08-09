use axum::{
    extract::State,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use tokio::sync::broadcast;

use crate::api::types::{ApiEvent, ObserverState, TransferState};

pub async fn ws_handler_observer(
    ws: WebSocketUpgrade,
    State(state): State<ObserverState>,
) -> impl IntoResponse {
    let rx = state.channels.api_tx.subscribe();
    ws.on_upgrade(move |socket| handle_socket(socket, rx))
}

pub async fn ws_handler_transfer(
    ws: WebSocketUpgrade,
    State(state): State<TransferState>,
) -> impl IntoResponse {
    let rx = state.api_tx.subscribe();
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
