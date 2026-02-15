import type { ComponentType } from "react";
import styles from "./EmptyState.module.css";

interface EmptyStateProps {
  icon: ComponentType<{ size?: number | string }>;
  title: string;
  description: string;
  action?: { label: string; onClick: () => void };
}

export function EmptyState({ icon: Icon, title, description, action }: EmptyStateProps) {
  return (
    <div className={styles.container}>
      <div className={styles.iconWrapper}>
        <Icon size={32} />
      </div>
      <div className={styles.title}>{title}</div>
      <div className={styles.description}>{description}</div>
      {action && (
        <button className="btn btn-primary" onClick={action.onClick}>
          {action.label}
        </button>
      )}
    </div>
  );
}
