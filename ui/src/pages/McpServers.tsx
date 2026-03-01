import { useQuery } from "../hooks/useQuery";
import StatusBadge from "../components/StatusBadge";

interface McpServer {
  id: string;
  name: string;
  command: string;
  args: string[];
  env: Record<string, string>;
  enabled: boolean;
  source?: string;
}

export default function McpServers() {
  const { data: servers } = useQuery<McpServer[]>("/mcp", 10000);

  const bySource = new Map<string, McpServer[]>();
  for (const s of servers ?? []) {
    const src = s.source ?? "Unknown";
    const list = bySource.get(src) ?? [];
    list.push(s);
    bySource.set(src, list);
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-xl font-bold text-[var(--text-primary)]">
          MCP Servers
        </h2>
        <p className="text-sm text-[var(--text-secondary)]">
          {(servers ?? []).length} configured across {bySource.size} source
          {bySource.size !== 1 ? "s" : ""}
        </p>
      </div>

      <p className="text-xs text-[var(--text-secondary)]">
        Discovered from ~/.claude/settings.json and Claude Desktop config.
      </p>

      {(servers ?? []).length === 0 ? (
        <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-16 text-center space-y-3">
          <p className="text-4xl">&#x1F50C;</p>
          <p className="text-[var(--text-primary)] font-medium">
            No MCP servers configured
          </p>
          <p className="text-sm text-[var(--text-secondary)] max-w-md mx-auto">
            MCP servers extend Claude with additional tools and capabilities.
            Configure them in{" "}
            <code className="text-xs bg-[var(--bg-tertiary)] px-1.5 py-0.5 rounded">
              ~/.claude/settings.json
            </code>{" "}
            or Claude Desktop settings.
          </p>
        </div>
      ) : (
        <div className="space-y-6">
          {Array.from(bySource.entries()).map(([source, sourceServers]) => (
            <div key={source}>
              <h3 className="text-sm font-semibold text-[var(--text-primary)] mb-3 flex items-center gap-2">
                <span className="w-2 h-2 rounded-full bg-[var(--accent)]" />
                {source}
                <span className="text-xs text-[var(--text-secondary)] font-normal">
                  ({sourceServers.length})
                </span>
              </h3>
              <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
                {sourceServers.map((server) => (
                  <div
                    key={server.id}
                    className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5 transition-all hover:border-[var(--accent)]/30"
                  >
                    <div className="flex items-start justify-between mb-3">
                      <div className="flex items-center gap-3">
                        <div className="w-10 h-10 rounded-lg bg-[var(--accent)]/10 flex items-center justify-center text-[var(--accent)] font-bold text-lg">
                          {server.name.charAt(0).toUpperCase()}
                        </div>
                        <div>
                          <h3 className="font-semibold text-[var(--text-primary)]">
                            {server.name}
                          </h3>
                          <p className="text-xs text-[var(--text-secondary)] font-mono">
                            {server.command}
                          </p>
                        </div>
                      </div>
                      <StatusBadge
                        status={server.enabled ? "enabled" : "disabled"}
                        size="sm"
                      />
                    </div>

                    <div className="space-y-2">
                      <div>
                        <p className="text-[10px] uppercase tracking-wider text-[var(--text-secondary)] mb-1">
                          Command
                        </p>
                        <pre className="text-xs text-[var(--text-primary)] bg-[var(--bg-tertiary)] rounded-lg p-2.5 overflow-x-auto font-mono">
                          {server.command} {server.args.join(" ")}
                        </pre>
                      </div>

                      {server.env &&
                        Object.keys(server.env).length > 0 && (
                          <div>
                            <p className="text-[10px] uppercase tracking-wider text-[var(--text-secondary)] mb-1">
                              Environment
                            </p>
                            <div className="space-y-1">
                              {Object.entries(server.env).map(([key, val]) => (
                                <div
                                  key={key}
                                  className="flex items-center gap-2 text-xs font-mono"
                                >
                                  <span className="text-[var(--text-secondary)]">
                                    {key}
                                  </span>
                                  <span className="text-[var(--text-primary)] bg-[var(--bg-tertiary)] px-1.5 py-0.5 rounded">
                                    {val}
                                  </span>
                                </div>
                              ))}
                            </div>
                          </div>
                        )}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
