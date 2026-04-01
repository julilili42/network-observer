import React, { useState } from "react";
import { colors, font } from "../theme";
import { THead, TRow, TD, Empty, Badge, Input, Button } from "./ui";
import { formatTimestamp } from "../utils/format";
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
    const ok = await api.sendPeerMessage(peer.ip, peer.port, message);
    setSending((prev) => ({ ...prev, [key]: false }));

    if (ok) {
      setDrafts((prev) => ({ ...prev, [key]: "" }));
    }
  };

  return (
    <div style={{ padding: 20 }}>
      <div
        style={{
          marginBottom: 16,
          padding: "10px 14px",
          background: colors.surface,
          border: `1px solid ${colors.border}`,
          borderRadius: 8,
          fontFamily: font.mono,
          fontSize: 11,
          color: colors.textMuted,
          display: "flex",
          alignItems: "center",
          gap: 8,
        }}
      >
        <span
          style={{
            width: 6,
            height: 6,
            borderRadius: "50%",
            background: peers.length > 0 ? colors.accent : colors.textMuted,
            boxShadow: peers.length > 0 ? `0 0 6px ${colors.accentBorder}` : "none",
            flexShrink: 0,
          }}
        />
        {peers.length === 0
          ? "Keine aktiven Peers — starte Discovery um Peers zu finden"
          : `${peers.length} aktive${peers.length === 1 ? "r" : ""} Peer${peers.length === 1 ? "" : "s"} im Netzwerk`}
      </div>

      <table style={{ width: "100%", borderCollapse: "collapse", fontFamily: font.mono, fontSize: 12 }}>
        <THead columns={["Name", "IP Adresse", "Port", "Zuletzt gesehen", "Status", "Nachricht"]} />
        <tbody>
          {peers.length === 0 && (
            <tr>
              <td colSpan={6}>
                <Empty message="Starte Discovery um Peers zu entdecken" />
              </td>
            </tr>
          )}
          {peers.map((peer, i) => {
            const key = peerKey(peer);
            const seenSecs = peer.last_seen.secs_since_epoch;
            const nowSecs = Math.floor(Date.now() / 1000);
            const ageSecs = nowSecs - seenSecs;
            const isActive = ageSecs < 35;

            return (
              <TRow key={key} index={i}>
                <TD color={colors.accent} style={{ fontWeight: 600 }}>{peer.name}</TD>
                <TD color={colors.blue}>{peer.ip}</TD>
                <TD color={colors.textSecondary}>{peer.port}</TD>
                <TD color={colors.textSecondary}>{formatTimestamp(peer.last_seen)}</TD>
                <TD>
                  <Badge
                    label={isActive ? "ONLINE" : "STALE"}
                    color={isActive ? colors.accent : colors.warn}
                  />
                </TD>
                <TD>
                  <div style={{ display: "flex", gap: 8, minWidth: 260 }}>
                    <Input
                      value={drafts[key] ?? ""}
                      onChange={(e) => {
                        const value = e.target.value;
                        setDrafts((prev) => ({ ...prev, [key]: value }));
                      }}
                      placeholder="Nachricht senden…"
                      onKeyDown={(e) => {
                        if (e.key === "Enter" && !sending[key]) void handleSend(peer);
                      }}
                    />
                    <Button
                      onClick={() => void handleSend(peer)}
                      disabled={!isActive || sending[key] || !(drafts[key] ?? "").trim()}
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