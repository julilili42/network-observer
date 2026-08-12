use std::path::PathBuf;

use reqwest::Client;
use uuid::Uuid;

use crate::transfer::{
    message::send_event,
    types::{
        FileMeta, FilePayload, Identity, PeerEvent, PeerInfo, PeerPayload, Transfer,
        TransferDirection, TransferError, TransferStatus, TransferStore,
    },
};

pub async fn find_peer_by_name(
    transfer_store: &TransferStore,
    recipient_name: &str,
) -> Option<PeerInfo> {
    let peers = transfer_store.peers.read().await;
    peers.values().find(|p| p.name == recipient_name).cloned()
}

pub async fn offer_transfer(
    peer: PeerInfo,
    path: PathBuf,
    transfer_store: &TransferStore,
    identity: &Identity,
    http: &Client,
) -> Result<Uuid, TransferError> {
    let transfer_id = Uuid::new_v4();
    let file_meta = tokio::fs::metadata(&path)
        .await
        .map_err(TransferError::IoFailed)?;
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(TransferError::InvalidFileName)?
        .to_owned();

    let transfer = Transfer {
        direction: TransferDirection::Outgoing,
        peer,
        meta: FileMeta {
            transfer_id,
            filename,
            size_byte: file_meta.len(),
        },
        path,
        status: TransferStatus::Offered,
    };

    tracing::info!(%transfer_id, "offering transfer");

    {
        let mut transfers = transfer_store.transfers.write().await;
        transfers.insert(transfer_id, transfer.clone());
    }

    let event = PeerEvent {
        from: identity.as_peer(),
        payload: PeerPayload::File(FilePayload::Offer {
            meta: transfer.meta,
        }),
    };

    let (ip, port) = (transfer.peer.ip, transfer.peer.port);

    let event_result = send_event(http, ip, port, &event).await;
    let mut transfer_map = transfer_store.transfers.write().await;

    match event_result {
        Ok(_) => Ok(transfer_id),
        Err(e) => {
            if let Some(transfer) = transfer_map.get_mut(&transfer_id) {
                transfer.status = TransferStatus::Failed(e.to_string());
            }
            Err(TransferError::SendFail(e))
        }
    }
}

pub async fn reject_transfer(
    transfer_id: Uuid,
    transfer_store: &TransferStore,
    identity: &Identity,
    http: &Client,
) -> Result<Uuid, TransferError> {
    tracing::info!(%transfer_id, "rejecting transfer");

    let (ip, port) = {
        let mut transfer_map = transfer_store.transfers.write().await;
        let Some(transfer) = transfer_map.get_mut(&transfer_id) else {
            return Err(TransferError::TransferNotFound);
        };
        transfer.status = TransferStatus::Rejected;
        (transfer.peer.ip, transfer.peer.port)
    };

    let payload = PeerPayload::File(FilePayload::Reject { transfer_id });

    let event = PeerEvent {
        from: identity.as_peer(),
        payload,
    };

    if let Err(e) = send_event(http, ip, port, &event).await {
        let mut transfer_map = transfer_store.transfers.write().await;
        let Some(transfer) = transfer_map.get_mut(&transfer_id) else {
            return Err(TransferError::TransferNotFound);
        };
        transfer.status = TransferStatus::Failed(e.to_string());

        return Err(TransferError::SendFail(e));
    };
    Ok(transfer_id)
}

pub async fn accept_transfer(
    transfer_id: Uuid,
    transfer_store: &TransferStore,
    identity: &Identity,
    http: &Client,
) -> Result<Uuid, TransferError> {
    tracing::info!(%transfer_id, "accepting transfer");

    let (ip, port) = {
        let mut transfer_map = transfer_store.transfers.write().await;
        let Some(transfer) = transfer_map.get_mut(&transfer_id) else {
            return Err(TransferError::TransferNotFound);
        };
        transfer.status = TransferStatus::Accepted;

        (transfer.peer.ip, transfer.peer.port)
    };

    let payload = PeerPayload::File(FilePayload::Accept { transfer_id });

    let event = PeerEvent {
        from: identity.as_peer(),
        payload,
    };

    if let Err(e) = send_event(http, ip, port, &event).await {
        let mut transfer_map = transfer_store.transfers.write().await;
        let Some(transfer) = transfer_map.get_mut(&transfer_id) else {
            return Err(TransferError::TransferNotFound);
        };
        transfer.status = TransferStatus::Failed(e.to_string());

        return Err(TransferError::SendFail(e));
    };
    Ok(transfer_id)
}
