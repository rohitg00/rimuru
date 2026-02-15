import { useState, useEffect } from 'react';
import { Timer, DollarSign, Hash, ChevronLeft, ChevronRight, X, Cpu } from 'lucide-react';
import { PtySessionInfo } from '../../types/pty';
import styles from './SessionInfoPanel.module.css';

interface SessionInfoPanelProps {
  session: PtySessionInfo;
  onTerminate: () => void;
  collapsed: boolean;
  onToggleCollapse: () => void;
}

function formatDuration(startedAt: string): string {
  const elapsed = Math.floor((Date.now() - new Date(startedAt).getTime()) / 1000);
  const hours = Math.floor(elapsed / 3600);
  const minutes = Math.floor((elapsed % 3600) / 60);
  const seconds = elapsed % 60;
  const pad = (n: number) => String(n).padStart(2, '0');
  if (hours > 0) {
    return `${pad(hours)}:${pad(minutes)}:${pad(seconds)}`;
  }
  return `${pad(minutes)}:${pad(seconds)}`;
}

function agentDotColor(agentType: string): string {
  const colors: Record<string, string> = {
    claude: '#7aa2f7',
    codex: '#9ece6a',
    cursor: '#e0af68',
    gemini: '#7dcfff',
  };
  return colors[agentType.toLowerCase()] || '#a9b1d6';
}

function statusClass(status: string): string {
  switch (status) {
    case 'Running': return styles.running;
    case 'Completed': return styles.completed;
    case 'Failed': return styles.failed;
    case 'Terminated': return styles.terminated;
    default: return '';
  }
}

export default function SessionInfoPanel({ session, onTerminate, collapsed, onToggleCollapse }: SessionInfoPanelProps) {
  const [duration, setDuration] = useState(() => formatDuration(session.started_at));

  useEffect(() => {
    setDuration(formatDuration(session.started_at));
    const interval = setInterval(() => {
      setDuration(formatDuration(session.started_at));
    }, 1000);
    return () => clearInterval(interval);
  }, [session.started_at]);

  if (collapsed) {
    return (
      <div className={`${styles.panel} ${styles.collapsed}`}>
        <button className={styles.toggleBtn} onClick={onToggleCollapse}>
          <ChevronLeft size={14} />
        </button>
        <span
          className={styles.dot}
          style={{ backgroundColor: agentDotColor(session.agent_type) }}
        />
        <span className={styles.collapsedName}>{session.agent_name}</span>
        <span className={`${styles.statusBadge} ${statusClass(session.status)}`}>
          {session.status}
        </span>
        <span className={styles.collapsedMeta}>${session.cumulative_cost_usd.toFixed(4)}</span>
        <span className={styles.collapsedMeta}>{duration}</span>
      </div>
    );
  }

  return (
    <div className={styles.panel}>
      <button className={styles.toggleBtn} onClick={onToggleCollapse}>
        <ChevronRight size={14} />
      </button>

      <div className={styles.section}>
        <div className={styles.agentHeader}>
          <span
            className={styles.dot}
            style={{ backgroundColor: agentDotColor(session.agent_type) }}
          />
          <span className={styles.agentName}>{session.agent_name}</span>
        </div>
      </div>

      <div className={styles.section}>
        <span className={styles.label}>Working Directory</span>
        <span className={styles.workDir} title={session.working_dir}>
          {session.working_dir}
        </span>
      </div>

      <div className={styles.section}>
        <span className={styles.label}>Status</span>
        <span className={`${styles.statusBadge} ${statusClass(session.status)}`}>
          {session.status}
        </span>
      </div>

      <div className={styles.section}>
        <span className={styles.label}>
          <Timer size={11} /> Duration
        </span>
        <span className={styles.value}>{duration}</span>
      </div>

      <div className={styles.section}>
        <span className={styles.label}>
          <DollarSign size={11} /> Cost
        </span>
        <span className={styles.value}>${session.cumulative_cost_usd.toFixed(4)}</span>
      </div>

      <div className={styles.section}>
        <span className={styles.label}>
          <Hash size={11} /> Tokens
        </span>
        <span className={styles.value}>{session.token_count.toLocaleString()}</span>
      </div>

      <div className={styles.section}>
        <span className={styles.label}>
          <Cpu size={11} /> PID
        </span>
        <span className={styles.value}>{session.pid || 'N/A'}</span>
      </div>

      {session.status === 'Running' && (
        <button className={styles.terminateBtn} onClick={onTerminate}>
          <X size={14} />
          Terminate
        </button>
      )}
    </div>
  );
}
