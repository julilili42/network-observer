use super::types::PeerInfo;
use mdns_sd::Error;
use mdns_sd::{ResolvedService, ServiceDaemon, ServiceEvent, ServiceInfo};
use std::{collections::HashMap, net::Ipv4Addr, sync::Arc};
use tokio::{sync::RwLock, task};

pub fn start_mdns(
    name: String,
    ip: Ipv4Addr,
    port: u16,
    peers: Arc<RwLock<HashMap<Ipv4Addr, PeerInfo>>>,
) -> Result<ServiceDaemon, Error> {
    let mdns = ServiceDaemon::new()?;

    register_service(&mdns, name.clone(), ip, port)?;
    browse_services(&mdns, name, ip, peers)?;

    Ok(mdns)
}

fn register_service(
    mdns: &ServiceDaemon,
    name: String,
    ip: Ipv4Addr,
    port: u16,
) -> Result<(), Error> {
    let service_type = "_network_sniffer._tcp.local.";
    let host_name = format!("{}.local.", name);

    let my_service = ServiceInfo::new(service_type, &name, &host_name, ip.to_string(), port, None)?;

    // Register with the daemon, which publishes the service.
    mdns.register(my_service)?;
    Ok(())
}

async fn resolve_service(
    info: Box<ResolvedService>,
    peers: &Arc<RwLock<HashMap<Ipv4Addr, PeerInfo>>>,
    self_ip: Ipv4Addr,
    self_name: &str,
) {
    if let Some(ipv4) = info.get_addresses_v4().iter().next() {
        let peer = PeerInfo {
            name: info.get_hostname().trim_end_matches(".local.").to_string(),
            ip: *ipv4,
            port: info.get_port(),
        };

        if peer.name == self_name || peer.ip == self_ip {
            return;
        }

        let mut peers = peers.write().await;
        peers.insert(peer.ip, peer);
    }
}

fn browse_services(
    mdns: &ServiceDaemon,
    self_name: String,
    self_ip: Ipv4Addr,
    peers: Arc<RwLock<HashMap<Ipv4Addr, PeerInfo>>>,
) -> Result<(), Error> {
    let service_type = "_network_sniffer._tcp.local.";

    let receiver = mdns.browse(service_type)?;

    task::spawn(async move {
        while let Ok(event) = receiver.recv_async().await {
            match event {
                ServiceEvent::ServiceResolved(info) => {
                    resolve_service(info, &peers, self_ip, &self_name).await;
                }
                ServiceEvent::ServiceRemoved(_type, fullname) => {
                    peers
                        .write()
                        .await
                        .retain(|_, p| !fullname.starts_with(&format!("{}.", p.name)));
                }
                _ => {}
            }
        }
    });

    Ok(())
}
