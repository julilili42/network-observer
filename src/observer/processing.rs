use crate::{
    api::types::ApiEvent,
    observer::types::{
        ArpPacket, HostEntry, ObserverEvent, ObserverStore, SessionKey, SessionStats,
        TransportPacket,
    },
};
use std::{collections::HashMap, net::Ipv4Addr};
extern crate pnet;
use tokio::sync::{broadcast, mpsc};

fn normalize_session(session: SessionKey) -> SessionKey {
    if (session.a_ip, session.a_port) < (session.b_ip, session.b_port) {
        session
    } else {
        SessionKey {
            a_ip: session.b_ip,
            a_port: session.b_port,
            b_ip: session.a_ip,
            b_port: session.a_port,
        }
    }
}

fn accumulate_stats(
    session_map: &mut HashMap<SessionKey, SessionStats>,
    session_key: SessionKey,
    packet_len: usize,
) {
    let normalized_key = normalize_session(session_key);

    let stats = session_map
        .entry(normalized_key)
        .or_insert_with(|| SessionStats {
            packets_total: 0,
            bytes_total: 0,
        });

    stats.packets_total += 1;
    stats.bytes_total += packet_len;
}

fn accumulate_arp(arp_table: &mut HashMap<Ipv4Addr, HostEntry>, packet: &ArpPacket) {
    arp_table.insert(
        packet.sender_ip,
        HostEntry {
            ip: packet.sender_ip,
            mac: packet.sender_mac,
            last_seen: std::time::SystemTime::now(),
        },
    );
}

async fn handle_transport(store: &ObserverStore, packet: &TransportPacket) {
    let session_map = &mut store.sessions.write().await;
    accumulate_stats(
        session_map,
        SessionKey {
            a_ip: packet.src_ip,
            a_port: packet.src_port,
            b_ip: packet.dst_ip,
            b_port: packet.dst_port,
        },
        packet.packet_len,
    );
}

async fn handle_arp(store: &ObserverStore, packet: &ArpPacket) {
    let table = &mut store.hosts.write().await;
    accumulate_arp(table, packet);
}

async fn handle_event(store: &ObserverStore, event: &ObserverEvent) {
    match event {
        ObserverEvent::Transport(packet) => handle_transport(store, packet).await,
        ObserverEvent::Arp(packet) => handle_arp(store, packet).await,
    }
}

pub fn spawn_observer_processing(
    store: ObserverStore,
    mut observer_rx: mpsc::Receiver<ObserverEvent>,
    api_tx: broadcast::Sender<ApiEvent>,
) {
    tokio::spawn(async move {
        while let Some(event) = observer_rx.recv().await {
            let mut events = store.events.write().await;
            events.push_back(event.clone());
            if events.len() > 1000 {
                events.pop_front();
            }
            drop(events);

            // process captured events
            handle_event(&store, &event).await;

            // Captured event -> Broadcast channel -> Websocket
            let _ = api_tx.send(event.into());
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_reverse_direction_to_same_session() {
        let session = SessionKey {
            a_ip: Ipv4Addr::LOCALHOST,
            a_port: 8080,
            b_ip: Ipv4Addr::BROADCAST,
            b_port: 4040,
        };
        let reverse_session = SessionKey {
            a_ip: Ipv4Addr::BROADCAST,
            a_port: 4040,
            b_ip: Ipv4Addr::LOCALHOST,
            b_port: 8080,
        };
        assert_eq!(
            normalize_session(session),
            normalize_session(reverse_session)
        )
    }

    #[test]
    fn accumulates_packet_count_and_bytes() {
        let mut session_map: HashMap<SessionKey, SessionStats> = HashMap::new();
        let session_key = SessionKey {
            a_ip: Ipv4Addr::LOCALHOST,
            a_port: 8080,
            b_ip: Ipv4Addr::BROADCAST,
            b_port: 4040,
        };

        accumulate_stats(&mut session_map, session_key, 100);
        accumulate_stats(&mut session_map, session_key, 50);

        let value = session_map
            .get(&session_key)
            .expect("a session should have been inserted");
        assert_eq!(value.bytes_total, 150);
        assert_eq!(value.packets_total, 2);
        assert_eq!(session_map.len(), 1);
    }
}
