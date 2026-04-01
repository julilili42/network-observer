use std::{
    net::Ipv4Addr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use crate::types::{DiscoveryOperation, DiscoveryPacket};
use crate::{constants::DISCOVERY_PORT, types::DiscoveryError};
use tokio::{net::UdpSocket, time::interval};

pub async fn discovery_sender(
    name: String,
    ip: Ipv4Addr,
    port: u16,
    running: Arc<AtomicBool>,
) -> Result<(), DiscoveryError> {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))
        .await
        .map_err(|e| DiscoveryError::Bind(e))?;
    socket
        .set_broadcast(true)
        .map_err(|_| DiscoveryError::Broadcast)?;

    let operation = DiscoveryOperation::Hello;

    let packet = DiscoveryPacket {
        name,
        ip,
        port,
        operation,
    };
    // serialized json string converted into bytes
    // '{"name":"MacBook","ip":"192.168.1.5","port":3000}' => [123, 34, 110, 97, 109, 101, 34, ...]
    let json = serde_json::to_vec(&packet).map_err(|_| DiscoveryError::Serialize)?;
    let broadcast_addr = format!("{}:{}", Ipv4Addr::BROADCAST, DISCOVERY_PORT);

    let mut ticker = interval(Duration::from_secs(10));

    while running.load(Ordering::Relaxed) {
        ticker.tick().await;
        let _ = socket.send_to(&json, &broadcast_addr).await;
    }

    // send bye when discovery is stopped
    let bye = DiscoveryPacket {
        operation: DiscoveryOperation::Bye,
        ..packet
    };
    let bye_json = serde_json::to_vec(&bye).map_err(|_| DiscoveryError::Serialize)?;
    let _ = socket.send_to(&bye_json, &broadcast_addr).await;

    Ok(())
}
