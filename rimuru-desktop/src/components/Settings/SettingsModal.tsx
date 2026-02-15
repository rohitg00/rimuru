import { useState, useEffect } from "react";
import { Settings as SettingsIcon, Keyboard, Palette, Bell, Zap, Info, X } from "lucide-react";
import { useAnimatedUnmount } from "@/hooks/useAnimatedUnmount";
import GeneralTab from "./GeneralTab";
import ShortcutsTab from "./ShortcutsTab";
import ThemesTab from "./ThemesTab";
import NotificationsTab from "./NotificationsTab";
import AICommandsTab from "./AICommandsTab";
import styles from "./SettingsModal.module.css";

type SettingsTab = "general" | "shortcuts" | "themes" | "notifications" | "ai-commands" | "about";

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
}

const TABS: Array<{ id: SettingsTab; label: string; icon: typeof SettingsIcon }> = [
  { id: "general", label: "General", icon: SettingsIcon },
  { id: "shortcuts", label: "Shortcuts", icon: Keyboard },
  { id: "themes", label: "Themes", icon: Palette },
  { id: "notifications", label: "Notifications", icon: Bell },
  { id: "ai-commands", label: "AI Commands", icon: Zap },
  { id: "about", label: "About", icon: Info },
];

export default function SettingsModal({ isOpen, onClose }: SettingsModalProps) {
  const [activeTab, setActiveTab] = useState<SettingsTab>("general");
  const { shouldRender, animationState } = useAnimatedUnmount(isOpen, 200);

  useEffect(() => {
    if (!isOpen) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [isOpen, onClose]);

  if (!shouldRender) return null;

  const renderTab = () => {
    switch (activeTab) {
      case "general": return <GeneralTab />;
      case "shortcuts": return <ShortcutsTab />;
      case "themes": return <ThemesTab />;
      case "notifications": return <NotificationsTab />;
      case "ai-commands": return <AICommandsTab />;
      case "about": return (
        <div className={styles.aboutTab}>
          <h3>Rimuru</h3>
          <p>AI Agent Orchestration & Cost Tracking</p>
          <p>Version 0.1.0</p>
        </div>
      );
    }
  };

  return (
    <div className={styles.overlay} data-state={animationState} onClick={onClose}>
      <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
        <div className={styles.sidebar}>
          <div className={styles.sidebarHeader}>Settings</div>
          {TABS.map((tab) => (
            <button
              key={tab.id}
              className={`${styles.sidebarTab} ${activeTab === tab.id ? styles.sidebarTabActive : ""}`}
              onClick={() => setActiveTab(tab.id)}
            >
              <tab.icon size={16} />
              {tab.label}
            </button>
          ))}
        </div>
        <div className={styles.content}>
          <div className={styles.contentHeader}>
            <h2>{TABS.find((t) => t.id === activeTab)?.label}</h2>
            <button className={styles.closeBtn} onClick={onClose} aria-label="Close dialog"><X size={18} /></button>
          </div>
          <div className={styles.contentBody}>{renderTab()}</div>
        </div>
      </div>
    </div>
  );
}
