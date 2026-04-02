import React, { useState } from "react";
import { colors, font } from "../theme";
import { THead, TRow, TD, Empty, Badge, Input, Button } from "./ui";
import { useApi } from "../context/ApiContext";
import type { PeerInfo } from "../types";

interface PeerTableProps {
  peers: PeerInfo[];
}

export function PeerTable({ peers }: PeerTableProps) {
  const api = useApi();
  const [drafts, setDrafts] = useState<Record<string, string>>({});
  const [sending, setSending] = useState<Record<string, boolean>>({});

  const peerKey = (peer: PeerInfo) => `${peer.ip}:${peer.port}`;

  const handleSend = async (peer: PeerInfo) => {
    const key = peerKey(peer);
    const message = (drafts[key] ?? "").trim();
    if (!message) return;

    setSending((prev) => ({ ...prev, [key]: true }));
    const ok = await api.sendPeerMessage(peer.name, message);
    setSending((prev) => ({ ...prev, [key]: false }));

    if (ok) setDrafts((prev) => ({ ...prev, [key]: "" }));
  };

  return (
    <div style={{ padding: 20 }}>
      <div style={{ marginBottom: 16, padding: "10px 14px", background: colors.surface, border: `1px solid ${colors.border}`, borderRadius: 8, fontFamily: font.mono, fontSize: 11, color: colors.textMuted, display: "flex", alignItems: "center", gap: 8 }}>
        <span style={{ width: 6, height: 6, borderRadius: "50%", background: peers.length > 0 ? colors.accent : colors.textMuted, boxShadow: peers.length > 0 ? `0 0 6px ${colors.accentBorder}` : "none", flexShrink: 0 }} />
        {peers.length === 0
          ? "Keine aktiven Peers — mDNS läuft automatisch"
          : `${peers.length} aktive${peers.length === 1 ? "r" : ""} Peer${peers.length === 1 ? "" : "s"} im Netzwerk`}
      </div>

      <table style={{ width: "100%", borderCollapse: "collapse", fontFamily: font.mono, fontSize: 12 }}>
        <THead columns={["Name", "IP Adresse", "Port", "Status", "Nachricht"]} />
        <tbody>
          {peers.length === 0 && (
            <tr><td colSpan={5}><Empty message="Noch keine Peers entdeckt" /></td></tr>
          )}
          {peers.map((peer, i) => {
            const key = peerKey(peer);
            return (
              <TRow key={key} index={i}>
                <TD color={colors.accent} style={{ fontWeight: 600 }}>{peer.name}</TD>
                <TD color={colors.blue}>{peer.ip}</TD>
                <TD color={colors.textSecondary}>{peer.port}</TD>
                <TD><Badge label="ONLINE" color={colors.accent} /></TD>
                <TD>
                  <div style={{ display: "flex", gap: 8, minWidth: 260 }}>
                    <Input
                      value={drafts[key] ?? ""}
                      onChange={(e) => setDrafts((prev) => ({ ...prev, [key]: e.target.value }))}
                      placeholder="Nachricht senden…"
                      onKeyDown={(e) => { if (e.key === "Enter" && !sending[key]) void handleSend(peer); }}
                    />
                    <Button
                      onClick={() => void handleSend(peer)}
                      disabled={sending[key] || !(drafts[key] ?? "").trim()}
                      color={colors.purple}
                      style={{ flex: "none", minWidth: 78 }}
                    >
                      {sending[key] ? "Sende…" : "Senden"}
                    </Button>
                  </div>
                </TD>
              </TRow>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}