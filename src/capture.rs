use crate::parser::parse_packet;
use crate::types::CapturedEvent;
use pcap::{Activated, Capture, Device};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;

extern crate pnet;

pub fn capture_packets(
    capturing_device: Device,
    filter_str: &str,
    packet_tx: mpsc::Sender<CapturedEvent>,
    running: Arc<AtomicBool>,
) -> Result<(), pcap::Error> {
    let mut cap = Capture::from_device(capturing_device)?
        .immediate_mode(true)
        .open()?
        .setnonblock()?;

    cap.filter(filter_str, true)?;

    run_capture_loop(running, cap, packet_tx)
}

// both Offline and Active implement the Activated trait
fn run_capture_loop<T: Activated>(
    running: Arc<AtomicBool>,
    mut capture: Capture<T>,
    packet_tx: mpsc::Sender<CapturedEvent>,
) -> Result<(), pcap::Error> {
    while running.load(Ordering::Relaxed) {
        match capture.next_packet() {
            Ok(packet) => {
                if let Some(packet_info) = parse_packet(&packet) {
                    tracing::debug!(packet = %packet_info, "captured");
                    let _ = packet_tx.blocking_send(packet_info);
                }
            }
            Err(pcap::Error::TimeoutExpired) => {
                std::thread::sleep(std::time::Duration::from_millis(10));
                continue;
            }
            Err(pcap::Error::NoMorePackets) => {
                break;
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TransportProtocol;
    use std::sync::atomic::AtomicBool;

    #[test]
    fn capture_packet_is_sent_as_event() {
        let capture = Capture::from_file("tests/fixtures/one_udp_packet.pcap")
            .expect("fixture should be readable");

        let running = Arc::new(AtomicBool::new(true));
        let (packet_tx, mut packet_rx) = mpsc::channel(16);

        run_capture_loop(running, capture, packet_tx).expect("capture should succeed");

        let event = packet_rx
            .blocking_recv()
            .expect("an packet should have been sent");
        let CapturedEvent::Transport(packet) = event else {
            panic!("expected transport packet, got {event:?}")
        };

        assert_eq!(matches!(packet.protocol, TransportProtocol::Udp), true);
    }
}
