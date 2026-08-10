use crate::transfer::{
    message::send_pending_file,
    types::{
        FilePayload, Identity, PeerEvent, PeerInfo, PeerPayload, TransferError, TransferStore,
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
    store: &TransferStore,
    event: &PeerEvent,
    http: &Client,
    identity: Identity,
    recipient: &PeerInfo,
) -> Result<(), TransferError> {
    match &event.payload {
        PeerPayload::Message(_) => {
            let mut messages = store.messages.write().await;
            messages
                .entry(event.from.clone())
                .or_default()
                .push(event.clone());
            Ok(())
        }
        PeerPayload::File(FilePayload::Data { filename, data, .. }) => {
            save_file(filename, Path::new("downloads/"), data)
                .await
                .map_err(TransferError::IoFailed)
        }
        PeerPayload::File(FilePayload::Accept { transfer_id }) => {
            send_pending_file(identity, store, http, *transfer_id, recipient).await
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
