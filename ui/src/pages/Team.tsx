import { useId, useMemo, useState } from "react";
import { useQuery } from "../hooks/useQuery";
import { apiPost } from "../api/client";
import { formatCost } from "../utils/format";

interface UserBreakdown {
  user_id: string;
  display_name: string | null;
  total_cost: number;
  total_input_tokens: number;
  total_output_tokens: number;
  record_count: number;
}

interface TeamCostAggregation {
  team_id: string;
  member_count: number;
  grand_total: number;
  total_records: number;
  per_user: UserBreakdown[];
  period_start: string | null;
  period_end: string | null;
}

function Header() {
  return (
    <div>
      <h1 className="text-2xl font-bold text-[var(--text-primary)]">Team</h1>
      <p className="text-sm text-[var(--text-secondary)] mt-1">
        Aggregate cost across all users on a team. JWT-gated endpoints; set{" "}
        <code className="mx-1 px-1 rounded bg-[var(--bg-tertiary)]">
          RIMURU_JWT_SECRET
        </code>{" "}
        or{" "}
        <code className="mx-1 px-1 rounded bg-[var(--bg-tertiary)]">
          RIMURU_ALLOW_TEAM_WITHOUT_JWT=1
        </code>{" "}
        for local dev.
      </p>
    </div>
  );
}

function StatCard({
  label,
  value,
  sub,
}: {
  label: string;
  value: string;
  sub?: string;
}) {
  return (
    <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
      <p className="text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)] mb-2">
        {label}
      </p>
      <p className="text-2xl font-bold text-[var(--text-primary)]">{value}</p>
      {sub && (
        <p className="text-xs text-[var(--text-secondary)] mt-1">{sub}</p>
      )}
    </div>
  );
}

function LeaderboardBars({ rows }: { rows: UserBreakdown[] }) {
  const max = Math.max(1, ...rows.map((r) => r.total_cost));
  if (rows.length === 0) {
    return (
      <p className="text-sm text-[var(--text-secondary)]">
        No spend recorded yet for this team.
      </p>
    );
  }
  return (
    <div className="space-y-2">
      {rows.map((r) => {
        const pct = (r.total_cost / max) * 100;
        return (
          <div key={r.user_id}>
            <div className="flex justify-between text-xs mb-1">
              <span className="text-[var(--text-primary)] font-medium">
                {r.display_name ?? r.user_id}
              </span>
              <span className="font-mono text-[var(--text-secondary)]">
                {formatCost(r.total_cost)}
              </span>
            </div>
            <div className="h-2 rounded-full bg-[var(--border)] overflow-hidden">
              <div
                className="h-full bg-[var(--accent)] transition-all duration-300"
                style={{ width: `${pct}%` }}
              />
            </div>
          </div>
        );
      })}
    </div>
  );
}

interface SelectorProps {
  teamId: string;
  setTeamId: (v: string) => void;
  onCreated: (msg: string) => void;
  onError: (msg: string) => void;
}

function TeamSelector({
  teamId,
  setTeamId,
  onCreated,
  onError,
}: SelectorProps) {
  const teamIdInputId = useId();
  const createNameId = useId();
  const [draftName, setDraftName] = useState("");

  async function createTeam() {
    const trimmedName = draftName.trim();
    if (!trimmedName) return;
    try {
      const res = await apiPost<{ id: string; name: string }>("/team/create", {
        name: trimmedName,
      });
      setTeamId(res.id);
      localStorage.setItem("rimuru.team_id", res.id);
      onCreated(`Created team ${res.name} (${res.id})`);
      setDraftName("");
    } catch (err) {
      onError(err instanceof Error ? err.message : "create failed");
    }
  }

  return (
    <div className="flex flex-col md:flex-row md:items-end gap-3">
      <div className="flex-1">
        <label
          htmlFor={teamIdInputId}
          className="block text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)] mb-1"
        >
          Active team id
        </label>
        <input
          id={teamIdInputId}
          value={teamId}
          onChange={(e) => {
            setTeamId(e.target.value);
            localStorage.setItem("rimuru.team_id", e.target.value);
          }}
          placeholder="paste team uuid"
          className="w-full rounded-lg border border-[var(--border)] bg-[var(--bg-tertiary)] px-3 py-2 text-sm font-mono text-[var(--text-primary)]"
        />
      </div>
      <div className="flex-1">
        <label
          htmlFor={createNameId}
          className="block text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)] mb-1"
        >
          Create new team
        </label>
        <div className="flex gap-2">
          <input
            id={createNameId}
            value={draftName}
            onChange={(e) => setDraftName(e.target.value)}
            placeholder="team name"
            className="flex-1 rounded-lg border border-[var(--border)] bg-[var(--bg-tertiary)] px-3 py-2 text-sm text-[var(--text-primary)]"
          />
          <button
            type="button"
            onClick={createTeam}
            disabled={!draftName.trim()}
            className="text-xs px-3 py-2 rounded-lg bg-[var(--accent)] text-white disabled:opacity-40"
          >
            create
          </button>
        </div>
      </div>
    </div>
  );
}

interface AddUserProps {
  teamId: string;
  onAdded: (msg: string) => void;
  onError: (msg: string) => void;
}

