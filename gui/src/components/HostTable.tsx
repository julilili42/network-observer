import React from "react";
import { colors, font } from "../theme";
import { THead, TRow, TD, Empty } from "./ui";
import { formatMac, formatTimestamp } from "../utils/format";
import type { HostEntry } from "../types";

interface HostTableProps {
  hosts: HostEntry[];
}

export function HostTable({ hosts }: HostTableProps) {
  return (
    <div style={{ padding: 20 }}>
      <table style={{ width: "100%", borderCollapse: "collapse", fontFamily: font.mono, fontSize: 12 }}>
        <THead columns={["IP Address", "MAC Address", "Last Seen"]} />
        <tbody>
          {hosts.length === 0 && (
            <tr>
              <td colSpan={3}><Empty message="No hosts discovered" /></td>
            </tr>
          )}
          {hosts.map((h, i) => (
            <TRow key={h.ip} index={i}>
              <TD color={colors.accent} style={{ fontWeight: 600 }}>{h.ip}</TD>
              <TD color={colors.textSecondary}>{formatMac(h.mac)}</TD>
              <TD color={colors.textSecondary}>{formatTimestamp(h.last_seen)}</TD>
            </TRow>
          ))}
        </tbody>
      </table>
    </div>
  );
}
