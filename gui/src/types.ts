export interface TransportPacket {
  src_ip: string;
  src_port: number;
  dst_ip: string;
  dst_port: number;
  len: number;
  protocol: "Tcp" | "Udp";
}

export interface ArpPacket {
  sender_ip: string;
  sender_mac: number[];
  target_ip: string;
  target_mac: number[];
  operation: "Request" | "Reply";
  oui: string | null;
  org: string | null;
}

export interface DiscoveryPacket {
  name: string;
  ip: string;
  port: number;
  operation: "Hello" | "Bye";
}

export interface PeerInfo {
  name: string;
  ip: string;
  port: number;
  last_seen: { secs_since_epoch: number; nanos_since_epoch: number };
}

export interface PeerMessage {
  from: PeerInfo;
  content: string;
  outgoing?: boolean;
}

export type MessagesEntry = [PeerInfo, PeerMessage[]];

// Backend returns Vec<(PeerInfo, Vec<PeerMessage>)>
export type MessageInbox = MessagesEntry[];

export type CapturedEvent =
  | { Transport: TransportPacket; Arp?: never; Discovery?: never; IncomingMessage?: never }
  | { Arp: ArpPacket; Transport?: never; Discovery?: never; IncomingMessage?: never }
  | { Discovery: DiscoveryPacket; Transport?: never; Arp?: never; IncomingMessage?: never }
  | { IncomingMessage: PeerMessage; Transport?: never; Arp?: never; Discovery?: never };

export interface HostEntry {
  ip: string;
  mac: number[] | string;
  last_seen: number | { secs_since_epoch: number; nanos_since_epoch: number };
}

export interface SessionKey {
  a_ip: string;
  a_port: number;
  b_ip: string;
  b_port: number;
}

export interface SessionStats {
  packets_total: number;
  bytes_total: number;
}

export type SessionEntry = [SessionKey, SessionStats];

export type Tab = "graph" | "packets" | "sessions" | "hosts" | "peers" | "messages";

export interface Toast {
  id: number;
  from: PeerInfo;
  content: string;
}