use crate::transfer::{
    message::send_pending_file,
    types::{
        FilePayload, Identity, PeerEvent, PeerPayload, Transfer, TransferDirection, TransferError,
        TransferStatus, TransferStore,
    },
};
use reqwest::Client;
use std::io::ErrorKind;
use std::path::Path;
use tokio::{
    fs::{self, OpenOptions},
    io::{self, AsyncWriteExt},
};

pub async fn handle_peer_event(
    transfer_store: &TransferStore,
    event: &PeerEvent,
    http: &Client,
    identity: Identity,
) -> Result<(), TransferError> {
    match &event.payload {
        PeerPayload::Message(_) => {
            let mut messages = transfer_store.messages.write().await;
            messages
                .entry(event.from.clone())
                .or_default()
                .push(event.clone());
            Ok(())
        }
        PeerPayload::File(FilePayload::Data {
            transfer_id, data, ..
        }) => {
            let filename = {
                let mut transfers = transfer_store.transfers.write().await;
                let transfer = transfers
                    .get_mut(&transfer_id)
                    .ok_or(TransferError::TransferNotFound)?;

                if transfer.direction != TransferDirection::Incoming
                    || transfer.status != TransferStatus::Accepted
                    || transfer.peer != event.from
                {
                    return Err(TransferError::InvalidTransferState);
                }

                transfer.status = TransferStatus::Transferring;
                transfer.meta.filename.clone()
            };

            let result = save_file(filename.as_str(), Path::new("downloads/"), data)
                .await
                .map_err(TransferError::IoFailed);

            if let Some(transfer) = transfer_store.transfers.write().await.get_mut(transfer_id) {
                transfer.status = match &result {
                    Ok(()) => TransferStatus::Completed,
                    Err(e) => TransferStatus::Failed(e.to_string()),
                }
            }

            result
        }
        PeerPayload::File(FilePayload::Offer { meta }) => {
            let safe_name = Path::new(&meta.filename)
                .file_name()
                .ok_or(TransferError::InvalidFileName)?;

            let transfer = Transfer {
                direction: TransferDirection::Incoming,
                peer: event.from.clone(),
                meta: meta.clone(),
                path: Path::new("downloads").join(safe_name),
                status: TransferStatus::Offered,
            };

            transfer_store
                .transfers
                .write()
                .await
                .entry(meta.transfer_id)
                .or_insert(transfer);

            Ok(())
        }
        PeerPayload::File(FilePayload::Accept { transfer_id }) => {
            {
                let mut transfers = transfer_store.transfers.write().await;
                let transfer = transfers
                    .get_mut(&transfer_id)
                    .ok_or(TransferError::TransferNotFound)?;

                if transfer.direction != TransferDirection::Outgoing
                    || transfer.status != TransferStatus::Offered
                    || transfer.peer != event.from
                {
                    return Err(TransferError::InvalidTransferState);
                }
            }

            send_pending_file(*transfer_id, transfer_store, identity, http).await
        }
        PeerPayload::File(FilePayload::Reject { transfer_id }) => {
            let mut transfers = transfer_store.transfers.write().await;
            let transfer = transfers
                .get_mut(&transfer_id)
                .ok_or(TransferError::TransferNotFound)?;

            if transfer.direction != TransferDirection::Outgoing
                || transfer.status != TransferStatus::Offered
                || transfer.peer != event.from
            {
                return Err(TransferError::InvalidTransferState);
            }

            transfer.status = TransferStatus::Rejected;
            Ok(())
        }

        _ => Ok(()),
    }
}

async fn save_file(file_name: &str, dir_path: &Path, data: &[u8]) -> Result<(), io::Error> {
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
