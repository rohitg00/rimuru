import { CheckCircle2, XCircle, AlertTriangle, Info, X } from "lucide-react";
import styles from "./Toast.module.css";

export type ToastType = "success" | "error" | "warning" | "info";

interface ToastProps {
  id: string;
  type: ToastType;
  title: string;
  description?: string;
  onDismiss: (id: string) => void;
  state?: string;
}

const icons = {
  success: CheckCircle2,
  error: XCircle,
  warning: AlertTriangle,
  info: Info,
};

const iconStyles = {
  success: styles.iconSuccess,
  error: styles.iconError,
  warning: styles.iconWarning,
  info: styles.iconInfo,
};

export function Toast({ id, type, title, description, onDismiss, state }: ToastProps) {
  const Icon = icons[type];
  return (
    <div className={`${styles.toast} ${styles[type]}`} data-state={state} role="alert">
      <Icon size={18} className={iconStyles[type]} />
      <div className={styles.body}>
        <div className={styles.title}>{title}</div>
        {description && <div className={styles.description}>{description}</div>}
      </div>
      <button className={styles.dismiss} onClick={() => onDismiss(id)} aria-label="Dismiss">
        <X size={14} />
      </button>
    </div>
  );
}
