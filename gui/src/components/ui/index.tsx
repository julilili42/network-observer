import React from "react";
import { colors, font } from "../../theme";

/* ── Label ─────────────────────────────────────────── */

export function Label({ children, style }: { children: React.ReactNode; style?: React.CSSProperties }) {
  return (
    <div
      style={{
        fontSize: 10,
        fontWeight: 600,
        fontFamily: font.mono,
        color: colors.textMuted,
        textTransform: "uppercase",
        letterSpacing: 1,
        marginBottom: 6,
        ...style,
      }}
    >
      {children}
    </div>
  );
}

/* ── Input ─────────────────────────────────────────── */

export function Input(props: React.InputHTMLAttributes<HTMLInputElement>) {
  const { style, ...rest } = props;
  return (
    <input
      {...rest}
      style={{
        width: "100%",
        boxSizing: "border-box",
        background: colors.bg,
        border: `1px solid ${colors.border}`,
        borderRadius: 6,
        padding: "7px 10px",
        color: colors.text,
        fontFamily: font.mono,
        fontSize: 12,
        outline: "none",
        transition: "border-color 0.2s",
        ...style,
      }}
      onFocus={(e) => { e.currentTarget.style.borderColor = colors.accent; }}
      onBlur={(e) => { e.currentTarget.style.borderColor = colors.border; }}
    />
  );
}

/* ── Button ────────────────────────────────────────── */

interface ButtonProps {
  children: React.ReactNode;
  onClick: () => void;
  disabled?: boolean;
  color?: string;
  style?: React.CSSProperties;
}

export function Button({ children, onClick, disabled, color = colors.accent, style }: ButtonProps) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      style={{
        flex: 1,
        padding: "7px 12px",
        background: disabled ? colors.border : `${color}10`,
        border: `1px solid ${disabled ? colors.border : `${color}40`}`,
        borderRadius: 6,
        color: disabled ? colors.textMuted : color,
        fontFamily: font.mono,
        fontSize: 11,
        fontWeight: 600,
        cursor: disabled ? "not-allowed" : "pointer",
        transition: "all 0.15s",
        ...style,
      }}
    >
      {children}
    </button>
  );
}

/* ── Status Indicator ──────────────────────────────── */

export function StatusIndicator({ connected }: { connected: boolean }) {
  const c = connected ? colors.accent : colors.danger;
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
      <div
        style={{
          width: 6,
          height: 6,
          borderRadius: "50%",
          background: c,
          boxShadow: connected ? `0 0 6px ${colors.accentBorder}` : "none",
        }}
      />
      <span style={{ fontFamily: font.mono, fontSize: 10, fontWeight: 600, color: c }}>
        {connected ? "LIVE" : "OFFLINE"}
      </span>
    </div>
  );
}

/* ── Metric ────────────────────────────────────────── */

function formatMetricValue(value: number): string {
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(1)}k`;
  return value.toString();
}

export function Metric({ label, value, color }: { label: string; value: number; color: string }) {
  return (
    <div
      style={{
        display: "flex",
        alignItems: "baseline",
        gap: 6,
        minWidth: 0,
        whiteSpace: "nowrap",
      }}
    >
      <span
        style={{
          fontFamily: font.mono,
          fontSize: 10,
          color: colors.textMuted,
          flexShrink: 0,
        }}
      >
        {label}
      </span>
      <span
        style={{
          fontFamily: font.mono,
          fontSize: 13,
          fontWeight: 700,
          color,
          fontVariantNumeric: "tabular-nums",
          whiteSpace: "nowrap",
        }}
      >
        {formatMetricValue(value)}
      </span>
    </div>
  );
}

/* ── Empty State ───────────────────────────────────── */

export function Empty({ message }: { message: string }) {
  return (
    <div
      style={{
        padding: 60,
        textAlign: "center",
        color: colors.textMuted,
        fontFamily: font.mono,
        fontSize: 12,
      }}
    >
      {message}
    </div>
  );
}

/* ── Table helpers ─────────────────────────────────── */

export function THead({ columns }: { columns: string[] }) {
  return (
    <thead>
      <tr style={{ borderBottom: `1px solid ${colors.border}` }}>
        {columns.map((col) => (
          <th
            key={col}
            style={{
              textAlign: "left",
              padding: "10px 14px",
              color: colors.textMuted,
              fontFamily: font.mono,
              fontWeight: 600,
              fontSize: 10,
              textTransform: "uppercase",
              letterSpacing: 0.8,
            }}
          >
            {col}
          </th>
        ))}
      </tr>
    </thead>
  );
}

export function TRow({ children, index }: { children: React.ReactNode; index: number }) {
  const bg = index % 2 === 0 ? "transparent" : "rgba(255,255,255,0.015)";
  return (
    <tr
      style={{ borderBottom: `1px solid ${colors.border}`, background: bg, transition: "background 0.12s" }}
      onMouseEnter={(e) => { e.currentTarget.style.background = colors.accentSoft; }}
      onMouseLeave={(e) => { e.currentTarget.style.background = bg; }}
    >
      {children}
    </tr>
  );
}

export function TD({ children, color, style }: { children: React.ReactNode; color?: string; style?: React.CSSProperties }) {
  return (
    <td style={{ padding: "8px 14px", color: color ?? colors.text, fontFamily: font.mono, fontSize: 12, ...style }}>
      {children}
    </td>
  );
}

export function Badge({ label, color }: { label: string; color: string }) {
  return (
    <span
      style={{
        padding: "2px 8px",
        borderRadius: 4,
        fontSize: 10,
        fontWeight: 600,
        fontFamily: font.mono,
        background: `${color}15`,
        color,
      }}
    >
      {label}
    </span>
  );
}
