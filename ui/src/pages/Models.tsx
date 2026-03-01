import { useMemo } from "react";
import { useQuery } from "../hooks/useQuery";
import type { ModelInfo } from "../api/types";
import { formatProvider, normalizeProvider } from "../api/types";

import { formatPrice, formatContext } from "../utils/format";

const PROVIDER_COLORS: Record<string, string> = {
  anthropic: "#CC785C",
  openai: "#10a37f",
  google: "#4285f4",
  openrouter: "#c084fc",
  local: "#a89984",
};

export default function Models() {
  const { data: models } = useQuery<ModelInfo[]>("/models", 30000);

  const grouped = useMemo(() => {
    const map = new Map<string, ModelInfo[]>();
    for (const m of models ?? []) {
      const key = normalizeProvider(m.provider);
      const list = map.get(key) ?? [];
      list.push(m);
      map.set(key, list);
    }
    for (const [, list] of map) {
      list.sort((a, b) => (b.context_window ?? 0) - (a.context_window ?? 0));
    }
    return map;
  }, [models]);

  const providers = useMemo(() => Array.from(grouped.keys()).sort(), [grouped]);

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-xl font-bold text-[var(--text-primary)]">Models</h2>
        <p className="text-sm text-[var(--text-secondary)]">
          {models?.length ?? 0} models across {providers.length} providers.
          Prices per 1M tokens.
        </p>
      </div>

      <div className="flex flex-wrap gap-3">
        {providers.map((p) => (
          <div
            key={p}
            className="flex items-center gap-2 px-3 py-1.5 rounded-lg border border-[var(--border)] bg-[var(--bg-secondary)]"
          >
            <span
              className="w-2.5 h-2.5 rounded-full"
              style={{ backgroundColor: PROVIDER_COLORS[p] ?? "var(--accent)" }}
            />
            <span className="text-sm font-medium text-[var(--text-primary)]">
              {formatProvider(p)}
            </span>
            <span className="text-xs text-[var(--text-secondary)]">
              {grouped.get(p)?.length ?? 0}
            </span>
          </div>
        ))}
      </div>

      {providers.map((provider) => {
        const providerModels = grouped.get(provider) ?? [];
        const color = PROVIDER_COLORS[provider] ?? "var(--accent)";

        return (
          <div key={provider} className="space-y-3">
            <div className="flex items-center gap-2">
              <span
                className="w-3 h-3 rounded-full"
                style={{ backgroundColor: color }}
              />
              <h3 className="text-lg font-semibold text-[var(--text-primary)]">
                {formatProvider(provider)}
              </h3>
            </div>

            <div className="overflow-x-auto rounded-xl border border-[var(--border)]">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-[var(--border)] bg-[var(--bg-tertiary)]">
                    <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">
                      Model
                    </th>
                    <th className="px-4 py-3 text-right text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">
                      Input
                    </th>
                    <th className="px-4 py-3 text-right text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">
                      Output
                    </th>
                    <th className="px-4 py-3 text-right text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">
                      Cache Read
                    </th>
                    <th className="px-4 py-3 text-right text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">
                      Cache Write
                    </th>
                    <th className="px-4 py-3 text-right text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">
                      Context
                    </th>
                    <th className="px-4 py-3 text-right text-xs font-medium uppercase tracking-wider text-[var(--text-secondary)]">
                      Max Output
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {providerModels.map((m) => (
                    <tr
                      key={m.id}
                      className="border-b border-[var(--border)] last:border-0 hover:bg-[var(--bg-tertiary)] transition-colors"
                    >
                      <td className="px-4 py-3">
                        <div>
                          <span className="font-medium text-[var(--text-primary)]">
                            {m.name}
                          </span>
                          <p className="text-[10px] text-[var(--text-secondary)] mt-0.5 font-mono">
                            {m.id}
                          </p>
                        </div>
                      </td>
                      <td className="px-4 py-3 text-right text-[var(--text-primary)] font-mono">
                        {formatPrice(m.input_price_per_million ?? 0)}
                      </td>
                      <td className="px-4 py-3 text-right text-[var(--text-primary)] font-mono">
                        {formatPrice(m.output_price_per_million ?? 0)}
                      </td>
                      <td className="px-4 py-3 text-right text-[var(--text-secondary)] font-mono">
                        {formatPrice(m.cache_read_price_per_million ?? 0)}
                      </td>
                      <td className="px-4 py-3 text-right text-[var(--text-secondary)] font-mono">
                        {formatPrice(m.cache_write_price_per_million ?? 0)}
                      </td>
                      <td className="px-4 py-3 text-right text-[var(--text-primary)] font-mono">
                        {formatContext(m.context_window ?? 0)}
                      </td>
                      <td className="px-4 py-3 text-right text-[var(--text-secondary)] font-mono">
                        {formatContext(m.max_output_tokens ?? 0)}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        );
      })}

      {(models ?? []).length === 0 && (
        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-16 text-center">
          <p className="text-[var(--text-secondary)]">No models configured</p>
        </div>
      )}
    </div>
  );
}
