import { useState, useEffect, useCallback } from "react";
import { events } from "@/lib/tauri";

export function useSessionCost(sessionId: string | null) {
  const [cost, setCost] = useState(0);
  const [tokens, setTokens] = useState(0);

  useEffect(() => {
    if (!sessionId) return;
    const unlisten = events.onCostRecorded((payload) => {
      if (payload.session_id === sessionId) {
        setCost((prev) => prev + payload.cost);
        setTokens((prev) => prev + payload.tokens);
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [sessionId]);

  const reset = useCallback(() => {
    setCost(0);
    setTokens(0);
  }, []);

  return { cost, tokens, reset };
}
