import React from "react";
import { colors, font } from "../theme";
import { THead, TRow, TD, Empty } from "./ui";
import { formatBytes } from "../utils/format";
import type { SessionEntry } from "../types";

interface SessionTableProps {
  sessions: SessionEntry[];
}

export function SessionTable({ sessions }: SessionTableProps) {
  const maxBytes = sessions.length > 0
    ? Math.max(...sessions.map(([, s]) => s.bytes_total))
    : 1;

  return (
    <div style={{ padding: 20 }}>
      <table style={{ width: "100%", borderCollapse: "collapse", fontFamily: font.mono, fontSize: 12 }}>
        <THead columns={["Endpoint A", "Endpoint B", "Packets", "Bytes", ""]} />
        <tbody>
          {sessions.length === 0 && (
            <tr>
              <td colSpan={5}><Empty message="No sessions recorded" /></td>
            </tr>
          )}
          {sessions.map(([key, stats], i) => {
            const pct = Math.min(100, (stats.bytes_total / maxBytes) * 100);
            return (
              <TRow key={i} index={i}>
                <TD color={colors.blue}>{key.a_ip}:{key.a_port}</TD>
                <TD color={colors.purple}>{key.b_ip}:{key.b_port}</TD>
                <TD>{stats.packets_total.toLocaleString()}</TD>
                <TD>{formatBytes(stats.bytes_total)}</TD>
                <TD>
                  <div style={{ height: 3, borderRadius: 2, background: colors.border, width: 72, overflow: "hidden" }}>
                    <div
                      style={{
                        height: "100%",
                        borderRadius: 2,
                        background: colors.accent,
                        width: `${pct}%`,
                        transition: "width 0.4s ease",
                      }}
                    />
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
