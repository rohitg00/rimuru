import { useState } from "react";
import { useQuery } from "../hooks/useQuery";
import type { PluginManifest } from "../api/types";
import StatusBadge from "../components/StatusBadge";

function PluginCard({ plugin }: { plugin: PluginManifest }) {
  return (
    <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5 transition-all hover:border-[var(--accent)]/30">
      <div className="flex items-start justify-between mb-3">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-lg bg-[var(--accent)]/10 flex items-center justify-center text-[var(--accent)] font-bold text-lg">
            {plugin.name.charAt(0).toUpperCase()}
          </div>
          <div>
            <h3 className="font-semibold text-[var(--text-primary)]">
              {plugin.name}
            </h3>
            <p className="text-xs text-[var(--text-secondary)]">
              v{plugin.version}
            </p>
          </div>
        </div>
        <StatusBadge
          status={plugin.enabled ? "enabled" : "disabled"}
          size="sm"
        />
      </div>

      {plugin.description && (
        <p className="text-sm text-[var(--text-secondary)] mb-3 line-clamp-2">
          {plugin.description}
        </p>
      )}

      <div className="flex items-center gap-2 mb-3">
        <span className="text-xs text-[var(--text-secondary)]">
          by {plugin.author}
        </span>
        {plugin.hooks.length > 0 && (
          <span className="text-xs px-1.5 py-0.5 rounded bg-[var(--bg-tertiary)] text-[var(--text-secondary)]">
            {plugin.hooks.length} hooks
          </span>
        )}
      </div>

      {plugin.hooks.length > 0 && (
        <div className="flex flex-wrap gap-1">
          {plugin.hooks.map((h) => (
            <span
              key={h}
              className="text-[10px] px-1.5 py-0.5 rounded bg-[var(--bg-tertiary)] text-[var(--text-secondary)] font-mono"
            >
              {h}
            </span>
          ))}
        </div>
      )}
    </div>
  );
}

export default function Plugins() {
  const { data: plugins } = useQuery<PluginManifest[]>("/plugins", 10000);
  const [filter, setFilter] = useState<"all" | "enabled" | "disabled">("all");
  const [search, setSearch] = useState("");

  const filtered = (plugins ?? []).filter((p) => {
    if (filter === "enabled" && !p.enabled) return false;
    if (filter === "disabled" && p.enabled) return false;
    if (search) {
      const q = search.toLowerCase();
      return (
        p.name.toLowerCase().includes(q) ||
        p.description.toLowerCase().includes(q) ||
        p.author.toLowerCase().includes(q)
      );
    }
    return true;
  });

  const enabledCount = (plugins ?? []).filter((p) => p.enabled).length;
  const totalCount = (plugins ?? []).length;

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h2 className="text-xl font-bold text-[var(--text-primary)]">
            Plugins
          </h2>
          <p className="text-sm text-[var(--text-secondary)]">
            {totalCount} installed, {enabledCount} enabled
          </p>
        </div>
        <input
          type="text"
          placeholder="Search plugins..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="px-3 py-2 text-sm rounded-lg bg-[var(--bg-secondary)] border border-[var(--border)] text-[var(--text-primary)] placeholder-[var(--text-secondary)] focus:outline-none focus:border-[var(--accent)] w-48"
        />
      </div>

      <div className="flex gap-2">
        {(["all", "enabled", "disabled"] as const).map((f) => (
          <button
            key={f}
            onClick={() => setFilter(f)}
            className={`px-3 py-1.5 text-xs font-medium rounded-lg border transition-colors ${
              filter === f
                ? "bg-[var(--accent)]/10 text-[var(--accent)] border-[var(--accent)]/30"
                : "bg-[var(--bg-secondary)] text-[var(--text-secondary)] border-[var(--border)] hover:text-[var(--text-primary)]"
            }`}
          >
            {f === "all"
              ? `All (${totalCount})`
              : f === "enabled"
                ? `Enabled (${enabledCount})`
                : `Disabled (${totalCount - enabledCount})`}
          </button>
        ))}
      </div>

      <p className="text-xs text-[var(--text-secondary)]">
        Discovered from ~/.claude/plugins. Manage via Claude Code CLI.
      </p>

      {filtered.length === 0 ? (
        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-16 text-center space-y-3">
          <p className="text-4xl">&#x1F9E9;</p>
          <p className="text-[var(--text-primary)] font-medium">
            {search ? "No plugins match your search" : "No plugins found"}
          </p>
          <p className="text-sm text-[var(--text-secondary)] max-w-md mx-auto">
            Install plugins using the Claude Code CLI with{" "}
            <code className="text-xs bg-[var(--bg-tertiary)] px-1.5 py-0.5 rounded">
              /install-plugin
            </code>
          </p>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
          {filtered.map((plugin) => (
            <PluginCard key={plugin.id} plugin={plugin} />
          ))}
        </div>
      )}
    </div>
  );
}
