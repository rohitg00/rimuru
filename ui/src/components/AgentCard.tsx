import type { Agent } from "../api/types";
import { formatAgentType } from "../api/types";
import StatusBadge from "./StatusBadge";

interface AgentCardProps {
  agent: Agent;
  onConnect: (id: string) => void;
  onDisconnect: (id: string) => void;
}

import { formatCost, formatTokens, timeAgo } from "../utils/format";

const AGENT_ICONS: Record<string, string> = {
  claude_code: "\u27C1",
  "claude-code": "\u27C1",
  cursor: "\u25EB",
  codex: "\u25CE",
  gemini_cli: "\u2726",
  "gemini-cli": "\u2726",
  opencode: "\u2B21",
  windsurf: "\u25C7",
  copilot: "\u2B22",
  goose: "\u2B23",
};

export default function AgentCard({
  agent,
  onConnect,
  onDisconnect,
}: AgentCardProps) {
  const icon = AGENT_ICONS[agent.agent_type] ?? "\u2B22";
  const isActive = agent.status === "connected" || agent.status === "busy";

  return (
    <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5 transition-all hover:border-[var(--accent)]/50 hover:shadow-lg hover:shadow-[var(--accent)]/5">
      <div className="flex items-start justify-between mb-4">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-lg bg-[var(--accent)]/10 flex items-center justify-center text-[var(--accent)] text-lg font-bold">
            {icon}
          </div>
          <div>
            <h3 className="font-semibold text-[var(--text-primary)]">
              {agent.name}
            </h3>
            <p className="text-xs text-[var(--text-secondary)]">
              {formatAgentType(agent.agent_type)}
            </p>
          </div>
        </div>
        <StatusBadge status={agent.status} />
      </div>

      <div className="grid grid-cols-2 gap-3 mb-4">
        <div className="rounded-lg bg-[var(--bg-tertiary)] p-2.5">
          <p className="text-[10px] uppercase tracking-wider text-[var(--text-secondary)] mb-0.5">
            Cost
          </p>
          <p className="text-sm font-semibold text-[var(--text-primary)]">
            {formatCost(agent.total_cost)}
          </p>
        </div>
        <div className="rounded-lg bg-[var(--bg-tertiary)] p-2.5">
          <p className="text-[10px] uppercase tracking-wider text-[var(--text-secondary)] mb-0.5">
            Tokens
          </p>
          <p className="text-sm font-semibold text-[var(--text-primary)]">
            {formatTokens(agent.total_tokens ?? 0)}
          </p>
        </div>
        <div className="rounded-lg bg-[var(--bg-tertiary)] p-2.5">
          <p className="text-[10px] uppercase tracking-wider text-[var(--text-secondary)] mb-0.5">
            Sessions
          </p>
          <p className="text-sm font-semibold text-[var(--text-primary)]">
            {agent.session_count}
          </p>
        </div>
        <div className="rounded-lg bg-[var(--bg-tertiary)] p-2.5">
          <p className="text-[10px] uppercase tracking-wider text-[var(--text-secondary)] mb-0.5">
            Model
          </p>
          <p className="text-sm font-semibold text-[var(--text-primary)] truncate">
            {agent.model ?? agent.version ?? "-"}
          </p>
        </div>
      </div>

      <div className="flex items-center justify-between">
        <span className="text-xs text-[var(--text-secondary)]">
          Active {timeAgo(agent.last_active ?? agent.last_seen ?? null)}
        </span>
        {isActive ? (
          <button
            onClick={() => onDisconnect(agent.id)}
            className="px-3 py-1.5 text-xs font-medium rounded-lg bg-[var(--error)]/10 text-[var(--error)] hover:bg-[var(--error)]/20 transition-colors"
          >
            Disconnect
          </button>
        ) : (
          <button
            onClick={() => onConnect(agent.id)}
            className="px-3 py-1.5 text-xs font-medium rounded-lg bg-[var(--accent)]/10 text-[var(--accent)] hover:bg-[var(--accent)]/20 transition-colors"
          >
            Connect
          </button>
        )}
      </div>
    </div>
  );
}
