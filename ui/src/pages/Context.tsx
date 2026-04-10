import { useMemo } from "react";
import { useQuery } from "../hooks/useQuery";
import { formatTokens } from "../utils/format";

interface UtilizationRecord {
  session_id: string;
  model: string;
  used_tokens: number;
  window_tokens: number;
  utilization_pct: number;
}

interface WasteRecord {
  session_id: string;
  total_tokens: number;
  schema_tokens: number;
  bash_tokens: number;
  mcp_tokens: number;
  waste_pct: number;
  savings: number;
}

export default function Context() {
  const { data: utilization } = useQuery<UtilizationRecord[]>(
    "/context/utilization",
    10000,
  );
  const { data: waste } = useQuery<WasteRecord[]>("/context/waste", 10000);

  const sessionsAnalyzed = useMemo(
    () => new Set([...(utilization ?? []).map((u) => u.session_id), ...(waste ?? []).map((w) => w.session_id)]).size,
    [utilization, waste],
  );

  const totalWasteTokens = useMemo(
    () =>
      (waste ?? []).reduce(
        (sum, w) => sum + w.schema_tokens + w.bash_tokens + w.mcp_tokens,
        0,
      ),
    [waste],
  );

  const avgUtilization = useMemo(() => {
    const rows = utilization ?? [];
    if (rows.length === 0) return 0;
    return rows.reduce((sum, u) => sum + u.utilization_pct, 0) / rows.length;
  }, [utilization]);

  function utilizationColor(pct: number): string {
    if (pct < 50) return "text-[var(--success)]";
    if (pct <= 80) return "text-[var(--warning)]";
    return "text-[var(--error)]";
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-xl font-bold text-[var(--text-primary)]">
          Context Observability
        </h2>
        <p className="text-sm text-[var(--text-secondary)]">
          Monitor context window utilization and token waste
        </p>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
          <p className="text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)] mb-1">
            Sessions Analyzed
          </p>
          <p className="text-2xl font-bold text-[var(--text-primary)]">
            {sessionsAnalyzed}
          </p>
        </div>
        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
          <p className="text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)] mb-1">
            Total Waste Tokens
          </p>
          <p className="text-2xl font-bold text-[var(--warning)]">
            {formatTokens(totalWasteTokens)}
          </p>
        </div>
        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
          <p className="text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)] mb-1">
            Avg Utilization
          </p>
          <p className={`text-2xl font-bold ${utilizationColor(avgUtilization)}`}>
            {avgUtilization.toFixed(1)}%
          </p>
        </div>
      </div>

      <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
        <h3 className="text-sm font-semibold text-[var(--text-primary)] mb-4">
          Context Utilization
        </h3>
        {(utilization ?? []).length === 0 ? (
          <p className="text-sm text-[var(--text-secondary)] text-center py-8">
            No utilization data yet
          </p>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-[var(--border)]">
                  <th className="text-left py-2 px-3 text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">
                    Session
                  </th>
                  <th className="text-left py-2 px-3 text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">
                    Model
                  </th>
                  <th className="text-right py-2 px-3 text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">
                    Used
                  </th>
                  <th className="text-right py-2 px-3 text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">
                    Window
                  </th>
                  <th className="text-right py-2 px-3 text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">
                    Utilization %
                  </th>
                </tr>
              </thead>
              <tbody>
                {(utilization ?? []).map((row) => (
                  <tr
                    key={row.session_id}
                    className="border-b border-[var(--border)] last:border-0 hover:bg-[var(--bg-tertiary)] transition-colors"
                  >
                    <td className="py-2 px-3 text-[var(--text-primary)] font-mono text-xs">
                      {row.session_id.slice(0, 8)}
                    </td>
                    <td className="py-2 px-3 text-[var(--text-primary)]">
                      {row.model}
                    </td>
                    <td className="py-2 px-3 text-right text-[var(--text-primary)]">
                      {formatTokens(row.used_tokens)}
                    </td>
                    <td className="py-2 px-3 text-right text-[var(--text-primary)]">
                      {formatTokens(row.window_tokens)}
                    </td>
                    <td className={`py-2 px-3 text-right font-semibold ${utilizationColor(row.utilization_pct)}`}>
                      {row.utilization_pct.toFixed(1)}%
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
        <h3 className="text-sm font-semibold text-[var(--text-primary)] mb-4">
          Token Waste Analysis
        </h3>
        {(waste ?? []).length === 0 ? (
          <p className="text-sm text-[var(--text-secondary)] text-center py-8">
            No waste data yet
          </p>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-[var(--border)]">
                  <th className="text-left py-2 px-3 text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">
                    Session
                  </th>
                  <th className="text-right py-2 px-3 text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">
                    Total
                  </th>
                  <th className="text-right py-2 px-3 text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">
                    Schemas
                  </th>
                  <th className="text-right py-2 px-3 text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">
                    Bash
                  </th>
                  <th className="text-right py-2 px-3 text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">
                    MCP
                  </th>
                  <th className="text-right py-2 px-3 text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">
                    Waste %
                  </th>
                  <th className="text-right py-2 px-3 text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">
                    Savings
                  </th>
                </tr>
              </thead>
              <tbody>
                {(waste ?? []).map((row) => (
                  <tr
                    key={row.session_id}
                    className="border-b border-[var(--border)] last:border-0 hover:bg-[var(--bg-tertiary)] transition-colors"
                  >
                    <td className="py-2 px-3 text-[var(--text-primary)] font-mono text-xs">
                      {row.session_id.slice(0, 8)}
                    </td>
                    <td className="py-2 px-3 text-right text-[var(--text-primary)]">
                      {formatTokens(row.total_tokens)}
                    </td>
                    <td className="py-2 px-3 text-right text-[var(--text-primary)]">
                      {formatTokens(row.schema_tokens)}
                    </td>
                    <td className="py-2 px-3 text-right text-[var(--text-primary)]">
                      {formatTokens(row.bash_tokens)}
                    </td>
                    <td className="py-2 px-3 text-right text-[var(--text-primary)]">
                      {formatTokens(row.mcp_tokens)}
                    </td>
                    <td className="py-2 px-3 text-right font-semibold text-[var(--warning)]">
                      {row.waste_pct.toFixed(1)}%
                    </td>
                    <td className="py-2 px-3 text-right text-[var(--success)]">
                      ${row.savings.toFixed(4)}
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
