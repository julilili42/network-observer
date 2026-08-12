use core::fmt;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::Hash, net::Ipv4Addr, path::PathBuf, sync::Arc};
use tokio::{io, sync::RwLock};
use uuid::Uuid;

#[derive(Clone)]
pub struct Store {
    pub peers: Arc<RwLock<HashMap<Ipv4Addr, PeerInfo>>>,
    pub messages: Arc<RwLock<HashMap<PeerInfo, Vec<Message>>>>,
    pub transfers: Arc<RwLock<HashMap<Uuid, Transfer>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PeerPayload {
    Message(Message),
    File(FilePayload),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TransferStatus {
    Offered,
    Accepted,
    Transferring,
    Completed,
    Rejected,
    Failed(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct Transfer {
    pub direction: TransferDirection,
    pub peer: PeerInfo,
    pub meta: FileMeta,
    pub path: PathBuf,
    pub status: TransferStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransferDirection {
    Incoming,
    Outgoing,
}

#[derive(Debug)]
pub enum TransferError {
    PeerNotFound,
    InvalidTransferState,
    TransferNotFound,
    InvalidFileName,
    SendFail(reqwest::Error),
    IoFailed(io::Error),
}

impl fmt::Display for TransferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransferError::PeerNotFound => write!(f, "peer not found"),
            TransferError::InvalidTransferState => write!(f, "transfer state not valid"),
            TransferError::TransferNotFound => write!(f, "transfer not found"),
            TransferError::InvalidFileName => write!(f, "file name not valid"),
            TransferError::SendFail(e) => write!(f, "send fail: {}", e),
            TransferError::IoFailed(e) => write!(f, "io operation failed: {}", e),
        }
    }
}

impl std::error::Error for TransferError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TransferError::SendFail(e) => Some(e),
            TransferError::IoFailed(e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMeta {
    pub transfer_id: Uuid,
    pub filename: String,
    pub size_byte: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilePayload {
    Offer { meta: FileMeta },
    Accept { transfer_id: Uuid },
    Reject { transfer_id: Uuid },
    Data { transfer_id: Uuid, data: Vec<u8> },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub content: String,
    pub direction: TransferDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerEvent {
    pub from: PeerInfo,
    pub payload: PeerPayload,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Hash, Eq)]
pub struct PeerInfo {
    pub name: String,
    pub ip: Ipv4Addr,
    pub port: u16,
}

impl fmt::Display for PeerInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Name: {}, ip: {}, Port: {}",
            self.name, self.ip, self.port
        )
    }
}
