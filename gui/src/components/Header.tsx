import { colors, font } from "../theme";
import { StatusIndicator, Metric } from "./ui";

interface HeaderProps {
  connected: boolean;
  hostCount: number;
  packetCount: number;
  sessionCount: number;
  peerCount: number;
}

export function Header({ connected, hostCount, packetCount, sessionCount, peerCount }: HeaderProps) {
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
      <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
        <div
          style={{
            border: `1px solid ${colors.accent}55`,
            borderRadius: 6,
            color: colors.accent,
            fontFamily: font.mono,
            fontSize: 11,
            fontWeight: 700,
            padding: "4px 7px",
          }}
        >
          LAN
        </div>
        <h1 style={{ margin: 0, fontSize: 16, fontWeight: 700, fontFamily: font.mono, letterSpacing: "-0.3px", color: colors.text }}>
          NETWORK OBSERVER
        </h1>
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
