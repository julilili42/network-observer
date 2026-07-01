import { useState, useCallback, useMemo } from "react";
import { colors } from "./theme";
import { MAX_EVENTS, DEFAULT_SERVER_PORT, getApiBase, getWsUrl } from "./utils/config";
import { createApi } from "./utils/api";
import { ApiContext, useApi } from "./context/ApiContext";
import { useWebSocket } from "./hooks/useWebSocket";
import { usePolling } from "./hooks/usePolling";

import { Header } from "./components/Header";
import { Sidebar } from "./components/Sidebar";
import { TabBar } from "./components/TabBar";
import { NetworkGraph } from "./components/NetworkGraph";
import { PacketTable } from "./components/PacketTable";
import { SessionTable } from "./components/SessionTable";
import { HostTable } from "./components/HostTable";
import { PeerTable } from "./components/PeerTable";
import { MessageInbox } from "./components/MessageInbox";

import type {
  CapturedEvent,
  HostEntry,
  SessionEntry,
  PeerInfo,
  Tab,
  MessageInbox as MessageInboxType,
} from "./types";

function AppInner({
  serverPort,
  wsUrl,
  onServerPortChange,
}: {
  serverPort: number;
  wsUrl: string;
  onServerPortChange: (port: number) => void;
}) {
  const api = useApi();

  const [hosts, setHosts] = useState<HostEntry[]>([]);
  const [packets, setPackets] = useState<CapturedEvent[]>([]);
  const [sessions, setSessions] = useState<SessionEntry[]>([]);
  const [peers, setPeers] = useState<PeerInfo[]>([]);
  const [events, setEvents] = useState<CapturedEvent[]>([]);
  const [messages, setMessages] = useState<MessageInboxType>([]);
  const [packetCount, setPacketCount] = useState(0);

  const [tab, setTab] = useState<Tab>("graph");
  const [scanning, setScanning] = useState(false);
  const [capturing, setCapturing] = useState(false);

  const [iface, setIface] = useState("eth0");
  const [filter, setFilter] = useState("");
  const [hostLimit, setHostLimit] = useState(256);

  const handleEvent = useCallback((ev: CapturedEvent) => {
    if (ev.Peer?.payload.Message) {
      api.fetchMessages().then((m: any) => { if (m) setMessages(m); });
      return;
    }

    setEvents((prev) => [...prev.slice(-(MAX_EVENTS - 1)), ev]);
    setPackets((prev) => [...prev.slice(-199), ev]);
    setPacketCount((prev) => prev + 1);

    if (ev.Arp) {
      setHosts((prev) => {
        const ip = ev.Arp!.sender_ip;
        const mac = ev.Arp!.sender_mac;
        const existing = prev.find((h) => h.ip === ip);
        if (existing) {
          return prev.map((h) =>
            h.ip === ip ? { ...h, mac, last_seen: Date.now() } : h
          );
        }
        return [...prev, { ip, mac, last_seen: Date.now() }];
      });
    }
  }, [api]);

  const connected = useWebSocket(wsUrl, handleEvent);

  usePolling(setHosts, connected ? undefined : setPackets, setSessions, setPeers, setMessages);

  const ensureCapture = async (): Promise<boolean> => {
    if (capturing) return true;
    const ok = await api.startCapture(iface, filter);
    if (ok) {
      setCapturing(true);
      setPacketCount(0);
      setPackets([]);
      setEvents([]);
      setSessions([]);
    }
    return ok;
  };

  const onStartScan = async () => {
    await ensureCapture();
    if (await api.startScan(iface, hostLimit)) {
      setHosts([]);
      setScanning(true);
    }
  };

  const onStopScan = async () => { await api.stopScan(); setScanning(false); };

  const onStartCapture = async () => {
    if (await api.startCapture(iface, filter)) {
      setCapturing(true);
      setPacketCount(0);
      setPackets([]);
      setEvents([]);
      setSessions([]);
    }
  };

  const onStopCapture = async () => { await api.stopCapture(); setCapturing(false); };

  return (
    <div style={{ background: colors.bg, color: colors.text, fontFamily: "'Inter', system-ui, sans-serif", height: "100vh", display: "flex", flexDirection: "column", overflow: "hidden" }}>
      <Header
        connected={connected}
        hostCount={hosts.length}
        packetCount={packetCount}
        sessionCount={sessions.length}
        peerCount={peers.length}
      />

      <div style={{ display: "flex", flex: 1, overflow: "hidden" }}>
        <Sidebar
          serverPort={serverPort}
          onServerPortChange={onServerPortChange}
          iface={iface}
          setIface={setIface}
          hostLimit={hostLimit}
          setHostLimit={setHostLimit}
          filter={filter}
          setFilter={setFilter}
          scanning={scanning}
          capturing={capturing}
          onStartScan={onStartScan}
          onStopScan={onStopScan}
          onStartCapture={onStartCapture}
          onStopCapture={onStopCapture}
          events={events}
        />

        <main style={{ flex: 1, display: "flex", flexDirection: "column", overflow: "hidden" }}>
          <TabBar active={tab} onChange={setTab} />
          <div style={{ flex: 1, overflow: "auto", position: "relative" }}>
            {tab === "graph" && <NetworkGraph hosts={hosts} />}
            {tab === "packets" && <PacketTable packets={packets} />}
            {tab === "sessions" && <SessionTable sessions={sessions} />}
            {tab === "hosts" && <HostTable hosts={hosts} />}
            {tab === "peers" && <PeerTable peers={peers} />}
            {tab === "messages" && (
              <div style={{ height: "100%", overflow: "hidden" }}>
                <MessageInbox
                  inbox={messages}
                  peers={peers}
                  onMessageSent={() => api.fetchMessages().then((m: any) => { if (m) setMessages(m); })}
                />
              </div>
            )}
          </div>
        </main>
      </div>

    </div>
  );
}

export default function App() {
  const [serverPort, setServerPort] = useState<number>(() => {
    const saved = localStorage.getItem("serverPort");
    return saved ? parseInt(saved, 10) : DEFAULT_SERVER_PORT;
  });

  const api = useMemo(() => createApi(getApiBase(serverPort)), [serverPort]);
  const wsUrl = getWsUrl(serverPort);

  const handlePortChange = (port: number) => {
    localStorage.setItem("serverPort", String(port));
    setServerPort(port);
  };

  return (
    <ApiContext.Provider value={api}>
      <AppInner serverPort={serverPort} wsUrl={wsUrl} onServerPortChange={handlePortChange} />
    </ApiContext.Provider>
  );
}
