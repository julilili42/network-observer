use std::net::Ipv4Addr;

use etherparse::{ArpPacketSlice, Ipv4Slice, SlicedPacket, TransportSlice};
use oui_data::lookup;
use pcap::Packet;

use crate::constants::DISCOVERY_PORT;
use crate::types::{
    ArpOperation, ArpPacket, CapturedEvent, DiscoveryPacket, TransportPacket, TransportProtocol,
};

extern crate pnet;

pub fn parse_packet(captured_packet: &Packet) -> Option<CapturedEvent> {
    let len = captured_packet.len();
    let sliced = SlicedPacket::from_ethernet(captured_packet.data).ok()?;

    match sliced.net? {
        etherparse::NetSlice::Arp(arp) => parse_arp_packet(arp),
        etherparse::NetSlice::Ipv4(ip) => parse_transport_packet(ip, sliced.transport?, len),
        _ => None,
    }
}

fn parse_arp_packet(arp: ArpPacketSlice) -> Option<CapturedEvent> {
    let (sender_bytes, target_bytes): ([u8; 4], [u8; 4]) = (
        arp.sender_protocol_addr().try_into().unwrap(),
        arp.target_protocol_addr().try_into().unwrap(),
    );
    let (sender_ip, target_ip) = (Ipv4Addr::from(sender_bytes), Ipv4Addr::from(target_bytes));
    let (sender_mac, target_mac): ([u8; 6], [u8; 6]) = (
        arp.sender_hw_addr().try_into().unwrap(),
        arp.target_hw_addr().try_into().unwrap(),
    );
    let operation = ArpOperation::from(arp.operation());

    let (oui, org) = match operation {
        ArpOperation::Reply => {
            let lookup_mac: String = sender_mac[..3]
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect();
            match lookup(&lookup_mac) {
                Some(record) => (
                    Some(record.oui().to_string()),
                    Some(record.organization().to_string()),
                ),
                None => (None, None),
            }
        }
        ArpOperation::Request => (None, None),
    };

    Some(CapturedEvent::Arp(ArpPacket {
        sender_ip,
        sender_mac,
        target_ip,
        target_mac,
        operation,
        oui,
        org,
    }))
}

fn parse_transport_packet(
    ip: Ipv4Slice,
    transport: TransportSlice,
    len: usize,
) -> Option<CapturedEvent> {
    let (src_ip, dst_ip) = (
        Ipv4Addr::from(ip.header().source()),
        Ipv4Addr::from(ip.header().destination()),
    );

    let (src_port, dst_port, payload, protocol) = match transport {
        etherparse::TransportSlice::Udp(u) => (
            u.source_port(),
            u.destination_port(),
            u.payload(),
            TransportProtocol::Udp,
        ),
        etherparse::TransportSlice::Tcp(t) => (
            t.source_port(),
            t.destination_port(),
            t.payload(),
            TransportProtocol::Tcp,
        ),
        _ => return None,
    };

    if src_port == DISCOVERY_PORT || dst_port == DISCOVERY_PORT {
        let payload = payload;
        let packet: DiscoveryPacket = serde_json::from_slice(payload).ok()?;
        return Some(CapturedEvent::Discovery(packet));
    };

    Some(CapturedEvent::Transport(TransportPacket {
        src_ip,
        src_port,
        dst_ip,
        dst_port,
        len,
        protocol,
    }))
}
