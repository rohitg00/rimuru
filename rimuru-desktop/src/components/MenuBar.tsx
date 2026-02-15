import { useState, useRef, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { ChevronDown } from "lucide-react";
import { formatShortcut } from "@/hooks/useKeyboardShortcuts";
import styles from "./MenuBar.module.css";

interface MenuItem {
  label: string;
  action?: () => void;
  shortcut?: { key: string; meta?: boolean; shift?: boolean; alt?: boolean };
  separator?: boolean;
  disabled?: boolean;
}

interface Menu {
  label: string;
  items: MenuItem[];
}

interface MenuBarProps {
  onShowAbout: () => void;
  onShowSettings: () => void;
  onShowShortcuts: () => void;
  onResetOnboarding: () => void;
}

export function MenuBar({
  onShowAbout,
  onShowSettings,
  onShowShortcuts,
  onResetOnboarding,
}: MenuBarProps) {
  const navigate = useNavigate();
  const [openMenu, setOpenMenu] = useState<string | null>(null);
  const menuBarRef = useRef<HTMLDivElement>(null);

  const menus: Menu[] = [
    {
      label: "File",
      items: [
        {
          label: "New Agent",
          action: () => {
            navigate("/agents");
            window.dispatchEvent(new CustomEvent("open-add-agent-modal"));
          },
          shortcut: { key: "n", meta: true },
        },
        { separator: true, label: "" },
        {
          label: "Settings",
          action: onShowSettings,
          shortcut: { key: ",", meta: true },
        },
        { separator: true, label: "" },
        {
          label: "Quit Rimuru",
          action: () => {
            window.close();
          },
          shortcut: { key: "q", meta: true },
        },
      ],
    },
    {
      label: "Edit",
      items: [
        {
          label: "Refresh Data",
          action: () => window.dispatchEvent(new CustomEvent("refresh-data")),
          shortcut: { key: "r", meta: true },
        },
        { separator: true, label: "" },
        {
          label: "Search",
          action: () => document.querySelector<HTMLInputElement>('[data-search-input]')?.focus(),
          shortcut: { key: "k", meta: true },
        },
      ],
    },
    {
      label: "View",
      items: [
        {
          label: "Dashboard",
          action: () => navigate("/"),
          shortcut: { key: "d", meta: true },
        },
        {
          label: "Agents",
          action: () => navigate("/agents"),
          shortcut: { key: "a", meta: true },
        },
        {
          label: "Sessions",
          action: () => navigate("/sessions"),
          shortcut: { key: "s", meta: true },
        },
        {
          label: "Costs",
          action: () => navigate("/costs"),
          shortcut: { key: "c", meta: true, shift: true },
        },
        {
          label: "Metrics",
          action: () => navigate("/metrics"),
          shortcut: { key: "m", meta: true },
        },
        { separator: true, label: "" },
        {
          label: "Skills",
          action: () => navigate("/skills"),
        },
        {
          label: "Plugins",
          action: () => navigate("/plugins"),
        },
        {
          label: "Hooks",
          action: () => navigate("/hooks"),
        },
      ],
    },
    {
      label: "Help",
      items: [
        {
          label: "Keyboard Shortcuts",
          action: onShowShortcuts,
          shortcut: { key: "?", shift: true },
        },
        {
          label: "Show Onboarding",
          action: onResetOnboarding,
        },
        { separator: true, label: "" },
        {
          label: "Documentation",
          action: () => window.open("https://github.com/rohitg00/rimuru#readme", "_blank"),
        },
        {
          label: "Report Issue",
          action: () => window.open("https://github.com/rohitg00/rimuru/issues", "_blank"),
        },
        { separator: true, label: "" },
        {
          label: "About Rimuru",
          action: onShowAbout,
        },
      ],
    },
  ];

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (menuBarRef.current && !menuBarRef.current.contains(e.target as Node)) {
        setOpenMenu(null);
      }
    };

    if (openMenu) {
      document.addEventListener("click", handleClickOutside);
      return () => document.removeEventListener("click", handleClickOutside);
    }
  }, [openMenu]);

  useEffect(() => {
    const handleEsc = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setOpenMenu(null);
      }
    };

    if (openMenu) {
      document.addEventListener("keydown", handleEsc);
      return () => document.removeEventListener("keydown", handleEsc);
    }
  }, [openMenu]);

  const handleMenuClick = (label: string) => {
    setOpenMenu(openMenu === label ? null : label);
  };

  const handleItemClick = (item: MenuItem) => {
    if (item.disabled || item.separator) return;
    item.action?.();
    setOpenMenu(null);
  };

  return (
    <div className={styles.menuBar} ref={menuBarRef}>
      {menus.map((menu) => (
        <div key={menu.label} className={styles.menuWrapper}>
          <button
            className={`${styles.menuButton} ${openMenu === menu.label ? styles.active : ""}`}
            onClick={() => handleMenuClick(menu.label)}
            onMouseEnter={() => openMenu && setOpenMenu(menu.label)}
          >
            {menu.label}
            <ChevronDown size={14} className={styles.chevron} />
          </button>

          {openMenu === menu.label && (
            <div className={styles.dropdown}>
              {menu.items.map((item, i) =>
                item.separator ? (
                  <div key={i} className={styles.separator} />
                ) : (
                  <button
                    key={i}
                    className={`${styles.menuItem} ${item.disabled ? styles.disabled : ""}`}
                    onClick={() => handleItemClick(item)}
                    disabled={item.disabled}
                  >
                    <span className={styles.itemLabel}>{item.label}</span>
                    {item.shortcut && (
                      <span className={styles.shortcut}>
                        {formatShortcut(item.shortcut)}
                      </span>
                    )}
                  </button>
                )
              )}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}
