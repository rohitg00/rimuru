import { useState, useRef, useEffect } from "react";
import { useTheme, ThemeName } from "../hooks/useTheme";
import styles from "./ThemeSwitcher.module.css";

interface ThemeOption {
  name: ThemeName;
  label: string;
  preview: {
    bg: string;
    accent: string;
    text: string;
  };
}

const themeOptions: ThemeOption[] = [
  {
    name: "tokyo-night",
    label: "Tokyo Night",
    preview: { bg: "#1a1b26", accent: "#7aa2f7", text: "#c0caf5" },
  },
  {
    name: "catppuccin",
    label: "Catppuccin",
    preview: { bg: "#1e1e2e", accent: "#89b4fa", text: "#cdd6f4" },
  },
  {
    name: "dracula",
    label: "Dracula",
    preview: { bg: "#282a36", accent: "#bd93f9", text: "#f8f8f2" },
  },
  {
    name: "nord",
    label: "Nord",
    preview: { bg: "#2e3440", accent: "#88c0d0", text: "#eceff4" },
  },
  {
    name: "light",
    label: "Light",
    preview: { bg: "#ffffff", accent: "#2563eb", text: "#18181b" },
  },
];

interface ThemeSwitcherProps {
  variant?: "dropdown" | "grid";
  showLabel?: boolean;
}

export function ThemeSwitcher({ variant = "dropdown", showLabel = true }: ThemeSwitcherProps) {
  const { theme, setTheme } = useTheme();
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  const currentTheme = themeOptions.find((t) => t.name === theme) ?? themeOptions[0];

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    }

    if (isOpen) {
      document.addEventListener("mousedown", handleClickOutside);
      return () => document.removeEventListener("mousedown", handleClickOutside);
    }
  }, [isOpen]);

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        setIsOpen(false);
      }
    }

    if (isOpen) {
      document.addEventListener("keydown", handleKeyDown);
      return () => document.removeEventListener("keydown", handleKeyDown);
    }
  }, [isOpen]);

  if (variant === "grid") {
    return (
      <div className={styles.grid}>
        {showLabel && <span className={styles.gridLabel}>Theme</span>}
        <div className={styles.gridOptions}>
          {themeOptions.map((option) => (
            <button
              key={option.name}
              className={`${styles.gridOption} ${theme === option.name ? styles.gridOptionActive : ""}`}
              onClick={() => setTheme(option.name)}
              title={option.label}
              aria-label={`Select ${option.label} theme`}
              aria-pressed={theme === option.name}
            >
              <div
                className={styles.previewSwatch}
                style={{
                  background: option.preview.bg,
                  borderColor: theme === option.name ? option.preview.accent : "transparent",
                }}
              >
                <div
                  className={styles.previewAccent}
                  style={{ background: option.preview.accent }}
                />
              </div>
              <span className={styles.optionLabel}>{option.label}</span>
            </button>
          ))}
        </div>
      </div>
    );
  }

  return (
    <div className={styles.dropdown} ref={dropdownRef}>
      <button
        className={styles.trigger}
        onClick={() => setIsOpen(!isOpen)}
        aria-expanded={isOpen}
        aria-haspopup="listbox"
        aria-label="Select theme"
      >
        <div
          className={styles.triggerPreview}
          style={{
            background: currentTheme.preview.bg,
            borderColor: currentTheme.preview.accent,
          }}
        >
          <div
            className={styles.previewAccent}
            style={{ background: currentTheme.preview.accent }}
          />
        </div>
        {showLabel && <span className={styles.triggerLabel}>{currentTheme.label}</span>}
        <svg
          className={`${styles.chevron} ${isOpen ? styles.chevronOpen : ""}`}
          width="16"
          height="16"
          viewBox="0 0 16 16"
          fill="none"
        >
          <path
            d="M4 6L8 10L12 6"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </svg>
      </button>

      {isOpen && (
        <div className={styles.menu} role="listbox" aria-label="Theme options">
          {themeOptions.map((option) => (
            <button
              key={option.name}
              className={`${styles.menuItem} ${theme === option.name ? styles.menuItemActive : ""}`}
              onClick={() => {
                setTheme(option.name);
                setIsOpen(false);
              }}
              role="option"
              aria-selected={theme === option.name}
            >
              <div
                className={styles.menuPreview}
                style={{
                  background: option.preview.bg,
                  borderColor: theme === option.name ? option.preview.accent : "transparent",
                }}
              >
                <div
                  className={styles.previewAccent}
                  style={{ background: option.preview.accent }}
                />
              </div>
              <span className={styles.menuLabel}>{option.label}</span>
              {theme === option.name && (
                <svg
                  className={styles.checkmark}
                  width="16"
                  height="16"
                  viewBox="0 0 16 16"
                  fill="none"
                >
                  <path
                    d="M13 4L6 12L3 9"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  />
                </svg>
              )}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

export default ThemeSwitcher;
