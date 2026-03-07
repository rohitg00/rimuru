import { useState, useMemo } from "react";
import { useQuery } from "../hooks/useQuery";
import type { CatalogResponse, HardwareInfo } from "../api/types";
import StatusBadge from "../components/StatusBadge";

function formatDownloads(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(0)}K`;
  return n.toString();
}

function formatParams(b: number): string {
  if (b >= 1) return `${b.toFixed(1)}B`;
  return `${(b * 1000).toFixed(0)}M`;
}

export default function Advisor() {
  const { data: catalog } = useQuery<CatalogResponse>("/models/catalog", 0);
  const { data: hwInfo } = useQuery<HardwareInfo>("/system", 0);
  const [filter, setFilter] = useState<"all" | "perfect" | "good" | "marginal" | "too_tight">("all");
  const [search, setSearch] = useState("");
  const [sortBy, setSortBy] = useState<"downloads" | "params" | "tok_s">("downloads");

  const entries = useMemo(() => {
    let list = catalog?.entries ?? [];
    if (filter !== "all") {
      list = list.filter((e) => e.fit_level === filter);
    }
    if (search) {
      const q = search.toLowerCase();
      list = list.filter(
        (e) =>
          e.name.toLowerCase().includes(q) ||
          e.provider.toLowerCase().includes(q) ||
          e.architecture.toLowerCase().includes(q) ||
          e.use_case.toLowerCase().includes(q),
      );
    }
    list = [...list].sort((a, b) => {
      if (sortBy === "downloads") return b.hf_downloads - a.hf_downloads;
      if (sortBy === "params") return a.params_b - b.params_b;
      return (b.estimated_tok_per_sec ?? 0) - (a.estimated_tok_per_sec ?? 0);
    });
    return list;
  }, [catalog, filter, search, sortBy]);

  const summary = catalog?.summary;

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-xl font-bold text-[var(--text-primary)]">Model Advisor</h2>
        <p className="text-sm text-[var(--text-secondary)]">
          {summary
            ? `${summary.perfect + summary.good + summary.marginal} of ${summary.catalog_size} models can run on your hardware`
            : "Analyzing models..."}
        </p>
      </div>

      {hwInfo && (
        <div className="flex flex-wrap gap-3 items-center rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] px-5 py-3">
          <span className="text-sm font-medium text-[var(--text-primary)]">{hwInfo.cpu_brand}</span>
          <span className="text-xs text-[var(--text-secondary)]">{(hwInfo.total_ram_mb / 1024).toFixed(0)} GB RAM</span>
          {hwInfo.gpu && (
            <span className="text-xs text-[var(--text-secondary)]">{hwInfo.gpu.name} ({(hwInfo.gpu.vram_mb / 1024).toFixed(0)} GB)</span>
          )}
          <span className="text-xs px-2 py-0.5 rounded-full bg-[var(--accent)]/15 text-[var(--accent)]">
            {hwInfo.backend.replace("_", " ").toUpperCase()}
          </span>
        </div>
      )}

      {summary && (
        <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
          {[
            { label: "Perfect", count: summary.perfect, level: "perfect" as const, color: "var(--success)" },
            { label: "Good", count: summary.good, level: "good" as const, color: "var(--accent)" },
            { label: "Marginal", count: summary.marginal, level: "marginal" as const, color: "var(--warning)" },
            { label: "Total Catalog", count: summary.catalog_size, level: "all" as const, color: "var(--text-secondary)" },
          ].map((s) => (
            <button
              key={s.label}
              onClick={() => setFilter(s.level === "all" ? "all" : s.level)}
              className={`rounded-xl border bg-[var(--bg-secondary)] p-4 text-center transition-colors ${
                filter === s.level ? "border-[var(--accent)]" : "border-[var(--border)] hover:border-[var(--text-secondary)]"
              }`}
            >
              <p className="text-2xl font-bold" style={{ color: s.color }}>
                {s.count}
              </p>
              <p className="text-xs text-[var(--text-secondary)] mt-1 uppercase tracking-wider">
                {s.label}
              </p>
            </button>
          ))}
        </div>
      )}

      <div className="flex flex-wrap gap-3">
        <input
          type="text"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="Search models..."
          className="flex-1 min-w-[200px] px-3 py-2 text-sm rounded-lg border border-[var(--border)] bg-[var(--bg-secondary)] text-[var(--text-primary)] placeholder-[var(--text-secondary)] outline-none focus:border-[var(--accent)]"
        />
        <select
          value={sortBy}
          onChange={(e) => setSortBy(e.target.value as typeof sortBy)}
          className="px-3 py-2 text-sm rounded-lg border border-[var(--border)] bg-[var(--bg-secondary)] text-[var(--text-primary)] outline-none"
        >
          <option value="downloads">Most Popular</option>
          <option value="tok_s">Fastest</option>
          <option value="params">Smallest First</option>
        </select>
      </div>

      <div className="text-xs text-[var(--text-secondary)]">{entries.length} models</div>

      <div className="overflow-x-auto rounded-xl border border-[var(--border)]">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-[var(--border)] bg-[var(--bg-tertiary)]">
              <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">Model</th>
              <th className="px-4 py-3 text-right text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">Params</th>
              <th className="px-4 py-3 text-right text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">Context</th>
              <th className="px-4 py-3 text-center text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">Fit</th>
              <th className="px-4 py-3 text-right text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">Quant</th>
              <th className="px-4 py-3 text-right text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">VRAM</th>
              <th className="px-4 py-3 text-right text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">Est. tok/s</th>
              <th className="px-4 py-3 text-right text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">Downloads</th>
            </tr>
          </thead>
          <tbody>
            {entries.slice(0, 100).map((e) => (
              <tr
                key={e.name}
                className="border-b border-[var(--border)] last:border-0 hover:bg-[var(--bg-tertiary)] transition-colors"
              >
                <td className="px-4 py-3">
                  <div>
                    <span className="font-medium text-[var(--text-primary)]">{e.name.split("/").pop()}</span>
                    <p className="text-[10px] text-[var(--text-secondary)] mt-0.5">
                      {e.provider} &middot; {e.architecture} &middot; {e.use_case}
                    </p>
                    {e.capabilities.length > 0 && (
                      <div className="flex gap-1 mt-0.5">
                        {e.capabilities.map((c) => (
                          <span key={c} className="text-[9px] px-1 py-0.5 rounded bg-[var(--accent)]/10 text-[var(--accent)]">{c}</span>
                        ))}
                      </div>
                    )}
                  </div>
                </td>
                <td className="px-4 py-3 text-right text-[var(--text-primary)] font-mono">{formatParams(e.params_b)}</td>
                <td className="px-4 py-3 text-right text-[var(--text-secondary)] font-mono">
                  {e.context_length >= 1000 ? `${(e.context_length / 1000).toFixed(0)}K` : e.context_length}
                </td>
                <td className="px-4 py-3 text-center">
                  <StatusBadge status={e.fit_level} size="sm" />
                </td>
                <td className="px-4 py-3 text-right text-[var(--text-secondary)] font-mono text-xs">
                  {e.best_quantization ?? "\u2014"}
                </td>
                <td className="px-4 py-3 text-right text-[var(--text-secondary)] font-mono text-xs">
                  {e.estimated_vram_mb ? `${(e.estimated_vram_mb / 1024).toFixed(1)}G` : "\u2014"}
                </td>
                <td className="px-4 py-3 text-right text-[var(--text-primary)] font-mono">
                  {e.estimated_tok_per_sec ? e.estimated_tok_per_sec.toFixed(1) : "\u2014"}
                </td>
                <td className="px-4 py-3 text-right text-[var(--text-secondary)] font-mono text-xs">
                  {formatDownloads(e.hf_downloads)}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {entries.length > 100 && (
        <p className="text-xs text-center text-[var(--text-secondary)]">
          Showing first 100 of {entries.length} models
        </p>
      )}
    </div>
  );
}
