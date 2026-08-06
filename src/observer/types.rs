use core::fmt;
use serde::Serialize;
use std::{
    collections::{HashMap, VecDeque},
    hash::Hash,
    net::Ipv4Addr,
    sync::{Arc, atomic::AtomicBool},
};
use tokio::sync::RwLock;

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

#[derive(Clone)]
pub struct ObserverStore {
    pub events: Arc<RwLock<VecDeque<ObserverEvent>>>,
    pub hosts: Arc<RwLock<HashMap<Ipv4Addr, HostEntry>>>,
    pub sessions: Arc<RwLock<HashMap<SessionKey, SessionStats>>>,
}

#[derive(Clone)]
pub struct Flags {
    pub capture_running: Arc<AtomicBool>,
    pub scan_running: Arc<AtomicBool>,
}

#[derive(Debug, Clone, Serialize)]
pub enum ObserverEvent {
    Transport(TransportPacket),
    Arp(ArpPacket),
}

impl fmt::Display for ObserverEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObserverEvent::Transport(transport_packet) => {
                write!(f, "{transport_packet}")
            }
            ObserverEvent::Arp(arp_packet) => {
                write!(f, "{arp_packet}")
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
    pub protocol: TransportProtocol,
    pub packet_len: usize,
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
            _ => ArpOperation::Request,
        }
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

#[derive(Debug, Clone, Copy, Serialize)]

pub struct HostEntry {
    pub ip: Ipv4Addr,
    pub mac: [u8; 6],
    pub last_seen: std::time::SystemTime,
}
