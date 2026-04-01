use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr},
    sync::Arc,
    time::SystemTime,
};

use crate::types::PeerInfo;
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use tokio::sync::RwLock;

pub fn start(
    name: String,
    ip: Ipv4Addr,
    port: u16,
    peers: Arc<RwLock<HashMap<Ipv4Addr, PeerInfo>>>,
) -> ServiceDaemon {
    let mdns = ServiceDaemon::new().expect("mDNS daemon failed");

    register_service(&mdns, name, ip, port);
    browse_services(&mdns, peers);

    mdns
}

pub fn register_service(mdns: &ServiceDaemon, name: String, ip: Ipv4Addr, port: u16) {
    let service_type = "_network_sniffer._tcp.local.";
    let instance_name = format!("{}.{}", name, service_type);
    let host_name = format!("{}.local.", name);

    println!("{}", host_name);

    let my_service = ServiceInfo::new(
        service_type,
        &instance_name,
        &host_name,
        ip.to_string(),
        port,
        None,
    )
    .unwrap();

    // Register with the daemon, which publishes the service.
    mdns.register(my_service)
        .expect("Failed to register our service");
}

pub fn browse_services(mdns: &ServiceDaemon, peers: Arc<RwLock<HashMap<Ipv4Addr, PeerInfo>>>) {
    let service_type = "_network_sniffer._tcp.local.";

    let receiver = mdns.browse(service_type).expect("Browse failed");

    std::thread::spawn(move || {
        for event in receiver {
            if let ServiceEvent::ServiceResolved(info) = event {
                if let Some(addr) = info.get_addresses().iter().next() {
                    if let IpAddr::V4(ipv4) = addr.to_ip_addr() {
                        let peer = PeerInfo {
                            name: info.get_hostname().to_string(),
                            ip: ipv4,
                            port: info.get_port(),
                            last_seen: SystemTime::now(),
                        };

                        let mut peers = peers.blocking_write();
                        peers.insert(peer.ip, peer);
                    }
                }
            }
        }
    });
}
