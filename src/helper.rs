extern crate pnet;
use pcap::Device;
use pnet::datalink::{self, NetworkInterface};
use reqwest::StatusCode;
use std::{
    net::Ipv4Addr,
    sync::atomic::{AtomicBool, Ordering},
};

pub fn get_interface_ipv4(interface: &NetworkInterface) -> Option<Ipv4Addr> {
    interface.ips.iter().find_map(|ip| match ip.ip() {
        std::net::IpAddr::V4(ipv4) => Some(ipv4),
        _ => None,
    })
}

pub fn find_pnet_interface(interface_name: &str) -> Option<NetworkInterface> {
    let interface_names_match = |iface: &NetworkInterface| iface.name == interface_name;

    // Find the network interface with the provided name
    let interfaces = datalink::interfaces();
    let interface = interfaces.into_iter().filter(interface_names_match).next();

    interface
}

pub fn find_pcap_interface(interface_name: &str) -> Result<Device, pcap::Error> {
    let devices = Device::list()?;

    for d in &devices {
        tracing::debug!("pcap device: {}", d.name);
    }

    let device = devices
        .into_iter()
        .find(|d| d.name == interface_name)
        .ok_or(pcap::Error::PcapError("Interface not found".into()));

    device
}

// change running flags
pub fn change_flag(flag: &AtomicBool) -> Result<(), StatusCode> {
    flag.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .map(|_| ())
        .map_err(|_| StatusCode::TOO_MANY_REQUESTS)
}
