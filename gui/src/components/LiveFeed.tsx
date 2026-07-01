import { useRef, useEffect } from "react";
import { colors, font } from "../theme";
import { formatEvent } from "../utils/format";
import { MAX_FEED_VISIBLE } from "../utils/config";
import type { CapturedEvent } from "../types";

interface LiveFeedProps {
  events: CapturedEvent[];
}

function eventColor(ev: CapturedEvent): string {
  if (ev.Arp) return colors.accent;
  return colors.blue;
}

export function LiveFeed({ events }: LiveFeedProps) {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    el.scrollTop = el.scrollHeight;
  }, [events.length]);

  const visible = events.slice(-MAX_FEED_VISIBLE);

  return (
    <div
      style={{
        flex: 1,
        minHeight: 0,
        display: "flex",
        flexDirection: "column",
        borderTop: `1px solid ${colors.border}`,
        overflow: "hidden",
      }}
    >
      <div
        style={{
          padding: "10px 16px",
          fontSize: 10,
          fontWeight: 600,
          fontFamily: font.mono,
          color: colors.textMuted,
          textTransform: "uppercase",
          letterSpacing: 1,
          flexShrink: 0,
        }}
      >
        Event Feed
      </div>

      <div
        ref={containerRef}
        style={{
          flex: 1,
          minHeight: 0,
          overflowY: "auto",
          overflowX: "hidden",
          padding: "0 8px 8px",
          fontFamily: font.mono,
          fontSize: 10,
          lineHeight: 1.7,
        }}
      >
        {visible.map((ev, i) => (
          <div
            key={i}
            style={{
              color: eventColor(ev),
              opacity: 0.85,
              padding: "1px 8px",
              borderRadius: 3,
              whiteSpace: "nowrap",
              overflow: "hidden",
              textOverflow: "ellipsis",
              borderLeft: "2px solid transparent",
            }}
          >
            {formatEvent(ev)}
          </div>
        ))}
      </div>
    </div>
  );
}
