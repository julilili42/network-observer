use crate::transfer::{
    file::{save_file, send_pending_file},
    types::{
        FilePayload, Message, PeerEvent, PeerInfo, PeerPayload, Store, Transfer, TransferDirection,
        TransferError, TransferStatus,
    },
};
use reqwest::Client;
use std::path::Path;

pub async fn handle_peer_event(
    transfer_store: &Store,
    event: &PeerEvent,
    http: &Client,
    identity: PeerInfo,
) -> Result<(), TransferError> {
    let sender = event.from.clone();
    match &event.payload {
        PeerPayload::Message(message) => {
            handle_message_event(message, transfer_store, sender).await
        }
        PeerPayload::File(file) => {
            handle_file_event(file, transfer_store, sender, http, identity).await
        }
    }
}

pub async fn handle_message_event(
    message: &Message,
    transfer_store: &Store,
    sender: PeerInfo,
) -> Result<(), TransferError> {
    let mut messages = transfer_store.messages.write().await;
    messages.entry(sender).or_default().push(Message {
        content: message.content.clone(),
        direction: TransferDirection::Incoming,
    });
    Ok(())
}

pub async fn handle_file_event(
    file: &FilePayload,
    transfer_store: &Store,
    sender: PeerInfo,
    http: &Client,
    identity: PeerInfo,
) -> Result<(), TransferError> {
    match file {
        FilePayload::Accept { transfer_id } => {
            {
                let mut transfers = transfer_store.transfers.write().await;
                let transfer = transfers
                    .get_mut(transfer_id)
                    .ok_or(TransferError::TransferNotFound)?;

                if transfer.direction != TransferDirection::Outgoing
                    || transfer.status != TransferStatus::Offered
                    || transfer.peer != sender
                {
                    return Err(TransferError::InvalidTransferState);
                }
            }

            send_pending_file(*transfer_id, transfer_store, identity, http).await
        }
        FilePayload::Offer { meta } => {
            let safe_name = Path::new(&meta.filename)
                .file_name()
                .ok_or(TransferError::InvalidFileName)?;

            let transfer = Transfer {
                direction: TransferDirection::Incoming,
                peer: sender.clone(),
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
        FilePayload::Data {
            transfer_id, data, ..
        } => {
            let filename = {
                let mut transfers = transfer_store.transfers.write().await;
                let transfer = transfers
                    .get_mut(transfer_id)
                    .ok_or(TransferError::TransferNotFound)?;

                if transfer.direction != TransferDirection::Incoming
                    || transfer.status != TransferStatus::Accepted
                    || transfer.peer != sender
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
        FilePayload::Reject { transfer_id } => {
            let mut transfers = transfer_store.transfers.write().await;
            let transfer = transfers
                .get_mut(transfer_id)
                .ok_or(TransferError::TransferNotFound)?;

            if transfer.direction != TransferDirection::Outgoing
                || transfer.status != TransferStatus::Offered
                || transfer.peer != sender
            {
                return Err(TransferError::InvalidTransferState);
            }

            transfer.status = TransferStatus::Rejected;
            Ok(())
        }
    }
}
