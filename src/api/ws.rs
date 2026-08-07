use axum::{
    extract::State,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use tokio::sync::broadcast;

use crate::api::types::{ApiEvent, AppState};

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
