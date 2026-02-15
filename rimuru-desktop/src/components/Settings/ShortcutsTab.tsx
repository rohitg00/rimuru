import { useState, useMemo } from "react";
import { useGlobalShortcuts, getShortcutsByCategory, formatShortcut } from "@/hooks/useKeyboardShortcuts";
import styles from "./SettingsModal.module.css";

export default function ShortcutsTab() {
  const shortcuts = useGlobalShortcuts();
  const [filter, setFilter] = useState("");

  const filteredCategories = useMemo(() => {
    const lowerFilter = filter.toLowerCase();
    const filtered = lowerFilter
      ? shortcuts.filter((s) => s.description.toLowerCase().includes(lowerFilter) || s.category.toLowerCase().includes(lowerFilter))
      : shortcuts;
    return getShortcutsByCategory(filtered);
  }, [shortcuts, filter]);

  return (
    <div>
      <input
        className={styles.filterInput}
        type="text"
        placeholder="Filter shortcuts..."
        value={filter}
        onChange={(e) => setFilter(e.target.value)}
      />
      {Array.from(filteredCategories.entries()).map(([category, items]) => (
        <div key={category} className={styles.shortcutGroup}>
          <div className={styles.shortcutGroupTitle}>{category}</div>
          {items.map((shortcut) => {
            return (
              <div key={shortcut.description} className={styles.shortcutRow}>
                <span className={styles.shortcutDesc}>{shortcut.description}</span>
                <div className={styles.shortcutKeys}>
                  <span className={styles.kbd}>{formatShortcut(shortcut)}</span>
                </div>
              </div>
            );
          })}
        </div>
      ))}
      {filteredCategories.size === 0 && (
        <div className={styles.placeholder}>No shortcuts match your filter</div>
      )}
    </div>
  );
}
