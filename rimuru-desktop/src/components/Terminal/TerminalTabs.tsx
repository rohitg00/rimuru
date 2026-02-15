import { X } from "lucide-react";
import type { PtySessionInfo } from "@/types/pty";
import { Tooltip } from "@/components/Tooltip/Tooltip";
import styles from "./TerminalTabs.module.css";
import clsx from "clsx";

interface TerminalTabsProps {
  sessions: PtySessionInfo[];
  activeSessionId: string;
  onSelectSession: (id: string) => void;
  onCloseSession: (id: string) => void;
}

const agentIcons: Record<string, string> = {
  claude_code: "\u27C1",
  open_code: "\u25C7",
  codex: "\u25CE",
  copilot: "\u25C8",
  cursor: "\u25EB",
  goose: "\u2B21",
};

export default function TerminalTabs({
  sessions,
  activeSessionId,
  onSelectSession,
  onCloseSession,
}: TerminalTabsProps) {
  return (
    <div className={styles.tabBar}>
      {sessions.map((session) => {
        const icon = agentIcons[session.agent_type] ?? "\u25CF";
        const isActive = session.id === activeSessionId;
        const isRunning = session.status === "Running";

        return (
          <button
            key={session.id}
            className={clsx(styles.tab, isActive && styles.active)}
            onClick={() => onSelectSession(session.id)}
          >
            <span className={styles.tabIcon}>{icon}</span>
            <span className={styles.tabName}>{session.agent_name}</span>
            <span className={clsx(styles.statusDot, isRunning && styles.running)} />
            <span className={styles.tabCost}>
              ${session.cumulative_cost_usd.toFixed(2)}
            </span>
            <Tooltip content="Close" shortcut="\u2318W">
              <span
                className={styles.closeBtn}
                onClick={(e) => {
                  e.stopPropagation();
                  onCloseSession(session.id);
                }}
                role="button"
                tabIndex={0}
              >
                <X size={12} />
              </span>
            </Tooltip>
          </button>
        );
      })}
    </div>
  );
}
