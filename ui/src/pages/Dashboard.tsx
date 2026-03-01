import { useQuery } from "../hooks/useQuery";
import { useStream } from "../hooks/useStream";
import type { StatsOverview, ActivityEvent, DailyCost } from "../api/types";
import { CostBarChart } from "../components/CostChart";

function StatCard({
  label,
  value,
  sub,
  color,
}: {
  label: string;
  value: string;
  sub?: string;
  color: string;
}) {
  return (
    <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
      <div className="flex items-start justify-between">
        <div>
          <p className="text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)] mb-1">
            {label}
          </p>
          <p className="text-2xl font-bold text-[var(--text-primary)]">
            {value}
          </p>
          {sub && (
            <p className="text-xs text-[var(--text-secondary)] mt-1">{sub}</p>
          )}
        </div>
        <div
          className="w-10 h-10 rounded-lg flex items-center justify-center"
          style={{ backgroundColor: `${color}20` }}
        >
          <div
            className="w-3 h-3 rounded-full"
            style={{ backgroundColor: color }}
          />
        </div>
      </div>
    </div>
  );
}

import { formatCost, formatTokens, timeAgo } from "../utils/format";

const EVENT_ICONS: Record<string, { icon: string; color: string }> = {
  agent_connected: { icon: "\u25B6", color: "var(--success)" },
  agent_disconnected: { icon: "\u25A0", color: "var(--error)" },
  session_started: { icon: "\u25CF", color: "var(--accent)" },
  session_ended: { icon: "\u25CB", color: "var(--text-secondary)" },
  cost_alert: { icon: "\u26A0", color: "var(--warning)" },
  plugin_installed: { icon: "\u2795", color: "var(--accent)" },
  hook_triggered: { icon: "\u26A1", color: "var(--warning)" },
  error: { icon: "\u2717", color: "var(--error)" },
};

export default function Dashboard() {
  const { data: stats } = useQuery<StatsOverview>("/stats", 5000);
  const { data: dailyCosts } = useQuery<DailyCost[]>(
    "/costs/daily?days=14",
    30000,
  );
  const { data: activity } = useQuery<ActivityEvent[]>(
    "/activity?limit=20",
    5000,
  );
  const { connected } = useStream("activity");

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-bold text-[var(--text-primary)]">
            Overview
          </h2>
          <p className="text-sm text-[var(--text-secondary)]">
            System status and recent activity
          </p>
        </div>
        <div className="flex items-center gap-2">
          <span
            className={`w-2 h-2 rounded-full ${connected ? "bg-[var(--success)] animate-pulse" : "bg-[var(--error)]"}`}
          />
          <span className="text-xs text-[var(--text-secondary)]">
            {connected ? "Live" : "Disconnected"}
          </span>
        </div>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          label="Total Cost"
          value={formatCost(stats?.total_cost ?? 0)}
          sub={`$${(stats?.total_cost_today ?? 0).toFixed(4)} today`}
          color="var(--accent)"
        />
        <StatCard
          label="Active Agents"
          value={`${stats?.active_agents ?? 0}`}
          sub={`${stats?.total_agents ?? 0} total`}
          color="var(--success)"
        />
        <StatCard
          label="Sessions"
          value={`${stats?.total_sessions ?? 0}`}
          sub={`${stats?.active_sessions ?? 0} active`}
          color="var(--warning)"
        />
        <StatCard
          label="Tokens"
          value={formatTokens(stats?.total_tokens ?? 0)}
          sub={`${stats?.models_used ?? 0} models`}
          color="var(--error)"
        />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2 rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
          <h3 className="text-sm font-semibold text-[var(--text-primary)] mb-4">
            Daily Cost (14 days)
          </h3>
          {dailyCosts && dailyCosts.length > 0 ? (
            <CostBarChart
              data={dailyCosts.map((d) => ({
                date: new Date(d.date).toLocaleDateString("en-US", {
                  month: "short",
                  day: "numeric",
                }),
                cost: d.total_cost ?? d.cost ?? 0,
              }))}
            />
          ) : (
            <div className="h-[300px] flex items-center justify-center text-sm text-[var(--text-secondary)]">
              No cost data yet
            </div>
          )}
        </div>

        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
          <h3 className="text-sm font-semibold text-[var(--text-primary)] mb-4">
            Activity Feed
          </h3>
          <div className="space-y-3 max-h-[340px] overflow-y-auto pr-1">
            {(activity ?? []).length === 0 ? (
              <p className="text-sm text-[var(--text-secondary)] text-center py-8">
                No recent activity
              </p>
            ) : (
              (activity ?? []).map((evt) => {
                const ei = EVENT_ICONS[evt.type] ?? EVENT_ICONS.error!;
                return (
                  <div
                    key={evt.id}
                    className="flex items-start gap-3 p-2 rounded-lg hover:bg-[var(--bg-tertiary)] transition-colors"
                  >
                    <span
                      className="mt-0.5 text-xs shrink-0"
                      style={{ color: ei.color }}
                    >
                      {ei.icon}
                    </span>
                    <div className="flex-1 min-w-0">
                      <p className="text-xs text-[var(--text-primary)] line-clamp-2">
                        {evt.message}
                      </p>
                      <p className="text-[10px] text-[var(--text-secondary)] mt-0.5">
                        {timeAgo(evt.timestamp)}
                      </p>
                    </div>
                  </div>
                );
              })
            )}
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5 text-center">
          <p className="text-3xl font-bold text-[var(--accent)]">
            {stats?.plugins_installed ?? 0}
          </p>
          <p className="text-xs text-[var(--text-secondary)] mt-1 uppercase tracking-wider">
            Plugins
          </p>
        </div>
        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5 text-center">
          <p className="text-3xl font-bold text-[var(--success)]">
            {stats?.hooks_active ?? 0}
          </p>
          <p className="text-xs text-[var(--text-secondary)] mt-1 uppercase tracking-wider">
            Active Hooks
          </p>
        </div>
        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5 text-center">
          <p className="text-3xl font-bold text-[var(--warning)]">
            {stats?.models_used ?? 0}
          </p>
          <p className="text-xs text-[var(--text-secondary)] mt-1 uppercase tracking-wider">
            Models
          </p>
        </div>
      </div>
    </div>
  );
}
