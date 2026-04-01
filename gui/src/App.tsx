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
import { MessageToast } from "./components/MessageToast";

import type {
  CapturedEvent,
  HostEntry,
  SessionEntry,
  PeerInfo,
  PeerMessage,
  Tab,
  Toast,
  MessageInbox as MessageInboxType,
} from "./types";

let toastCounter = 0;

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
  const [toasts, setToasts] = useState<Toast[]>([]);

  const [tab, setTab] = useState<Tab>("graph");
  const [scanning, setScanning] = useState(false);
  const [capturing, setCapturing] = useState(false);
  const [discoveryRunning, setDiscoveryRunning] = useState(false);

  const [iface, setIface] = useState("eth0");
  const [filter, setFilter] = useState("");
  const [hostLimit, setHostLimit] = useState(256);
  const [discoveryName, setDiscoveryName] = useState("MyDevice");
  const [discoveryPort, setDiscoveryPort] = useState(serverPort);

  const dismissToast = useCallback((id: number) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  const isSelfPeer = useCallback(
    (peer: PeerInfo) => peer.port === serverPort,
    [serverPort]
  );

  const handleEvent = useCallback((ev: CapturedEvent) => {
    if (ev.IncomingMessage) {
      const msg = ev.IncomingMessage;

      if (!isSelfPeer(msg.from)) {
        setToasts((prev) => [
          ...prev.slice(-4),
          { id: ++toastCounter, from: msg.from, content: msg.content },
        ]);
      }

      api.fetchMessages().then((m: any) => {
        if (m) setMessages(m);
      });

      return;
    }

    setEvents((prev) => [...prev.slice(-(MAX_EVENTS - 1)), ev]);
    setPackets((prev) => [...prev.slice(-199), ev]);
    setPacketCount((prev) => prev + 1);

    if (ev.Arp) {
      setHosts((prev) => {
        const ip = ev.Arp.sender_ip;
        const mac = ev.Arp.sender_mac;
        const existing = prev.find((h) => h.ip === ip);
        if (existing) {
          return prev.map((h) =>
            h.ip === ip ? { ...h, mac, last_seen: Date.now() } : h
          );
        }
        return [...prev, { ip, mac, last_seen: Date.now() }];
      });
    }

    if (ev.Discovery) {
      const d = ev.Discovery;
      if (d.operation === "Bye") {
        setPeers((prev) => prev.filter((p) => p.ip !== d.ip));
      }
    }
  }, [api, isSelfPeer]);

  const connected = useWebSocket(wsUrl, handleEvent);

  usePolling(
    setHosts,
    connected ? undefined : setPackets,
    setSessions,
    setPeers,
    setMessages,
  );

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

  const onStopScan = async () => {
    await api.stopScan();
    setScanning(false);
  };

  const onStartCapture = async () => {
    if (await api.startCapture(iface, filter)) {
      setCapturing(true);
      setPacketCount(0);
      setPackets([]);
      setEvents([]);
      setSessions([]);
    }
  };

  const onStopCapture = async () => {
    await api.stopCapture();
    setCapturing(false);
  };

  const onStartDiscovery = async () => {
    await ensureCapture();
    if (await api.startDiscovery(iface, discoveryName, discoveryPort)) {
      setDiscoveryRunning(true);
    }
  };

  const onStopDiscovery = async () => {
    await api.stopDiscovery();
    setDiscoveryRunning(false);
  };

  const handleLocalOutgoingMessage = useCallback((peer: PeerInfo, content: string) => {
    const selfPeer: PeerInfo = {
      name: `localhost:${serverPort}`,
      ip: "127.0.0.1",
      port: serverPort,
      last_seen: {
        secs_since_epoch: Math.floor(Date.now() / 1000),
        nanos_since_epoch: 0,
      },
    };

    const localMessage: PeerMessage = {
      from: selfPeer,
      content,
      outgoing: true,
    };

    setMessages((prev) => {
      const index = prev.findIndex(
        ([p]) => p.ip === peer.ip && p.port === peer.port
      );

      if (index === -1) {
        return [...prev, [peer, [localMessage]]];
      }

      return prev.map((entry, i) =>
        i === index ? [entry[0], [...entry[1], localMessage]] : entry
      );
    });
  }, [serverPort]);

  return (
    <div
      style={{
        background: colors.bg,
        color: colors.text,
        fontFamily: "'Inter', system-ui, sans-serif",
        height: "100vh",
        display: "flex",
        flexDirection: "column",
        overflow: "hidden",
      }}
    >
      <Header
        connected={connected}
        hostCount={hosts.length}
        packetCount={packetCount}
        sessionCount={sessions.length}
        peerCount={peers.length}
        serverPort={serverPort}
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
          discoveryName={discoveryName}
          setDiscoveryName={setDiscoveryName}
          discoveryPort={discoveryPort}
          setDiscoveryPort={setDiscoveryPort}
          scanning={scanning}
          capturing={capturing}
          discoveryRunning={discoveryRunning}
          onStartScan={onStartScan}
          onStopScan={onStopScan}
          onStartCapture={onStartCapture}
          onStopCapture={onStopCapture}
          onStartDiscovery={onStartDiscovery}
          onStopDiscovery={onStopDiscovery}
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
                  selfPort={serverPort}
                  onLocalOutgoingMessage={handleLocalOutgoingMessage}
                />
              </div>
            )}
          </div>
        </main>
      </div>

      <MessageToast toasts={toasts} onDismiss={dismissToast} />
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
      <AppInner
        serverPort={serverPort}
        wsUrl={wsUrl}
        onServerPortChange={handlePortChange}
      />
    </ApiContext.Provider>
  );
}