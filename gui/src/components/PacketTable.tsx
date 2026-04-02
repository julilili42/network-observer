import React from "react";
import { colors, font } from "../theme";
import { THead, TRow, TD, Badge, Empty } from "./ui";
import type { CapturedEvent } from "../types";

interface PacketTableProps {
  packets: CapturedEvent[];
}

export function PacketTable({ packets }: PacketTableProps) {
  const rows = packets.slice(-200).reverse();

  return (
    <div style={{ padding: 20 }}>
      <table
        style={{
          width: "100%",
          borderCollapse: "collapse",
          fontFamily: font.mono,
          fontSize: 12,
        }}
      >
        <THead columns={["Type", "Source", "Destination", "Info", "Size"]} />
        <tbody>
          {rows.length === 0 && (
            <tr>
              <td colSpan={5}>
                <Empty message="No packets captured yet" />
              </td>
            </tr>
          )}
          {rows.map((pkt, i) => {
            const t = pkt.Transport;
            const a = pkt.Arp;

            return (
              <TRow key={i} index={i}>
                <TD>
                  <Badge
                    label={t ? "TCP/UDP" : "ARP"}
                    color={t ? colors.blue : colors.accent}
                  />
                </TD>
                <TD>
                  {t ? `${t.src_ip}:${t.src_port}` : a ? a.sender_ip : "—"}
                </TD>
                <TD>
                  {t ? `${t.dst_ip}:${t.dst_port}` : a ? a.target_ip : "—"}
                </TD>
                <TD color={colors.textSecondary}>
                  {t ? t.protocol : a ? a.operation : "—"}
                </TD>
                <TD color={colors.textSecondary}>{t ? `${t.len}` : "—"}</TD>
              </TRow>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
