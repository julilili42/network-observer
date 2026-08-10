use std::net::Ipv4Addr;

use super::types::{FilePayload, PeerEvent, PeerInfo, PeerPayload};
use crate::transfer::types::{Identity, MessagePayload, TransferError, TransferStore};
use reqwest::Client;
use uuid::Uuid;

pub async fn send_event(
    http: &Client,
    ip: Ipv4Addr,
    port: u16,
    event: &PeerEvent,
) -> Result<(), reqwest::Error> {
    let url = format!("https://{}:{}/incoming", ip, port);

    http.post(url)
        .json(event)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

pub async fn send_pending_file(
    identity: Identity,
    transfer_store: &TransferStore,
    http: &Client,
    transfer_id: Uuid,
    recipient: &PeerInfo,
) -> Result<(), TransferError> {
    let transfer = {
        let mut transfers = transfer_store.pending.write().await;
        transfers.remove(&transfer_id)
    };

    let Some(transfer) = transfer else {
        return Err(TransferError::NoPending);
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

    if let Err(e) = send_event(http, recipient.ip, recipient.port, &event).await {
        return Err(TransferError::SendFail(e));
    };

    tracing::info!(transfer_id=%transfer_id, "file sent successfully");
    Ok(())
}

pub async fn send_message(
    identity: &Identity,
    transfer_store: &TransferStore,
    http: &Client,
    req_name: &str,
    req_content: &str,
) -> Result<(), TransferError> {
    // lookup recipient info from name
    let recipient: Option<PeerInfo> = {
        let peers = transfer_store.peers.read().await;
        peers.values().find(|p| p.name == req_name).cloned()
    };

    let Some(recipient) = recipient else {
        return Err(TransferError::PeerNotFound);
    };

    let sender = PeerInfo {
        name: identity.name.clone(),
        ip: identity.ip,
        port: identity.port,
    };

    let from = sender.clone();
    let payload_store = PeerPayload::Message(MessagePayload {
        content: req_content.into(),
        outgoing: true,
    });

    // save outgoing message event
    let mut messages = transfer_store.messages.write().await;
    messages
        .entry(recipient.clone())
        .or_default()
        .push(PeerEvent {
            from: from.clone(),
            payload: payload_store,
        });
    drop(messages);

    // send message to peer
    // false from receiver perspective
    let payload_send = PeerPayload::Message(MessagePayload {
        content: req_content.into(),
        outgoing: false,
    });

    let event = &PeerEvent {
        from,
        payload: payload_send,
    };

    if let Err(e) = send_event(http, recipient.ip, recipient.port, event).await {
        return Err(TransferError::SendFail(e));
    };

    Ok(())
}
