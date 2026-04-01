import React from "react";
import { colors, font } from "../theme";
import { StatusIndicator, Metric } from "./ui";

interface HeaderProps {
  connected: boolean;
  hostCount: number;
  packetCount: number;
  sessionCount: number;
  peerCount: number;
  serverPort: number;
}

export function Header({ connected, hostCount, packetCount, sessionCount, peerCount, serverPort }: HeaderProps) {
  return (
    <header
      style={{
        padding: "14px 24px",
        borderBottom: `1px solid ${colors.border}`,
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        background: colors.surface,
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
        <div
          style={{
            width: 32,
            height: 32,
            borderRadius: 8,
            background: colors.accent,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
          }}
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke={colors.bg} strokeWidth="2.5" strokeLinecap="round">
            <circle cx="12" cy="12" r="3" />
            <path d="M12 1v4M12 19v4M4.22 4.22l2.83 2.83M16.95 16.95l2.83 2.83M1 12h4M19 12h4M4.22 19.78l2.83-2.83M16.95 7.05l2.83-2.83" />
          </svg>
        </div>
        <div>
          <h1 style={{ margin: 0, fontSize: 16, fontWeight: 700, fontFamily: font.mono, letterSpacing: "-0.3px", color: colors.text }}>
            NETWORK SNIFFER
          </h1>
          <span style={{ fontSize: 11, color: colors.textMuted, fontFamily: font.sans }}>
            localhost:{serverPort}
          </span>
        </div>
      </div>

      <div style={{ display: "flex", alignItems: "center", gap: 16, minWidth: 0, flexShrink: 0 }}>
        <Metric label="Hosts" value={hostCount} color={colors.accent} />
        <Metric label="Packets" value={packetCount} color={colors.blue} />
        <Metric label="Sessions" value={sessionCount} color={colors.purple} />
        <Metric label="Peers" value={peerCount} color={colors.warn} />
        <div style={{ width: 1, height: 20, background: colors.border, flexShrink: 0 }} />
        <StatusIndicator connected={connected} />
      </div>
    </header>
  );
}