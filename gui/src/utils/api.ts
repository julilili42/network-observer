import type { HostEntry, CapturedEvent, SessionEntry, SessionKey, SessionStats, PeerInfo, MessageInbox } from "../types";

export function createApi(baseUrl: string) {
  async function post<T>(path: string, body: T): Promise<boolean> {
    try {
      const res = await fetch(`${baseUrl}${path}`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
      });
      if (!res.ok) {
        const text = await res.text().catch(() => "");
        console.error(`POST ${path} failed`, res.status, text);
        return false;
      }
      return true;
    } catch (err) {
      console.error(`POST ${path} error`, err);
      return false;
    }
  }

  async function del(path: string): Promise<boolean> {
    try {
      const res = await fetch(`${baseUrl}${path}`, { method: "DELETE" });
      if (!res.ok) {
        const text = await res.text().catch(() => "");
        console.error(`DELETE ${path} failed`, res.status, text);
        return false;
      }
      return true;
    } catch (err) {
      console.error(`DELETE ${path} error`, err);
      return false;
    }
  }

  async function get<T>(path: string): Promise<T | null> {
    try {
      const res = await fetch(`${baseUrl}${path}`);
      if (!res.ok) {
        console.error(`GET ${path} failed`, res.status);
        return null;
      }
      return res.json();
    } catch (err) {
      console.error(`GET ${path} error`, err);
      return null;
    }
  }

  function isSessionKey(value: unknown): value is SessionKey {
    if (!value || typeof value !== "object") return false;
    const v = value as Record<string, unknown>;
    return (
      typeof v.a_ip === "string" &&
      typeof v.a_port === "number" &&
      typeof v.b_ip === "string" &&
      typeof v.b_port === "number"
    );
  }

  function isSessionStats(value: unknown): value is SessionStats {
    if (!value || typeof value !== "object") return false;
    const v = value as Record<string, unknown>;
    return (
      typeof v.packets_total === "number" &&
      typeof v.bytes_total === "number"
    );
  }

  function normalizeSessions(data: unknown): SessionEntry[] | null {
    if (!Array.isArray(data)) return null;
    const result: SessionEntry[] = [];
    for (const item of data) {
      if (!Array.isArray(item) || item.length !== 2) continue;
      const [key, stats] = item;
      if (!isSessionKey(key) || !isSessionStats(stats)) continue;
      result.push([key, stats]);
    }
    return result;
  }

  return {
    startScan: (iface: string, hostLimit: number) =>
      post("/scan", { interface: iface, host_limit: hostLimit }),
    stopScan: () => del("/scan"),

    startCapture: (iface: string, filter: string) =>
      post("/capture", { interface: iface, filter }),
    stopCapture: () => del("/capture"),

    startDiscovery: (iface: string, name: string, port: number) =>
      post("/discovery", { interface: iface, name, port }),
    stopDiscovery: () => del("/discovery"),

    sendPeerMessage: (ip: string, port: number, content: string) =>
      post("/peers/outgoing_message", { ip, port, content }),

    fetchHosts: () => get<HostEntry[]>("/hosts"),
    fetchPackets: () => get<CapturedEvent[]>("/packets"),
    fetchPeers: () => get<PeerInfo[]>("/peers"),
    fetchMessages: () => get<MessageInbox>("/peers/messages"),
    fetchSessions: async (): Promise<SessionEntry[] | null> => {
      const raw = await get<unknown>("/sessions");
      return normalizeSessions(raw);
    },
  };
}

export type ApiInstance = ReturnType<typeof createApi>;
