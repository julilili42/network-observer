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

export interface PeerInfo {
  name: string;
  ip: string;
  port: number;
}

export interface PeerMessage {
  from: PeerInfo;
  content: string;
  outgoing: boolean;
}

export interface PeerEvent {
  from: PeerInfo;
  payload: { Message?: { content: string; outgoing: boolean } };
}

export type MessagesEntry = [PeerInfo, PeerMessage[]];
export type MessageInbox = MessagesEntry[];

export type CapturedEvent =
  | { Transport: TransportPacket; Arp?: never; Peer?: never }
  | { Arp: ArpPacket; Transport?: never; Peer?: never }
  | { Peer: PeerEvent; Transport?: never; Arp?: never };

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
