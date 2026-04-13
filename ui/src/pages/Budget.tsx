import { useState } from "react";
import { useQuery } from "../hooks/useQuery";
import { apiPost } from "../api/client";
import { formatCost } from "../utils/format";

interface BudgetStatus {
  monthly_limit: number;
  monthly_spent: number;
  monthly_remaining: number;
  daily_limit: number;
  daily_spent: number;
  daily_remaining: number;
  session_limit: number;
  agent_daily_limit: number;
  alert_threshold: number;
  action_on_exceed: string;
  status: string;
  burn_rate_daily: number;
  projected_monthly: number;
  days_in_month: number;
}

interface BudgetAlert {
  timestamp: string;
  alert_type: string;
  message: string;
  monthly_spent: number;
  daily_spent: number;
  limit_hit: string;
}

interface AlertsResponse {
  alerts: BudgetAlert[];
  count: number;
  total: number;
}

function statusColor(status: string): string {
  switch (status) {
    case "exceeded":
      return "var(--error)";
    case "warning":
      return "var(--warning)";
    default:
      return "var(--success)";
  }
}

function CapBar({
  label,
  spent,
  limit,
  threshold,
}: {
  label: string;
  spent: number;
  limit: number;
  threshold: number;
}) {
  if (limit <= 0) {
    return (
      <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
        <div className="flex items-baseline justify-between mb-2">
          <p className="text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">
            {label}
          </p>
          <p className="text-xs text-[var(--text-secondary)]">disabled</p>
        </div>
        <p className="text-2xl font-bold text-[var(--text-primary)]">
          {formatCost(spent)}
        </p>
        <p className="text-xs text-[var(--text-secondary)] mt-1">
          set a limit in Settings to enable
        </p>
      </div>
    );
  }

  const pct = Math.min(100, (spent / limit) * 100);
  const exceeded = spent >= limit;
  const warn = !exceeded && spent >= limit * threshold;
  const color = exceeded
    ? "var(--error)"
    : warn
    ? "var(--warning)"
    : "var(--success)";
  const remaining = Math.max(0, limit - spent);

  return (
    <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
      <div className="flex items-baseline justify-between mb-2">
        <p className="text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">
          {label}
        </p>
        <p className="text-xs font-mono" style={{ color }}>
          {pct.toFixed(0)}%
        </p>
      </div>
      <p className="text-2xl font-bold text-[var(--text-primary)]">
        {formatCost(spent)}
        <span className="text-sm text-[var(--text-secondary)] font-normal">
          {" "}
          / {formatCost(limit)}
        </span>
      </p>
      <p className="text-xs text-[var(--text-secondary)] mt-1">
        {formatCost(remaining)} remaining
      </p>
      <div className="mt-3 h-2 w-full rounded-full bg-[var(--border)] overflow-hidden">
        <div
          className="h-full transition-all duration-300"
          style={{ width: `${pct}%`, backgroundColor: color }}
        />
      </div>
    </div>
  );
}

