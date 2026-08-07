use std::net::Ipv4Addr;

use reqwest::Client;
use uuid::Uuid;

use super::types::{FilePayload, PeerEvent, PeerInfo, PeerPayload};
use crate::transfer::types::{Identity, TransferStore};

pub async fn send_event(http: &Client, ip: Ipv4Addr, port: u16, event: &PeerEvent) {
    let res = http
        .post(format!("https://{}:{}/peers/incoming", ip, port))
        .json(event)
        .send()
        .await;

    if let Err(e) = res {
        tracing::error!(error = %e, "event send failed");
    }
}

pub async fn send_pending_file(
    identity: Identity,
    transfer_store: &TransferStore,
    http: &Client,
    transfer_id: Uuid,
    recipient: PeerInfo,
) {
    let transfer = {
        let mut transfers = transfer_store.pending.write().await;
        transfers.remove(&transfer_id)
    };

    let Some(transfer) = transfer else {
        tracing::warn!(transfer_id=%transfer_id, "No pending transfer found");
        return;
    };

    let sender = PeerInfo {
        name: identity.name,
        ip: identity.ip,
        port: identity.port,
    };

    let event = PeerEvent {
        from: sender,
        payload: PeerPayload::File(FilePayload::Data {
            transfer_id,
            filename: transfer.filename,
            data: transfer.data,
        }),
    };

    send_event(http, recipient.ip, recipient.port, &event).await;

    tracing::info!(transfer_id=%transfer_id, "file sent successfully");
}
