import { createContext, useContext, useEffect, useState, ReactNode } from "react";
import { load } from "@tauri-apps/plugin-store";
import type { Store } from "@tauri-apps/plugin-store";

export type ThemeName = "rimuru" | "tokyo-night" | "catppuccin" | "dracula" | "nord" | "light" | "monokai" | "gruvbox-dark" | "github-light" | "solarized" | "one-light" | "gruvbox-light" | "ayu-light" | "catppuccin-latte" | "pedurple";

interface ThemeContextValue {
  theme: ThemeName;
  setTheme: (theme: ThemeName) => void;
  themes: ThemeName[];
}

const ThemeContext = createContext<ThemeContextValue | null>(null);

const THEMES: ThemeName[] = ["rimuru", "tokyo-night", "catppuccin", "dracula", "nord", "light", "monokai", "gruvbox-dark", "github-light", "solarized", "one-light", "gruvbox-light", "ayu-light", "catppuccin-latte", "pedurple"];

const themeVariables: Record<ThemeName, Record<string, string>> = {
  rimuru: {
    "--color-bg-primary": "#0d1b2a",
    "--color-bg-secondary": "#132234",
    "--color-bg-tertiary": "#1c3148",
    "--color-text-primary": "#dceaf5",
    "--color-text-secondary": "#a8c8e8",
    "--color-text-muted": "#5a84a5",
    "--color-accent": "#5cc6d0",
    "--color-accent-hover": "#7dd8e0",
    "--color-success": "#6ecf8e",
    "--color-warning": "#f0c060",
    "--color-error": "#e8758a",
    "--color-info": "#a8cce8",
    "--color-border": "#1c3148",
    "--color-sidebar": "#091420",
  },
  "tokyo-night": {
    "--color-bg-primary": "#1a1b26",
    "--color-bg-secondary": "#24283b",
    "--color-bg-tertiary": "#414868",
    "--color-text-primary": "#c0caf5",
    "--color-text-secondary": "#a9b1d6",
    "--color-text-muted": "#565f89",
    "--color-accent": "#7aa2f7",
    "--color-accent-hover": "#89b4fa",
    "--color-success": "#9ece6a",
    "--color-warning": "#e0af68",
    "--color-error": "#f7768e",
    "--color-info": "#7dcfff",
    "--color-border": "#414868",
    "--color-sidebar": "#1f2335",
  },
  catppuccin: {
    "--color-bg-primary": "#1e1e2e",
    "--color-bg-secondary": "#313244",
    "--color-bg-tertiary": "#45475a",
    "--color-text-primary": "#cdd6f4",
    "--color-text-secondary": "#bac2de",
    "--color-text-muted": "#6c7086",
    "--color-accent": "#89b4fa",
    "--color-accent-hover": "#b4befe",
    "--color-success": "#a6e3a1",
    "--color-warning": "#f9e2af",
    "--color-error": "#f38ba8",
    "--color-info": "#89dceb",
    "--color-border": "#45475a",
    "--color-sidebar": "#181825",
  },
  dracula: {
    "--color-bg-primary": "#282a36",
    "--color-bg-secondary": "#44475a",
    "--color-bg-tertiary": "#6272a4",
    "--color-text-primary": "#f8f8f2",
    "--color-text-secondary": "#f8f8f2",
    "--color-text-muted": "#6272a4",
    "--color-accent": "#bd93f9",
    "--color-accent-hover": "#ff79c6",
    "--color-success": "#50fa7b",
    "--color-warning": "#ffb86c",
    "--color-error": "#ff5555",
    "--color-info": "#8be9fd",
    "--color-border": "#6272a4",
    "--color-sidebar": "#21222c",
  },
  nord: {
    "--color-bg-primary": "#2e3440",
    "--color-bg-secondary": "#3b4252",
    "--color-bg-tertiary": "#434c5e",
    "--color-text-primary": "#eceff4",
    "--color-text-secondary": "#e5e9f0",
    "--color-text-muted": "#4c566a",
    "--color-accent": "#88c0d0",
    "--color-accent-hover": "#81a1c1",
    "--color-success": "#a3be8c",
    "--color-warning": "#ebcb8b",
    "--color-error": "#bf616a",
    "--color-info": "#5e81ac",
    "--color-border": "#4c566a",
    "--color-sidebar": "#242933",
  },
  light: {
    "--color-bg-primary": "#ffffff",
    "--color-bg-secondary": "#f4f4f5",
    "--color-bg-tertiary": "#e4e4e7",
    "--color-text-primary": "#18181b",
    "--color-text-secondary": "#3f3f46",
    "--color-text-muted": "#a1a1aa",
    "--color-accent": "#2563eb",
    "--color-accent-hover": "#1d4ed8",
    "--color-success": "#16a34a",
    "--color-warning": "#ca8a04",
    "--color-error": "#dc2626",
    "--color-info": "#0284c7",
    "--color-border": "#e4e4e7",
    "--color-sidebar": "#fafafa",
  },
  monokai: {
    "--color-bg-primary": "#272822",
    "--color-bg-secondary": "#3e3d32",
    "--color-bg-tertiary": "#75715e",
    "--color-text-primary": "#f8f8f2",
    "--color-text-secondary": "#e6db74",
    "--color-text-muted": "#75715e",
    "--color-accent": "#a6e22e",
    "--color-accent-hover": "#66d9ef",
    "--color-success": "#a6e22e",
    "--color-warning": "#fd971f",
    "--color-error": "#f92672",
    "--color-info": "#66d9ef",
    "--color-border": "#49483e",
    "--color-sidebar": "#1e1f1c",
  },
  "gruvbox-dark": {
    "--color-bg-primary": "#282828",
    "--color-bg-secondary": "#3c3836",
    "--color-bg-tertiary": "#504945",
    "--color-text-primary": "#ebdbb2",
    "--color-text-secondary": "#d5c4a1",
    "--color-text-muted": "#665c54",
    "--color-accent": "#fe8019",
    "--color-accent-hover": "#fabd2f",
    "--color-success": "#b8bb26",
    "--color-warning": "#fabd2f",
    "--color-error": "#fb4934",
    "--color-info": "#83a598",
    "--color-border": "#504945",
    "--color-sidebar": "#1d2021",
  },
  "github-light": {
    "--color-bg-primary": "#ffffff",
    "--color-bg-secondary": "#f6f8fa",
    "--color-bg-tertiary": "#d0d7de",
    "--color-text-primary": "#1f2328",
    "--color-text-secondary": "#656d76",
    "--color-text-muted": "#8c959f",
    "--color-accent": "#0969da",
    "--color-accent-hover": "#0550ae",
    "--color-success": "#1a7f37",
    "--color-warning": "#9a6700",
    "--color-error": "#cf222e",
    "--color-info": "#0969da",
    "--color-border": "#d0d7de",
    "--color-sidebar": "#f6f8fa",
  },
  solarized: {
    "--color-bg-primary": "#002b36",
    "--color-bg-secondary": "#073642",
    "--color-bg-tertiary": "#586e75",
    "--color-text-primary": "#839496",
    "--color-text-secondary": "#93a1a1",
    "--color-text-muted": "#586e75",
    "--color-accent": "#268bd2",
    "--color-accent-hover": "#2aa198",
    "--color-success": "#859900",
    "--color-warning": "#b58900",
    "--color-error": "#dc322f",
    "--color-info": "#268bd2",
    "--color-border": "#073642",
    "--color-sidebar": "#001e26",
  },
  "one-light": {
    "--color-bg-primary": "#fafafa",
    "--color-bg-secondary": "#f0f0f0",
    "--color-bg-tertiary": "#e0e0e0",
    "--color-text-primary": "#383a42",
    "--color-text-secondary": "#696c77",
    "--color-text-muted": "#a0a1a7",
    "--color-accent": "#4078f2",
    "--color-accent-hover": "#3d6ce7",
    "--color-success": "#50a14f",
    "--color-warning": "#c18401",
    "--color-error": "#e45649",
    "--color-info": "#4078f2",
    "--color-border": "#e0e0e0",
    "--color-sidebar": "#f0f0f0",
  },
  "gruvbox-light": {
    "--color-bg-primary": "#fbf1c7",
    "--color-bg-secondary": "#ebdbb2",
    "--color-bg-tertiary": "#d5c4a1",
    "--color-text-primary": "#3c3836",
    "--color-text-secondary": "#504945",
    "--color-text-muted": "#7c6f64",
    "--color-accent": "#d65d0e",
    "--color-accent-hover": "#af3a03",
    "--color-success": "#79740e",
    "--color-warning": "#b57614",
    "--color-error": "#cc241d",
    "--color-info": "#076678",
    "--color-border": "#d5c4a1",
    "--color-sidebar": "#f2e5bc",
  },
  "ayu-light": {
    "--color-bg-primary": "#fafafa",
    "--color-bg-secondary": "#f3f4f5",
    "--color-bg-tertiary": "#e7e8e9",
    "--color-text-primary": "#575f66",
    "--color-text-secondary": "#828c99",
    "--color-text-muted": "#abb0b6",
    "--color-accent": "#ff6a00",
    "--color-accent-hover": "#e65100",
    "--color-success": "#86b300",
    "--color-warning": "#f29718",
    "--color-error": "#f51818",
    "--color-info": "#36a3d9",
    "--color-border": "#e7e8e9",
    "--color-sidebar": "#f3f4f5",
  },
  "catppuccin-latte": {
    "--color-bg-primary": "#eff1f5",
    "--color-bg-secondary": "#e6e9ef",
    "--color-bg-tertiary": "#ccd0da",
    "--color-text-primary": "#4c4f69",
    "--color-text-secondary": "#5c5f77",
    "--color-text-muted": "#9ca0b0",
    "--color-accent": "#1e66f5",
    "--color-accent-hover": "#7287fd",
    "--color-success": "#40a02b",
    "--color-warning": "#df8e1d",
    "--color-error": "#d20f39",
    "--color-info": "#209fb5",
    "--color-border": "#ccd0da",
    "--color-sidebar": "#e6e9ef",
  },
  pedurple: {
    "--color-bg-primary": "#1a1025",
    "--color-bg-secondary": "#261838",
    "--color-bg-tertiary": "#3d2a54",
    "--color-text-primary": "#e0d4ed",
    "--color-text-secondary": "#c4b4d6",
    "--color-text-muted": "#7a6b8a",
    "--color-accent": "#9b59b6",
    "--color-accent-hover": "#8e44ad",
    "--color-success": "#2ecc71",
    "--color-warning": "#f39c12",
    "--color-error": "#e74c3c",
    "--color-info": "#3498db",
    "--color-border": "#3d2a54",
    "--color-sidebar": "#150d1f",
  },
};

