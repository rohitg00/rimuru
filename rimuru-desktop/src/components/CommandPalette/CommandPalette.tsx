import { useState, useEffect, useRef, useMemo, useCallback, type ReactNode } from "react";
import { Search, Terminal, Layout, Eye, Zap, ArrowRight, X } from "lucide-react";
import { useAnimatedUnmount } from "@/hooks/useAnimatedUnmount";
import type { PtySessionInfo } from "@/types/pty";
import { EmptyState } from "@/components/EmptyState/EmptyState";
import styles from "./CommandPalette.module.css";

interface CommandItem {
  id: string;
  label: string;
  category: string;
  icon: ReactNode;
  shortcut?: string;
  action: () => void;
}

interface CommandPaletteProps {
  isOpen: boolean;
  onClose: () => void;
  onNavigate: (path: string) => void;
  onLaunchAgent: (agentType: string) => void;
  onToggleSidebar: () => void;
  onToggleInfoPanel: () => void;
  sessions: PtySessionInfo[];
  onSwitchSession: (sessionId: string) => void;
  onTerminateSession: (sessionId: string) => void;
}

const RECENT_KEY = "rimuru-recent-commands";
const MAX_RECENT = 5;

function getRecentIds(): string[] {
  try {
    const stored = localStorage.getItem(RECENT_KEY);
    return stored ? JSON.parse(stored) : [];
  } catch {
    return [];
  }
}

function saveRecentId(id: string) {
  const recent = getRecentIds().filter((r) => r !== id);
  recent.unshift(id);
  localStorage.setItem(RECENT_KEY, JSON.stringify(recent.slice(0, MAX_RECENT)));
}

