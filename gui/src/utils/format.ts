import type { CapturedEvent, HostEntry } from "../types";

export function formatMac(mac: number[] | string | undefined): string {
  if (!mac) return "—";
  if (typeof mac === "string") return mac;
  return mac.map((b) => b.toString(16).padStart(2, "0")).join(":");
}

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1048576).toFixed(1)} MB`;
}

export function formatTimestamp(
  ts:
    | HostEntry["last_seen"]
    | { secs_since_epoch: number; nanos_since_epoch: number }
    | undefined
): string {
  if (!ts) return "—";
  if (typeof ts === "number") return new Date(ts).toLocaleTimeString();
  if (typeof ts === "object" && "secs_since_epoch" in ts) {
    return new Date(ts.secs_since_epoch * 1000).toLocaleTimeString();
  }
  return "—";
}

export function formatEvent(ev: CapturedEvent): string {
  if (ev.Arp) {
    return `ARP ${ev.Arp.operation}  ${ev.Arp.sender_ip} → ${ev.Arp.target_ip}`;
  }
  if (ev.Transport) {
    const t = ev.Transport;
    return `${t.protocol}  ${t.src_ip}:${t.src_port} → ${t.dst_ip}:${t.dst_port}`;
  }
  return "Unknown event";
}
