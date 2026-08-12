use std::{
    io,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use reqwest::Client;
use tokio::{fs, fs::OpenOptions, io::AsyncWriteExt};
use uuid::Uuid;

use crate::transfer::{
    message::send_event,
    types::{
        FileMeta, FilePayload, PeerEvent, PeerInfo, PeerPayload, Store, Transfer,
        TransferDirection, TransferError, TransferStatus,
    },
};

pub async fn save_file(file_name: &str, dir_path: &Path, data: &[u8]) -> Result<(), io::Error> {
    fs::create_dir_all(dir_path).await?;

    let safe_name = Path::new(file_name)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or(io::Error::new(
            ErrorKind::InvalidFilename,
            "invalid filename",
        ))?;

    let path = dir_path.join(safe_name);

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await?;

    file.write_all(data).await?;
    tracing::info!(file_name = %file_name, "File saved to {:?}", dir_path);

    Ok(())
}

pub async fn send_pending_file(
    transfer_id: Uuid,
    transfer_store: &Store,
    identity: PeerInfo,
    http: &Client,
) -> Result<(), TransferError> {
    let (ip, port, path) = {
        let mut transfer_map = transfer_store.transfers.write().await;
        let Some(transfer) = transfer_map.get_mut(&transfer_id) else {
            return Err(TransferError::TransferNotFound);
        };
        transfer.status = TransferStatus::Transferring;
        (transfer.peer.ip, transfer.peer.port, transfer.path.clone())
    };

    let result: Result<(), TransferError> = async {
        let data = tokio::fs::read(path)
            .await
            .map_err(TransferError::IoFailed)?;

        let event = PeerEvent {
            from: identity.clone(),
            payload: PeerPayload::File(FilePayload::Data { transfer_id, data }),
        };

        send_event(http, ip, port, &event)
            .await
            .map_err(TransferError::SendFail)?;
        Ok(())
    }
    .await;

    if let Some(transfer) = transfer_store.transfers.write().await.get_mut(&transfer_id) {
        transfer.status = match &result {
            Ok(()) => TransferStatus::Completed,
            Err(e) => TransferStatus::Failed(e.to_string()),
        }
    };

    tracing::info!(transfer_id=%transfer_id, "file sent successfully");
    result
}

pub async fn offer_transfer(
    peer: PeerInfo,
    path: PathBuf,
    transfer_store: &Store,
    identity: &PeerInfo,
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
        from: identity.clone(),
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
    transfer_store: &Store,
    identity: &PeerInfo,
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

    let event = PeerEvent {
        from: identity.clone(),
        payload: PeerPayload::File(FilePayload::Reject { transfer_id }),
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
    transfer_store: &Store,
    identity: &PeerInfo,
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

    let event = PeerEvent {
        from: identity.clone(),
        payload: PeerPayload::File(FilePayload::Accept { transfer_id }),
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
