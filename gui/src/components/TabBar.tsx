import { colors, font } from "../theme";
import type { Tab } from "../types";

interface TabBarProps {
  active: Tab;
  onChange: (tab: Tab) => void;
}

const tabs: { id: Tab; label: string }[] = [
  { id: "graph", label: "Network Graph" },
  { id: "peers", label: "Peers" },
  { id: "messages", label: "Messages" },
  { id: "packets", label: "Packets" },
  { id: "sessions", label: "Sessions" },
  { id: "hosts", label: "Hosts" },
];

export function TabBar({ active, onChange }: TabBarProps) {
  return (
    <nav
      style={{
        display: "flex",
        borderBottom: `1px solid ${colors.border}`,
        background: colors.surface,
      }}
    >
      {tabs.map((t) => (
        <button
          key={t.id}
          onClick={() => onChange(t.id)}
          style={{
            background: "transparent",
            border: "none",
            cursor: "pointer",
            padding: "12px 20px",
            fontFamily: font.mono,
            fontSize: 11,
            fontWeight: 600,
            color: active === t.id ? colors.text : colors.textMuted,
            borderBottom:
              active === t.id
                ? `2px solid ${t.id === "messages" ? colors.purple : colors.accent}`
                : "2px solid transparent",
            transition: "all 0.15s",
          }}
        >
          {t.label}
        </button>
      ))}
    </nav>
  );
}
