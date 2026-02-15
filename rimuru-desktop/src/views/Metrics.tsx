import { useSystemMetrics, useMetricsHistory } from "@/hooks/useMetrics";
import { LineChart, Line, XAxis, YAxis, Tooltip, ResponsiveContainer } from "recharts";
import Gauge from "@/components/Gauge";
import styles from "./Metrics.module.css";

export default function Metrics() {
  const { data: currentMetrics } = useSystemMetrics();
  const { data: history } = useMetricsHistory(24);

  const formattedHistory = history?.map((h) => ({
    ...h,
    time: new Date(h.timestamp).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }),
  }));

  return (
    <div className={styles.container}>
      <h1 className={styles.title}>System Metrics</h1>

      <div className={styles.gaugeGrid}>
        <div className={styles.gaugeCard}>
          <h3 className={styles.gaugeTitle}>CPU Usage</h3>
          <Gauge value={currentMetrics?.cpu_usage ?? 0} max={100} unit="%" color="var(--color-accent)" />
        </div>

        <div className={styles.gaugeCard}>
          <h3 className={styles.gaugeTitle}>Memory Usage</h3>
          <Gauge
            value={currentMetrics?.memory_usage_percent ?? 0}
            max={100}
            unit="%"
            color="var(--color-success)"
          />
          <div className={styles.memoryDetail}>
            {currentMetrics?.memory_used_mb ?? 0} MB / {currentMetrics?.memory_total_mb ?? 0} MB
          </div>
        </div>

        <div className={styles.gaugeCard}>
          <h3 className={styles.gaugeTitle}>Active Sessions</h3>
          <div className={styles.sessionCount}>{currentMetrics?.active_sessions ?? 0}</div>
        </div>
      </div>

      <div className={styles.chartCard}>
        <h3 className={styles.chartTitle}>CPU Usage (24h)</h3>
        {formattedHistory && formattedHistory.length > 0 ? (
          <ResponsiveContainer width="100%" height={200}>
            <LineChart data={formattedHistory}>
              <XAxis
                dataKey="time"
                tick={{ fontSize: 10, fill: "var(--color-text-muted)" }}
                axisLine={false}
              />
              <YAxis
                domain={[0, 100]}
                tick={{ fontSize: 10, fill: "var(--color-text-muted)" }}
                axisLine={false}
                tickFormatter={(v) => `${v}%`}
              />
              <Tooltip
                formatter={(value: number) => `${value.toFixed(1)}%`}
                contentStyle={{
                  background: "var(--color-bg-secondary)",
                  border: "1px solid var(--color-border)",
                  borderRadius: "var(--radius-sm)",
                }}
              />
              <Line
                type="monotone"
                dataKey="cpu_usage"
                stroke="var(--color-accent)"
                strokeWidth={2}
                dot={false}
              />
            </LineChart>
          </ResponsiveContainer>
        ) : (
          <div className={styles.empty}>No historical data</div>
        )}
      </div>

      <div className={styles.chartCard}>
        <h3 className={styles.chartTitle}>Memory Usage (24h)</h3>
        {formattedHistory && formattedHistory.length > 0 ? (
          <ResponsiveContainer width="100%" height={200}>
            <LineChart data={formattedHistory}>
              <XAxis
                dataKey="time"
                tick={{ fontSize: 10, fill: "var(--color-text-muted)" }}
                axisLine={false}
              />
              <YAxis
                domain={[0, 100]}
                tick={{ fontSize: 10, fill: "var(--color-text-muted)" }}
                axisLine={false}
                tickFormatter={(v) => `${v}%`}
              />
              <Tooltip
                formatter={(value: number) => `${value.toFixed(1)}%`}
                contentStyle={{
                  background: "var(--color-bg-secondary)",
                  border: "1px solid var(--color-border)",
                  borderRadius: "var(--radius-sm)",
                }}
              />
              <Line
                type="monotone"
                dataKey="memory_usage_percent"
                stroke="var(--color-success)"
                strokeWidth={2}
                dot={false}
              />
            </LineChart>
          </ResponsiveContainer>
        ) : (
          <div className={styles.empty}>No historical data</div>
        )}
      </div>
    </div>
  );
}
