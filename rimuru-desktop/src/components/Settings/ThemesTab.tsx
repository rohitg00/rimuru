import { useTheme, ThemeName } from "@/hooks/useTheme";
import styles from "./SettingsModal.module.css";

interface ThemeOption {
  id: ThemeName;
  name: string;
  colors: string[];
}

const DARK_THEMES: ThemeOption[] = [
  { id: "tokyo-night", name: "Tokyo Night", colors: ["#1a1b26", "#7aa2f7", "#9ece6a", "#f7768e", "#e0af68"] },
  { id: "catppuccin", name: "Catppuccin", colors: ["#1e1e2e", "#89b4fa", "#a6e3a1", "#f38ba8", "#f9e2af"] },
  { id: "dracula", name: "Dracula", colors: ["#282a36", "#bd93f9", "#50fa7b", "#ff5555", "#ffb86c"] },
  { id: "nord", name: "Nord", colors: ["#2e3440", "#88c0d0", "#a3be8c", "#bf616a", "#ebcb8b"] },
  { id: "monokai", name: "Monokai", colors: ["#272822", "#a6e22e", "#a6e22e", "#f92672", "#fd971f"] },
  { id: "gruvbox-dark", name: "Gruvbox Dark", colors: ["#282828", "#fe8019", "#b8bb26", "#fb4934", "#fabd2f"] },
  { id: "solarized", name: "Solarized", colors: ["#002b36", "#268bd2", "#859900", "#dc322f", "#b58900"] },
];

const LIGHT_THEMES: ThemeOption[] = [
  { id: "light", name: "Light", colors: ["#ffffff", "#2563eb", "#16a34a", "#dc2626", "#ca8a04"] },
  { id: "github-light", name: "GitHub Light", colors: ["#ffffff", "#0969da", "#1a7f37", "#cf222e", "#9a6700"] },
  { id: "one-light", name: "One Light", colors: ["#fafafa", "#4078f2", "#50a14f", "#e45649", "#c18401"] },
  { id: "gruvbox-light", name: "Gruvbox Light", colors: ["#fbf1c7", "#d65d0e", "#79740e", "#cc241d", "#b57614"] },
  { id: "ayu-light", name: "Ayu Light", colors: ["#fafafa", "#ff6a00", "#86b300", "#f51818", "#f29718"] },
  { id: "catppuccin-latte", name: "Catppuccin Latte", colors: ["#eff1f5", "#1e66f5", "#40a02b", "#d20f39", "#df8e1d"] },
];

const VIBE_THEMES: ThemeOption[] = [
  { id: "pedurple", name: "Pedurple", colors: ["#1a1025", "#9b59b6", "#2ecc71", "#e74c3c", "#f39c12"] },
];

export default function ThemesTab() {
  const { theme, setTheme } = useTheme();

  const renderGrid = (themes: ThemeOption[]) => (
    <div className={styles.themeGrid}>
      {themes.map((t) => (
        <button
          key={t.id}
          className={`${styles.themeCard} ${theme === t.id ? styles.themeCardActive : ""}`}
          onClick={() => setTheme(t.id)}
        >
          <div className={styles.themeName}>{t.name}</div>
          <div className={styles.themeColors}>
            {t.colors.map((color, i) => (
              <div key={i} className={styles.colorDot} style={{ backgroundColor: color }} />
            ))}
          </div>
        </button>
      ))}
    </div>
  );

  return (
    <div>
      <div className={styles.themeSection}>
        <div className={styles.themeSectionTitle}>Dark</div>
        {renderGrid(DARK_THEMES)}
      </div>
      <div className={styles.themeSection}>
        <div className={styles.themeSectionTitle}>Light</div>
        {renderGrid(LIGHT_THEMES)}
      </div>
      <div className={styles.themeSection}>
        <div className={styles.themeSectionTitle}>Vibe</div>
        {renderGrid(VIBE_THEMES)}
      </div>
    </div>
  );
}
