import { useState } from "react";
import { useQuery } from "../hooks/useQuery";
import type { Session } from "../api/types";
import { formatAgentType } from "../api/types";
import StatusBadge from "../components/StatusBadge";
import DataTable from "../components/DataTable";

type ViewMode = "table" | "timeline";

import { formatCost, formatTokens, formatDuration } from "../utils/format";

function TimelineView({ sessions }: { sessions: Session[] }) {
  const sorted = [...sessions].sort(
    (a, b) =>
      new Date(b.started_at).getTime() - new Date(a.started_at).getTime(),
  );

  const grouped = new Map<string, Session[]>();
  for (const s of sorted) {
    const day = new Date(s.started_at).toLocaleDateString("en-US", {
      weekday: "short",
      month: "short",
      day: "numeric",
    });
    const list = grouped.get(day) ?? [];
    list.push(s);
    grouped.set(day, list);
  }

  return (
    <div className="space-y-6">
      {Array.from(grouped.entries()).map(([day, items]) => (
        <div key={day}>
          <h3 className="text-xs font-semibold text-[var(--text-secondary)] uppercase tracking-wider mb-3">
            {day}
          </h3>
          <div className="space-y-2">
            {items.map((s) => (
              <div
                key={s.id}
                className="flex items-center gap-4 rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-4 hover:border-[var(--accent)]/30 transition-colors"
              >
                <div
                  className="w-1.5 h-12 rounded-full"
                  style={{
                    backgroundColor:
                      s.status === "active"
                        ? "var(--success)"
                        : s.status === "completed"
                          ? "var(--accent)"
                          : s.status === "failed"
                            ? "var(--error)"
                            : "var(--warning)",
                  }}
                />
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 mb-1">
                    <span className="text-sm font-medium text-[var(--text-primary)] truncate">
                      {s.agent_name ?? formatAgentType(s.agent_type ?? "-")}
                    </span>
                    <StatusBadge status={s.status} size="sm" />
                  </div>
                  <div className="flex items-center gap-3 text-xs text-[var(--text-secondary)]">
                    <span>{s.model}</span>
                    <span>{s.messages} msgs</span>
                    <span>{formatDuration(s.duration_ms ?? 0)}</span>
                  </div>
                </div>
                <div className="text-right shrink-0">
                  <p className="text-sm font-semibold text-[var(--text-primary)]">
                    {formatCost(s.cost ?? s.total_cost ?? 0)}
                  </p>
                  <p className="text-[10px] text-[var(--text-secondary)]">
                    {formatTokens(s.input_tokens + s.output_tokens)} tokens
                  </p>
                </div>
                <div className="text-xs text-[var(--text-secondary)] shrink-0 w-16 text-right">
                  {new Date(s.started_at).toLocaleTimeString("en-US", {
                    hour: "2-digit",
                    minute: "2-digit",
                  })}
                </div>
              </div>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}

export default function Sessions() {
  const { data: sessions } = useQuery<Session[]>("/sessions", 5000);
  const [view, setView] = useState<ViewMode>("table");
  const [statusFilter, setStatusFilter] = useState<string>("all");

  const filtered = (sessions ?? []).filter(
    (s) => statusFilter === "all" || s.status === statusFilter,
  );

  const columns = [
    {
      key: "agent_name",
      label: "Agent",
      render: (row: Record<string, unknown>) => (
        <span className="font-medium">
          {String(
            row.agent_name ?? formatAgentType(String(row.agent_type ?? "-")),
          )}
        </span>
      ),
    },
    {
      key: "status",
      label: "Status",
      render: (row: Record<string, unknown>) => (
        <StatusBadge status={String(row.status)} size="sm" />
      ),
    },
    { key: "model", label: "Model" },
    {
      key: "messages",
      label: "Messages",
      render: (row: Record<string, unknown>) => String(row.messages),
    },
    {
      key: "cost",
      label: "Cost",
      render: (row: Record<string, unknown>) =>
        formatCost((row.cost ?? row.total_cost ?? 0) as number),
    },
    {
      key: "duration_ms",
      label: "Duration",
      render: (row: Record<string, unknown>) =>
        formatDuration((row.duration_ms ?? 0) as number),
    },
    {
      key: "input_tokens",
      label: "Tokens",
      render: (row: Record<string, unknown>) =>
        formatTokens(
          (row.input_tokens as number) + (row.output_tokens as number),
        ),
    },
    {
      key: "started_at",
      label: "Started",
      render: (row: Record<string, unknown>) =>
        new Date(String(row.started_at)).toLocaleString("en-US", {
          month: "short",
          day: "numeric",
          hour: "2-digit",
          minute: "2-digit",
        }),
    },
  ];

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h2 className="text-xl font-bold text-[var(--text-primary)]">
            Sessions
          </h2>
          <p className="text-sm text-[var(--text-secondary)]">
            {sessions?.length ?? 0} sessions recorded
          </p>
        </div>
        <div className="flex items-center gap-3">
          <select
            value={statusFilter}
            onChange={(e) => setStatusFilter(e.target.value)}
            className="px-3 py-2 text-sm rounded-lg bg-[var(--bg-secondary)] border border-[var(--border)] text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)]"
          >
            <option value="all">All Status</option>
            <option value="active">Active</option>
            <option value="completed">Completed</option>
            <option value="failed">Failed</option>
            <option value="paused">Paused</option>
          </select>
          <div className="flex rounded-lg border border-[var(--border)] overflow-hidden">
            <button
              onClick={() => setView("table")}
              className={`px-3 py-1.5 text-xs font-medium transition-colors ${
                view === "table"
                  ? "bg-[var(--accent)]/10 text-[var(--accent)]"
                  : "bg-[var(--bg-secondary)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]"
              }`}
            >
              Table
            </button>
            <button
              onClick={() => setView("timeline")}
              className={`px-3 py-1.5 text-xs font-medium border-l border-[var(--border)] transition-colors ${
                view === "timeline"
                  ? "bg-[var(--accent)]/10 text-[var(--accent)]"
                  : "bg-[var(--bg-secondary)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]"
              }`}
            >
              Timeline
            </button>
          </div>
        </div>
      </div>

      {view === "table" ? (
        <DataTable
          columns={columns}
          data={filtered as unknown as Record<string, unknown>[]}
          keyField="id"
          searchable
          searchFields={["agent_name", "model", "status"]}
          emptyMessage="No sessions found"
        />
      ) : (
        <TimelineView sessions={filtered} />
      )}
    </div>
  );
}
