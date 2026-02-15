import { useState, useEffect, useCallback } from "react";
import { ChevronRight, RefreshCw } from "lucide-react";
import { commands } from "@/lib/tauri";
import styles from "./DiscoveredSessions.module.css";

interface DiscoveredSession {
  provider: string;
  project_name: string;
  project_path: string;
  last_active: string | null;
  session_count: number;
}

interface DiscoveredSessionsProps {
  onLaunchFromDiscovered: (projectPath: string, agentType: string) => void;
}

const PROVIDER_COLORS: Record<string, string> = {
  "Claude Code": "#a78bfa",
  Codex: "#34d399",
  Goose: "#fbbf24",
  Cursor: "#60a5fa",
};

const PROVIDER_AGENT_MAP: Record<string, string> = {
  "Claude Code": "claude_code",
  Codex: "codex",
  Goose: "goose",
  Cursor: "cursor",
};

function relativeTime(isoDate: string): string {
  const now = Date.now();
  const then = new Date(isoDate).getTime();
  const diffSec = Math.floor((now - then) / 1000);

  if (diffSec < 60) return "just now";
  if (diffSec < 3600) return `${Math.floor(diffSec / 60)}m ago`;
  if (diffSec < 86400) return `${Math.floor(diffSec / 3600)}h ago`;
  return `${Math.floor(diffSec / 86400)}d ago`;
}

function groupByProvider(sessions: DiscoveredSession[]): Record<string, DiscoveredSession[]> {
  const groups: Record<string, DiscoveredSession[]> = {};
  for (const s of sessions) {
    if (!groups[s.provider]) groups[s.provider] = [];
    groups[s.provider].push(s);
  }
  return groups;
}

export default function DiscoveredSessions({ onLaunchFromDiscovered }: DiscoveredSessionsProps) {
  const [sessions, setSessions] = useState<DiscoveredSession[]>([]);
  const [expanded, setExpanded] = useState(true);
  const [loading, setLoading] = useState(false);

  const fetchSessions = useCallback(async () => {
    setLoading(true);
    try {
      const result = await commands.discoverSessions();
      setSessions(result);
    } catch {
      setSessions([]);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchSessions();
  }, [fetchSessions]);

  const grouped = groupByProvider(sessions);
  const providerNames = Object.keys(grouped);

  return (
    <div className={styles.container}>
      <div className={styles.header} onClick={() => setExpanded((v) => !v)}>
        <div className={styles.headerLeft}>
          <ChevronRight
            size={14}
            className={`${styles.chevron} ${expanded ? styles.chevronOpen : ""}`}
          />
          <span className={styles.title}>Discovered Sessions</span>
          {sessions.length > 0 && (
            <span className={styles.count}>{sessions.length}</span>
          )}
        </div>
        <button
          className={styles.refreshBtn}
          onClick={(e) => {
            e.stopPropagation();
            fetchSessions();
          }}
          title="Refresh"
        >
          <RefreshCw size={12} className={loading ? "spin" : ""} />
        </button>
      </div>

      {expanded && (
        <div className={styles.list}>
          {sessions.length === 0 && (
            <div className={styles.empty}>
              {loading ? "Scanning..." : "No existing sessions found"}
            </div>
          )}
          {providerNames.map((provider) => (
            <div key={provider} className={styles.providerGroup}>
              <div className={styles.providerLabel}>
                <span
                  className={styles.providerDot}
                  style={{ background: PROVIDER_COLORS[provider] ?? "#888" }}
                />
                {provider}
              </div>
              {grouped[provider].map((session, i) => (
                <div key={`${provider}-${i}`} className={styles.item}>
                  <div className={styles.itemLeft}>
                    <div className={styles.info}>
                      <span className={styles.name}>{session.project_name}</span>
                      <div className={styles.meta}>
                        <span className={styles.path}>{session.project_path}</span>
                        {session.last_active && (
                          <span>{relativeTime(session.last_active)}</span>
                        )}
                        {session.session_count > 0 && (
                          <span className={styles.sessionCount}>
                            {session.session_count}
                          </span>
                        )}
                      </div>
                    </div>
                  </div>
                  <button
                    className={styles.resumeBtn}
                    onClick={() =>
                      onLaunchFromDiscovered(
                        session.project_path,
                        PROVIDER_AGENT_MAP[session.provider] ?? "claude_code"
                      )
                    }
                  >
                    Resume
                  </button>
                </div>
              ))}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
