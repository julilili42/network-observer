use std::path::Path;

use crate::transfer::types::{FilePayload, PeerEvent, PeerPayload, TransferStore};

pub async fn handle_peer_event(store: &TransferStore, event: &PeerEvent) {
    match &event.payload {
        PeerPayload::Message(_) => {
            let mut messages = store.messages.write().await;
            messages
                .entry(event.from.clone())
                .or_default()
                .push(event.clone());
        }
        PeerPayload::File(FilePayload::Data { filename, data, .. }) => {
            save_file(filename, data).await;
        }
        _ => {}
    }
}

async fn save_file(filename: &str, data: &[u8]) {
    if let Err(e) = tokio::fs::create_dir_all("downloads").await {
        tracing::error!(error=%e, "failed to create download dir");
        return;
    }

    let safe_name = Path::new(filename)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown file");

    let path = format!("downloads/{}", safe_name);

    match tokio::fs::write(&path, data).await {
        Ok(_) => tracing::info!(filename = %safe_name, "File saved to downloads/"),
        Err(e) => tracing::error!(error = %e, "failed to write file"),
    }
}
