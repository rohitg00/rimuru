import React, { useEffect, useRef, useCallback, useImperativeHandle } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { WebglAddon } from "@xterm/addon-webgl";
import { SearchAddon } from "@xterm/addon-search";
import "@xterm/xterm/css/xterm.css";
import { commands, events } from "@/lib/tauri";
import styles from "./TerminalPane.module.css";

export interface TerminalSearchHandle {
  findNext: (term: string, options?: { regex?: boolean; caseSensitive?: boolean }) => boolean;
  findPrevious: (term: string, options?: { regex?: boolean; caseSensitive?: boolean }) => boolean;
  clearSearch: () => void;
}

interface TerminalPaneProps {
  sessionId: string;
  onExit?: (exitCode: number | null, success: boolean) => void;
}

function getCssVar(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}

function buildTheme(): Record<string, string> {
  return {
    background: getCssVar("--color-bg-primary") || "#1a1b26",
    foreground: getCssVar("--color-text-primary") || "#c0caf5",
    cursor: getCssVar("--color-accent") || "#7aa2f7",
    cursorAccent: getCssVar("--color-bg-primary") || "#1a1b26",
    selectionBackground: getCssVar("--color-bg-tertiary") || "#414868",
    selectionForeground: getCssVar("--color-text-primary") || "#c0caf5",
    black: getCssVar("--color-bg-primary") || "#1a1b26",
    red: getCssVar("--color-error") || "#f7768e",
    green: getCssVar("--color-success") || "#9ece6a",
    yellow: getCssVar("--color-warning") || "#e0af68",
    blue: getCssVar("--color-accent") || "#7aa2f7",
    magenta: getCssVar("--color-accent-hover") || "#89b4fa",
    cyan: getCssVar("--color-info") || "#7dcfff",
    white: getCssVar("--color-text-primary") || "#c0caf5",
    brightBlack: getCssVar("--color-text-muted") || "#565f89",
    brightRed: getCssVar("--color-error") || "#f7768e",
    brightGreen: getCssVar("--color-success") || "#9ece6a",
    brightYellow: getCssVar("--color-warning") || "#e0af68",
    brightBlue: getCssVar("--color-accent") || "#7aa2f7",
    brightMagenta: getCssVar("--color-accent-hover") || "#89b4fa",
    brightCyan: getCssVar("--color-info") || "#7dcfff",
    brightWhite: getCssVar("--color-text-primary") || "#c0caf5",
  };
}

const TerminalPane = React.forwardRef<TerminalSearchHandle, TerminalPaneProps>(
  function TerminalPane({ sessionId, onExit }, ref) {
  const containerRef = useRef<HTMLDivElement>(null);
  const termRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const searchAddonRef = useRef<SearchAddon | null>(null);
  const mountedRef = useRef(false);

  useImperativeHandle(ref, () => ({
    findNext: (term, options) => searchAddonRef.current?.findNext(term, options) ?? false,
    findPrevious: (term, options) => searchAddonRef.current?.findPrevious(term, options) ?? false,
    clearSearch: () => searchAddonRef.current?.clearDecorations(),
  }));

  const handleResize = useCallback(() => {
    const fitAddon = fitAddonRef.current;
    const term = termRef.current;
    if (!fitAddon || !term) return;
    try {
      fitAddon.fit();
      commands.resizeSession(sessionId, term.cols, term.rows);
    } catch {
      // ignore resize errors during teardown
    }
  }, [sessionId]);

  useEffect(() => {
    if (!containerRef.current || mountedRef.current) return;
    mountedRef.current = true;

    const term = new Terminal({
      theme: buildTheme(),
      fontFamily: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', Menlo, monospace",
      fontSize: 13,
      lineHeight: 1.4,
      cursorBlink: true,
      cursorStyle: "bar",
      scrollback: 10000,
      allowProposedApi: true,
    });

    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);

    const searchAddon = new SearchAddon();
    term.loadAddon(searchAddon);

    term.open(containerRef.current);

    try {
      const webglAddon = new WebglAddon();
      webglAddon.onContextLoss(() => webglAddon.dispose());
      term.loadAddon(webglAddon);
    } catch {
      // WebGL not available, canvas renderer is fine
    }

    termRef.current = term;
    fitAddonRef.current = fitAddon;
    searchAddonRef.current = searchAddon;

    requestAnimationFrame(() => {
      const el = containerRef.current;
      if (el && el.clientWidth > 0 && el.clientHeight > 0) {
        fitAddon.fit();
      }
    });

    term.onData((data) => {
      const base64 = btoa(data);
      commands.writeToSession(sessionId, base64);
    });

    const outputUnlisten = events.onPtyOutput(sessionId, (payload) => {
      const raw = atob(payload.data);
      const bytes = new Uint8Array(raw.length);
      for (let i = 0; i < raw.length; i++) {
        bytes[i] = raw.charCodeAt(i);
      }
      term.write(bytes);
    });

    const exitUnlisten = events.onPtyExit((payload) => {
      if (payload.session_id === sessionId) {
        const code = payload.exit_code !== null ? ` with code ${payload.exit_code}` : "";
        term.write(`\r\n\x1b[90m[Process exited${code}]\x1b[0m\r\n`);
        onExit?.(payload.exit_code, payload.success);
      }
    });

    const observer = new ResizeObserver(() => handleResize());
    observer.observe(containerRef.current);

    return () => {
      observer.disconnect();
      outputUnlisten.then((fn) => fn());
      exitUnlisten.then((fn) => fn());
      term.dispose();
      termRef.current = null;
      fitAddonRef.current = null;
      searchAddonRef.current = null;
      mountedRef.current = false;
    };
  }, [sessionId, onExit, handleResize]);

  return <div ref={containerRef} className={styles.container} />;
  }
);

export default TerminalPane;
