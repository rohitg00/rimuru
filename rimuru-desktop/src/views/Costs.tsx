import { useState } from "react";
import { useCostSummary, useCostBreakdown, useCostHistory } from "@/hooks/useCosts";
import { commands } from "@/lib/tauri";
import { useToast } from "@/components/Toast/ToastProvider";
import { PieChart, Pie, Cell, BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, LineChart, Line } from "recharts";
import styles from "./Costs.module.css";

const COLORS = [
  "var(--color-accent)",
  "var(--color-success)",
  "var(--color-warning)",
  "var(--color-error)",
  "var(--color-info)",
];

export default function Costs() {
  const { toast } = useToast();
  const [timeRange, setTimeRange] = useState<"today" | "week" | "month" | "year">("month");
  const [exporting, setExporting] = useState(false);
  const { data: summary } = useCostSummary({ range: timeRange });
  const { data: breakdown } = useCostBreakdown({ range: timeRange });
  const { data: history } = useCostHistory(30);

  const handleExport = async (format: "csv" | "json") => {
    setExporting(true);
    try {
      const data = await commands.exportCosts(format, { range: timeRange });
      const blob = new Blob([data], { type: format === "csv" ? "text/csv" : "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `costs-export.${format}`;
      a.click();
      URL.revokeObjectURL(url);
      toast({ type: "success", title: "Export complete", description: `Costs exported as ${format.toUpperCase()}` });
    } catch (e) {
      console.error("Export failed:", e);
      toast({ type: "error", title: "Export failed", description: String(e) });
    } finally {
      setExporting(false);
    }
  };

  const projectedMonthly = summary
    ? (summary.total_cost / (timeRange === "today" ? 1 : timeRange === "week" ? 7 : 30)) * 30
    : 0;

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <h1 className={styles.title}>Cost Analytics</h1>
        <div style={{ display: "flex", gap: "0.5rem", alignItems: "center" }}>
          <select
            value={timeRange}
            onChange={(e) => setTimeRange(e.target.value as typeof timeRange)}
            className={styles.timeSelect}
          >
            <option value="today">Today</option>
            <option value="week">This Week</option>
            <option value="month">This Month</option>
            <option value="year">This Year</option>
          </select>
          <button
            className="btn btn-secondary"
            onClick={() => handleExport("csv")}
            disabled={exporting}
          >
            Export CSV
          </button>
          <button
            className="btn btn-secondary"
            onClick={() => handleExport("json")}
            disabled={exporting}
          >
            Export JSON
          </button>
        </div>
      </div>

      <div className={styles.summaryGrid}>
        <div className={styles.summaryCard}>
          <span className={styles.summaryLabel}>Total Cost</span>
          <span className={styles.summaryValue}>
            ${(summary?.total_cost ?? 0).toFixed(2)}
          </span>
        </div>
        <div className={styles.summaryCard}>
          <span className={styles.summaryLabel}>Total Tokens</span>
          <span className={styles.summaryValue}>
            {(summary?.total_tokens ?? 0).toLocaleString()}
          </span>
        </div>
        <div className={styles.summaryCard}>
          <span className={styles.summaryLabel}>Sessions</span>
          <span className={styles.summaryValue}>{summary?.session_count ?? 0}</span>
        </div>
        <div className={styles.summaryCard}>
          <span className={styles.summaryLabel}>Projected Monthly</span>
          <span className={styles.summaryValue}>${projectedMonthly.toFixed(2)}</span>
        </div>
      </div>

      <div className={styles.chartsGrid}>
        <div className={styles.chartCard}>
          <h3 className={styles.chartTitle}>Cost by Agent</h3>
          {breakdown?.by_agent && breakdown.by_agent.length > 0 ? (
            <ResponsiveContainer width="100%" height={250}>
              <PieChart>
                <Pie
                  data={breakdown.by_agent}
                  dataKey="cost"
                  nameKey="name"
                  cx="50%"
                  cy="50%"
                  innerRadius={40}
                  outerRadius={80}
                  label={({ name, percentage }) => `${name} (${percentage.toFixed(0)}%)`}
                >
                  {breakdown.by_agent.map((_, index) => (
                    <Cell key={`cell-agent-${index}`} fill={COLORS[index % COLORS.length]} />
                  ))}
                </Pie>
                <Tooltip
                  formatter={(value: number) => `$${value.toFixed(4)}`}
                  contentStyle={{
                    background: "var(--color-bg-secondary)",
                    border: "1px solid var(--color-border)",
                    borderRadius: "var(--radius-sm)",
                  }}
                />
              </PieChart>
            </ResponsiveContainer>
          ) : (
            <div className={styles.empty}>
              <span>No data available</span>
              <p>Cost data will appear once sessions generate usage</p>
            </div>
          )}
        </div>

        <div className={styles.chartCard}>
          <h3 className={styles.chartTitle}>Cost by Model</h3>
          {breakdown?.by_model && breakdown.by_model.length > 0 ? (
            <ResponsiveContainer width="100%" height={250}>
              <PieChart>
                <Pie
                  data={breakdown.by_model}
                  dataKey="cost"
                  nameKey="name"
                  cx="50%"
                  cy="50%"
                  outerRadius={80}
                  label={({ name, percentage }) => `${name} (${percentage.toFixed(0)}%)`}
                >
                  {breakdown.by_model.map((_, index) => (
                    <Cell key={`cell-model-${index}`} fill={COLORS[index % COLORS.length]} />
                  ))}
                </Pie>
                <Tooltip
                  formatter={(value: number) => `$${value.toFixed(4)}`}
                  contentStyle={{
                    background: "var(--color-bg-secondary)",
                    border: "1px solid var(--color-border)",
                    borderRadius: "var(--radius-sm)",
                  }}
                />
              </PieChart>
            </ResponsiveContainer>
          ) : (
            <div className={styles.empty}>
              <span>No data available</span>
              <p>Cost data will appear once sessions generate usage</p>
            </div>
          )}
        </div>
      </div>

      <div className={styles.chartCard}>
        <h3 className={styles.chartTitle}>Token Usage by Model</h3>
        {breakdown?.by_model && breakdown.by_model.length > 0 ? (
          <ResponsiveContainer width="100%" height={250}>
            <BarChart data={breakdown.by_model}>
              <XAxis
                dataKey="name"
                tick={{ fontSize: 11, fill: "var(--color-text-muted)" }}
                axisLine={false}
              />
              <YAxis
                tick={{ fontSize: 11, fill: "var(--color-text-muted)" }}
                axisLine={false}
              />
              <Tooltip
                formatter={(value: number) => value.toLocaleString()}
                contentStyle={{
                  background: "var(--color-bg-secondary)",
                  border: "1px solid var(--color-border)",
                  borderRadius: "var(--radius-sm)",
                }}
              />
              <Bar dataKey="tokens" fill="var(--color-accent)" radius={[4, 4, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        ) : (
          <div className={styles.empty}>No data available</div>
        )}
      </div>

      <div className={styles.chartCard}>
        <h3 className={styles.chartTitle}>Cost Over Time (30 days)</h3>
        {history && history.length > 0 ? (
          <ResponsiveContainer width="100%" height={300}>
            <LineChart data={history}>
              <XAxis
                dataKey="date"
                tick={{ fontSize: 11, fill: "var(--color-text-muted)" }}
                axisLine={false}
              />
              <YAxis
                tick={{ fontSize: 11, fill: "var(--color-text-muted)" }}
                axisLine={false}
                tickFormatter={(value) => `$${value.toFixed(2)}`}
              />
              <Tooltip
                formatter={(value: number) => `$${value.toFixed(4)}`}
                contentStyle={{
                  background: "var(--color-bg-secondary)",
                  border: "1px solid var(--color-border)",
                  borderRadius: "var(--radius-sm)",
                }}
              />
              <Line
                type="monotone"
                dataKey="cost"
                stroke="var(--color-accent)"
                strokeWidth={2}
                dot={false}
              />
            </LineChart>
          </ResponsiveContainer>
        ) : (
          <div className={styles.empty}>No data available</div>
        )}
      </div>
    </div>
  );
}