export default function CommandPalette({
  isOpen,
  onClose,
  onNavigate,
  onLaunchAgent,
  onToggleSidebar,
  onToggleInfoPanel,
  sessions,
  onSwitchSession,
  onTerminateSession,
}: CommandPaletteProps) {
  const { shouldRender, animationState } = useAnimatedUnmount(isOpen, 200);
  const [query, setQuery] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const resultsRef = useRef<HTMLDivElement>(null);

  const allCommands = useMemo<CommandItem[]>(() => {
    const nav: CommandItem[] = [
      { id: "nav-dashboard", label: "Dashboard", category: "Navigation", icon: <Layout size={16} />, action: () => onNavigate("/") },
      { id: "nav-agents", label: "Agents", category: "Navigation", icon: <ArrowRight size={16} />, action: () => onNavigate("/agents") },
      { id: "nav-sessions", label: "Sessions", category: "Navigation", icon: <ArrowRight size={16} />, action: () => onNavigate("/sessions") },
      { id: "nav-costs", label: "Costs", category: "Navigation", icon: <ArrowRight size={16} />, action: () => onNavigate("/costs") },
      { id: "nav-metrics", label: "Metrics", category: "Navigation", icon: <ArrowRight size={16} />, action: () => onNavigate("/metrics") },
      { id: "nav-skills", label: "Skills", category: "Navigation", icon: <ArrowRight size={16} />, action: () => onNavigate("/skills") },
      { id: "nav-plugins", label: "Plugins", category: "Navigation", icon: <ArrowRight size={16} />, action: () => onNavigate("/plugins") },
      { id: "nav-hooks", label: "Hooks", category: "Navigation", icon: <ArrowRight size={16} />, action: () => onNavigate("/hooks") },
      { id: "nav-settings", label: "Settings", category: "Navigation", icon: <ArrowRight size={16} />, action: () => onNavigate("/settings") },
      { id: "nav-orchestrate", label: "Orchestrate", category: "Navigation", icon: <Terminal size={16} />, action: () => onNavigate("/orchestrate") },
      { id: "nav-playbooks", label: "Playbooks", category: "Navigation", icon: <ArrowRight size={16} />, action: () => onNavigate("/playbooks") },
      { id: "nav-groupchat", label: "Group Chat", category: "Navigation", icon: <ArrowRight size={16} />, action: () => onNavigate("/groupchat") },
    ];

    const sessionCmds: CommandItem[] = sessions.flatMap((s) => [
      { id: `session-switch-${s.id}`, label: `Switch to: ${s.agent_name}`, category: "Sessions", icon: <Terminal size={16} />, action: () => onSwitchSession(s.id) },
      { id: `session-term-${s.id}`, label: `Terminate: ${s.agent_name}`, category: "Sessions", icon: <X size={16} />, action: () => onTerminateSession(s.id) },
    ]);

    const launch: CommandItem[] = [
      { id: "launch-claude", label: "Launch Claude Code", category: "Actions", icon: <Zap size={16} />, action: () => onLaunchAgent("claude_code") },
      { id: "launch-codex", label: "Launch Codex", category: "Actions", icon: <Zap size={16} />, action: () => onLaunchAgent("codex") },
      { id: "launch-goose", label: "Launch Goose", category: "Actions", icon: <Zap size={16} />, action: () => onLaunchAgent("goose") },
      { id: "launch-opencode", label: "Launch OpenCode", category: "Actions", icon: <Zap size={16} />, action: () => onLaunchAgent("open_code") },
      { id: "launch-cursor", label: "Launch Cursor", category: "Actions", icon: <Zap size={16} />, action: () => onLaunchAgent("cursor") },
      { id: "launch-copilot", label: "Launch Copilot", category: "Actions", icon: <Zap size={16} />, action: () => onLaunchAgent("copilot") },
    ];

    const view: CommandItem[] = [
      { id: "view-sidebar", label: "Toggle Sidebar", category: "View", icon: <Eye size={16} />, shortcut: "\u2318B", action: onToggleSidebar },
      { id: "view-info", label: "Toggle Info Panel", category: "View", icon: <Eye size={16} />, shortcut: "\u2318I", action: onToggleInfoPanel },
    ];

    return [...sessionCmds, ...nav, ...launch, ...view];
  }, [sessions, onNavigate, onLaunchAgent, onSwitchSession, onTerminateSession, onToggleSidebar, onToggleInfoPanel]);

  const filtered = useMemo(() => {
    if (!query.trim()) {
      const recentIds = getRecentIds();
      const recent = recentIds
        .map((id) => allCommands.find((c) => c.id === id))
        .filter(Boolean) as CommandItem[];
      if (recent.length > 0) {
        return [
          ...recent.map((c) => ({ ...c, category: "Recent" })),
          ...allCommands,
        ];
      }
      return allCommands;
    }
    const lower = query.toLowerCase();
    return allCommands.filter((c) => c.label.toLowerCase().includes(lower));
  }, [query, allCommands]);

  const grouped = useMemo(() => {
    const groups: { category: string; items: CommandItem[] }[] = [];
    const seen = new Map<string, number>();
    for (const item of filtered) {
      const idx = seen.get(item.category);
      if (idx !== undefined) {
        groups[idx].items.push(item);
      } else {
        seen.set(item.category, groups.length);
        groups.push({ category: item.category, items: [item] });
      }
    }
    return groups;
  }, [filtered]);

  const flatItems = useMemo(() => filtered, [filtered]);

  useEffect(() => {
    if (isOpen) {
      setQuery("");
      setActiveIndex(0);
      requestAnimationFrame(() => inputRef.current?.focus());
    }
  }, [isOpen]);

  useEffect(() => {
    setActiveIndex(0);
  }, [query]);

  const executeItem = useCallback(
    (item: CommandItem) => {
      saveRecentId(item.id);
      onClose();
      item.action();
    },
    [onClose]
  );

  useEffect(() => {
    if (!isOpen) return;

    const handler = (e: KeyboardEvent) => {
      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          setActiveIndex((i) => (i < flatItems.length - 1 ? i + 1 : 0));
          break;
        case "ArrowUp":
          e.preventDefault();
          setActiveIndex((i) => (i > 0 ? i - 1 : flatItems.length - 1));
          break;
        case "Enter":
          e.preventDefault();
          if (flatItems[activeIndex]) {
            executeItem(flatItems[activeIndex]);
          }
          break;
        case "Escape":
          e.preventDefault();
          onClose();
          break;
      }
    };

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [isOpen, flatItems, activeIndex, executeItem, onClose]);

  useEffect(() => {
    if (!resultsRef.current) return;
    const active = resultsRef.current.querySelector(`.${styles.itemActive}`);
    active?.scrollIntoView({ block: "nearest" });
  }, [activeIndex]);

  if (!shouldRender) return null;

  let itemIndex = -1;

  return (
    <div className={styles.overlay} data-state={animationState} onClick={onClose}>
      <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
        <div className={styles.searchContainer}>
          <Search size={18} className={styles.searchIcon} />
          <input
            ref={inputRef}
            className={styles.searchInput}
            type="text"
            placeholder="Type a command..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
          <kbd className={styles.itemShortcut}>ESC</kbd>
        </div>
        <div className={styles.results} ref={resultsRef}>
          {grouped.length === 0 && (
            <EmptyState icon={Search} title="No matching commands" description="Try a different search term" />
          )}
          {grouped.map((group) => (
            <div key={group.category}>
              <div className={styles.category}>{group.category}</div>
              {group.items.map((item) => {
                itemIndex++;
                const idx = itemIndex;
                return (
                  <div
                    key={item.id + "-" + idx}
                    className={`${styles.item} ${idx === activeIndex ? styles.itemActive : ""}`}
                    onClick={() => executeItem(item)}
                    onMouseEnter={() => setActiveIndex(idx)}
                  >
                    <span className={styles.itemIcon}>{item.icon}</span>
                    <span className={styles.itemLabel}>{item.label}</span>
                    {item.shortcut && (
                      <span className={styles.itemShortcut}>{item.shortcut}</span>
                    )}
                  </div>
                );
              })}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
