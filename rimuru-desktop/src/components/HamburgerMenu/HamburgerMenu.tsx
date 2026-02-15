import { useEffect, useRef } from "react";
import {
  Plus, BookOpen, Keyboard, Settings, ScrollText,
  Activity, BarChart3, RefreshCw, ExternalLink, Info
} from "lucide-react";
import styles from "./HamburgerMenu.module.css";

interface MenuItem {
  id: string;
  label: string;
  icon: typeof Plus;
  shortcut?: string;
  divider?: boolean;
}

const MENU_ITEMS: MenuItem[] = [
  { id: "new-wizard", label: "New Agent Wizard", icon: Plus, shortcut: "\u2318\u21e7N" },
  { id: "tour", label: "Introductory Tour", icon: BookOpen },
  { id: "shortcuts", label: "Keyboard Shortcuts", icon: Keyboard, shortcut: "\u2318/" },
  { id: "settings", label: "Settings", icon: Settings, shortcut: "\u2318," },
  { id: "divider-1", label: "", icon: Plus, divider: true },
  { id: "logs", label: "System Logs", icon: ScrollText, shortcut: "\u2325\u2318L" },
  { id: "process", label: "Process Monitor", icon: Activity, shortcut: "\u2325\u2318P" },
  { id: "usage", label: "Usage Dashboard", icon: BarChart3, shortcut: "\u2325\u2318U" },
  { id: "divider-2", label: "", icon: Plus, divider: true },
  { id: "updates", label: "Check for Updates", icon: RefreshCw },
  { id: "docs", label: "Documentation", icon: ExternalLink },
  { id: "about", label: "About", icon: Info },
];

interface HamburgerMenuProps {
  isOpen: boolean;
  onClose: () => void;
  onAction: (action: string) => void;
}

export default function HamburgerMenu({ isOpen, onClose, onAction }: HamburgerMenuProps) {
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!isOpen) return;
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        onClose();
      }
    };
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("mousedown", handleClickOutside);
    document.addEventListener("keydown", handleEscape);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
      document.removeEventListener("keydown", handleEscape);
    };
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  return (
    <div ref={menuRef} className={styles.menu}>
      {MENU_ITEMS.map((item) =>
        item.divider ? (
          <div key={item.id} className={styles.divider} />
        ) : (
          <button
            key={item.id}
            className={styles.item}
            onClick={() => { onAction(item.id); onClose(); }}
          >
            <item.icon size={15} />
            <span className={styles.itemLabel}>{item.label}</span>
            {item.shortcut && <span className={styles.itemShortcut}>{item.shortcut}</span>}
          </button>
        )
      )}
    </div>
  );
}
