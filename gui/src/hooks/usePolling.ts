import { useEffect } from "react";
import { POLL_INTERVAL_MS } from "../utils/config";
import { useApi } from "../context/ApiContext";
import type {
  HostEntry,
  CapturedEvent,
  SessionEntry,
  PeerInfo,
  MessageInbox,
} from "../types";

export function usePolling(
  setHosts: React.Dispatch<React.SetStateAction<HostEntry[]>>,
  setPackets?: React.Dispatch<React.SetStateAction<CapturedEvent[]>>,
  setSessions?: React.Dispatch<React.SetStateAction<SessionEntry[]>>,
  setPeers?: React.Dispatch<React.SetStateAction<PeerInfo[]>>,
  setMessages?: React.Dispatch<React.SetStateAction<MessageInbox>>,
) {
  const api = useApi();

  useEffect(() => {
    const poll = async () => {
      const [h, p, s, peers, messages] = await Promise.all([
        api.fetchHosts(),
        setPackets ? api.fetchPackets() : Promise.resolve(null),
        setSessions ? api.fetchSessions() : Promise.resolve(null),
        setPeers ? api.fetchPeers() : Promise.resolve(null),
        setMessages ? api.fetchMessages() : Promise.resolve(null),
      ]);

      if (h) setHosts(h);
      if (p && setPackets) setPackets(p);
      if (s && setSessions) setSessions(s);
      if (peers && setPeers) setPeers(peers);
      if (messages && setMessages) setMessages(messages);
    };

    poll();
    const id = setInterval(poll, POLL_INTERVAL_MS);
    return () => clearInterval(id);
  }, [api, setHosts, setPackets, setSessions, setPeers, setMessages]);
}