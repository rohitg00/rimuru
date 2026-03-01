import { useState } from "react";
import { useQuery } from "../hooks/useQuery";
import { apiPost, apiDelete } from "../api/client";
import type { Agent, AgentStatus } from "../api/types";
import AgentCard from "../components/AgentCard";

const STATUS_FILTERS: (AgentStatus | "all")[] = [
  "all",
  "connected",
  "busy",
  "idle",
  "disconnected",
  "error",
];

export default function Agents() {
  const { data: agents, refetch } = useQuery<Agent[]>("/agents", 5000);
  const [filter, setFilter] = useState<AgentStatus | "all">("all");
  const [search, setSearch] = useState("");

  const filtered = (agents ?? []).filter((a) => {
    if (filter !== "all" && a.status !== filter) return false;
    if (search) {
      const q = search.toLowerCase();
      return (
        a.name.toLowerCase().includes(q) ||
        a.agent_type.toLowerCase().includes(q) ||
        (a.model ?? "").toLowerCase().includes(q)
      );
    }
    return true;
  });

  async function handleConnect(id: string) {
    try {
      await apiPost(`/agents/${id}/connect`);
      refetch();
    } catch {
      // handled by UI
    }
  }

  async function handleDisconnect(id: string) {
    try {
      await apiDelete(`/agents/${id}/connect`);
      refetch();
    } catch {
      // handled by UI
    }
  }

  const statusCounts = (agents ?? []).reduce(
    (acc, a) => {
      acc[a.status] = (acc[a.status] ?? 0) + 1;
      return acc;
    },
    {} as Record<string, number>,
  );

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h2 className="text-xl font-bold text-[var(--text-primary)]">
            Agents
          </h2>
          <p className="text-sm text-[var(--text-secondary)]">
            {agents?.length ?? 0} agents registered
          </p>
        </div>
        <div className="flex items-center gap-3">
          <input
            type="text"
            placeholder="Search agents..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="px-3 py-2 text-sm rounded-lg bg-[var(--bg-secondary)] border border-[var(--border)] text-[var(--text-primary)] placeholder-[var(--text-secondary)] focus:outline-none focus:border-[var(--accent)] w-48"
          />
        </div>
      </div>

      <div className="flex flex-wrap items-center gap-2">
        {STATUS_FILTERS.map((s) => {
          const count =
            s === "all" ? agents?.length ?? 0 : statusCounts[s] ?? 0;
          return (
            <button
              key={s}
              onClick={() => setFilter(s)}
              className={`px-3 py-1.5 text-xs font-medium rounded-lg border transition-colors ${
                filter === s
                  ? "bg-[var(--accent)]/10 text-[var(--accent)] border-[var(--accent)]/30"
                  : "bg-[var(--bg-secondary)] text-[var(--text-secondary)] border-[var(--border)] hover:text-[var(--text-primary)]"
              }`}
            >
              {s === "all" ? "All" : s.charAt(0).toUpperCase() + s.slice(1)}
              <span className="ml-1.5 text-[10px] opacity-60">{count}</span>
            </button>
          );
        })}
      </div>

      {filtered.length === 0 ? (
        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-16 text-center">
          <p className="text-[var(--text-secondary)]">
            {search
              ? "No agents match your search"
              : "No agents registered yet"}
          </p>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
          {filtered.map((agent) => (
            <AgentCard
              key={agent.id}
              agent={agent}
              onConnect={handleConnect}
              onDisconnect={handleDisconnect}
            />
          ))}
        </div>
      )}
    </div>
  );
}
