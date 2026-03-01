import { useState, useEffect, useRef } from "react";

interface ThemeDef {
  name: string;
  label: string;
  vars: Record<string, string>;
}

const THEMES: ThemeDef[] = [
  {
    name: "catppuccin-mocha",
    label: "Catppuccin Mocha",
    vars: {
      "--bg-primary": "#1e1e2e",
      "--bg-secondary": "#181825",
      "--bg-tertiary": "#313244",
      "--text-primary": "#cdd6f4",
      "--text-secondary": "#a6adc8",
      "--accent": "#89b4fa",
      "--accent-hover": "#74c7ec",
      "--success": "#a6e3a1",
      "--warning": "#f9e2af",
      "--error": "#f38ba8",
      "--border": "#45475a",
    },
  },
  {
    name: "catppuccin-latte",
    label: "Catppuccin Latte",
    vars: {
      "--bg-primary": "#eff1f5",
      "--bg-secondary": "#e6e9ef",
      "--bg-tertiary": "#ccd0da",
      "--text-primary": "#4c4f69",
      "--text-secondary": "#6c6f85",
      "--accent": "#1e66f5",
      "--accent-hover": "#04a5e5",
      "--success": "#40a02b",
      "--warning": "#df8e1d",
      "--error": "#d20f39",
      "--border": "#bcc0cc",
    },
  },
  {
    name: "gruvbox-dark",
    label: "Gruvbox Dark",
    vars: {
      "--bg-primary": "#282828",
      "--bg-secondary": "#1d2021",
      "--bg-tertiary": "#3c3836",
      "--text-primary": "#ebdbb2",
      "--text-secondary": "#a89984",
      "--accent": "#fabd2f",
      "--accent-hover": "#fe8019",
      "--success": "#b8bb26",
      "--warning": "#fabd2f",
      "--error": "#fb4934",
      "--border": "#504945",
    },
  },
  {
    name: "gruvbox-light",
    label: "Gruvbox Light",
    vars: {
      "--bg-primary": "#fbf1c7",
      "--bg-secondary": "#f2e5bc",
      "--bg-tertiary": "#ebdbb2",
      "--text-primary": "#3c3836",
      "--text-secondary": "#665c54",
      "--accent": "#b57614",
      "--accent-hover": "#af3a03",
      "--success": "#79740e",
      "--warning": "#b57614",
      "--error": "#cc241d",
      "--border": "#d5c4a1",
    },
  },
  {
    name: "tokyo-night",
    label: "Tokyo Night",
    vars: {
      "--bg-primary": "#1a1b26",
      "--bg-secondary": "#16161e",
      "--bg-tertiary": "#292e42",
      "--text-primary": "#c0caf5",
      "--text-secondary": "#565f89",
      "--accent": "#7aa2f7",
      "--accent-hover": "#2ac3de",
      "--success": "#9ece6a",
      "--warning": "#e0af68",
      "--error": "#f7768e",
      "--border": "#3b4261",
    },
  },
  {
    name: "nord",
    label: "Nord",
    vars: {
      "--bg-primary": "#2e3440",
      "--bg-secondary": "#282c34",
      "--bg-tertiary": "#3b4252",
      "--text-primary": "#eceff4",
      "--text-secondary": "#d8dee9",
      "--accent": "#88c0d0",
      "--accent-hover": "#81a1c1",
      "--success": "#a3be8c",
      "--warning": "#ebcb8b",
      "--error": "#bf616a",
      "--border": "#4c566a",
    },
  },
  {
    name: "dracula",
    label: "Dracula",
    vars: {
      "--bg-primary": "#282a36",
      "--bg-secondary": "#21222c",
      "--bg-tertiary": "#44475a",
      "--text-primary": "#f8f8f2",
      "--text-secondary": "#6272a4",
      "--accent": "#bd93f9",
      "--accent-hover": "#ff79c6",
      "--success": "#50fa7b",
      "--warning": "#f1fa8c",
      "--error": "#ff5555",
      "--border": "#44475a",
    },
  },
  {
    name: "solarized-dark",
    label: "Solarized Dark",
    vars: {
      "--bg-primary": "#002b36",
      "--bg-secondary": "#001e26",
      "--bg-tertiary": "#073642",
      "--text-primary": "#839496",
      "--text-secondary": "#586e75",
      "--accent": "#268bd2",
      "--accent-hover": "#2aa198",
      "--success": "#859900",
      "--warning": "#b58900",
      "--error": "#dc322f",
      "--border": "#073642",
    },
  },
  {
    name: "solarized-light",
    label: "Solarized Light",
    vars: {
      "--bg-primary": "#fdf6e3",
      "--bg-secondary": "#eee8d5",
      "--bg-tertiary": "#e4dfcb",
      "--text-primary": "#657b83",
      "--text-secondary": "#93a1a1",
      "--accent": "#268bd2",
      "--accent-hover": "#2aa198",
      "--success": "#859900",
      "--warning": "#b58900",
      "--error": "#dc322f",
      "--border": "#d3cbb7",
    },
  },
  {
    name: "one-dark",
    label: "One Dark",
    vars: {
      "--bg-primary": "#282c34",
      "--bg-secondary": "#21252b",
      "--bg-tertiary": "#2c313c",
      "--text-primary": "#abb2bf",
      "--text-secondary": "#636d83",
      "--accent": "#61afef",
      "--accent-hover": "#c678dd",
      "--success": "#98c379",
      "--warning": "#e5c07b",
      "--error": "#e06c75",
      "--border": "#3e4451",
    },
  },
  {
    name: "monokai",
    label: "Monokai",
    vars: {
      "--bg-primary": "#272822",
      "--bg-secondary": "#1e1f1c",
      "--bg-tertiary": "#3e3d32",
      "--text-primary": "#f8f8f2",
      "--text-secondary": "#75715e",
      "--accent": "#66d9ef",
      "--accent-hover": "#a6e22e",
      "--success": "#a6e22e",
      "--warning": "#e6db74",
      "--error": "#f92672",
      "--border": "#49483e",
    },
  },
  {
    name: "github-dark",
    label: "GitHub Dark",
    vars: {
      "--bg-primary": "#0d1117",
      "--bg-secondary": "#010409",
      "--bg-tertiary": "#161b22",
      "--text-primary": "#e6edf3",
      "--text-secondary": "#7d8590",
      "--accent": "#58a6ff",
      "--accent-hover": "#79c0ff",
      "--success": "#3fb950",
      "--warning": "#d29922",
      "--error": "#f85149",
      "--border": "#30363d",
    },
  },
  {
    name: "github-light",
    label: "GitHub Light",
    vars: {
      "--bg-primary": "#ffffff",
      "--bg-secondary": "#f6f8fa",
      "--bg-tertiary": "#eaeef2",
      "--text-primary": "#1f2328",
      "--text-secondary": "#656d76",
      "--accent": "#0969da",
      "--accent-hover": "#0550ae",
      "--success": "#1a7f37",
      "--warning": "#9a6700",
      "--error": "#cf222e",
      "--border": "#d0d7de",
    },
  },
  {
    name: "rose-pine",
    label: "Ros\u00e9 Pine",
    vars: {
      "--bg-primary": "#191724",
      "--bg-secondary": "#1f1d2e",
      "--bg-tertiary": "#26233a",
      "--text-primary": "#e0def4",
      "--text-secondary": "#908caa",
      "--accent": "#c4a7e7",
      "--accent-hover": "#ebbcba",
      "--success": "#9ccfd8",
      "--warning": "#f6c177",
      "--error": "#eb6f92",
      "--border": "#393552",
    },
  },
  {
    name: "ayu-dark",
    label: "Ayu Dark",
    vars: {
      "--bg-primary": "#0b0e14",
      "--bg-secondary": "#0d1017",
      "--bg-tertiary": "#131721",
      "--text-primary": "#bfbdb6",
      "--text-secondary": "#565b66",
      "--accent": "#e6b450",
      "--accent-hover": "#ffb454",
      "--success": "#7fd962",
      "--warning": "#e6b450",
      "--error": "#d95757",
      "--border": "#1c2029",
    },
  },
];

