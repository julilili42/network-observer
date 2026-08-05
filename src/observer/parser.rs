use crate::types::{ArpOperation, ArpPacket, CapturedEvent, TransportPacket, TransportProtocol};
use etherparse::{ArpPacketSlice, Ipv4Slice, NetSlice, SlicedPacket, TransportSlice};
use oui_data::lookup;
use std::net::Ipv4Addr;
extern crate pnet;

pub fn parse_packet(captured_packet: &[u8]) -> Option<CapturedEvent> {
    let sliced = SlicedPacket::from_ethernet(captured_packet).ok()?;
    let packet_len = captured_packet.len();
    match (sliced.net, sliced.transport) {
        (Some(NetSlice::Arp(arp)), _) => parse_arp_packet(arp),
        (Some(NetSlice::Ipv4(ip)), Some(transport_slice)) => {
            parse_flow(ip, transport_slice, packet_len)
        }
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
    let (oui, org) = oui_lookup(operation, sender_mac);

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

fn parse_flow(
    ipv4_slice: Ipv4Slice,
    transport_slice: TransportSlice,
    packet_len: usize,
) -> Option<CapturedEvent> {
    let (src_ip, dst_ip) = (
        Ipv4Addr::from(ipv4_slice.header().source()),
        Ipv4Addr::from(ipv4_slice.header().destination()),
    );

    let (src_port, dst_port, protocol) = match transport_slice {
        TransportSlice::Udp(udp_slice) => (
            udp_slice.source_port(),
            udp_slice.destination_port(),
            TransportProtocol::Udp,
        ),
        TransportSlice::Tcp(tcp_slice) => (
            tcp_slice.source_port(),
            tcp_slice.destination_port(),
            TransportProtocol::Tcp,
        ),
        _ => return None,
    };

    Some(CapturedEvent::Transport(TransportPacket {
        src_ip,
        src_port,
        dst_ip,
        dst_port,
        protocol,
        packet_len,
    }))
}

fn oui_lookup(operation: ArpOperation, sender_mac: [u8; 6]) -> (Option<String>, Option<String>) {
    match operation {
        ArpOperation::Reply => {
            let lookup_mac: String = sender_mac[..3]
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect();

            lookup(&lookup_mac).map_or((None, None), |record| {
                (
                    Some(record.oui().to_string()),
                    Some(record.organization().to_string()),
                )
            })
        }
        ArpOperation::Request => (None, None),
    }
}
