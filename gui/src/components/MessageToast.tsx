import React, { useEffect } from "react";
import { colors, font } from "../theme";
import type { Toast } from "../types";

interface MessageToastProps {
  toasts: Toast[];
  onDismiss: (id: number) => void;
}

export function MessageToast({ toasts, onDismiss }: MessageToastProps) {
  return (
    <div
      style={{
        position: "fixed",
        bottom: 24,
        right: 24,
        zIndex: 9999,
        display: "flex",
        flexDirection: "column",
        gap: 10,
        pointerEvents: "none",
      }}
    >
      {toasts.map((toast) => (
        <ToastItem key={toast.id} toast={toast} onDismiss={onDismiss} />
      ))}
    </div>
  );
}

function ToastItem({ toast, onDismiss }: { toast: Toast; onDismiss: (id: number) => void }) {
  useEffect(() => {
    const timer = setTimeout(() => onDismiss(toast.id), 5000);
    return () => clearTimeout(timer);
  }, [toast.id, onDismiss]);

  return (
    <div
      style={{
        pointerEvents: "auto",
        display: "flex",
        flexDirection: "column",
        gap: 4,
        padding: "12px 14px",
        background: colors.surface,
        border: `1px solid ${colors.purple}40`,
        borderLeft: `3px solid ${colors.purple}`,
        borderRadius: 10,
        boxShadow: "0 8px 24px rgba(0,0,0,0.4)",
        minWidth: 260,
        maxWidth: 360,
        cursor: "pointer",
      }}
      onClick={() => onDismiss(toast.id)}
    >
      <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
        <div
          style={{
            width: 6,
            height: 6,
            borderRadius: "50%",
            background: colors.purple,
            flexShrink: 0,
          }}
        />
        <span
          style={{
            fontFamily: font.mono,
            fontSize: 10,
            fontWeight: 700,
            color: colors.purple,
            textTransform: "uppercase",
            letterSpacing: 0.8,
          }}
        >
          Nachricht von {toast.from.name}
        </span>
        <span
          style={{
            marginLeft: "auto",
            fontFamily: font.mono,
            fontSize: 10,
            color: colors.textMuted,
          }}
        >
          ×
        </span>
      </div>
      <span
        style={{
          fontFamily: font.mono,
          fontSize: 13,
          color: colors.text,
          paddingLeft: 14,
          lineHeight: 1.4,
          wordBreak: "break-word",
        }}
      >
        {toast.content}
      </span>
      <span
        style={{
          fontFamily: font.mono,
          fontSize: 10,
          color: colors.textMuted,
          paddingLeft: 14,
        }}
      >
        {toast.from.ip}:{toast.from.port}
      </span>
    </div>
  );
}
