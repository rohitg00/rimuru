import { Play, Check, X, Clock, Shield, SkipForward } from "lucide-react";
import styles from "./StepCard.module.css";

export type StepStatus = "pending" | "running" | "waiting_approval" | "completed" | "failed" | "skipped";

interface StepCardProps {
  index: number;
  name: string;
  prompt: string;
  agentType: string;
  gate: string;
  status: StepStatus;
  isLast: boolean;
  onLaunch: () => void;
  onApprove?: () => void;
  onSkip: () => void;
  canLaunch: boolean;
}

const STATUS_ICON: Record<StepStatus, React.ReactNode> = {
  pending: null,
  running: null,
  waiting_approval: <Shield size={12} />,
  completed: <Check size={12} />,
  failed: <X size={12} />,
  skipped: <SkipForward size={10} />,
};

const CIRCLE_STYLES: Record<StepStatus, string> = {
  pending: styles.stepCirclePending,
  running: styles.stepCircleRunning,
  waiting_approval: styles.stepCircleWaiting,
  completed: styles.stepCircleCompleted,
  failed: styles.stepCircleFailed,
  skipped: styles.stepCirclePending,
};

const CONTENT_STYLES: Record<StepStatus, string> = {
  pending: "",
  running: styles.stepContentRunning,
  waiting_approval: styles.stepContentWaiting,
  completed: styles.stepContentCompleted,
  failed: styles.stepContentFailed,
  skipped: styles.stepContentCompleted,
};

export default function StepCard({
  index,
  name,
  prompt,
  agentType,
  gate,
  status,
  isLast,
  onLaunch,
  onApprove,
  onSkip,
  canLaunch,
}: StepCardProps) {
  const truncatedPrompt = prompt.length > 100 ? prompt.slice(0, 100) + "..." : prompt;
  const icon = STATUS_ICON[status];

  return (
    <div className={styles.step}>
      <div className={styles.stepIndicator}>
        <div className={`${styles.stepCircle} ${CIRCLE_STYLES[status]}`}>
          {icon ?? index + 1}
        </div>
        {!isLast && (
          <div
            className={`${styles.connector} ${
              status === "completed" ? styles.connectorActive : ""
            }`}
          />
        )}
      </div>
      <div className={`${styles.stepContent} ${CONTENT_STYLES[status]}`}>
        <div className={styles.stepTop}>
          <span className={styles.stepName}>{name}</span>
          <div className={styles.badges}>
            <span className={`${styles.badge} ${styles.badgeAgent}`}>{agentType}</span>
            <span
              className={`${styles.badge} ${
                gate === "approval" ? styles.badgeApproval : ""
              }`}
            >
              {gate === "approval" ? "approval" : "auto"}
            </span>
          </div>
        </div>
        <p className={styles.promptPreview}>{truncatedPrompt}</p>
        <div className={styles.stepActions}>
          {status === "pending" && canLaunch && (
            <button className={styles.launchBtn} onClick={onLaunch}>
              <Play size={10} />
              Launch
            </button>
          )}
          {status === "waiting_approval" && (
            <button className={styles.launchBtn} onClick={onApprove ?? onLaunch}>
              <Check size={10} />
              Approve
            </button>
          )}
          {(status === "pending" || status === "waiting_approval") && (
            <button className={styles.skipBtn} onClick={onSkip}>
              <SkipForward size={10} />
              Skip
            </button>
          )}
          {status === "running" && (
            <span className={styles.badge}>
              <Clock size={10} /> Running...
            </span>
          )}
        </div>
      </div>
    </div>
  );
}
