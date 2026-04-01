use std::{
    net::{IpAddr, Ipv4Addr},
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use std::sync::Arc;

extern crate pnet;

use pnet::datalink::{Channel::Ethernet, DataLinkSender};
use pnet::packet::Packet as PnetPacket;
use pnet::packet::ethernet::MutableEthernetPacket;
use pnet::{
    datalink::{self, NetworkInterface},
    packet::{
        MutablePacket,
        arp::{ArpHardwareTypes, ArpOperations, MutableArpPacket},
        ethernet::EtherTypes,
    },
    util::MacAddr,
};

use crate::types::ScanError;

fn construct_ethernet_frame<'a>(
    buffer: &'a mut [u8],
    src_mac: MacAddr,
    dst_mac: MacAddr,
) -> Option<MutableEthernetPacket<'a>> {
    let mut ethernet_packet = MutableEthernetPacket::new(buffer)?;

    ethernet_packet.set_source(src_mac);
    ethernet_packet.set_destination(dst_mac);
    ethernet_packet.set_ethertype(EtherTypes::Arp);

    Some(ethernet_packet)
}

fn construct_arp_request<'a>(
    buffer: &'a mut [u8],
    sender_mac: MacAddr,
    sender_ip: Ipv4Addr,
    target_ip: Ipv4Addr,
) -> Option<MutableEthernetPacket<'a>> {
    let mut ethernet_packet = construct_ethernet_frame(buffer, sender_mac, MacAddr::broadcast())?;

    let payload = ethernet_packet.payload_mut();
    let mut arp_packet = MutableArpPacket::new(payload)?;
    arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
    arp_packet.set_protocol_type(EtherTypes::Ipv4);
    arp_packet.set_hw_addr_len(6);
    arp_packet.set_proto_addr_len(4);
    arp_packet.set_operation(ArpOperations::Request);
    arp_packet.set_sender_hw_addr(sender_mac);
    arp_packet.set_sender_proto_addr(sender_ip);
    arp_packet.set_target_hw_addr(MacAddr::zero());
    arp_packet.set_target_proto_addr(target_ip);

    Some(ethernet_packet)
}

pub fn arp_scan(
    interface: NetworkInterface,
    sender_mac: MacAddr,
    sender_ip: Ipv4Addr,
    running: Arc<AtomicBool>,
    host_limit: u32,
) -> Result<(), ScanError> {
    let (mut tx, _): (Box<dyn DataLinkSender>, _) =
        match datalink::channel(&interface, Default::default()) {
            Ok(Ethernet(tx, rx)) => (tx, rx),
            Ok(_) => return Err(ScanError::UnsupportedChannel),
            Err(e) => return Err(ScanError::ChannelOpen(e)),
        };

    let Some((ip, prefix)) = get_nework_config(&interface) else {
        running.store(false, Ordering::Relaxed);
        return Err(ScanError::NoConfigFound);
    };

    if prefix >= 31 {
        running.store(false, Ordering::Relaxed);
        return Err(ScanError::NoUsableRange);
    }

    let (start, end) = cidr_range(ip, prefix);

    let host_count = end.saturating_sub(start).saturating_sub(1);
    if host_count > host_limit {
        running.store(false, Ordering::Relaxed);
        return Err(ScanError::LargeNetwork);
    } else {
        tracing::debug!(name = %interface.name, ip = %ip, prefix = %prefix, count = %host_count, "Start ARP scan");
        tracing::debug!(start = %u32_to_ipv4(start), end = %u32_to_ipv4(end), "Range");
    }

    for ip in (start + 1)..end {
        if !running.load(Ordering::Relaxed) {
            break;
        }
        let target_ip = u32_to_ipv4(ip);
        if target_ip == sender_ip {
            continue;
        }
        let mut ethernet_buffer = [0u8; 42];

        match construct_arp_request(&mut ethernet_buffer, sender_mac, sender_ip, target_ip) {
            Some(packet) => match tx.send_to(packet.packet(), None) {
                Some(Ok(_)) => tracing::debug!(target = %target_ip, "Sending ARP"),
                Some(Err(e)) => return Err(ScanError::SendError(e)),
                None => {}
            },
            None => {}
        }

        std::thread::sleep(Duration::from_millis(2));
    }
    running.store(false, Ordering::Relaxed);
    Ok(())
}

fn get_nework_config(interface: &NetworkInterface) -> Option<(Ipv4Addr, u8)> {
    interface.ips.iter().find_map(|ip| {
        if let IpAddr::V4(ipv4) = ip.ip() {
            Some((ipv4, ip.prefix()))
        } else {
            None
        }
    })
}

fn ipv4_to_u32(ip: Ipv4Addr) -> u32 {
    u32::from_be_bytes(ip.octets())
}

fn u32_to_ipv4(n: u32) -> Ipv4Addr {
    Ipv4Addr::from(n)
}

fn cidr_range(ip: Ipv4Addr, prefix: u8) -> (u32, u32) {
    let ip_u32 = ipv4_to_u32(ip);

    let mask = if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix)
    };

    let network = ip_u32 & mask;
    let broadcast = network | !mask;

    (network, broadcast)
}
