import { useMemo } from "react";
import { useQuery } from "../hooks/useQuery";
import type { CostRecord, DailyCost } from "../api/types";
import {
  CostBarChart,
  CostLineChart,
  CostPieChart,
} from "../components/CostChart";

import { formatCost } from "../utils/format";

export default function Costs() {
  const { data: records } = useQuery<CostRecord[]>("/costs", 10000);
  const { data: dailyCosts } = useQuery<DailyCost[]>(
    "/costs/daily?days=30",
    30000,
  );

  const totalCost = useMemo(
    () =>
      (records ?? []).reduce(
        (sum, r) => sum + (r.cost ?? r.total_cost ?? 0),
        0,
      ),
    [records],
  );

  const todayCost = useMemo(() => {
    const today = new Date().toISOString().split("T")[0];
    return (records ?? [])
      .filter((r) => (r.timestamp ?? r.recorded_at ?? "").startsWith(today!))
      .reduce((sum, r) => sum + (r.cost ?? r.total_cost ?? 0), 0);
  }, [records]);

  const costByAgent = useMemo(() => {
    const map = new Map<string, number>();
    for (const r of records ?? []) {
      const name = r.agent_name ?? "unknown";
      map.set(name, (map.get(name) ?? 0) + (r.cost ?? r.total_cost ?? 0));
    }
    return Array.from(map.entries())
      .map(([name, cost]) => ({ name, cost }))
      .sort((a, b) => b.cost - a.cost);
  }, [records]);

  const tokenBreakdown = useMemo(() => {
    const totalInput = (records ?? []).reduce(
      (s, r) => s + (r.input_tokens ?? 0),
      0,
    );
    const totalOutput = (records ?? []).reduce(
      (s, r) => s + (r.output_tokens ?? 0),
      0,
    );
    if (totalInput === 0 && totalOutput === 0) return [];
    return [
      { name: "Input Tokens", cost: totalInput },
      { name: "Output Tokens", cost: totalOutput },
    ];
  }, [records]);

  const dailyBarData = useMemo(
    () =>
      (dailyCosts ?? []).map((d) => ({
        date: new Date(d.date).toLocaleDateString("en-US", {
          month: "short",
          day: "numeric",
        }),
        cost: d.total_cost ?? d.cost ?? 0,
      })),
    [dailyCosts],
  );

  const cumulativeData = useMemo(() => {
    let running = 0;
    return (dailyCosts ?? []).map((d) => {
      running += d.total_cost ?? d.cost ?? 0;
      return {
        date: new Date(d.date).toLocaleDateString("en-US", {
          month: "short",
          day: "numeric",
        }),
        cost: running,
      };
    });
  }, [dailyCosts]);

  const avgDailyCost = useMemo(() => {
    if (!dailyCosts || dailyCosts.length === 0) return 0;
    const sum = dailyCosts.reduce(
      (s, d) => s + (d.total_cost ?? d.cost ?? 0),
      0,
    );
    return sum / dailyCosts.length;
  }, [dailyCosts]);

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-xl font-bold text-[var(--text-primary)]">
          Cost Analytics
        </h2>
        <p className="text-sm text-[var(--text-secondary)]">
          Track and analyze your AI spending
        </p>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
          <p className="text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)] mb-1">
            Total Cost
          </p>
          <p className="text-2xl font-bold text-[var(--text-primary)]">
            {formatCost(totalCost)}
          </p>
        </div>
        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
          <p className="text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)] mb-1">
            Today
          </p>
          <p className="text-2xl font-bold text-[var(--accent)]">
            {formatCost(todayCost)}
          </p>
        </div>
        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
          <p className="text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)] mb-1">
            Daily Average
          </p>
          <p className="text-2xl font-bold text-[var(--warning)]">
            {formatCost(avgDailyCost)}
          </p>
        </div>
        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
          <p className="text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)] mb-1">
            Projected Monthly
          </p>
          <p className="text-2xl font-bold text-[var(--error)]">
            {formatCost(avgDailyCost * 30)}
          </p>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
          <h3 className="text-sm font-semibold text-[var(--text-primary)] mb-4">
            Cost by Agent
          </h3>
          {costByAgent.length > 0 ? (
            <CostPieChart data={costByAgent} />
          ) : (
            <div className="h-[300px] flex items-center justify-center text-sm text-[var(--text-secondary)]">
              No data
            </div>
          )}
        </div>

        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
          <h3 className="text-sm font-semibold text-[var(--text-primary)] mb-4">
            Token Distribution
          </h3>
          {tokenBreakdown.length > 0 ? (
            <CostPieChart data={tokenBreakdown} />
          ) : (
            <div className="h-[300px] flex items-center justify-center text-sm text-[var(--text-secondary)]">
              No data
            </div>
          )}
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
          <h3 className="text-sm font-semibold text-[var(--text-primary)] mb-4">
            Daily Cost (30 days)
          </h3>
          {dailyBarData.length > 0 ? (
            <CostBarChart data={dailyBarData} />
          ) : (
            <div className="h-[300px] flex items-center justify-center text-sm text-[var(--text-secondary)]">
              No data
            </div>
          )}
        </div>

        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
          <h3 className="text-sm font-semibold text-[var(--text-primary)] mb-4">
            Cumulative Cost Trend
          </h3>
          {cumulativeData.length > 0 ? (
            <CostLineChart data={cumulativeData} />
          ) : (
            <div className="h-[300px] flex items-center justify-center text-sm text-[var(--text-secondary)]">
              No data
            </div>
          )}
        </div>
      </div>

      {costByAgent.length > 0 && (
        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
          <h3 className="text-sm font-semibold text-[var(--text-primary)] mb-4">
            Cost Breakdown
          </h3>
          <div className="space-y-3">
            {costByAgent.map((entry) => {
              const pct = totalCost > 0 ? (entry.cost / totalCost) * 100 : 0;
              return (
                <div key={entry.name} className="flex items-center gap-3">
                  <span className="text-sm text-[var(--text-primary)] w-32 truncate">
                    {entry.name}
                  </span>
                  <div className="flex-1 h-2 rounded-full bg-[var(--bg-tertiary)] overflow-hidden">
                    <div
                      className="h-full rounded-full bg-[var(--accent)] transition-all duration-500"
                      style={{ width: `${pct}%` }}
                    />
                  </div>
                  <span className="text-xs text-[var(--text-secondary)] w-16 text-right">
                    {formatCost(entry.cost)}
                  </span>
                  <span className="text-xs text-[var(--text-secondary)] w-12 text-right">
                    {pct.toFixed(1)}%
                  </span>
                </div>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}
