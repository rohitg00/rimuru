import React from "react";
import { FolderTree, Clock, Play } from "lucide-react";
import { FilesTab } from "./FilesTab";
import { HistoryTab } from "./HistoryTab";
import { AutoRunTab } from "./AutoRunTab";
import styles from "./RightPanel.module.css";

interface RightPanelProps {
  visible: boolean;
  activeTab: "files" | "history" | "autorun";
  onTabChange: (tab: "files" | "history" | "autorun") => void;
  sessionWorkingDir?: string;
  sessionId?: string;
}

const tabs = [
  { key: "files" as const, label: "Files", icon: FolderTree },
  { key: "history" as const, label: "History", icon: Clock },
  { key: "autorun" as const, label: "Auto Run", icon: Play },
];

export const RightPanel: React.FC<RightPanelProps> = ({
  visible,
  activeTab,
  onTabChange,
  sessionWorkingDir,
  sessionId,
}) => {
  return (
    <div className={`${styles.panel} ${!visible ? styles.panelHidden : ''}`}>
      <div className={styles.tabBar}>
        {tabs.map(({ key, label, icon: Icon }) => (
          <button
            key={key}
            className={`${styles.tab} ${activeTab === key ? styles.tabActive : ""}`}
            onClick={() => onTabChange(key)}
          >
            <Icon size={14} />
            {label}
          </button>
        ))}
      </div>
      <div className={styles.content}>
        {activeTab === "files" && <FilesTab workingDir={sessionWorkingDir} />}
        {activeTab === "history" && <HistoryTab sessionId={sessionId} />}
        {activeTab === "autorun" && <AutoRunTab sessionId={sessionId} />}
      </div>
    </div>
  );
};
