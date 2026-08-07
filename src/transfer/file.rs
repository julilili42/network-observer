use reqwest::Client;
use uuid::Uuid;

use crate::transfer::{
    message::send_event,
    types::{
        FileMeta, FilePayload, Identity, PeerEvent, PeerInfo, PeerPayload, PendingTransfer,
        TransferError, TransferStore,
    },
};

async fn find_peer_by_name(
    transfer_store: &TransferStore,
    recipient_name: &str,
) -> Option<PeerInfo> {
    let peers = transfer_store.peers.read().await;
    peers.values().find(|p| p.name == recipient_name).cloned()
}

pub async fn offer_file(
    recipient_name: &str,
    file_name: &str,
    data: Vec<u8>,
    transfer_store: &TransferStore,
    identity: &Identity,
    http: &Client,
) -> Result<(), TransferError> {
    let Some(recipient) = find_peer_by_name(transfer_store, recipient_name).await else {
        return Err(TransferError::PeerNotFound);
    };

    let transfer_id = Uuid::new_v4();
    tracing::info!(%transfer_id, "created transfer");
    let size_bytes = data.len() as u64;

    let mut pending_transfers = transfer_store.pending.write().await;
    pending_transfers.insert(
        transfer_id,
        PendingTransfer {
            filename: file_name.into(),
            data,
        },
    );
    drop(pending_transfers);

    let payload = PeerPayload::File(FilePayload::Offer {
        meta: FileMeta {
            transfer_id,
            filename: file_name.into(),
            size_byte: size_bytes,
        },
    });

    let event = PeerEvent {
        from: identity.as_peer(),
        payload,
    };

    if let Err(error) = send_event(http, recipient.ip, recipient.port, &event).await {
        let mut pending_transfers = transfer_store.pending.write().await;
        pending_transfers.remove(&transfer_id);
        drop(pending_transfers);

        return Err(TransferError::SendFail(error));
    };

    Ok(())
}

pub async fn reject_file(
    recipient_name: &str,
    transfer_id: Uuid,
    transfer_store: &TransferStore,
    identity: &Identity,
    http: &Client,
) -> Result<(), TransferError> {
    let Some(sender) = find_peer_by_name(transfer_store, recipient_name).await else {
        return Err(TransferError::PeerNotFound);
    };

    let payload = PeerPayload::File(FilePayload::Reject { transfer_id });

    let event = PeerEvent {
        from: identity.as_peer(),
        payload,
    };

    if let Err(error) = send_event(http, sender.ip, sender.port, &event).await {
        return Err(TransferError::SendFail(error));
    };

    Ok(())
}

pub async fn accept_file(
    recipient_name: &str,
    transfer_id: Uuid,
    transfer_store: &TransferStore,
    identity: &Identity,
    http: &Client,
) -> Result<(), TransferError> {
    let Some(sender) = find_peer_by_name(transfer_store, recipient_name).await else {
        return Err(TransferError::PeerNotFound);
    };

    let payload = PeerPayload::File(FilePayload::Accept { transfer_id });
    let event = PeerEvent {
        from: identity.as_peer(),
        payload,
    };

    if let Err(error) = send_event(http, sender.ip, sender.port, &event).await {
        return Err(TransferError::SendFail(error));
    };

    Ok(())
}