function AddUserForm({ teamId, onAdded, onError }: AddUserProps) {
  const addUserId = useId();
  const addUserDisplayId = useId();
  const [newUser, setNewUser] = useState("");
  const [newDisplay, setNewDisplay] = useState("");

  async function addUser() {
    const trimmedUser = newUser.trim();
    const trimmedDisplay = newDisplay.trim();
    if (!trimmedUser) return;
    try {
      await apiPost("/team/add_user", {
        team_id: teamId,
        user_id: trimmedUser,
        display_name: trimmedDisplay || undefined,
      });
      onAdded(`Added ${trimmedUser} to team`);
      setNewUser("");
      setNewDisplay("");
    } catch (err) {
      onError(err instanceof Error ? err.message : "add failed");
    }
  }

  return (
    <div className="flex flex-col md:flex-row md:items-end gap-3 pt-2 border-t border-[var(--border)]">
      <div className="flex-1">
        <label
          htmlFor={addUserId}
          className="block text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)] mb-1"
        >
          Add user_id
        </label>
        <input
          id={addUserId}
          value={newUser}
          onChange={(e) => setNewUser(e.target.value)}
          placeholder="alice"
          className="w-full rounded-lg border border-[var(--border)] bg-[var(--bg-tertiary)] px-3 py-2 text-sm font-mono text-[var(--text-primary)]"
        />
      </div>
      <div className="flex-1">
        <label
          htmlFor={addUserDisplayId}
          className="block text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)] mb-1"
        >
          Display name (optional)
        </label>
        <input
          id={addUserDisplayId}
          value={newDisplay}
          onChange={(e) => setNewDisplay(e.target.value)}
          placeholder="Alice Smith"
          className="w-full rounded-lg border border-[var(--border)] bg-[var(--bg-tertiary)] px-3 py-2 text-sm text-[var(--text-primary)]"
        />
      </div>
      <button
        type="button"
        onClick={addUser}
        disabled={!newUser.trim()}
        className="text-xs px-3 py-2 rounded-lg bg-[var(--accent)] text-white disabled:opacity-40"
      >
        add user
      </button>
    </div>
  );
}

function LoadedTeam({ teamId }: { teamId: string }) {
  const { data: agg, error } = useQuery<TeamCostAggregation>(
    `/team/costs?team_id=${encodeURIComponent(teamId)}`,
    10000,
  );

  // Only treat the first entry as the "top spender" when their spend is
  // strictly positive. Otherwise the card shows a spurious $0.00 winner.
  const topSpender = useMemo(
    () => agg?.per_user.find((u) => u.total_cost > 0),
    [agg],
  );

  if (error) {
    return (
      <div className="rounded-xl border border-[var(--error)] bg-[var(--bg-secondary)] p-5">
        <p className="text-sm font-semibold text-[var(--error)]">
          Failed to load team data
        </p>
        <p className="text-xs text-[var(--text-secondary)] mt-1 font-mono">
          {error}
        </p>
      </div>
    );
  }

  return (
    <>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <StatCard
          label="Team Spend"
          value={formatCost(agg?.grand_total ?? 0)}
          sub={`${agg?.total_records ?? 0} cost records`}
        />
        <StatCard
          label="Members"
          value={String(agg?.member_count ?? 0)}
          sub="users on this team"
        />
        <StatCard
          label="Top Spender"
          value={
            topSpender
              ? (topSpender.display_name ?? topSpender.user_id)
              : "—"
          }
          sub={topSpender ? formatCost(topSpender.total_cost) : "no spend yet"}
        />
      </div>

      <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
        <h2 className="text-lg font-semibold text-[var(--text-primary)] mb-4">
          Leaderboard
        </h2>
        <LeaderboardBars rows={agg?.per_user ?? []} />
      </div>

      <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
        <h2 className="text-lg font-semibold text-[var(--text-primary)] mb-4">
          Per-user breakdown
        </h2>
        {!agg || agg.per_user.length === 0 ? (
          <p className="text-sm text-[var(--text-secondary)]">
            No per-user cost data yet.
          </p>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="text-left text-xs uppercase tracking-wider text-[var(--text-secondary)] border-b border-[var(--border)]">
                  <th className="py-2 pr-4">User</th>
                  <th className="py-2 pr-4">Records</th>
                  <th className="py-2 pr-4">Input tok</th>
                  <th className="py-2 pr-4">Output tok</th>
                  <th className="py-2">Total</th>
                </tr>
              </thead>
              <tbody>
                {agg.per_user.map((u) => (
                  <tr
                    key={u.user_id}
                    className="border-b border-[var(--border)] last:border-b-0"
                  >
                    <td className="py-2 pr-4">
                      <p className="text-[var(--text-primary)] font-medium">
                        {u.display_name ?? u.user_id}
                      </p>
                      {u.display_name && (
                        <p className="text-xs font-mono text-[var(--text-secondary)]">
                          {u.user_id}
                        </p>
                      )}
                    </td>
                    <td className="py-2 pr-4 font-mono">{u.record_count}</td>
                    <td className="py-2 pr-4 font-mono">
                      {u.total_input_tokens.toLocaleString()}
                    </td>
                    <td className="py-2 pr-4 font-mono">
                      {u.total_output_tokens.toLocaleString()}
                    </td>
                    <td className="py-2 font-mono">
                      {formatCost(u.total_cost)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </>
  );
}

export default function Team() {
  const [teamId, setTeamId] = useState<string>(
    () => localStorage.getItem("rimuru.team_id") ?? "",
  );
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  return (
    <div className="p-6 space-y-6">
      <Header />

      <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5 space-y-4">
        <TeamSelector
          teamId={teamId}
          setTeamId={setTeamId}
          onCreated={(m) => {
            setMessage(m);
            setError(null);
          }}
          onError={(m) => {
            setError(m);
            setMessage(null);
          }}
        />

        {teamId && (
          <AddUserForm
            teamId={teamId}
            onAdded={(m) => {
              setMessage(m);
              setError(null);
            }}
            onError={(m) => {
              setError(m);
              setMessage(null);
            }}
          />
        )}

        {message && <p className="text-xs text-[var(--success)]">{message}</p>}
        {error && <p className="text-xs text-[var(--error)]">{error}</p>}
      </div>

      {teamId && <LoadedTeam teamId={teamId} />}
    </div>
  );
}
