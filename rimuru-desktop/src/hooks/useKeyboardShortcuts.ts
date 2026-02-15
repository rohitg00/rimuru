import { useEffect, useCallback, useRef } from "react";
import { useNavigate } from "react-router-dom";

export interface Shortcut {
  key: string;
  ctrl?: boolean;
  meta?: boolean;
  shift?: boolean;
  alt?: boolean;
  description: string;
  category: string;
  action: () => void;
}

interface UseKeyboardShortcutsOptions {
  enabled?: boolean;
  preventDefault?: boolean;
}

const isMac = typeof navigator !== "undefined" && /Mac/.test(navigator.platform);

export function formatShortcut(shortcut: Omit<Shortcut, "action" | "description" | "category">): string {
  const parts: string[] = [];

  if (shortcut.ctrl || shortcut.meta) {
    parts.push(isMac ? "⌘" : "Ctrl");
  }
  if (shortcut.alt) {
    parts.push(isMac ? "⌥" : "Alt");
  }
  if (shortcut.shift) {
    parts.push(isMac ? "⇧" : "Shift");
  }

  const key = shortcut.key.length === 1 ? shortcut.key.toUpperCase() : shortcut.key;
  parts.push(key);

  return parts.join(isMac ? "" : "+");
}

export function useKeyboardShortcuts(
  shortcuts: Shortcut[],
  options: UseKeyboardShortcutsOptions = {}
) {
  const { enabled = true, preventDefault = true } = options;
  const shortcutsRef = useRef(shortcuts);
  shortcutsRef.current = shortcuts;

  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      if (!enabled) return;

      const target = event.target as HTMLElement;
      if (
        target.tagName === "INPUT" ||
        target.tagName === "TEXTAREA" ||
        target.tagName === "SELECT" ||
        target.isContentEditable
      ) {
        return;
      }

      for (const shortcut of shortcutsRef.current) {
        const ctrlOrMeta = shortcut.ctrl || shortcut.meta;
        const modifierMatch =
          (!ctrlOrMeta || (isMac ? event.metaKey : event.ctrlKey)) &&
          (!shortcut.shift || event.shiftKey) &&
          (!shortcut.alt || event.altKey);

        if (
          modifierMatch &&
          event.key.toLowerCase() === shortcut.key.toLowerCase()
        ) {
          if (preventDefault) {
            event.preventDefault();
          }
          shortcut.action();
          return;
        }
      }
    },
    [enabled, preventDefault]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);
}

export function useGlobalShortcuts() {
  const navigate = useNavigate();

  const shortcuts: Shortcut[] = [
    {
      key: "d",
      meta: true,
      description: "Go to Dashboard",
      category: "Navigation",
      action: () => navigate("/"),
    },
    {
      key: "a",
      meta: true,
      description: "Go to Agents",
      category: "Navigation",
      action: () => navigate("/agents"),
    },
    {
      key: "s",
      meta: true,
      description: "Go to Sessions",
      category: "Navigation",
      action: () => navigate("/sessions"),
    },
    {
      key: "c",
      meta: true,
      shift: true,
      description: "Go to Costs",
      category: "Navigation",
      action: () => navigate("/costs"),
    },
    {
      key: "m",
      meta: true,
      description: "Go to Metrics",
      category: "Navigation",
      action: () => navigate("/metrics"),
    },
    {
      key: "k",
      meta: true,
      shift: true,
      description: "Go to Skills",
      category: "Navigation",
      action: () => navigate("/skills"),
    },
    {
      key: "p",
      meta: true,
      shift: true,
      description: "Go to Plugins",
      category: "Navigation",
      action: () => navigate("/plugins"),
    },
    {
      key: ",",
      meta: true,
      description: "Go to Settings",
      category: "Navigation",
      action: () => navigate("/settings"),
    },
    {
      key: "?",
      shift: true,
      description: "Show keyboard shortcuts",
      category: "Help",
      action: () => {
        const event = new CustomEvent("show-shortcuts-help");
        window.dispatchEvent(event);
      },
    },
    {
      key: "Escape",
      description: "Close modal or dialog",
      category: "General",
      action: () => {
        const event = new CustomEvent("close-modal");
        window.dispatchEvent(event);
      },
    },
    { key: "n", meta: true, shift: true, description: "Launch Agent Wizard", category: "Agent Management", action: () => window.dispatchEvent(new CustomEvent("launch-wizard")) },
    { key: "Enter", meta: true, description: "Quick Launch Agent", category: "Agent Management", action: () => window.dispatchEvent(new CustomEvent("quick-launch")) },
    { key: "t", meta: true, shift: true, description: "Terminate Active Agent", category: "Agent Management", action: () => window.dispatchEvent(new CustomEvent("terminate-active")) },
    { key: "l", meta: true, shift: true, description: "Focus Input", category: "Agent Management", action: () => window.dispatchEvent(new CustomEvent("focus-input")) },
    { key: "e", meta: true, description: "Toggle Right Panel", category: "Panel Toggles", action: () => window.dispatchEvent(new CustomEvent("toggle-right-panel")) },
    { key: "1", meta: true, description: "Files Tab", category: "Panel Toggles", action: () => window.dispatchEvent(new CustomEvent("panel-tab", { detail: "files" })) },
    { key: "2", meta: true, description: "History Tab", category: "Panel Toggles", action: () => window.dispatchEvent(new CustomEvent("panel-tab", { detail: "history" })) },
    { key: "3", meta: true, description: "Auto Run Tab", category: "Panel Toggles", action: () => window.dispatchEvent(new CustomEvent("panel-tab", { detail: "autorun" })) },
    { key: "f", meta: true, shift: true, description: "Zen Mode", category: "Panel Toggles", action: () => window.dispatchEvent(new CustomEvent("zen-mode")) },
    { key: "g", meta: true, description: "Find Next", category: "Search", action: () => window.dispatchEvent(new CustomEvent("find-next")) },
    { key: "g", meta: true, shift: true, description: "Find Previous", category: "Search", action: () => window.dispatchEvent(new CustomEvent("find-prev")) },
    { key: "r", meta: true, shift: true, description: "Start Playbook", category: "Playbook", action: () => window.dispatchEvent(new CustomEvent("playbook-start")) },
    { key: "l", meta: true, alt: true, description: "System Logs", category: "System", action: () => navigate("/logs") },
    { key: "u", meta: true, alt: true, description: "Usage Dashboard", category: "System", action: () => navigate("/usage") },
  ];

  useKeyboardShortcuts(shortcuts);

  return shortcuts;
}

export function getShortcutsByCategory(shortcuts: Shortcut[]): Map<string, Shortcut[]> {
  const map = new Map<string, Shortcut[]>();

  for (const shortcut of shortcuts) {
    const existing = map.get(shortcut.category) || [];
    existing.push(shortcut);
    map.set(shortcut.category, existing);
  }

  return map;
}
