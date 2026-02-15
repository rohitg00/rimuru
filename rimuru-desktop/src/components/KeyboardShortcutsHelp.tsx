import { useEffect, useState } from "react";
import { X, Keyboard } from "lucide-react";
import {
  Shortcut,
  formatShortcut,
  getShortcutsByCategory,
} from "@/hooks/useKeyboardShortcuts";
import styles from "./KeyboardShortcutsHelp.module.css";

interface KeyboardShortcutsHelpProps {
  shortcuts: Shortcut[];
}

export function KeyboardShortcutsHelp({ shortcuts }: KeyboardShortcutsHelpProps) {
  const [isOpen, setIsOpen] = useState(false);

  useEffect(() => {
    const handleShowShortcuts = () => setIsOpen(true);
    const handleCloseModal = () => setIsOpen(false);

    window.addEventListener("show-shortcuts-help", handleShowShortcuts);
    window.addEventListener("close-modal", handleCloseModal);

    return () => {
      window.removeEventListener("show-shortcuts-help", handleShowShortcuts);
      window.removeEventListener("close-modal", handleCloseModal);
    };
  }, []);

  useEffect(() => {
    if (isOpen) {
      const handleEsc = (e: KeyboardEvent) => {
        if (e.key === "Escape") {
          setIsOpen(false);
        }
      };
      window.addEventListener("keydown", handleEsc);
      return () => window.removeEventListener("keydown", handleEsc);
    }
  }, [isOpen]);

  if (!isOpen) return null;

  const shortcutsByCategory = getShortcutsByCategory(shortcuts);

  return (
    <div className={styles.overlay} onClick={() => setIsOpen(false)}>
      <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
        <div className={styles.header}>
          <div className={styles.titleGroup}>
            <Keyboard size={20} />
            <h2 className={styles.title}>Keyboard Shortcuts</h2>
          </div>
          <button
            className={styles.closeBtn}
            onClick={() => setIsOpen(false)}
            aria-label="Close"
          >
            <X size={20} />
          </button>
        </div>

        <div className={styles.content}>
          {Array.from(shortcutsByCategory.entries()).map(([category, categoryShortcuts]) => (
            <div key={category} className={styles.category}>
              <h3 className={styles.categoryTitle}>{category}</h3>
              <div className={styles.shortcuts}>
                {categoryShortcuts.map((shortcut, i) => (
                  <div key={i} className={styles.shortcut}>
                    <span className={styles.description}>
                      {shortcut.description}
                    </span>
                    <kbd className={styles.key}>
                      {formatShortcut(shortcut)}
                    </kbd>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>

        <div className={styles.footer}>
          <span className={styles.hint}>
            Press <kbd className={styles.key}>?</kbd> to toggle this help
          </span>
        </div>
      </div>
    </div>
  );
}
