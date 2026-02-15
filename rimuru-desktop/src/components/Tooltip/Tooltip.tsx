import { useState, useRef, useCallback, type ReactNode } from "react";
import styles from "./Tooltip.module.css";

interface TooltipProps {
  content: string;
  shortcut?: string;
  side?: "top" | "bottom" | "left" | "right";
  children: ReactNode;
}

export function Tooltip({ content, shortcut, side = "top", children }: TooltipProps) {
  const [visible, setVisible] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout>>();

  const show = useCallback(() => {
    timerRef.current = setTimeout(() => setVisible(true), 400);
  }, []);

  const hide = useCallback(() => {
    clearTimeout(timerRef.current);
    setVisible(false);
  }, []);

  return (
    <span className={styles.wrapper} onMouseEnter={show} onMouseLeave={hide} onFocus={show} onBlur={hide}>
      {children}
      {visible && (
        <span className={`${styles.tooltip} ${styles[side]}`} role="tooltip">
          {content}
          {shortcut && <kbd className={styles.kbd}>{shortcut}</kbd>}
        </span>
      )}
    </span>
  );
}
