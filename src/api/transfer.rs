use crate::transfer::message::send_event;
use crate::transfer::types::{FileMeta, FilePayload, PeerPayload, PendingTransfer};
use crate::{
    api::AppState,
    transfer::{
        message::send_pending_file,
        processing::handle_peer_event,
        types::{MessagePayload, PeerEvent, PeerInfo},
    },
};
use axum::{Json, extract::State};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub async fn get_peers(State(state): State<AppState>) -> Json<Vec<PeerInfo>> {
    let map = state.transfer.peers.read().await;
    Json(map.values().cloned().collect())
}

pub async fn get_messages(State(state): State<AppState>) -> Json<Vec<(PeerInfo, Vec<PeerEvent>)>> {
    let map = state.transfer.messages.read().await;
    Json(map.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
}

#[derive(Serialize, Deserialize)]
pub struct OutgoingFileOffer {
    pub recipient_name: String,
    pub file_name: String,
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct AcceptRejectRequest {
    pub transfer_id: Uuid,
    pub from_name: String,
}

pub async fn handle_outgoing_file_offer(
    State(state): State<AppState>,
    Json(req): Json<OutgoingFileOffer>,
) -> StatusCode {
    let Some(recipient) = state.find_peer_by_name(&req.recipient_name).await else {
        return StatusCode::NOT_FOUND;
    };

    let transfer_id = Uuid::new_v4();
    tracing::info!(%transfer_id, "created transfer");
    let size_bytes = req.data.len() as u64;

    state.transfer.pending.write().await.insert(
        transfer_id,
        PendingTransfer {
            filename: req.file_name.clone(),
            data: req.data,
        },
    );

    let payload = PeerPayload::File(FilePayload::Offer {
        meta: FileMeta {
            transfer_id,
            filename: req.file_name,
            size_byte: size_bytes,
        },
    });

    state.send_to(&recipient, payload).await
}

pub async fn handle_outgoing_file_accept(
    State(state): State<AppState>,
    Json(req): Json<AcceptRejectRequest>,
) -> StatusCode {
    let Some(sender) = state.find_peer_by_name(&req.from_name).await else {
        return StatusCode::NOT_FOUND;
    };

    let payload = PeerPayload::File(FilePayload::Accept {
        transfer_id: req.transfer_id,
    });

    state.send_to(&sender, payload).await
}

pub async fn handle_outgoing_file_reject(
    State(state): State<AppState>,
    Json(req): Json<AcceptRejectRequest>,
) -> StatusCode {
    let Some(sender) = state.find_peer_by_name(&req.from_name).await else {
        return StatusCode::NOT_FOUND;
    };

    let payload = PeerPayload::File(FilePayload::Reject {
        transfer_id: req.transfer_id,
    });

    state.send_to(&sender, payload).await
}

#[derive(Deserialize)]
pub struct SendMessageRequest {
    pub name: String,
    pub content: String,
}

pub async fn handle_outgoing_message(
    State(state): State<AppState>,
    Json(req): Json<SendMessageRequest>,
) -> StatusCode {
    // lookup recipient info from name
    let recipient: Option<PeerInfo> = {
        let peers = state.transfer.peers.read().await;
        peers.values().find(|p| p.name == req.name).cloned()
    };

    let Some(recipient) = recipient else {
        return StatusCode::NOT_FOUND;
    };

    let sender = PeerInfo {
        name: state.identity.name,
        ip: state.identity.ip,
        port: state.identity.port,
    };

    // save outgoing message event
    let mut messages = state.transfer.messages.write().await;
    let from = sender.clone();
    let payload_store = PeerPayload::Message(MessagePayload {
        content: req.content.clone(),
        outgoing: true,
    });

    messages
        .entry(recipient.clone())
        .or_default()
        .push(PeerEvent {
            from: from.clone(),
            payload: payload_store,
        });

    // send message to peer
    // false from receiver perspective
    let payload_send = PeerPayload::Message(MessagePayload {
        content: req.content,
        outgoing: false,
    });

    let event = &PeerEvent {
        from,
        payload: payload_send,
    };

    send_event(&state.http, recipient.ip, recipient.port, event).await;

    StatusCode::OK
}

pub async fn handle_incoming(
    State(state): State<AppState>,
    Json(event): Json<PeerEvent>,
) -> StatusCode {
    tracing::debug!(sender = %event.from, "Incoming peer event");

    handle_peer_event(&state.transfer, &event).await;
    // side effect: peer accepted file offer -> send file
    if let PeerPayload::File(FilePayload::Accept { transfer_id }) = event.payload {
        let recipient = event.from.clone();

        tokio::spawn(async move {
            send_pending_file(
                state.identity,
                &state.transfer,
                &state.http,
                transfer_id,
                recipient,
            )
            .await
        });
    }

    // send message to event_processing -> sends it to ws
    let _ = state.channels.api_tx.send(event.into());

    StatusCode::OK
}
