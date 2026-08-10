use core::fmt;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::Hash, net::Ipv4Addr, sync::Arc};
use tokio::{io, sync::RwLock};
use uuid::Uuid;

#[derive(Debug)]
pub enum TransferError {
    PeerNotFound,
    SendFail(reqwest::Error),
    NoPending,
    IoFailed(io::Error),
}

impl fmt::Display for TransferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransferError::PeerNotFound => write!(f, "peer not found"),
            TransferError::SendFail(e) => write!(f, "send fail: {}", e),
            TransferError::NoPending => write!(f, "no pending transfer"),
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

#[derive(Clone)]
pub struct Identity {
    pub name: String,
    pub ip: Ipv4Addr,
    pub port: u16,
}

impl Identity {
    pub fn as_peer(&self) -> PeerInfo {
        PeerInfo {
            name: self.name.clone(),
            ip: self.ip,
            port: self.port,
        }
    }
}

#[derive(Clone)]
pub struct TransferStore {
    pub peers: Arc<RwLock<HashMap<Ipv4Addr, PeerInfo>>>,
    pub messages: Arc<RwLock<HashMap<PeerInfo, Vec<PeerEvent>>>>,
    pub pending: Arc<RwLock<HashMap<Uuid, PendingTransfer>>>,
}

#[derive(Clone)]
pub struct PendingTransfer {
    pub filename: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMeta {
    pub transfer_id: Uuid,
    pub filename: String,
    pub size_byte: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilePayload {
    Offer {
        meta: FileMeta,
    },
    Accept {
        transfer_id: Uuid,
    },
    Reject {
        transfer_id: Uuid,
    },
    Data {
        transfer_id: Uuid,
        filename: String,
        data: Vec<u8>,
    },
    Complete {
        meta: FileMeta,
    },
}

impl fmt::Display for FilePayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FilePayload::Offer { meta } => {
                write!(f, "FILE OFFER {} ({})", meta.filename, meta.transfer_id)
            }
            FilePayload::Accept { transfer_id } => {
                write!(f, "FILE ACCEPTED {} ", transfer_id)
            }
            FilePayload::Reject { transfer_id } => write!(f, "FILE REJECTED {}", transfer_id),
            FilePayload::Data {
                transfer_id,
                filename,
                data,
            } => write!(
                f,
                "FILE DATA {}: {:?} with id {}",
                filename, data, transfer_id
            ),
            FilePayload::Complete { meta } => write!(f, "FILE COMPLETE {}", meta.filename),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessagePayload {
    pub content: String,
    pub outgoing: bool,
}

impl fmt::Display for MessagePayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MESSAGE {:?} (outgoing: {})",
            self.content, self.outgoing
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PeerPayload {
    Message(MessagePayload),
    File(FilePayload),
}

impl fmt::Display for PeerPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PeerPayload::Message(message) => {
                write!(f, "MESSAGE {message}")
            }
            PeerPayload::File(file) => {
                write!(f, "{file}")
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerEvent {
    pub from: PeerInfo,
    pub payload: PeerPayload,
}

impl fmt::Display for PeerEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PEER EVENT {} from {}", self.payload, self.from)
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
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

impl PartialEq for PeerInfo {
    fn eq(&self, other: &Self) -> bool {
        self.ip == other.ip && self.port == other.port
    }
}

impl Eq for PeerInfo {}

impl Hash for PeerInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ip.hash(state);
        self.port.hash(state);
    }
}
