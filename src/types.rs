use core::fmt;
use serde::{Deserialize, Serialize};
use std::{hash::Hash, net::Ipv4Addr};

use crate::message::PeerMessage;

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

#[derive(Debug, Clone, Serialize)]
pub enum CapturedEvent {
    Transport(TransportPacket),
    Arp(ArpPacket),
    IncomingMessage(PeerMessage),
}

impl fmt::Display for CapturedEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CapturedEvent::Transport(packet) => write!(
                f,
                "Transport: {}:{} -> {}:{}",
                packet.src_ip, packet.src_port, packet.dst_ip, packet.dst_port
            ),
            CapturedEvent::Arp(packet) => {
                write!(
                    f,
                    "ARP {:?}: {}:{:?} -> {}:{:?}",
                    packet.operation,
                    packet.sender_ip,
                    packet.sender_mac,
                    packet.target_ip,
                    packet.target_mac
                )?;

                if let (Some(oui), Some(org)) = (packet.oui.clone(), packet.org.clone()) {
                    write!(f, " ({} at {})", oui, org)?
                }

                Ok(())
            }
            CapturedEvent::IncomingMessage(message) => {
                write!(
                    f,
                    "MESSAGE {:?} received from {:?}",
                    message.content, message.from
                )?;
                Ok(())
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
