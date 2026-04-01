import { useState, useEffect, useRef } from "react";
import type { CapturedEvent } from "../types";

export function useWebSocket(wsUrl: string, onEvent: (ev: CapturedEvent) => void): boolean {
  const [connected, setConnected] = useState(false);
  const callbackRef = useRef(onEvent);
  callbackRef.current = onEvent;

  useEffect(() => {
    let ws: WebSocket;
    let timer: ReturnType<typeof setTimeout>;

    const connect = () => {
      try {
        ws = new WebSocket(wsUrl);
        ws.onopen = () => setConnected(true);
        ws.onclose = () => {
          setConnected(false);
          timer = setTimeout(connect, 3000);
        };
        ws.onerror = () => ws.close();
        ws.onmessage = (msg) => {
          try {
            callbackRef.current(JSON.parse(msg.data));
          } catch { /* skip malformed */ }
        };
      } catch { /* retry on next timer */ }
    };

    connect();
    return () => {
      ws?.close();
      clearTimeout(timer);
    };
  }, [wsUrl]); // reconnect wenn URL sich ändert

  return connected;
}