export function getThemes() {
  return THEMES;
}

export function applyTheme(name: string) {
  const theme = THEMES.find((t) => t.name === name);
  if (!theme) return;
  document.documentElement.setAttribute("data-theme", name);
  const root = document.documentElement;
  for (const [key, val] of Object.entries(theme.vars)) {
    root.style.setProperty(key, val);
  }
  localStorage.setItem("rimuru-theme", name);
}

export function getStoredTheme(): string {
  return localStorage.getItem("rimuru-theme") ?? "catppuccin-mocha";
}

export default function ThemeSwitcher() {
  const [open, setOpen] = useState(false);
  const [current, setCurrent] = useState(getStoredTheme);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    applyTheme(current);
  }, [current]);

  useEffect(() => {
    function handleClick(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    }
    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, []);

  return (
    <div className="relative" ref={ref}>
      <button
        onClick={() => setOpen((v) => !v)}
        className="flex items-center gap-2 px-3 py-1.5 text-xs rounded-lg bg-[var(--bg-tertiary)] text-[var(--text-secondary)] border border-[var(--border)] hover:text-[var(--text-primary)] hover:border-[var(--accent)]/50 transition-colors"
      >
        <span
          className="w-3 h-3 rounded-full border border-[var(--border)]"
          style={{ backgroundColor: "var(--accent)" }}
        />
        Theme
      </button>

      {open && (
        <div className="absolute right-0 top-full mt-2 w-56 max-h-80 overflow-y-auto rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] shadow-2xl z-50">
          <div className="p-2">
            {THEMES.map((t) => (
              <button
                key={t.name}
                onClick={() => {
                  setCurrent(t.name);
                  setOpen(false);
                }}
                className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg text-left text-sm transition-colors ${
                  current === t.name
                    ? "bg-[var(--accent)]/10 text-[var(--accent)]"
                    : "text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)]"
                }`}
              >
                <div className="flex gap-0.5">
                  {[t.vars["--bg-primary"], t.vars["--accent"], t.vars["--success"], t.vars["--error"]].map(
                    (c, i) => (
                      <span
                        key={i}
                        className="w-3 h-3 rounded-full border border-black/20"
                        style={{ backgroundColor: c }}
                      />
                    ),
                  )}
                </div>
                <span className="flex-1 truncate">{t.label}</span>
                {current === t.name && (
                  <span className="text-[var(--accent)]">\u2713</span>
                )}
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
