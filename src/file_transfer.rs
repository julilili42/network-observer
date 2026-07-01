use axum::{Json, extract::State};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    api::{AppState, PendingTransfer},
    types::{FileMeta, FilePayload, PeerPayload},
};

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

    state.store.pending_transfer.write().await.insert(
        transfer_id.clone(),
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
