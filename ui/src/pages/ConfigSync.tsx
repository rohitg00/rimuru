import { useCallback, useEffect, useState } from "react";
import { apiGet, apiPost } from "../api/client";

interface McpServer {
  command: string;
  args: string[];
  env?: Record<string, string>;
  disabled?: boolean;
}

interface SyncCanonical {
  mcp_servers: Record<string, McpServer>;
  allowed_tools: string[];
  denied_tools: string[];
  custom_instructions: string | null;
  model_preferences: Record<string, string>;
}

interface AgentDiff {
  mcp_servers: { added: string[]; removed: string[]; changed: string[] };
  allowed_tools: { added: string[]; removed: string[] };
  denied_tools: { added: string[]; removed: string[] };
  model_preferences: { added: string[]; removed: string[]; changed: string[] };
  custom_instructions_changed: boolean;
}

interface ExportResponse {
  canonical: SyncCanonical;
  per_agent: Record<string, SyncCanonical>;
  errors: Record<string, string>;
  exported_at: string;
}

interface DiffResponse {
  diffs: Record<string, AgentDiff>;
  target_source: "explicit" | "merged_canonical";
}

interface ImportEntry {
  config_file: string;
  diff: AgentDiff;
  applied: boolean;
  reason?: string;
  error?: string;
  backup_file?: string | null;
}

interface ImportResponse {
  results: Record<string, ImportEntry>;
  applied: boolean;
  imported_at: string;
}

function emptyDiff(): AgentDiff {
  return {
    mcp_servers: { added: [], removed: [], changed: [] },
    allowed_tools: { added: [], removed: [] },
    denied_tools: { added: [], removed: [] },
    model_preferences: { added: [], removed: [], changed: [] },
    custom_instructions_changed: false,
  };
}

function diffCount(d: AgentDiff): number {
  return (
    d.mcp_servers.added.length +
    d.mcp_servers.removed.length +
    d.mcp_servers.changed.length +
    d.allowed_tools.added.length +
    d.allowed_tools.removed.length +
    d.denied_tools.added.length +
    d.denied_tools.removed.length +
    d.model_preferences.added.length +
    d.model_preferences.removed.length +
    d.model_preferences.changed.length +
    (d.custom_instructions_changed ? 1 : 0)
  );
}

function DiffRow({ label, items }: { label: string; items: string[] }) {
  if (items.length === 0) return null;
  return (
    <div className="flex flex-wrap items-baseline gap-2 py-1">
      <span className="text-xs uppercase tracking-wider text-[var(--text-secondary)] shrink-0">
        {label}
      </span>
      <span className="font-mono text-xs text-[var(--text-primary)] break-all">
        {items.join(", ")}
      </span>
    </div>
  );
}

function DiffCard({ agent, diff }: { agent: string; diff: AgentDiff }) {
  const count = diffCount(diff);
  return (
    <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-4">
      <div className="flex items-baseline justify-between mb-2">
        <span className="text-sm font-semibold text-[var(--text-primary)]">
          {agent}
        </span>
        <span
          className="text-xs px-2 py-0.5 rounded-full"
          style={{
            color: count === 0 ? "var(--success)" : "var(--warning)",
            backgroundColor:
              count === 0
                ? "var(--success)20"
                : "var(--warning)20",
          }}
        >
          {count === 0 ? "in sync" : `${count} change${count === 1 ? "" : "s"}`}
        </span>
      </div>
      {count === 0 ? (
        <p className="text-xs text-[var(--text-secondary)]">
          Matches the merged canonical state.
        </p>
      ) : (
        <div>
          <DiffRow label="mcp added" items={diff.mcp_servers.added} />
          <DiffRow label="mcp changed" items={diff.mcp_servers.changed} />
          <DiffRow label="mcp removed" items={diff.mcp_servers.removed} />
          <DiffRow label="allow added" items={diff.allowed_tools.added} />
          <DiffRow label="allow removed" items={diff.allowed_tools.removed} />
          <DiffRow label="deny added" items={diff.denied_tools.added} />
          <DiffRow label="deny removed" items={diff.denied_tools.removed} />
          <DiffRow label="model added" items={diff.model_preferences.added} />
          <DiffRow
            label="model changed"
            items={diff.model_preferences.changed}
          />
          <DiffRow
            label="model removed"
            items={diff.model_preferences.removed}
          />
          {diff.custom_instructions_changed && (
            <div className="text-xs text-[var(--text-primary)] mt-1">
              custom instructions changed
            </div>
          )}
        </div>
      )}
    </div>
  );
}