export default function Budget() {
  const { data: status, refetch } = useQuery<BudgetStatus>(
    "/budget/status",
    5000,
  );
  const { data: alertsResp, refetch: refetchAlerts } = useQuery<AlertsResponse>(
    "/budget/alerts?limit=20",
    10000,
  );

  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState({
    monthly_limit: 0,
    daily_limit: 0,
    session_limit: 0,
    daily_agent_limit: 0,
    alert_threshold: 0.8,
    action: "alert",
  });
  const [saving, setSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);

  function startEditing() {
    if (!status) return;
    setDraft({
      monthly_limit: status.monthly_limit,
      daily_limit: status.daily_limit,
      session_limit: status.session_limit,
      daily_agent_limit: status.agent_daily_limit ?? 0,
      alert_threshold: status.alert_threshold,
      action: status.action_on_exceed,
    });
    setSaveError(null);
    setEditing(true);
  }

  async function save() {
    setSaving(true);
    setSaveError(null);
    try {
      await apiPost("/budget/set", draft);
      setEditing(false);
      refetch();
      refetchAlerts();
    } catch (err) {
      setSaveError(err instanceof Error ? err.message : "save failed");
    } finally {
      setSaving(false);
    }
  }

  if (!status) {
    return (
      <div className="p-6 text-[var(--text-secondary)]">Loading budget...</div>
    );
  }

  const projectedOver =
    status.monthly_limit > 0 && status.projected_monthly > status.monthly_limit;

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-start justify-between">
        <div>
          <h1 className="text-2xl font-bold text-[var(--text-primary)]">
            Budget
          </h1>
          <p className="text-sm text-[var(--text-secondary)] mt-1">
            Real-time cost caps. Threshold and exceed events fire hooks; with
            <code className="mx-1 px-1 rounded bg-[var(--bg-tertiary)]">block</code>
            action, exceeded caps reject cost recording.
          </p>
        </div>
        <span
          className="px-3 py-1 rounded-full text-xs font-semibold uppercase tracking-wider"
          style={{
            color: statusColor(status.status),
            backgroundColor: `${statusColor(status.status)}20`,
          }}
        >
          {status.status}
        </span>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <CapBar
          label="Monthly"
          spent={status.monthly_spent}
          limit={status.monthly_limit}
          threshold={status.alert_threshold}
        />
        <CapBar
          label="Daily"
          spent={status.daily_spent}
          limit={status.daily_limit}
          threshold={status.alert_threshold}
        />
        <CapBar
          label="Per Session"
          spent={0}
          limit={status.session_limit}
          threshold={status.alert_threshold}
        />
        <CapBar
          label="Per Agent / Day"
          spent={0}
          limit={status.agent_daily_limit ?? 0}
          threshold={status.alert_threshold}
        />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
          <p className="text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)] mb-2">
            Burn Rate
          </p>
          <p className="text-2xl font-bold text-[var(--text-primary)]">
            {formatCost(status.burn_rate_daily)}
            <span className="text-sm text-[var(--text-secondary)] font-normal">
              {" "}
              / day
            </span>
          </p>
          <p className="text-xs text-[var(--text-secondary)] mt-1">
            average across the current month
          </p>
        </div>
        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
          <p className="text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)] mb-2">
            Projected Monthly
          </p>
          <p
            className="text-2xl font-bold"
            style={{
              color: projectedOver
                ? "var(--error)"
                : "var(--text-primary)",
            }}
          >
            {formatCost(status.projected_monthly)}
          </p>
          <p className="text-xs text-[var(--text-secondary)] mt-1">
            burn_rate * {status.days_in_month ?? 30} days
            {projectedOver ? " — over cap" : ""}
          </p>
        </div>
        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
          <p className="text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)] mb-2">
            Action on Exceed
          </p>
          <p className="text-2xl font-bold text-[var(--text-primary)] uppercase">
            {status.action_on_exceed}
          </p>
          <p className="text-xs text-[var(--text-secondary)] mt-1">
            alert threshold: {(status.alert_threshold * 100).toFixed(0)}%
          </p>
        </div>
      </div>

      <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
        <div className="flex items-baseline justify-between mb-4">
          <h2 className="text-lg font-semibold text-[var(--text-primary)]">
            Configure caps
          </h2>
          {!editing ? (
            <button
              type="button"
              className="text-xs px-3 py-1 rounded-lg border border-[var(--border)] text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)]"
              onClick={startEditing}
            >
              edit
            </button>
          ) : (
            <div className="flex gap-2">
              <button
                type="button"
                className="text-xs px-3 py-1 rounded-lg border border-[var(--border)] text-[var(--text-secondary)]"
                onClick={() => setEditing(false)}
                disabled={saving}
              >
                cancel
              </button>
              <button
                type="button"
                className="text-xs px-3 py-1 rounded-lg bg-[var(--accent)] text-white"
                onClick={save}
                disabled={saving}
              >
                {saving ? "saving..." : "save"}
              </button>
            </div>
          )}
        </div>

        {editing ? (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <NumberField
              label="Monthly limit ($)"
              value={draft.monthly_limit}
              onChange={(v) => setDraft({ ...draft, monthly_limit: v })}
            />
            <NumberField
              label="Daily limit ($)"
              value={draft.daily_limit}
              onChange={(v) => setDraft({ ...draft, daily_limit: v })}
            />
            <NumberField
              label="Per-session limit ($)"
              value={draft.session_limit}
              onChange={(v) => setDraft({ ...draft, session_limit: v })}
            />
            <NumberField
              label="Per-agent daily limit ($)"
              value={draft.daily_agent_limit}
              onChange={(v) =>
                setDraft({ ...draft, daily_agent_limit: v })
              }
            />
            <NumberField
              label="Alert threshold (0.0 - 1.0)"
              value={draft.alert_threshold}
              step={0.05}
              onChange={(v) => setDraft({ ...draft, alert_threshold: v })}
            />
            <div>
              <label className="block text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)] mb-1">
                Action on exceed
              </label>
              <select
                className="w-full rounded-lg border border-[var(--border)] bg-[var(--bg-tertiary)] px-3 py-2 text-sm text-[var(--text-primary)]"
                value={draft.action}
                onChange={(e) =>
                  setDraft({ ...draft, action: e.target.value })
                }
              >
                <option value="alert">alert</option>
                <option value="warn">warn</option>
                <option value="block">block</option>
              </select>
            </div>
          </div>
        ) : (
          <p className="text-sm text-[var(--text-secondary)]">
            All limits are in USD. Set any limit to <code>0.0</code> to disable
            it.
          </p>
        )}
        {saveError && (
          <p className="mt-3 text-xs text-[var(--error)]">{saveError}</p>
        )}
      </div>

      <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
        <div className="flex items-baseline justify-between mb-4">
          <h2 className="text-lg font-semibold text-[var(--text-primary)]">
            Recent alerts
          </h2>
          <p className="text-xs text-[var(--text-secondary)]">
            {alertsResp?.total ?? 0} total
          </p>
        </div>
        {!alertsResp || alertsResp.alerts.length === 0 ? (
          <p className="text-sm text-[var(--text-secondary)]">
            No alerts yet. Threshold crossings and exceeded caps will appear
            here.
          </p>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="text-left text-xs uppercase tracking-wider text-[var(--text-secondary)] border-b border-[var(--border)]">
                  <th className="py-2 pr-4">Time</th>
                  <th className="py-2 pr-4">Type</th>
                  <th className="py-2 pr-4">Hit</th>
                  <th className="py-2 pr-4">Monthly</th>
                  <th className="py-2 pr-4">Daily</th>
                  <th className="py-2">Message</th>
                </tr>
              </thead>
              <tbody>
                {alertsResp.alerts.map((a) => (
                  <tr
                    key={a.timestamp + a.message}
                    className="border-b border-[var(--border)] last:border-b-0"
                  >
                    <td className="py-2 pr-4 font-mono text-xs text-[var(--text-secondary)]">
                      {new Date(a.timestamp).toLocaleString()}
                    </td>
                    <td className="py-2 pr-4">
                      <span
                        className="px-2 py-0.5 rounded-full text-xs font-semibold uppercase"
                        style={{
                          color: statusColor(a.alert_type),
                          backgroundColor: `${statusColor(a.alert_type)}20`,
                        }}
                      >
                        {a.alert_type}
                      </span>
                    </td>
                    <td className="py-2 pr-4 text-xs text-[var(--text-secondary)]">
                      {a.limit_hit}
                    </td>
                    <td className="py-2 pr-4 font-mono">
                      {formatCost(a.monthly_spent)}
                    </td>
                    <td className="py-2 pr-4 font-mono">
                      {formatCost(a.daily_spent)}
                    </td>
                    <td className="py-2 text-xs text-[var(--text-secondary)]">
                      {a.message}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}

function NumberField({
  label,
  value,
  step = 0.01,
  onChange,
}: {
  label: string;
  value: number;
  step?: number;
  onChange: (v: number) => void;
}) {
  return (
    <div>
      <label className="block text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)] mb-1">
        {label}
      </label>
      <input
        type="number"
        min={0}
        step={step}
        value={value}
        onChange={(e) => onChange(parseFloat(e.target.value) || 0)}
        className="w-full rounded-lg border border-[var(--border)] bg-[var(--bg-tertiary)] px-3 py-2 text-sm font-mono text-[var(--text-primary)]"
      />
    </div>
  );
}
