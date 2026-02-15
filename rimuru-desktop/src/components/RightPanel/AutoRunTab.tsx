import React from "react";
import { Play, Pause } from "lucide-react";
import styles from "./AutoRunTab.module.css";

interface AutoRunTabProps {
  sessionId?: string;
}

const MOCK_TASKS = [
  { name: "Lint codebase", status: "completed" as const },
  { name: "Run unit tests", status: "completed" as const },
  { name: "Review changed files", status: "active" as const },
  { name: "Generate summary report", status: "pending" as const },
  { name: "Post review comments", status: "pending" as const },
];

export const AutoRunTab: React.FC<AutoRunTabProps> = ({ sessionId: _sessionId }) => {
  const completed = MOCK_TASKS.filter((t) => t.status === "completed").length;
  const progress = (completed / MOCK_TASKS.length) * 100;

  return (
    <div className={styles.container}>
      <div className={styles.playbookName}>Code Review Pipeline</div>
      <div className={styles.progressBar}>
        <div className={styles.progressFill} style={{ width: `${progress}%` }} />
      </div>
      <div className={styles.taskList}>
        {MOCK_TASKS.map((task, i) => (
          <div
            key={i}
            className={`${styles.task} ${
              task.status === "completed" ? styles.taskCompleted : ""
            } ${task.status === "active" ? styles.taskActive : ""}`}
          >
            <input
              type="checkbox"
              className={styles.checkbox}
              checked={task.status === "completed"}
              readOnly
            />
            <span>{task.name}</span>
          </div>
        ))}
      </div>
      <div className={styles.actions}>
        <button className={styles.task} style={{ cursor: "pointer" }}>
          <Play size={14} /> Run
        </button>
        <button className={styles.task} style={{ cursor: "pointer" }}>
          <Pause size={14} /> Pause
        </button>
      </div>
    </div>
  );
};