function applyTheme(themeName: ThemeName) {
  const root = document.documentElement;
  const vars = themeVariables[themeName];

  root.setAttribute("data-theme", themeName);

  for (const [key, value] of Object.entries(vars)) {
    root.style.setProperty(key, value);
  }
}

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setThemeState] = useState<ThemeName>("rimuru");
  const [store, setStore] = useState<Store | null>(null);

  useEffect(() => {
    const initStore = async () => {
      try {
        const s = await load("settings.json", { defaults: {}, autoSave: true });
        setStore(s);
        const savedTheme = await s.get<ThemeName>("theme");
        if (savedTheme && THEMES.includes(savedTheme)) {
          setThemeState(savedTheme);
          applyTheme(savedTheme);
        }
      } catch (e) {
        console.error("Failed to load theme from store:", e);
      }
    };
    initStore();
  }, []);

  useEffect(() => {
    applyTheme(theme);
  }, [theme]);

  const setTheme = async (newTheme: ThemeName) => {
    setThemeState(newTheme);
    if (store) {
      try {
        await store.set("theme", newTheme);
        await store.save();
      } catch (e) {
        console.error("Failed to save theme:", e);
      }
    }
  };

  return (
    <ThemeContext.Provider value={{ theme, setTheme, themes: THEMES }}>
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme() {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error("useTheme must be used within a ThemeProvider");
  }
  return context;
}
