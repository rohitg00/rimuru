import { Clock, Zap, Play } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { AgentWithStatus } from "@/lib/tauri";
import styles from "./AgentCard.module.css";
import clsx from "clsx";

interface AgentCardProps {
  agent: AgentWithStatus;
  onClick?: () => void;
}

const agentIcons: Record<string, string> = {
  claude_code: "⟁",
  open_code: "◇",
  codex: "◎",
  copilot: "◈",
  cursor: "◫",
  goose: "⬡",
};

function getRelativeTime(dateStr: string): string {
  const now = Date.now();
  const then = new Date(dateStr).getTime();
  const diffSecs = Math.floor((now - then) / 1000);
  if (diffSecs < 60) return "just now";
  const diffMins = Math.floor(diffSecs / 60);
  if (diffMins < 60) return `${diffMins} minute${diffMins === 1 ? "" : "s"} ago`;
  const diffHours = Math.floor(diffMins / 60);
  if (diffHours < 24) return `${diffHours} hour${diffHours === 1 ? "" : "s"} ago`;
  const diffDays = Math.floor(diffHours / 24);
  return `${diffDays} day${diffDays === 1 ? "" : "s"} ago`;
}

export default function AgentCard({ agent, onClick }: AgentCardProps) {
  const navigate = useNavigate();
  const icon = agentIcons[agent.agent.agent_type] ?? "●";
  const lastActive = agent.agent.last_active
    ? getRelativeTime(agent.agent.last_active)
    : "No activity";
  const hasExecutable = ["claude_code", "codex", "goose", "open_code", "cursor", "copilot"].includes(agent.agent.agent_type);

  const handleLaunch = (e: React.MouseEvent) => {
    e.stopPropagation();
    navigate(`/orchestrate?agent=${agent.agent.agent_type}`);
  };

  return (
    <div className={styles.card} onClick={onClick} role="button" tabIndex={0}>
      <div className={styles.header}>
        <div className={styles.iconWrapper}>
          <span className={styles.icon}>{icon}</span>
        </div>
        <div className={styles.info}>
          <h3 className={styles.name}>{agent.agent.name}</h3>
          <span className={styles.type}>{agent.agent.agent_type}</span>
        </div>
        <span
          className={clsx(
            "badge",
            agent.is_active ? "badge-success" : "badge-warning"
          )}
        >
          {agent.is_active ? "Active" : "Idle"}
        </span>
      </div>

      <div className={styles.stats}>
        <div className={styles.stat}>
          <Clock size={14} />
          <span>Last active: {lastActive}</span>
        </div>
        <div className={styles.stat}>
          <Zap size={14} />
          <span>{agent.session_count} sessions</span>
        </div>
        {hasExecutable && (
          <button className={styles.launchBtn} onClick={handleLaunch}>
            <Play size={12} />
            Launch
          </button>
        )}
      </div>
    </div>
  );
}
