import { Bot, Clock, DollarSign, Activity } from "lucide-react";
import { useAgents } from "@/hooks/useAgents";
import { useActiveSessions } from "@/hooks/useSessions";
import { useCostSummary, useCostHistory } from "@/hooks/useCosts";
import { useSystemMetrics } from "@/hooks/useMetrics";
import StatsCard from "@/components/StatsCard";
import MiniChart from "@/components/MiniChart";
import ActivityFeed from "@/components/ActivityFeed";
import {
  StatsCardSkeleton,
  ChartCardSkeleton,
  ListItemSkeleton,
} from "@/components/Skeleton";
import styles from "./Dashboard.module.css";

export default function Dashboard() {
  const { data: agents, isLoading: agentsLoading } = useAgents();
  const { data: activeSessions, isLoading: sessionsLoading } = useActiveSessions();
  const { data: costSummary, isLoading: costLoading } = useCostSummary({ range: "today" });
  const { data: costHistory, isLoading: historyLoading } = useCostHistory(7);
  const { data: metrics, isLoading: metricsLoading } = useSystemMetrics();

  const statsLoading = agentsLoading || sessionsLoading || costLoading || metricsLoading;
  const chartsLoading = historyLoading;

  const healthScore = metrics
    ? Math.max(0, 100 - metrics.cpu_usage - (metrics.memory_usage_percent / 2))
    : 100;

  return (
    <div className={styles.dashboard}>
      <h1 className={styles.title}>Dashboard</h1>

      <div className={styles.statsGrid}>
        {statsLoading ? (
          <>
            <StatsCardSkeleton />
            <StatsCardSkeleton />
            <StatsCardSkeleton />
            <StatsCardSkeleton />
          </>
        ) : (
          <>
            <StatsCard
              icon={Bot}
              label="Total Agents"
              value={agents?.length ?? 0}
              trend={{ value: 0, direction: "neutral" }}
            />
            <StatsCard
              icon={Clock}
              label="Active Sessions"
              value={activeSessions?.length ?? 0}
              trend={{ value: 0, direction: "neutral" }}
            />
            <StatsCard
              icon={DollarSign}
              label="Today's Cost"
              value={`$${(costSummary?.total_cost ?? 0).toFixed(2)}`}
              trend={{ value: 0, direction: "neutral" }}
            />
            <StatsCard
              icon={Activity}
              label="System Health"
              value={`${healthScore.toFixed(0)}%`}
              trend={{
                value: healthScore >= 80 ? 1 : healthScore >= 50 ? 0 : -1,
                direction: healthScore >= 80 ? "up" : healthScore >= 50 ? "neutral" : "down",
              }}
            />
          </>
        )}
      </div>

      <div className={styles.chartsRow}>
        {chartsLoading ? (
          <>
            <ChartCardSkeleton />
            <ChartCardSkeleton />
          </>
        ) : (
          <>
            <div className={styles.chartCard}>
              <h3 className={styles.chartTitle}>Cost Trend (7 days)</h3>
              <MiniChart
                data={costHistory ?? []}
                dataKey="cost"
                xKey="date"
                color="var(--color-accent)"
              />
            </div>
            <div className={styles.chartCard}>
              <h3 className={styles.chartTitle}>Token Usage (7 days)</h3>
              <MiniChart
                data={costHistory ?? []}
                dataKey="tokens"
                xKey="date"
                color="var(--color-info)"
              />
            </div>
          </>
        )}
      </div>

      <div className={styles.bottomRow}>
        <div className={styles.activityCard}>
          <h3 className={styles.cardTitle}>Recent Activity</h3>
          <ActivityFeed />
        </div>

        <div className={styles.agentsCard}>
          <h3 className={styles.cardTitle}>Agent Status</h3>
          <div className={styles.agentsList}>
            {agentsLoading ? (
              <>
                <ListItemSkeleton />
                <ListItemSkeleton />
                <ListItemSkeleton />
              </>
            ) : agents && agents.length > 0 ? (
              agents.slice(0, 5).map((a) => (
                <div key={a.agent.id} className={styles.agentItem}>
                  <span className={styles.agentName}>{a.agent.name}</span>
                  <span
                    className={`badge ${a.is_active ? "badge-success" : "badge-warning"}`}
                  >
                    {a.is_active ? "Active" : "Idle"}
                  </span>
                </div>
              ))
            ) : (
              <p className={styles.empty}>No agents registered</p>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
