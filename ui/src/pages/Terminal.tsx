import { useEffect, useRef, useState } from "react";

export default function Terminal() {
  const termRef = useRef<HTMLDivElement>(null);
  const terminalRef = useRef<unknown>(null);
  const [loaded, setLoaded] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let mounted = true;
    let xtermInstance: { dispose: () => void } | null = null;

    async function init() {
      try {
        const { Terminal } = await import("@xterm/xterm");
        const { FitAddon } = await import("@xterm/addon-fit");

        if (!mounted || !termRef.current) return;

        const computedBg = getComputedStyle(document.documentElement)
          .getPropertyValue("--bg-primary")
          .trim();
        const computedFg = getComputedStyle(document.documentElement)
          .getPropertyValue("--text-primary")
          .trim();
        const computedAccent = getComputedStyle(document.documentElement)
          .getPropertyValue("--accent")
          .trim();

        const term = new Terminal({
          cursorBlink: true,
          fontSize: 13,
          fontFamily:
            "'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace",
          theme: {
            background: computedBg || "#1e1e2e",
            foreground: computedFg || "#cdd6f4",
            cursor: computedAccent || "#89b4fa",
            selectionBackground: (computedAccent || "#89b4fa") + "40",
          },
          allowProposedApi: true,
        });

        const fitAddon = new FitAddon();
        term.loadAddon(fitAddon);
        term.open(termRef.current);
        fitAddon.fit();

        xtermInstance = term;
        terminalRef.current = term;

        const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
        const host = window.location.hostname;
        const wsUrl = `${protocol}//${host}:3112/terminal`;

        let ws: WebSocket | null = null;
        let wsAttempted = false;

        function connectWs() {
          if (wsAttempted) return;
          wsAttempted = true;

          ws = new WebSocket(wsUrl);

          ws.onopen = () => {
            term.writeln("\x1b[1;32mConnected to Rimuru terminal\x1b[0m");
            term.writeln(
              "\x1b[90mType commands to interact with the server\x1b[0m",
            );
            term.writeln("");
            term.write("\x1b[1;34mrimuru\x1b[0m \x1b[90m>\x1b[0m ");
          };

          ws.onmessage = (evt) => {
            term.write(evt.data);
          };

          ws.onerror = () => {
            term.writeln(
              "\x1b[33mRunning in local mode (no WebSocket server)\x1b[0m",
            );
            term.writeln(
              "\x1b[90mType \x1b[36mhelp\x1b[90m for available commands\x1b[0m",
            );
            term.writeln("");
            term.write("\x1b[1;34mrimuru\x1b[0m \x1b[90m>\x1b[0m ");
          };

          ws.onclose = () => {};
        }

        let inputBuffer = "";

        term.onData((data) => {
          if (data === "\r") {
            term.writeln("");
            if (ws && ws.readyState === WebSocket.OPEN) {
              ws.send(JSON.stringify({ type: "command", data: inputBuffer }));
            } else {
              handleLocalCommand(term, inputBuffer);
            }
            inputBuffer = "";
            term.write("\x1b[1;34mrimuru\x1b[0m \x1b[90m>\x1b[0m ");
          } else if (data === "\x7f") {
            if (inputBuffer.length > 0) {
              inputBuffer = inputBuffer.slice(0, -1);
              term.write("\b \b");
            }
          } else if (data >= " ") {
            inputBuffer += data;
            term.write(data);
          }
        });

        connectWs();

        const resizeObserver = new ResizeObserver(() => {
          fitAddon.fit();
        });
        resizeObserver.observe(termRef.current);

        if (mounted) setLoaded(true);

        return () => {
          resizeObserver.disconnect();
          if (ws) ws.close();
        };
      } catch (err) {
        if (mounted) {
          setError(
            err instanceof Error ? err.message : "Failed to load terminal",
          );
        }
      }
    }

    init();

    return () => {
      mounted = false;
      if (xtermInstance) {
        xtermInstance.dispose();
      }
    };
  }, []);

  return (
    <div className="space-y-4 h-full flex flex-col">
      <div className="flex items-center justify-between shrink-0">
        <div>
          <h2 className="text-xl font-bold text-[var(--text-primary)]">
            Terminal
          </h2>
          <p className="text-sm text-[var(--text-secondary)]">
            Interactive terminal connected to Rimuru server
          </p>
        </div>
        <div className="flex items-center gap-2">
          <div className="flex gap-1.5">
            <span className="w-3 h-3 rounded-full bg-[var(--error)]" />
            <span className="w-3 h-3 rounded-full bg-[var(--warning)]" />
            <span className="w-3 h-3 rounded-full bg-[var(--success)]" />
          </div>
        </div>
      </div>

      {error ? (
        <div className="flex-1 rounded-xl border border-[var(--error)]/30 bg-[var(--error)]/5 flex items-center justify-center">
          <div className="text-center">
            <p className="text-[var(--error)] mb-2">Failed to load terminal</p>
            <p className="text-xs text-[var(--text-secondary)]">{error}</p>
          </div>
        </div>
      ) : (
        <div
          className={`flex-1 rounded-xl border border-[var(--border)] bg-[var(--bg-primary)] overflow-hidden ${
            !loaded ? "flex items-center justify-center" : ""
          }`}
        >
          {!loaded && (
            <p className="text-sm text-[var(--text-secondary)] animate-pulse">
              Loading terminal...
            </p>
          )}
          <div
            ref={termRef}
            className="w-full h-full"
            style={{ display: loaded ? "block" : "none", padding: "8px" }}
          />
        </div>
      )}
    </div>
  );
}

