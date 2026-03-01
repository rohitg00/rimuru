import { useState, useMemo } from "react";
import { useQuery } from "../hooks/useQuery";
import type { SystemMetrics, MetricsTimeline } from "../api/types";
import MetricsChart, { Gauge } from "../components/MetricsChart";

import { formatUptime } from "../utils/format";

export default function Metrics() {
  const { data: rawMetrics } = useQuery<SystemMetrics>("/metrics", 2000);
  const { data: timeline } = useQuery<MetricsTimeline>(
    "/metrics/timeline?minutes=30",
    5000,
  );
  const [refreshRate] = useState(2000);

  const raw = (rawMetrics?.metrics ?? rawMetrics) as Record<
    string,
    number
  > | null;
  const metrics = raw
    ? {
        cpu_percent: (raw.cpu_percent ?? raw.cpu_usage_percent ?? 0) as number,
        memory_used_mb: (raw.memory_used_mb ?? 0) as number,
        memory_total_mb: (raw.memory_total_mb ?? 0) as number,
        heap_used_mb: (raw.heap_used_mb ?? 0) as number,
        event_loop_lag_ms: (raw.event_loop_lag_ms ?? 0) as number,
        requests_per_second: (raw.requests_per_second ??
          (raw.requests_per_minute ?? 0) / 60) as number,
        active_connections: (raw.active_connections ??
          raw.active_agents ??
          0) as number,
        uptime_seconds: (raw.uptime_seconds ?? raw.uptime_secs ?? 0) as number,
        total_cost_today: (raw.total_cost_today ?? 0) as number,
      }
    : null;

  const cpuData = useMemo(
    () =>
      (timeline?.timestamps ?? []).map((ts, i) => ({
        time: new Date(ts).toLocaleTimeString("en-US", {
          hour: "2-digit",
          minute: "2-digit",
        }),
        value: timeline?.cpu[i] ?? 0,
      })),
    [timeline],
  );

  const memData = useMemo(
    () =>
      (timeline?.timestamps ?? []).map((ts, i) => ({
        time: new Date(ts).toLocaleTimeString("en-US", {
          hour: "2-digit",
          minute: "2-digit",
        }),
        value: timeline?.memory[i] ?? 0,
      })),
    [timeline],
  );

  const reqData = useMemo(
    () =>
      (timeline?.timestamps ?? []).map((ts, i) => ({
        time: new Date(ts).toLocaleTimeString("en-US", {
          hour: "2-digit",
          minute: "2-digit",
        }),
        value: timeline?.requests[i] ?? 0,
      })),
    [timeline],
  );

  const connData = useMemo(
    () =>
      (timeline?.timestamps ?? []).map((ts, i) => ({
        time: new Date(ts).toLocaleTimeString("en-US", {
          hour: "2-digit",
          minute: "2-digit",
        }),
        value: timeline?.connections[i] ?? 0,
      })),
    [timeline],
  );

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-bold text-[var(--text-primary)]">
            System Metrics
          </h2>
          <p className="text-sm text-[var(--text-secondary)]">
            Real-time system performance (refreshing every {refreshRate / 1000}
            s)
          </p>
        </div>
        {metrics && (
          <div className="text-right">
            <p className="text-xs text-[var(--text-secondary)]">Uptime</p>
            <p className="text-sm font-semibold text-[var(--text-primary)]">
              {formatUptime(metrics.uptime_seconds ?? 0)}
            </p>
          </div>
        )}
      </div>

      <div className="flex flex-wrap items-center justify-center gap-8 rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-8">
        <Gauge
          value={metrics?.cpu_percent ?? 0}
          max={100}
          label="CPU"
          unit="%"
          color="var(--accent)"
        />
        <Gauge
          value={metrics?.memory_used_mb ?? 0}
          max={metrics?.memory_total_mb ?? 100}
          label="Memory"
          unit="MB"
          color="var(--success)"
        />
        <Gauge
          value={metrics?.heap_used_mb ?? 0}
          max={metrics?.memory_used_mb ?? 100}
          label="Heap"
          unit="MB"
          color="var(--warning)"
        />
        <Gauge
          value={metrics?.event_loop_lag_ms ?? 0}
          max={100}
          label="Event Loop"
          unit="ms"
          color="var(--error)"
        />
        <Gauge
          value={metrics?.requests_per_second ?? 0}
          max={Math.max(100, (metrics?.requests_per_second ?? 0) * 1.5)}
          label="Req/s"
          unit=""
          color="#a78bfa"
        />
        <Gauge
          value={metrics?.active_connections ?? 0}
          max={Math.max(10, (metrics?.active_connections ?? 0) * 2)}
          label="Connections"
          unit=""
          color="#f472b6"
        />
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-4">
          <p className="text-[10px] uppercase tracking-wider text-[var(--text-secondary)] mb-1">
            CPU Usage
          </p>
          <p className="text-xl font-bold text-[var(--text-primary)]">
            {(metrics?.cpu_percent ?? 0).toFixed(1)}%
          </p>
        </div>
        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-4">
          <p className="text-[10px] uppercase tracking-wider text-[var(--text-secondary)] mb-1">
            Memory
          </p>
          <p className="text-xl font-bold text-[var(--text-primary)]">
            {(metrics?.memory_used_mb ?? 0).toFixed(0)} /{" "}
            {(metrics?.memory_total_mb ?? 0).toFixed(0)} MB
          </p>
        </div>
        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-4">
          <p className="text-[10px] uppercase tracking-wider text-[var(--text-secondary)] mb-1">
            Active Connections
          </p>
          <p className="text-xl font-bold text-[var(--text-primary)]">
            {metrics?.active_connections ?? 0}
          </p>
        </div>
        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-4">
          <p className="text-[10px] uppercase tracking-wider text-[var(--text-secondary)] mb-1">
            Requests/sec
          </p>
          <p className="text-xl font-bold text-[var(--text-primary)]">
            {(metrics?.requests_per_second ?? 0).toFixed(1)}
          </p>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {[
          {
            data: cpuData,
            color: "var(--accent)",
            label: "CPU Usage",
            unit: "%",
            max: 100,
          },
          {
            data: memData,
            color: "var(--success)",
            label: "Memory Usage",
            unit: "MB",
          },
          {
            data: reqData,
            color: "var(--warning)",
            label: "Requests per Second",
            unit: "/s",
          },
          { data: connData, color: "#a78bfa", label: "Active Connections" },
        ].map((chart) => (
          <div
            key={chart.label}
            className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5"
          >
            {chart.data.length > 0 ? (
              <MetricsChart
                data={chart.data}
                color={chart.color}
                label={chart.label}
                unit={chart.unit}
                max={chart.max}
              />
            ) : (
              <div>
                <p className="text-xs font-semibold text-[var(--text-primary)] mb-2">
                  {chart.label}
                </p>
                <div className="h-[200px] flex items-center justify-center text-sm text-[var(--text-secondary)]">
                  Collecting data...
                </div>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
