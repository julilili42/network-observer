use super::types::PeerEvent;
use crate::transfer::{
    file::handle_file_event,
    message::handle_message_event,
    types::{PeerInfo, PeerPayload, Store, TransferError},
};
use reqwest::Client;
use std::net::Ipv4Addr;

pub async fn send_event(
    http: &Client,
    ip: Ipv4Addr,
    port: u16,
    event: &PeerEvent,
) -> Result<(), reqwest::Error> {
    let url = format!("https://{}:{}/events", ip, port);

    http.post(url)
        .json(event)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

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
