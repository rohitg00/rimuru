import { useState, useEffect } from "react";
import { commands } from "@/lib/tauri";
import type { CostSummary, CostBreakdown, CostHistoryPoint } from "@/lib/tauri";
import styles from "./UsageDashboard.module.css";

export default function UsageDashboard() {
  const [summary, setSummary] = useState<CostSummary | null>(null);
  const [breakdown, setBreakdown] = useState<CostBreakdown | null>(null);
  const [history, setHistory] = useState<CostHistoryPoint[]>([]);

  useEffect(() => {
    commands.getCostSummary().then(setSummary).catch(() => {});
    commands.getCostBreakdown().then(setBreakdown).catch(() => {});
    commands.getCostHistory(30).then(setHistory).catch(() => {});
  }, []);

  const maxHistoryCost = Math.max(...history.map((h) => h.cost), 0.01);
  const maxAgentCost = Math.max(...(breakdown?.by_agent.map((a) => a.cost) ?? []), 0.01);

  const topAgent = breakdown?.by_agent.reduce(
    (top, a) => (a.cost > (top?.cost ?? 0) ? a : top),
    breakdown.by_agent[0]
  );

  return (
    <div className={styles.container}>
      <div className={styles.header}>Usage Dashboard</div>

      <div className={styles.cards}>
        <div className={styles.card}>
          <div className={styles.cardLabel}>Total Sessions</div>
          <div className={styles.cardValue}>{summary?.session_count ?? 0}</div>
        </div>
        <div className={styles.card}>
          <div className={styles.cardLabel}>Total Tokens</div>
          <div className={styles.cardValue}>{summary?.total_tokens?.toLocaleString() ?? 0}</div>
        </div>
        <div className={styles.card}>
          <div className={styles.cardLabel}>Total Cost</div>
          <div className={styles.cardValue}>${(summary?.total_cost ?? 0).toFixed(2)}</div>
        </div>
        <div className={styles.card}>
          <div className={styles.cardLabel}>Most Used Agent</div>
          <div className={styles.cardValue}>{topAgent?.name ?? "N/A"}</div>
        </div>
      </div>

      <div className={styles.section}>
        <div className={styles.sectionTitle}>Cost History (30 days)</div>
        <div className={styles.chart}>
          {history.map((point, i) => (
            <div
              key={i}
              className={styles.chartBar}
              style={{ height: `${(point.cost / maxHistoryCost) * 100}%` }}
              title={`${point.date}: $${point.cost.toFixed(2)}`}
            />
          ))}
        </div>
      </div>

      <div className={styles.section}>
        <div className={styles.sectionTitle}>Cost by Agent</div>
        {breakdown?.by_agent.map((agent) => (
          <div key={agent.name} className={styles.agentBar}>
            <span className={styles.agentName}>{agent.name}</span>
            <div
              className={styles.agentBarFill}
              style={{ width: `${(agent.cost / maxAgentCost) * 100}%`, flex: "1" }}
            />
            <span className={styles.agentCost}>${agent.cost.toFixed(2)}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
