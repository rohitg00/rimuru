import { useMemo } from "react";
import { useQuery } from "../hooks/useQuery";
import { formatTokens } from "../utils/format";

interface ProxyTool {
  name: string;
  server: string;
  description: string;
  schema_tokens: number;
}

interface ProxyStat {
  tool: string;
  calls: number;
  input_tokens: number;
  output_tokens: number;
  cache_hits: number;
  avg_latency_ms: number;
}

export default function McpProxy() {
  const { data: tools } = useQuery<ProxyTool[]>("/mcp/proxy/tools", 10000);
  const { data: stats } = useQuery<ProxyStat[]>("/mcp/proxy/stats", 10000);

  const connectedServers = useMemo(
    () => new Set((tools ?? []).map((t) => t.server)).size,
    [tools],
  );

  const totalTools = (tools ?? []).length;

  const totalCalls = useMemo(
    () => (stats ?? []).reduce((sum, s) => sum + s.calls, 0),
    [stats],
  );

  const cacheHitRate = useMemo(() => {
    const rows = stats ?? [];
    if (rows.length === 0) return 0;
    const totalHits = rows.reduce((sum, s) => sum + s.cache_hits, 0);
    const total = rows.reduce((sum, s) => sum + s.calls, 0);
    if (total === 0) return 0;
    return (totalHits / total) * 100;
  }, [stats]);

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-xl font-bold text-[var(--text-primary)]">
          MCP Proxy
        </h2>
        <p className="text-sm text-[var(--text-secondary)]">
          Monitor MCP tool usage, caching, and latency
        </p>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
          <p className="text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)] mb-1">
            Connected Servers
          </p>
          <p className="text-2xl font-bold text-[var(--text-primary)]">
            {connectedServers}
          </p>
        </div>
        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
          <p className="text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)] mb-1">
            Total Tools
          </p>
          <p className="text-2xl font-bold text-[var(--accent)]">
            {totalTools}
          </p>
        </div>
        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
          <p className="text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)] mb-1">
            Total Calls
          </p>
          <p className="text-2xl font-bold text-[var(--warning)]">
            {totalCalls.toLocaleString()}
          </p>
        </div>
        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
          <p className="text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)] mb-1">
            Cache Hit Rate
          </p>
          <p className="text-2xl font-bold text-[var(--success)]">
            {cacheHitRate.toFixed(1)}%
          </p>
        </div>
      </div>

      <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
        <h3 className="text-sm font-semibold text-[var(--text-primary)] mb-4">
          Tools
        </h3>
        {(tools ?? []).length === 0 ? (
          <p className="text-sm text-[var(--text-secondary)] text-center py-8">
            No tools discovered yet
          </p>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-[var(--border)]">
                  <th className="text-left py-2 px-3 text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">
                    Name
                  </th>
                  <th className="text-left py-2 px-3 text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">
                    Server
                  </th>
                  <th className="text-left py-2 px-3 text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">
                    Description
                  </th>
                  <th className="text-right py-2 px-3 text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">
                    Schema Tokens
                  </th>
                </tr>
              </thead>
              <tbody>
                {(tools ?? []).map((row) => (
                  <tr
                    key={`${row.server}-${row.name}`}
                    className="border-b border-[var(--border)] last:border-0 hover:bg-[var(--bg-tertiary)] transition-colors"
                  >
                    <td className="py-2 px-3 text-[var(--text-primary)] font-mono text-xs">
                      {row.name}
                    </td>
                    <td className="py-2 px-3 text-[var(--text-primary)]">
                      {row.server}
                    </td>
                    <td className="py-2 px-3 text-[var(--text-secondary)] max-w-xs truncate">
                      {row.description}
                    </td>
                    <td className="py-2 px-3 text-right text-[var(--text-primary)]">
                      {formatTokens(row.schema_tokens)}
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
          Usage Stats
        </h3>
        {(stats ?? []).length === 0 ? (
          <p className="text-sm text-[var(--text-secondary)] text-center py-8">
            No usage stats yet
          </p>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-[var(--border)]">
                  <th className="text-left py-2 px-3 text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">
                    Tool
                  </th>
                  <th className="text-right py-2 px-3 text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">
                    Calls
                  </th>
                  <th className="text-right py-2 px-3 text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">
                    Input Tokens
                  </th>
                  <th className="text-right py-2 px-3 text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">
                    Output Tokens
                  </th>
                  <th className="text-right py-2 px-3 text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">
                    Cache Hits
                  </th>
                  <th className="text-right py-2 px-3 text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">
                    Avg Latency
                  </th>
                </tr>
              </thead>
              <tbody>
                {(stats ?? []).map((row) => (
                  <tr
                    key={row.tool}
                    className="border-b border-[var(--border)] last:border-0 hover:bg-[var(--bg-tertiary)] transition-colors"
                  >
                    <td className="py-2 px-3 text-[var(--text-primary)] font-mono text-xs">
                      {row.tool}
                    </td>
                    <td className="py-2 px-3 text-right text-[var(--text-primary)]">
                      {row.calls.toLocaleString()}
                    </td>
                    <td className="py-2 px-3 text-right text-[var(--text-primary)]">
                      {formatTokens(row.input_tokens)}
                    </td>
                    <td className="py-2 px-3 text-right text-[var(--text-primary)]">
                      {formatTokens(row.output_tokens)}
                    </td>
                    <td className="py-2 px-3 text-right text-[var(--success)]">
                      {row.cache_hits.toLocaleString()}
                    </td>
                    <td className="py-2 px-3 text-right text-[var(--text-primary)]">
                      {row.avg_latency_ms.toFixed(0)}ms
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
