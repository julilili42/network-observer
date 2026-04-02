import React, { useEffect, useMemo, useRef, useState } from "react";
import { colors, font } from "../theme";
import { Input, Button, Empty } from "./ui";
import { useApi } from "../context/ApiContext";
import type { MessageInbox as MessageInboxType, PeerInfo, PeerMessage } from "../types";

interface MessageInboxProps {
  inbox: MessageInboxType;
  peers: PeerInfo[];
  onMessageSent: () => void;
}

export function MessageInbox({ inbox, peers, onMessageSent }: MessageInboxProps) {
  const api = useApi();
  const [selectedPeer, setSelectedPeer] = useState<PeerInfo | null>(null);
  const [draft, setDraft] = useState("");
  const [sending, setSending] = useState(false);
  const bottomRef = useRef<HTMLDivElement>(null);

  const allPeers = useMemo<PeerInfo[]>(() => {
    const map = new Map<string, PeerInfo>();
    for (const [peer] of inbox) {
      map.set(`${peer.ip}:${peer.port}`, peer);
    }
    for (const peer of peers) {
      map.set(`${peer.ip}:${peer.port}`, peer);
    }
    return Array.from(map.values());
  }, [inbox, peers]);

  useEffect(() => {
    if (!selectedPeer && allPeers.length > 0) { setSelectedPeer(allPeers[0]); return; }
    if (selectedPeer) {
      const refreshed = allPeers.find((p) => p.ip === selectedPeer.ip && p.port === selectedPeer.port);
      if (refreshed) setSelectedPeer(refreshed);
      else if (allPeers.length > 0) setSelectedPeer(allPeers[0]);
      else setSelectedPeer(null);
    }
  }, [allPeers, selectedPeer]);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [selectedPeer, inbox]);

  const selectedMessages: PeerMessage[] = selectedPeer
    ? (inbox.find(([peer]) => peer.ip === selectedPeer.ip && peer.port === selectedPeer.port)?.[1] ?? [])
    : [];

  const handleSend = async () => {
    if (!selectedPeer || !draft.trim()) return;
    const content = draft.trim();
    setSending(true);
    const ok = await api.sendPeerMessage(selectedPeer.name, content);
    setSending(false);
    if (ok) {
      setDraft("");
      onMessageSent();
    }
  };

  return (
    <div style={{ display: "flex", height: "100%", overflow: "hidden" }}>
      {/* Peer-Liste */}
      <div style={{ width: 240, minWidth: 240, borderRight: `1px solid ${colors.border}`, display: "flex", flexDirection: "column", overflow: "hidden", background: colors.surface }}>
        <div style={{ padding: "12px 14px", borderBottom: `1px solid ${colors.border}`, fontFamily: font.mono, fontSize: 10, fontWeight: 600, color: colors.textMuted, textTransform: "uppercase", letterSpacing: 1 }}>
          Conversations
        </div>
        <div style={{ flex: 1, overflowY: "auto" }}>
          {allPeers.length === 0 && (
            <div style={{ padding: 20, fontFamily: font.mono, fontSize: 11, color: colors.textMuted, textAlign: "center", lineHeight: 1.6 }}>
              Noch keine Peers.<br />mDNS läuft automatisch.
            </div>
          )}
          {allPeers.map((peer) => {
            const key = `${peer.ip}:${peer.port}`;
            const entry = inbox.find(([p]) => p.ip === peer.ip && p.port === peer.port);
            const msgs = entry?.[1] ?? [];
            const lastMsg = msgs[msgs.length - 1];
            const selected = selectedPeer?.ip === peer.ip && selectedPeer?.port === peer.port;

            return (
              <div key={key} onClick={() => setSelectedPeer(peer)} style={{ padding: "10px 14px", cursor: "pointer", borderBottom: `1px solid ${colors.border}`, background: selected ? colors.accentSoft : "transparent" }}>
                <div style={{ display: "flex", alignItems: "center", gap: 6, marginBottom: 3 }}>
                  <div style={{ width: 6, height: 6, borderRadius: "50%", background: colors.accent, flexShrink: 0 }} />
                  <span style={{ fontFamily: font.mono, fontSize: 11, fontWeight: 700, color: selected ? colors.accent : colors.text, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                    {peer.name}
                  </span>
                  {msgs.length > 0 && (
                    <span style={{ marginLeft: "auto", fontFamily: font.mono, fontSize: 9, color: colors.textMuted }}>{msgs.length}</span>
                  )}
                </div>
                <div style={{ fontFamily: font.mono, fontSize: 10, color: colors.textMuted, paddingLeft: 12 }}>{peer.ip}:{peer.port}</div>
                {lastMsg && (
                  <div style={{ fontFamily: font.sans, fontSize: 11, color: colors.textSecondary, paddingLeft: 12, marginTop: 2, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                    {lastMsg.outgoing ? `Du: ${lastMsg.content}` : lastMsg.content}
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </div>

      {/* Chat-Bereich */}
      <div style={{ flex: 1, display: "flex", flexDirection: "column", overflow: "hidden" }}>
        {!selectedPeer ? (
          <div style={{ flex: 1, display: "flex", alignItems: "center", justifyContent: "center" }}>
            <Empty message="Wähle einen Peer aus" />
          </div>
        ) : (
          <>
            <div style={{ padding: "12px 20px", borderBottom: `1px solid ${colors.border}`, display: "flex", alignItems: "center", gap: 10, background: colors.surface }}>
              <div style={{ width: 8, height: 8, borderRadius: "50%", background: colors.accent }} />
              <div>
                <div style={{ fontFamily: font.mono, fontSize: 13, fontWeight: 700, color: colors.text }}>{selectedPeer.name}</div>
                <div style={{ fontFamily: font.mono, fontSize: 10, color: colors.textMuted }}>{selectedPeer.ip}:{selectedPeer.port}</div>
              </div>
            </div>

            <div style={{ flex: 1, overflowY: "auto", padding: "16px 20px", display: "flex", flexDirection: "column", gap: 10 }}>
              {selectedMessages.length === 0 && (
                <div style={{ flex: 1, display: "flex", alignItems: "center", justifyContent: "center" }}>
                  <Empty message="Noch keine Nachrichten" />
                </div>
              )}
              {selectedMessages.map((msg, i) => <MessageBubble key={i} msg={msg} />)}
              <div ref={bottomRef} />
            </div>

            <div style={{ padding: "12px 20px", borderTop: `1px solid ${colors.border}`, display: "flex", gap: 10, background: colors.surface }}>
              <Input
                value={draft}
                onChange={(e) => setDraft(e.target.value)}
                placeholder={`Nachricht an ${selectedPeer.name}…`}
                onKeyDown={(e) => { if (e.key === "Enter" && !sending) void handleSend(); }}
                style={{ flex: 1 }}
              />
              <Button
                onClick={handleSend}
                disabled={sending || !draft.trim()}
                color={colors.purple}
                style={{ flex: "none", minWidth: 90 }}
              >
                {sending ? "Sende…" : "Senden"}
              </Button>
            </div>
          </>
        )}
      </div>
    </div>
  );
}

function MessageBubble({ msg }: { msg: PeerMessage }) {
  const outgoing = msg.outgoing;
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 4, maxWidth: "80%", alignSelf: outgoing ? "flex-end" : "flex-start" }}>
      <div style={{ display: "flex", alignItems: "baseline", gap: 8, justifyContent: outgoing ? "flex-end" : "flex-start" }}>
        <span style={{ fontFamily: font.mono, fontSize: 10, fontWeight: 700, color: outgoing ? colors.accent : colors.purple }}>
          {outgoing ? "Du" : msg.from.name}
        </span>
        <span style={{ fontFamily: font.mono, fontSize: 9, color: colors.textMuted }}>{msg.from.ip}:{msg.from.port}</span>
      </div>
      <div style={{ padding: "8px 12px", background: outgoing ? `${colors.accent}12` : `${colors.purple}12`, border: outgoing ? `1px solid ${colors.accent}25` : `1px solid ${colors.purple}25`, borderRadius: outgoing ? "12px 4px 12px 12px" : "4px 12px 12px 12px", fontFamily: font.sans, fontSize: 13, color: colors.text, lineHeight: 1.5, wordBreak: "break-word" }}>
        {msg.content}
      </div>
    </div>
  );
}