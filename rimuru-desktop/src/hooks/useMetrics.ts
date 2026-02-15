import { useQuery } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { commands, events, SystemMetrics, MetricsHistoryPoint, MetricsUpdatePayload } from "@/lib/tauri";

export function useSystemMetrics() {
  return useQuery<SystemMetrics>({
    queryKey: ["metrics", "system"],
    queryFn: commands.getSystemMetrics,
    refetchInterval: 5000,
  });
}

export function useMetricsHistory(hours?: number) {
  return useQuery<MetricsHistoryPoint[]>({
    queryKey: ["metrics", "history", hours],
    queryFn: () => commands.getMetricsHistory(hours),
  });
}

export function useRealtimeMetrics() {
  const [metrics, setMetrics] = useState<MetricsUpdatePayload | null>(null);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    events.onMetricsUpdate((payload) => {
      setMetrics(payload);
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, []);

  return metrics;
}
