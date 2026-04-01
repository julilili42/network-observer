use crate::parser::parse_packet;
use crate::types::CapturedEvent;
use pcap::{Capture, Device};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

extern crate pnet;

pub fn capture_packets(
    capturing_device: Device,
    filter_str: &str,
    packet_tx: tokio::sync::mpsc::Sender<CapturedEvent>,
    running: Arc<AtomicBool>,
) -> Result<(), pcap::Error> {
    let mut cap = Capture::from_device(capturing_device)?
        .immediate_mode(true)
        .open()?
        .setnonblock()?;

    cap.filter(filter_str, true)?;

    while running.load(Ordering::Relaxed) {
        match cap.next_packet() {
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
            Err(e) => {
                tracing::error!(error = %e, "packet capture failed");
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
    }

    Ok(())
}