async function handleLocalCommand(
  term: { writeln: (s: string) => void; write: (s: string) => void },
  command: string,
) {
  const trimmed = command.trim();
  if (!trimmed) return;

  switch (trimmed) {
    case "help":
      term.writeln("\x1b[1mAvailable commands:\x1b[0m");
      term.writeln("  \x1b[36mhelp\x1b[0m        Show this help");
      term.writeln("  \x1b[36mstatus\x1b[0m      Show server status");
      term.writeln("  \x1b[36magents\x1b[0m      List connected agents");
      term.writeln("  \x1b[36msessions\x1b[0m    Show session stats");
      term.writeln("  \x1b[36mcosts\x1b[0m       Show cost summary");
      term.writeln("  \x1b[36mhealth\x1b[0m      Health check");
      term.writeln("  \x1b[36mclear\x1b[0m       Clear terminal");
      term.writeln("  \x1b[36mversion\x1b[0m     Show version");
      break;
    case "status":
    case "health":
      try {
        const r = await fetch("/api/health");
        const d = await r.json();
        term.writeln(`\x1b[1mHealth:\x1b[0m ${d.status ?? "ok"}`);
        if (d.agents_connected != null)
          term.writeln(`  Agents connected: ${d.agents_connected}`);
        if (d.uptime_secs != null) term.writeln(`  Uptime: ${d.uptime_secs}s`);
      } catch {
        term.writeln("\x1b[31mFailed to fetch status\x1b[0m");
      }
      break;
    case "agents":
      try {
        const r = await fetch("/api/agents");
        const agents = await r.json();
        if (agents.length === 0) {
          term.writeln("\x1b[33mNo agents registered\x1b[0m");
          break;
        }
        for (const a of agents) {
          const color = a.status === "connected" ? "32" : "31";
          term.writeln(
            `  \x1b[${color}m${a.status}\x1b[0m  ${a.name} (${a.agent_type}) v${a.version ?? "?"}`,
          );
        }
      } catch {
        term.writeln("\x1b[31mFailed to fetch agents\x1b[0m");
      }
      break;
    case "sessions":
      try {
        const r = await fetch("/api/stats");
        const s = await r.json();
        term.writeln(`  Total sessions: ${s.total_sessions}`);
        term.writeln(`  Active: ${s.active_sessions}`);
        term.writeln(`  Total cost: $${(s.total_cost ?? 0).toFixed(2)}`);
      } catch {
        term.writeln("\x1b[31mFailed to fetch sessions\x1b[0m");
      }
      break;
    case "costs":
      try {
        const r = await fetch("/api/stats");
        const s = await r.json();
        term.writeln(
          `  Total cost: \x1b[33m$${(s.total_cost ?? 0).toFixed(2)}\x1b[0m`,
        );
        term.writeln(
          `  Today: \x1b[36m$${(s.total_cost_today ?? 0).toFixed(4)}\x1b[0m`,
        );
        term.writeln(`  Tokens: ${s.total_tokens}`);
        term.writeln(`  Models: ${s.models_used}`);
      } catch {
        term.writeln("\x1b[31mFailed to fetch costs\x1b[0m");
      }
      break;
    case "clear":
      term.writeln("\x1b[2J\x1b[H");
      break;
    case "version":
      term.writeln("Rimuru v0.1.0");
      break;
    default:
      term.writeln(`\x1b[31mUnknown command: ${trimmed}\x1b[0m`);
      term.writeln("Type \x1b[36mhelp\x1b[0m for available commands");
  }
}
