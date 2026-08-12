use super::types::{PeerEvent, PeerInfo, PeerPayload};
use crate::transfer::{
    event::send_event,
    types::{Message, Store, TransferDirection, TransferError},
};
use reqwest::Client;

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

pub async fn send_message(
    identity: &PeerInfo,
    transfer_store: &Store,
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

    // save outgoing message event
    let mut messages = transfer_store.messages.write().await;
    messages
        .entry(recipient.clone())
        .or_default()
        .push(Message {
            content: req_content.into(),
            direction: TransferDirection::Outgoing,
        });
    drop(messages);

    // send message to peer
    // false from receiver perspective
    let payload_send = PeerPayload::Message(Message {
        content: req_content.into(),
        direction: TransferDirection::Incoming,
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
