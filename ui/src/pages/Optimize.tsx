import { useEffect, useMemo, useState } from "react";
import { apiGet, apiPost } from "../api/client";
import { formatCost } from "../utils/format";

interface Recommendation {
  id: string;
  category: string;
  description: string;
  estimated_savings_tokens: number;
  estimated_savings_dollars: number;
  confidence: number;
  source: string;
  created_at: string;
}

interface RecommendationsResponse {
  recommendations: Recommendation[];
  total_count: number;
  total_savings_tokens: number;
  total_savings_dollars: number;
  generated_at: string;
  note: string;
}

interface AppliedRecommendation {
  id: string;
  category: string;
  description: string;
  applied_at: string;
  savings_tokens: number;
  savings_dollars: number;
}

interface AppliedResponse {
  applied: AppliedRecommendation[];
  count: number;
  total_savings_tokens: number;
  total_savings_dollars: number;
}

const CATEGORIES = [
  "mcp_schema",
  "output_verbose",
  "model_mismatch",
  "repeated_calls",
  "file_reread",
] as const;

const CATEGORY_LABELS: Record<string, string> = {
  mcp_schema: "MCP schema",
  output_verbose: "Output compression",
  model_mismatch: "Model routing",
  repeated_calls: "Repeated calls",
  file_reread: "File rereads",
};

function confidenceLabel(c: number): string {
  if (c >= 0.8) return "high";
  if (c >= 0.6) return "medium";
  if (c >= 0.4) return "low";
  return "speculative";
}

