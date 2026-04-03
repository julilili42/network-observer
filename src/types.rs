use core::fmt;
use serde::{Deserialize, Serialize};
use std::{hash::Hash, net::Ipv4Addr};
use uuid::Uuid;

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone, Serialize)]
pub struct SessionKey {
    pub a_ip: Ipv4Addr,
    pub a_port: u16,
    pub b_ip: Ipv4Addr,
    pub b_port: u16,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct SessionStats {
    pub packets_total: usize,
    pub bytes_total: usize,
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

#[derive(Debug, Clone, Serialize)]
pub enum CapturedEvent {
    Transport(TransportPacket),
    Arp(ArpPacket),
    Peer(PeerEvent),
}

impl fmt::Display for CapturedEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CapturedEvent::Transport(transport_packet) => {
                write!(f, "{transport_packet}")
            }
            CapturedEvent::Arp(arp_packet) => {
                write!(f, "{arp_packet}")
            }
            CapturedEvent::Peer(peer_event) => {
                write!(f, "{peer_event}")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct TransportPacket {
    pub src_ip: Ipv4Addr,
    pub src_port: u16,
    pub dst_ip: Ipv4Addr,
    pub dst_port: u16,
    pub len: usize,
    pub protocol: TransportProtocol,
}

impl fmt::Display for TransportPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Transport: {}:{} -> {}:{}",
            self.src_ip, self.src_port, self.dst_ip, self.dst_port
        )
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum TransportProtocol {
    Tcp,
    Udp,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArpPacket {
    pub sender_ip: Ipv4Addr,
    pub sender_mac: [u8; 6],
    pub target_ip: Ipv4Addr,
    pub target_mac: [u8; 6],
    pub operation: ArpOperation,
    pub oui: Option<String>,
    pub org: Option<String>,
}

impl fmt::Display for ArpPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ARP {:?}: {}:{:?} -> {}:{:?}",
            self.operation, self.sender_ip, self.sender_mac, self.target_ip, self.target_mac
        )?;

        if let (Some(oui), Some(org)) = (self.oui.clone(), self.org.clone()) {
            write!(f, " ({} at {})", oui, org)?
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize)]

pub enum ArpOperation {
    Request,
    Reply,
}

impl From<etherparse::ArpOperation> for ArpOperation {
    fn from(op: etherparse::ArpOperation) -> Self {
        match op {
            etherparse::ArpOperation::REQUEST => ArpOperation::Request,
            etherparse::ArpOperation::REPLY => ArpOperation::Reply,
            _ => return ArpOperation::Request,
        }
    }
}
#[derive(Debug, Clone, Copy, Serialize)]

pub struct HostEntry {
    pub ip: Ipv4Addr,
    pub mac: [u8; 6],
    pub last_seen: std::time::SystemTime,
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

#[derive(Debug)]
pub enum ScanError {
    UnsupportedChannel,
    ChannelOpen(std::io::Error),
    NoConfigFound,
    NoUsableRange,
    LargeNetwork,
    SendError(std::io::Error),
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScanError::UnsupportedChannel => write!(f, "unsupported channel type"),
            ScanError::ChannelOpen(e) => write!(f, "channel open failed: {}", e),
            ScanError::NoConfigFound => write!(f, "no IPv4 config on interface"),
            ScanError::NoUsableRange => write!(f, "prefix too large to scan"),
            ScanError::LargeNetwork => write!(f, "network exceeds host limit"),
            ScanError::SendError(e) => write!(f, "send error: {}", e),
        }
    }
}

impl std::error::Error for ScanError {}
