import { useState, useEffect, useRef, useMemo } from "react";

interface CommandAction {
  id: string;
  label: string;
  description?: string;
  shortcut?: string;
  section: string;
  action: () => void;
}

interface CommandPaletteProps {
  navigate: (path: string) => void;
}

export default function CommandPalette({ navigate }: CommandPaletteProps) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [selected, setSelected] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  const actions: CommandAction[] = useMemo(
    () => [
      { id: "nav-dashboard", label: "Dashboard", description: "Overview and stats", shortcut: "1", section: "Navigation", action: () => navigate("") },
      { id: "nav-agents", label: "Agents", description: "Manage connected agents", shortcut: "2", section: "Navigation", action: () => navigate("agents") },
      { id: "nav-sessions", label: "Sessions", description: "View session history", shortcut: "3", section: "Navigation", action: () => navigate("sessions") },
      { id: "nav-costs", label: "Costs", description: "Cost analytics", shortcut: "4", section: "Navigation", action: () => navigate("costs") },
      { id: "nav-models", label: "Models", description: "Model pricing", shortcut: "5", section: "Navigation", action: () => navigate("models") },
      { id: "nav-metrics", label: "Metrics", description: "System metrics", shortcut: "6", section: "Navigation", action: () => navigate("metrics") },
      { id: "nav-plugins", label: "Plugins", description: "Manage plugins", shortcut: "7", section: "Navigation", action: () => navigate("plugins") },
      { id: "nav-hooks", label: "Hooks", description: "Hook configuration", shortcut: "8", section: "Navigation", action: () => navigate("hooks") },
      { id: "nav-settings", label: "Settings", description: "Configuration", shortcut: "9", section: "Navigation", action: () => navigate("settings") },
      { id: "nav-terminal", label: "Terminal", description: "Embedded terminal", shortcut: "0", section: "Navigation", action: () => navigate("terminal") },
      { id: "act-refresh", label: "Refresh Data", description: "Reload all data", section: "Actions", action: () => window.location.reload() },
      { id: "act-theme", label: "Toggle Theme", description: "Switch theme", section: "Actions", action: () => document.querySelector<HTMLButtonElement>("[data-theme-btn]")?.click() },
    ],
    [navigate],
  );

  const filtered = useMemo(() => {
    if (!query) return actions;
    const q = query.toLowerCase();
    return actions.filter(
      (a) =>
        a.label.toLowerCase().includes(q) ||
        a.description?.toLowerCase().includes(q) ||
        a.section.toLowerCase().includes(q),
    );
  }, [actions, query]);

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        setOpen((v) => !v);
        setQuery("");
        setSelected(0);
      }
      if (e.key === "Escape") {
        setOpen(false);
      }
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  useEffect(() => {
    if (open) {
      requestAnimationFrame(() => inputRef.current?.focus());
    }
  }, [open]);

  useEffect(() => {
    setSelected(0);
  }, [query]);

  useEffect(() => {
    const el = listRef.current?.children[selected] as HTMLElement | undefined;
    el?.scrollIntoView({ block: "nearest" });
  }, [selected]);

  function handleKeyDown(e: React.KeyboardEvent) {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelected((s) => Math.min(s + 1, filtered.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelected((s) => Math.max(s - 1, 0));
    } else if (e.key === "Enter" && filtered[selected]) {
      e.preventDefault();
      filtered[selected].action();
      setOpen(false);
    }
  }

  if (!open) return null;

  const sections = new Map<string, CommandAction[]>();
  for (const a of filtered) {
    const list = sections.get(a.section) ?? [];
    list.push(a);
    sections.set(a.section, list);
  }

  return (
    <div className="fixed inset-0 z-[100] flex items-start justify-center pt-[20vh]">
      <div
        className="absolute inset-0 bg-black/50 backdrop-blur-sm"
        onClick={() => setOpen(false)}
      />

      <div className="relative w-full max-w-lg rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] shadow-2xl overflow-hidden">
        <div className="flex items-center border-b border-[var(--border)] px-4">
          <svg className="w-4 h-4 text-[var(--text-secondary)] shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          <input
            ref={inputRef}
            type="text"
            placeholder="Search commands..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            className="flex-1 px-3 py-3 text-sm bg-transparent text-[var(--text-primary)] placeholder-[var(--text-secondary)] focus:outline-none"
          />
          <kbd className="hidden sm:inline-flex items-center px-1.5 py-0.5 text-[10px] font-medium text-[var(--text-secondary)] bg-[var(--bg-tertiary)] rounded border border-[var(--border)]">
            ESC
          </kbd>
        </div>

        <div ref={listRef} className="max-h-80 overflow-y-auto p-2">
          {filtered.length === 0 ? (
            <div className="px-4 py-8 text-center text-sm text-[var(--text-secondary)]">
              No matching commands
            </div>
          ) : (
            Array.from(sections.entries()).map(([section, items]) => (
              <div key={section}>
                <p className="px-3 py-1.5 text-[10px] font-medium uppercase tracking-wider text-[var(--text-secondary)]">
                  {section}
                </p>
                {items.map((item) => {
                  const idx = filtered.indexOf(item);
                  return (
                    <button
                      key={item.id}
                      onClick={() => {
                        item.action();
                        setOpen(false);
                      }}
                      onMouseEnter={() => setSelected(idx)}
                      className={`w-full flex items-center justify-between px-3 py-2 rounded-lg text-left transition-colors ${
                        idx === selected
                          ? "bg-[var(--accent)]/10 text-[var(--accent)]"
                          : "text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)]"
                      }`}
                    >
                      <div>
                        <p className="text-sm font-medium">{item.label}</p>
                        {item.description && (
                          <p className="text-xs text-[var(--text-secondary)]">
                            {item.description}
                          </p>
                        )}
                      </div>
                      {item.shortcut && (
                        <kbd className="ml-2 px-1.5 py-0.5 text-[10px] font-medium text-[var(--text-secondary)] bg-[var(--bg-tertiary)] rounded border border-[var(--border)]">
                          {item.shortcut}
                        </kbd>
                      )}
                    </button>
                  );
                })}
              </div>
            ))
          )}
        </div>

        <div className="border-t border-[var(--border)] px-4 py-2 flex items-center gap-4 text-[10px] text-[var(--text-secondary)]">
          <span><kbd className="px-1 py-0.5 bg-[var(--bg-tertiary)] rounded border border-[var(--border)]">\u2191\u2193</kbd> navigate</span>
          <span><kbd className="px-1 py-0.5 bg-[var(--bg-tertiary)] rounded border border-[var(--border)]">\u23CE</kbd> select</span>
          <span><kbd className="px-1 py-0.5 bg-[var(--bg-tertiary)] rounded border border-[var(--border)]">esc</kbd> close</span>
        </div>
      </div>
    </div>
  );
}
