import React, { useRef, useEffect } from "react";
import styles from "./HistoryTab.module.css";

interface HistoryTabProps {
  sessionId?: string;
}

const MOCK_HISTORY = [
  { type: "AUTO" as const, message: "Analyzed codebase structure and identified 3 potential improvements", cost: 0.024, timestamp: "2 min ago" },
  { type: "USER" as const, message: "Fix the authentication bug in login flow", cost: 0.0, timestamp: "5 min ago" },
  { type: "AUTO" as const, message: "Applied fix to auth middleware, updated token validation logic", cost: 0.051, timestamp: "4 min ago" },
  { type: "USER" as const, message: "Run the test suite and report results", cost: 0.0, timestamp: "8 min ago" },
  { type: "AUTO" as const, message: "All 47 tests passing. Coverage increased from 82% to 89%", cost: 0.033, timestamp: "7 min ago" },
];

export const HistoryTab: React.FC<HistoryTabProps> = ({ sessionId: _sessionId }) => {
  const listRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (listRef.current) {
      listRef.current.scrollTop = listRef.current.scrollHeight;
    }
  }, []);

  return (
    <div className={styles.container}>
      <div className={styles.heatmapPlaceholder}>Activity</div>
      <div className={styles.list} ref={listRef}>
        {MOCK_HISTORY.map((entry, i) => (
          <div key={i} className={styles.entry}>
            <div className={styles.entryHeader}>
              <span className={`${styles.badge} ${entry.type === "AUTO" ? styles.badgeAuto : styles.badgeUser}`}>
                {entry.type}
              </span>
            </div>
            <div className={styles.message}>
              {entry.message.length > 60 ? entry.message.slice(0, 60) + "..." : entry.message}
            </div>
            <div className={styles.entryFooter}>
              <span>${entry.cost.toFixed(3)}</span>
              <span>{entry.timestamp}</span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