function formatTokens(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}k`;
  return String(n);
}

export default function Optimize() {
  const [pending, setPending] = useState<RecommendationsResponse | null>(null);
  const [applied, setApplied] = useState<AppliedResponse | null>(null);
  const [tab, setTab] = useState<"pending" | "applied">("pending");
  const [category, setCategory] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [actioningId, setActioningId] = useState<string | null>(null);

  async function refresh() {
    setLoading(true);
    setError(null);
    try {
      const [recs, app] = await Promise.all([
        apiGet<RecommendationsResponse>("/optimize/recommendations"),
        apiGet<AppliedResponse>("/optimize/applied"),
      ]);
      setPending(recs);
      setApplied(app);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load");
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void refresh();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function apply(rec: Recommendation) {
    setActioningId(rec.id);
    try {
      await apiPost("/optimize/apply", { recommendation: rec });
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Apply failed");
    } finally {
      setActioningId(null);
    }
  }

  const filteredPending = useMemo(() => {
    if (!pending) return [];
    if (!category) return pending.recommendations;
    return pending.recommendations.filter((r) => r.category === category);
  }, [pending, category]);

  const totalDollars =
    tab === "pending"
      ? pending?.total_savings_dollars ?? 0
      : applied?.total_savings_dollars ?? 0;
  const totalTokens =
    tab === "pending"
      ? pending?.total_savings_tokens ?? 0
      : applied?.total_savings_tokens ?? 0;

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-start justify-between">
        <div>
          <h1 className="text-2xl font-bold text-[var(--text-primary)]">
            Optimize
          </h1>
          <p className="text-sm text-[var(--text-secondary)] mt-1">
            Actionable recommendations mined from rimuru's own cost records,
            session data, and MCP proxy stats.
          </p>
        </div>
        <button
          type="button"
          onClick={() => void refresh()}
          disabled={loading}
          className="text-xs px-3 py-1 rounded-lg border border-[var(--border)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] disabled:opacity-40"
        >
          {loading ? "scanning..." : "rescan"}
        </button>
      </div>

      {error && (
        <div className="rounded-lg border border-[var(--error)]/30 bg-[var(--error)]/10 px-4 py-3 text-sm text-[var(--error)]">
          {error}
        </div>
      )}

      <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-6">
        <p className="text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)] mb-1">
          {tab === "pending" ? "Potential monthly savings" : "Realized savings"}
        </p>
        <p className="text-3xl font-bold text-[var(--text-primary)]">
          {formatCost(totalDollars)}
          <span className="text-sm text-[var(--text-secondary)] font-normal">
            {" "}
            / {formatTokens(totalTokens)} tokens
          </span>
        </p>
        {tab === "pending" && pending && (
          <p className="text-xs text-[var(--text-secondary)] mt-2">
            {pending.total_count} open recommendation
            {pending.total_count === 1 ? "" : "s"} across {CATEGORIES.length}{" "}
            categories.
          </p>
        )}
      </div>

      <div className="flex items-center justify-between gap-4">
        <div className="flex gap-1 rounded-lg border border-[var(--border)] p-1 bg-[var(--bg-secondary)]">
          <button
            type="button"
            onClick={() => setTab("pending")}
            className={`px-3 py-1 text-xs rounded-md ${
              tab === "pending"
                ? "bg-[var(--accent)] text-white"
                : "text-[var(--text-secondary)]"
            }`}
          >
            Pending {pending ? `(${pending.total_count})` : ""}
          </button>
          <button
            type="button"
            onClick={() => setTab("applied")}
            className={`px-3 py-1 text-xs rounded-md ${
              tab === "applied"
                ? "bg-[var(--accent)] text-white"
                : "text-[var(--text-secondary)]"
            }`}
          >
            Applied {applied ? `(${applied.count})` : ""}
          </button>
        </div>

        {tab === "pending" && (
          <div className="flex gap-1 flex-wrap">
            <button
              type="button"
              onClick={() => setCategory(null)}
              className={`px-3 py-1 text-xs rounded-full border ${
                category === null
                  ? "bg-[var(--accent)] text-white border-[var(--accent)]"
                  : "text-[var(--text-secondary)] border-[var(--border)]"
              }`}
            >
              all
            </button>
            {CATEGORIES.map((cat) => (
              <button
                key={cat}
                type="button"
                onClick={() => setCategory(cat)}
                className={`px-3 py-1 text-xs rounded-full border ${
                  category === cat
                    ? "bg-[var(--accent)] text-white border-[var(--accent)]"
                    : "text-[var(--text-secondary)] border-[var(--border)]"
                }`}
              >
                {CATEGORY_LABELS[cat]}
              </button>
            ))}
          </div>
        )}
      </div>

      {tab === "pending" && (
        <div className="space-y-3">
          {filteredPending.length === 0 ? (
            <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5 text-sm text-[var(--text-secondary)]">
              {category
                ? `No ${CATEGORY_LABELS[category] ?? category} recommendations right now.`
                : "No recommendations yet. Rimuru needs some cost records and MCP proxy stats to analyze — come back after a few sessions."}
            </div>
          ) : (
            filteredPending.map((rec) => (
              <div
                key={rec.id}
                className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5"
              >
                <div className="flex items-start justify-between gap-4">
                  <div className="flex-1">
                    <div className="flex items-baseline gap-2 mb-2">
                      <span
                        className="text-xs px-2 py-0.5 rounded-full font-semibold uppercase tracking-wider"
                        style={{
                          color: "var(--accent)",
                          backgroundColor:
                            "color-mix(in srgb, var(--accent) 12%, transparent)",
                        }}
                      >
                        {CATEGORY_LABELS[rec.category] ?? rec.category}
                      </span>
                      <span
                        className="text-xs text-[var(--text-secondary)]"
                        title={`confidence ${(rec.confidence * 100).toFixed(0)}%`}
                      >
                        {confidenceLabel(rec.confidence)} confidence
                      </span>
                    </div>
                    <p className="text-sm text-[var(--text-primary)]">
                      {rec.description}
                    </p>
                    <p className="text-xs text-[var(--text-secondary)] mt-2">
                      source: {rec.source}
                    </p>
                  </div>
                  <div className="text-right shrink-0">
                    <p className="text-lg font-bold text-[var(--success)]">
                      {formatCost(rec.estimated_savings_dollars)}
                    </p>
                    <p className="text-xs text-[var(--text-secondary)]">
                      {formatTokens(rec.estimated_savings_tokens)} tokens
                    </p>
                    <button
                      type="button"
                      onClick={() => void apply(rec)}
                      disabled={actioningId === rec.id}
                      className="mt-3 text-xs px-3 py-1 rounded-lg border border-[var(--accent)] text-[var(--accent)] hover:bg-[var(--accent)] hover:text-white disabled:opacity-40"
                    >
                      {actioningId === rec.id ? "acknowledging..." : "acknowledge"}
                    </button>
                  </div>
                </div>
              </div>
            ))
          )}
          {pending?.note && (
            <div className="rounded-lg border border-[var(--border)] bg-[var(--bg-tertiary)]/40 p-3 text-xs text-[var(--text-secondary)]">
              {pending.note}
            </div>
          )}
        </div>
      )}

      {tab === "applied" && (
        <div className="space-y-3">
          {applied?.applied.length === 0 ? (
            <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5 text-sm text-[var(--text-secondary)]">
              No recommendations acknowledged yet. Click Acknowledge on a
              pending card to record it here.
            </div>
          ) : (
            applied?.applied.map((rec) => (
              <div
                key={rec.id}
                className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5"
              >
                <div className="flex items-start justify-between gap-4">
                  <div className="flex-1">
                    <div className="flex items-baseline gap-2 mb-2">
                      <span
                        className="text-xs px-2 py-0.5 rounded-full font-semibold uppercase"
                        style={{
                          color: "var(--success)",
                          backgroundColor:
                            "color-mix(in srgb, var(--success) 12%, transparent)",
                        }}
                      >
                        {CATEGORY_LABELS[rec.category] ?? rec.category}
                      </span>
                      <span className="text-xs text-[var(--text-secondary)]">
                        {new Date(rec.applied_at).toLocaleString()}
                      </span>
                    </div>
                    <p className="text-sm text-[var(--text-primary)]">
                      {rec.description}
                    </p>
                  </div>
                  <div className="text-right shrink-0">
                    <p className="text-lg font-bold text-[var(--success)]">
                      {formatCost(rec.savings_dollars)}
                    </p>
                    <p className="text-xs text-[var(--text-secondary)]">
                      {formatTokens(rec.savings_tokens)} tokens
                    </p>
                  </div>
                </div>
              </div>
            ))
          )}
        </div>
      )}
    </div>
  );
}
