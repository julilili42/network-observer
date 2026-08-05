use crate::{
    api::Store,
    types::{
        ArpPacket, CapturedEvent, FilePayload, HostEntry, PeerEvent, PeerPayload, SessionKey,
        SessionStats, TransportPacket,
    },
};
use std::{collections::HashMap, net::Ipv4Addr, path::Path};
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

// do not add messages to event buffer
fn should_buffer(event: &CapturedEvent) -> bool {
    matches!(event, CapturedEvent::Transport(_) | CapturedEvent::Arp(_))
}

async fn handle_transport(store: &Store, packet: &TransportPacket) {
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

async fn handle_arp(store: &Store, packet: &ArpPacket) {
    let table = &mut store.hosts.write().await;
    accumulate_arp(table, packet);
}

async fn handle_peer_event(store: &Store, event: &PeerEvent) {
    match &event.payload {
        PeerPayload::Message(_) => {
            let mut messages = store.messages.write().await;
            messages
                .entry(event.from.clone())
                .or_default()
                .push(event.clone());
        }
        PeerPayload::File(FilePayload::Data { filename, data, .. }) => {
            save_file(filename, data).await;
        }
        _ => {}
    }
}

async fn save_file(filename: &str, data: &[u8]) {
    if let Err(e) = tokio::fs::create_dir_all("downloads").await {
        tracing::error!(error=%e, "failed to create download dir");
        return;
    }

    let safe_name = Path::new(filename)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown file");

    let path = format!("downloads/{}", safe_name);

    match tokio::fs::write(&path, data).await {
        Ok(_) => tracing::info!(filename = %safe_name, "File saved to downloads/"),
        Err(e) => tracing::error!(error = %e, "failed to write file"),
    }
}

async fn handle_event(store: &Store, event: &CapturedEvent) {
    match event {
        CapturedEvent::Transport(packet) => handle_transport(store, packet).await,
        CapturedEvent::Arp(packet) => handle_arp(store, packet).await,
        CapturedEvent::Peer(peer_event) => handle_peer_event(store, peer_event).await,
    }
}

pub fn spawn_event_processing(
    store: Store,
    mut internal_rx: mpsc::Receiver<CapturedEvent>,
    external_tx: broadcast::Sender<CapturedEvent>,
) {
    tokio::spawn(async move {
        while let Some(event) = internal_rx.recv().await {
            // save captured events
            if should_buffer(&event) {
                let mut buf = store.events.write().await;
                buf.push_back(event.clone());
                if buf.len() > 1000 {
                    buf.pop_front();
                }
            }

            // process captured events
            handle_event(&store, &event).await;

            // Captured event -> Broadcast channel -> Websocket
            let _ = external_tx.send(event);
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
