import { Plus, Wand2 } from "lucide-react";
import { Tooltip } from "@/components/Tooltip/Tooltip";
import styles from "./BottomBar.module.css";

const isMac = typeof navigator !== "undefined" && /Mac/.test(navigator.platform);
const modKey = isMac ? "\u2318" : "Ctrl";

interface BottomBarProps {
  onNewAgent: () => void;
  onWizard: () => void;
  viewMode: "terminal" | "conversation";
  onViewModeChange: (mode: "terminal" | "conversation") => void;
  inputMode: "smart" | "raw";
  onInputModeChange: (mode: "smart" | "raw") => void;
}

export default function BottomBar({
  onNewAgent,
  onWizard,
  viewMode,
  onViewModeChange,
  inputMode,
  onInputModeChange,
}: BottomBarProps) {
  return (
    <div className={styles.bar}>
      <div className={styles.left}>
        <Tooltip content="New Agent" shortcut="\u2318N">
          <button className={styles.barBtn} onClick={onNewAgent}>
            <Plus size={14} />
            New Agent
          </button>
        </Tooltip>
        <Tooltip content="Launch Wizard" shortcut="\u2318\u21E7N">
          <button className={styles.barBtn} onClick={onWizard}>
            <Wand2 size={14} />
            Wizard
          </button>
        </Tooltip>
      </div>
      <div className={styles.center}>
        <button
          className={`${styles.pill} ${viewMode === "terminal" ? styles.pillActive : ""}`}
          onClick={() => onViewModeChange("terminal")}
        >
          Terminal
        </button>
        <button
          className={`${styles.pill} ${viewMode === "conversation" ? styles.pillActive : ""}`}
          onClick={() => onViewModeChange("conversation")}
        >
          AI View
        </button>
        <span className={styles.divider} />
        <button
          className={`${styles.pill} ${inputMode === "smart" ? styles.pillActive : ""}`}
          onClick={() => onInputModeChange("smart")}
        >
          Smart
        </button>
        <button
          className={`${styles.pill} ${inputMode === "raw" ? styles.pillActive : ""}`}
          onClick={() => onInputModeChange("raw")}
        >
          Raw
        </button>
      </div>
      <div className={styles.right}>
        <span className={styles.hint}>{modKey}+Enter to send</span>
      </div>
    </div>
  );
}
