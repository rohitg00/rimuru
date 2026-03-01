import { useState, useEffect, useRef, useCallback } from "react";
import type { StreamEvent } from "../api/types";

interface UseStreamResult {
  connected: boolean;
  events: StreamEvent[];
  error: string | null;
  clear: () => void;
}

export function useStream(channel: string, maxEvents = 100): UseStreamResult {
  const [connected, setConnected] = useState(false);
  const [events, setEvents] = useState<StreamEvent[]>([]);
  const [error, setError] = useState<string | null>(null);
  const wsRef = useRef<WebSocket | null>(null);

  const clear = useCallback(() => {
    setEvents([]);
  }, []);

  useEffect(() => {
    const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    const host = window.location.hostname;
    const port = 3112;
    const url = `${protocol}//${host}:${port}/stream/${channel}`;

    let reconnectTimer: ReturnType<typeof setTimeout> | undefined;
    let mounted = true;
    let attempts = 0;
    const MAX_ATTEMPTS = 1;

    function connect() {
      if (!mounted || attempts >= MAX_ATTEMPTS) return;
      attempts++;

      let ws: WebSocket;
      try {
        ws = new WebSocket(url);
      } catch {
        return;
      }
      wsRef.current = ws;

      ws.onopen = () => {
        if (mounted) {
          setConnected(true);
          setError(null);
          attempts = 0;
        }
      };

      ws.onmessage = (evt) => {
        if (!mounted) return;
        try {
          const event: StreamEvent = JSON.parse(evt.data);
          setEvents((prev) => {
            const next = [event, ...prev];
            return next.length > maxEvents ? next.slice(0, maxEvents) : next;
          });
        } catch {
          // skip malformed messages
        }
      };

      ws.onerror = () => {
        if (mounted) setConnected(false);
      };

      ws.onclose = () => {
        if (mounted) setConnected(false);
      };
    }

    connect();

    return () => {
      mounted = false;
      if (reconnectTimer) clearTimeout(reconnectTimer);
      if (wsRef.current) {
        wsRef.current.close();
        wsRef.current = null;
      }
    };
  }, [channel, maxEvents]);

  return { connected, events, error, clear };
}