export default function ConfigSync() {
  const [canonical, setCanonical] = useState<SyncCanonical | null>(null);
  const [diffs, setDiffs] = useState<Record<string, AgentDiff>>({});
  // Agent list comes from /sync/export.per_agent (only installed
  // agents) unioned with export.errors (failed reads stay visible).
  // Deriving from /sync/diff's response would render every supported
  // adapter even on a clean machine, which misled the UI into showing
  // "in sync" for agents that were never installed.
  const [agentNames, setAgentNames] = useState<string[]>([]);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [loading, setLoading] = useState(false);
  const [syncing, setSyncing] = useState(false);
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [lastImport, setLastImport] = useState<ImportResponse | null>(null);
  const [pageError, setPageError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    setPageError(null);
    try {
      const [exp, diff] = await Promise.all([
        apiGet<ExportResponse>("/sync/export"),
        apiGet<DiffResponse>("/sync/diff"),
      ]);
      setCanonical(exp.canonical);
      setDiffs(diff.diffs);
      setErrors(exp.errors);
      setAgentNames(
        [
          ...new Set([
            ...Object.keys(exp.per_agent),
            ...Object.keys(exp.errors),
          ]),
        ].sort(),
      );
    } catch (err) {
      setPageError(err instanceof Error ? err.message : "Failed to load sync state");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  async function applySync() {
    if (!canonical) return;
    setSyncing(true);
    setPageError(null);
    try {
      const resp = await apiPost<ImportResponse>("/sync/import", {
        canonical,
        apply: true,
      });
      setLastImport(resp);
      setConfirmOpen(false);
      await refresh();
    } catch (err) {
      setPageError(err instanceof Error ? err.message : "Sync failed");
    } finally {
      setSyncing(false);
    }
  }

  const totalChanges = agentNames.reduce(
    (acc, name) => acc + diffCount(diffs[name] ?? emptyDiff()),
    0,
  );
  // Block Sync-all whenever export reported a read error. The import
  // handler's write gate (LoadState::Failed) refuses to write that
  // specific agent, but disabling the button keeps the user from
  // clicking through and wondering why the error entry was skipped.
  const hasReadErrors = Object.keys(errors).length > 0;

  return (
    <div className="space-y-4 max-w-3xl">
      <div className="flex items-baseline justify-between">
        <div>
          <h3 className="text-lg font-bold text-[var(--text-primary)]">
            Cross-agent config sync
          </h3>
          <p className="text-sm text-[var(--text-secondary)]">
            MCP servers, allowed/denied tools, and custom instructions from
            every installed agent merged into one canonical state.
          </p>
        </div>
        <button
          type="button"
          onClick={() => void refresh()}
          disabled={loading}
          className="text-xs px-3 py-1 rounded-lg border border-[var(--border)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] disabled:opacity-40"
        >
          {loading ? "refreshing..." : "refresh"}
        </button>
      </div>

      {pageError && (
        <div className="rounded-lg border border-[var(--error)]/30 bg-[var(--error)]/10 px-4 py-3 text-sm text-[var(--error)]">
          {pageError}
        </div>
      )}

      {Object.entries(errors).length > 0 && (
        <div className="rounded-lg border border-[var(--warning)]/30 bg-[var(--warning)]/10 px-4 py-3 text-xs text-[var(--warning)]">
          <div className="font-semibold mb-1">Read errors</div>
          {Object.entries(errors).map(([agent, err]) => (
            <div key={agent}>
              <span className="font-mono">{agent}</span>: {err}
            </div>
          ))}
        </div>
      )}

      {agentNames.length === 0 && !loading ? (
        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5 text-sm text-[var(--text-secondary)]">
          No installed agents detected. Config sync picks up Claude Code,
          Cursor, Codex, and Gemini CLI today.
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
          {agentNames.map((name) => (
            <DiffCard key={name} agent={name} diff={diffs[name] ?? emptyDiff()} />
          ))}
        </div>
      )}

      <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm font-semibold text-[var(--text-primary)]">
              Sync all agents
            </p>
            <p className="text-xs text-[var(--text-secondary)] mt-0.5">
              {hasReadErrors
                ? "Resolve the read errors above before syncing — rimuru refuses to overwrite configs it couldn't read."
                : totalChanges === 0
                  ? "All agents already match. No changes to apply."
                  : `${totalChanges} change${totalChanges === 1 ? "" : "s"} across ${agentNames.length} agent${agentNames.length === 1 ? "" : "s"}. Backups are taken before write.`}
            </p>
          </div>
          <button
            type="button"
            onClick={() => setConfirmOpen(true)}
            disabled={
              totalChanges === 0 || syncing || !canonical || hasReadErrors
            }
            className="px-4 py-2 text-sm font-medium rounded-lg bg-[var(--accent)] text-white disabled:opacity-40 disabled:cursor-not-allowed"
          >
            Sync all
          </button>
        </div>
      </div>

      {confirmOpen && (
        <div className="rounded-xl border border-[var(--warning)]/40 bg-[var(--warning)]/10 p-5 space-y-3">
          <p className="text-sm text-[var(--text-primary)]">
            This will rewrite every installed agent's native config to match
            the merged canonical state. Each file gets a timestamped backup
            before the write. Continue?
          </p>
          <div className="flex gap-2">
            <button
              type="button"
              onClick={() => setConfirmOpen(false)}
              disabled={syncing}
              className="text-xs px-3 py-1 rounded-lg border border-[var(--border)] text-[var(--text-secondary)]"
            >
              cancel
            </button>
            <button
              type="button"
              onClick={() => void applySync()}
              disabled={syncing}
              className="text-xs px-3 py-1 rounded-lg bg-[var(--warning)] text-white"
            >
              {syncing ? "applying..." : "yes, sync all"}
            </button>
          </div>
        </div>
      )}

      {lastImport && (
        <div className="rounded-xl border border-[var(--success)]/40 bg-[var(--success)]/10 p-5">
          <p className="text-sm font-semibold text-[var(--success)] mb-2">
            Last sync {new Date(lastImport.imported_at).toLocaleString()}
          </p>
          <div className="space-y-1 text-xs">
            {Object.entries(lastImport.results).map(([agent, entry]) => (
              <div key={agent} className="flex items-baseline gap-2">
                <span className="font-semibold text-[var(--text-primary)] w-28">
                  {agent}
                </span>
                <span
                  style={{
                    color: entry.applied
                      ? "var(--success)"
                      : entry.error
                      ? "var(--error)"
                      : "var(--text-secondary)",
                  }}
                >
                  {entry.applied
                    ? "applied"
                    : entry.error
                    ? `error: ${entry.error}`
                    : `skipped (${entry.reason ?? "unknown"})`}
                </span>
                {entry.backup_file && (
                  <span className="text-[var(--text-secondary)] font-mono">
                    backup: {entry.backup_file.split("/").slice(-1)[0]}
                  </span>
                )}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
