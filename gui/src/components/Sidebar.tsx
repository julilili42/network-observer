import { useState } from "react";
import { colors, font } from "../theme";
import { Label, Input, Button } from "./ui";
import { LiveFeed } from "./LiveFeed";
import type { CapturedEvent } from "../types";

interface SidebarProps {
  serverPort: number;
  onServerPortChange: (port: number) => void;
  iface: string;
  setIface: (v: string) => void;
  hostLimit: number;
  setHostLimit: (v: number) => void;
  filter: string;
  setFilter: (v: string) => void;
  scanning: boolean;
  capturing: boolean;
  onStartScan: () => void;
  onStopScan: () => void;
  onStartCapture: () => void;
  onStopCapture: () => void;
  events: CapturedEvent[];
}

const PRESET_PORTS = [3000, 3001, 3002];

export function Sidebar({
  serverPort,
  onServerPortChange,
  iface,
  setIface,
  hostLimit,
  setHostLimit,
  filter,
  setFilter,
  scanning,
  capturing,
  onStartScan,
  onStopScan,
  onStartCapture,
  onStopCapture,
  events,
}: SidebarProps) {
  const [customPort, setCustomPort] = useState("");

  const handleCustomPort = () => {
    const p = parseInt(customPort, 10);
    if (p > 0 && p < 65536) {
      onServerPortChange(p);
      setCustomPort("");
    }
  };

  return (
    <aside
      style={{
        width: 264,
        minWidth: 264,
        borderRight: `1px solid ${colors.border}`,
        background: colors.surface,
        display: "flex",
        flexDirection: "column",
        overflow: "hidden",
      }}
    >
      <div
        style={{
          padding: "18px 16px",
          display: "flex",
          flexDirection: "column",
          gap: 16,
          overflowY: "auto",
        }}
      >
        {/* Server Port */}
        <div>
          <Label>Server Port</Label>
          <div style={{ display: "flex", gap: 5, marginBottom: 6 }}>
            {PRESET_PORTS.map((p) => (
              <button
                key={p}
                onClick={() => onServerPortChange(p)}
                style={{
                  flex: 1,
                  padding: "6px 0",
                  borderRadius: 6,
                  border: `1px solid ${
                    serverPort === p ? colors.accent + "60" : colors.border
                  }`,
                  background:
                    serverPort === p ? colors.accent + "15" : "transparent",
                  color: serverPort === p ? colors.accent : colors.textMuted,
                  fontFamily: font.mono,
                  fontSize: 11,
                  fontWeight: 600,
                  cursor: "pointer",
                  transition: "all 0.15s",
                }}
              >
                {p}
              </button>
            ))}
          </div>
          <div style={{ display: "flex", gap: 6 }}>
            <Input
              value={customPort}
              onChange={(e) => setCustomPort(e.currentTarget.value)}
              placeholder="Custom port…"
              type="number"
              onKeyDown={(e) => {
                if (e.key === "Enter") handleCustomPort();
              }}
              style={{ flex: 1 }}
            />
            <Button
              onClick={handleCustomPort}
              disabled={!customPort}
              color={colors.blue}
              style={{ flex: "none", padding: "7px 10px" }}
            >
              OK
            </Button>
          </div>
        </div>

        <div style={{ height: 1, background: colors.border, margin: "0 -16px" }} />

        {/* Interface */}
        <div>
          <Label>Interface</Label>
          <Input
            value={iface}
            onChange={(e) => setIface(e.currentTarget.value)}
            placeholder="eth0"
          />
        </div>

        {/* Packet Capture */}
        <div>
          <Label>Packet Capture</Label>
          <Input
            value={filter}
            onChange={(e) => setFilter(e.currentTarget.value)}
            placeholder="BPF filter"
          />
          <div style={{ display: "flex", gap: 6, marginTop: 8 }}>
            <Button
              onClick={onStartCapture}
              disabled={capturing}
              color={colors.blue}
            >
              {capturing ? "Capturing…" : "Start Capture"}
            </Button>
            {capturing && (
              <Button
                onClick={onStopCapture}
                color={colors.danger}
                style={{ flex: "none", padding: "7px 10px" }}
              >
                Stop
              </Button>
            )}
          </div>
        </div>

        {/* ARP Scan */}
        <div>
          <Label>ARP Scan</Label>
          <Input
            value={hostLimit}
            onChange={(e) =>
              setHostLimit(parseInt(e.currentTarget.value || "0", 10))
            }
            placeholder="Host limit"
            type="number"
          />
          <div style={{ display: "flex", gap: 6, marginTop: 8 }}>
            <Button
              onClick={onStartScan}
              disabled={scanning}
              color={colors.accent}
            >
              {scanning ? "Scanning…" : "Start Scan"}
            </Button>
            {scanning && (
              <Button
                onClick={onStopScan}
                color={colors.danger}
                style={{ flex: "none", padding: "7px 10px" }}
              >
                Stop
              </Button>
            )}
          </div>
        </div>
      </div>

      <LiveFeed events={events} />
    </aside>
  );
}
