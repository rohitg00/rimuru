import { useState, useEffect, useCallback, useRef } from "react";
import { useSearchParams, useNavigate } from "react-router-dom";
import { Plus, Terminal as TerminalIcon, Keyboard, Wifi, GitBranch } from "lucide-react";
import { commands } from "@/lib/tauri";
import { useAgents } from "@/hooks/useAgents";
import type { PtySessionInfo, LaunchRequest } from "@/types/pty";
import type { GitInfo } from "@/lib/tauri";
import type { TerminalSearchHandle } from "@/components/Terminal/TerminalPane";
import TerminalPane from "@/components/Terminal/TerminalPane";
import TerminalTabs from "@/components/Terminal/TerminalTabs";
import LaunchWizard from "@/components/LaunchWizard/LaunchWizard";
import PromptInput from "@/components/Terminal/PromptInput";
import { RightPanel } from "@/components/RightPanel/RightPanel";
import BottomBar from "@/components/BottomBar/BottomBar";
import CommandPalette from "@/components/CommandPalette/CommandPalette";
import SearchOverlay from "@/components/Terminal/SearchOverlay";
import AIConversationView from "@/components/Terminal/AIConversationView";
import DiscoveredSessions from "@/components/Terminal/DiscoveredSessions";
import RemoteControlModal from "@/components/RemoteControl/RemoteControlModal";
import styles from "./Orchestrate.module.css";

const STATUS_STYLES: Record<string, string> = {
  Running: styles.statusRunning,
  Completed: styles.statusCompleted,
  Failed: styles.statusFailed,
};

export default function Orchestrate() {
  const [searchParams, setSearchParams] = useSearchParams();
  const navigate = useNavigate();
  const { data: agents } = useAgents();
  const [sessions, setSessions] = useState<PtySessionInfo[]>([]);
  const [activeSessionId, setActiveSessionId] = useState<string | null>(null);
  const [showLaunchWizard, setShowLaunchWizard] = useState(false);
  const [sidebarVisible, setSidebarVisible] = useState(true);
  const [_infoPanelCollapsed, setInfoPanelCollapsed] = useState(false);
  const [inputMode, setInputMode] = useState<"smart" | "raw">("raw");
  const [showPalette, setShowPalette] = useState(false);
  const [showSearch, setShowSearch] = useState(false);
  const [viewMode, setViewMode] = useState<"terminal" | "conversation">("terminal");
  const [showRemote, setShowRemote] = useState(false);
  const [rightPanelVisible, setRightPanelVisible] = useState(true);
  const [rightPanelTab, setRightPanelTab] = useState<"files" | "history" | "autorun">("files");
  const [gitInfo, setGitInfo] = useState<GitInfo | null>(null);
  const terminalRefs = useRef<Map<string, TerminalSearchHandle>>(new Map());

  const preSelectedAgent = searchParams.get("agent") ?? undefined;
  const activeSession = sessions.find((s) => s.id === activeSessionId) ?? null;

  useEffect(() => {
    if (preSelectedAgent) {
      setShowLaunchWizard(true);
      setSearchParams({}, { replace: true });
    }
  }, [preSelectedAgent, setSearchParams]);

  const fetchSessions = useCallback(async () => {
    try {
      const live = await commands.listLiveSessions();
      setSessions(live);
    } catch {
      // backend may not be ready
    }
  }, []);

  useEffect(() => {
    fetchSessions();
    const interval = setInterval(fetchSessions, 2000);
    return () => clearInterval(interval);
  }, [fetchSessions]);

  const handleLaunch = async (request: LaunchRequest) => {
    try {
      const sessionId = await commands.launchSession(request);
      setActiveSessionId(sessionId);
      await fetchSessions();
    } catch (err) {
      console.error("Failed to launch session:", err);
    }
  };

  const handleCloseSession = async (sessionId: string) => {
    try {
      await commands.terminateSession(sessionId);
      await fetchSessions();
      if (activeSessionId === sessionId) {
        const remaining = sessions.filter((s) => s.id !== sessionId);
        setActiveSessionId(remaining.length > 0 ? remaining[0].id : null);
      }
    } catch (err) {
      console.error("Failed to terminate session:", err);
    }
  };

  const handleExit = useCallback(
    (_exitCode: number | null, _success: boolean) => {
      fetchSessions();
    },
    [fetchSessions]
  );

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (!e.metaKey) return;

      switch (e.key) {
        case "k":
          e.preventDefault();
          setShowPalette(true);
          break;
        case "f":
          e.preventDefault();
          setShowSearch((v) => !v);
          break;
        case "j":
          e.preventDefault();
          setViewMode((v) => (v === "terminal" ? "conversation" : "terminal"));
          break;
        case "n":
          e.preventDefault();
          setShowLaunchWizard(true);
          break;
        case "w":
          e.preventDefault();
          if (activeSessionId) handleCloseSession(activeSessionId);
          break;
        case "[": {
          e.preventDefault();
          if (sessions.length < 2 || !activeSessionId) break;
          const idx = sessions.findIndex((s) => s.id === activeSessionId);
          const prev = idx > 0 ? idx - 1 : sessions.length - 1;
          setActiveSessionId(sessions[prev].id);
          break;
        }
        case "]": {
          e.preventDefault();
          if (sessions.length < 2 || !activeSessionId) break;
          const idx = sessions.findIndex((s) => s.id === activeSessionId);
          const next = idx < sessions.length - 1 ? idx + 1 : 0;
          setActiveSessionId(sessions[next].id);
          break;
        }
        case "e":
          e.preventDefault();
          setRightPanelVisible((v) => !v);
          break;
        case "b":
          e.preventDefault();
          setSidebarVisible((v) => !v);
          break;
        case "i":
          e.preventDefault();
          setInfoPanelCollapsed((v) => !v);
          break;
        case "T":
          if (e.shiftKey && activeSessionId) {
            e.preventDefault();
            commands.terminateSession(activeSessionId);
            fetchSessions();
          }
          break;
      }
    };

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [activeSessionId, sessions, fetchSessions]);

  useEffect(() => {
    if (!activeSession?.working_dir) {
      setGitInfo(null);
      return;
    }
    commands.getGitInfo(activeSession.working_dir).then(setGitInfo).catch(() => setGitInfo(null));
  }, [activeSession?.working_dir]);

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <h1 className={styles.title}>Orchestrate</h1>
        {gitInfo && (
          <div className={styles.branchPill}>
            <GitBranch size={14} />
            <span>{gitInfo.branch}</span>
            <span className={gitInfo.is_clean ? styles.branchClean : styles.branchDirty}>
              {gitInfo.is_clean ? "Clean" : "Modified"}
            </span>
          </div>
        )}
        <div style={{ display: "flex", gap: 8 }}>
          <button
            className="btn btn-secondary"
            onClick={() => setShowRemote(true)}
          >
            <Wifi size={16} />
            Remote
          </button>
          <button
            className="btn btn-primary"
            onClick={() => setShowLaunchWizard(true)}
          >
            <Plus size={16} />
            New Session
          </button>
        </div>
      </div>

      {sessions.length === 0 ? (
        <div className={styles.empty}>
          <TerminalIcon size={48} strokeWidth={1} />
          <p className={styles.emptyTitle}>No active sessions</p>
          <p className={styles.emptyDesc}>
            Launch an agent session to start orchestrating
          </p>
          <button
            className="btn btn-primary"
            onClick={() => setShowLaunchWizard(true)}
          >
            <Plus size={16} />
            Launch Session
          </button>
          <div className={styles.shortcutHints}>
            <Keyboard size={14} />
            <span>
              <kbd>&#8984;N</kbd> New &middot; <kbd>&#8984;B</kbd> Sidebar &middot; <kbd>&#8984;[</kbd><kbd>&#8984;]</kbd> Switch
            </span>
          </div>
        </div>
      ) : (
        <div className={styles.workspace}>
          {sidebarVisible && (
            <div className={styles.sessionList}>
              <DiscoveredSessions
                onLaunchFromDiscovered={(projectPath, agentType) => {
                  handleLaunch({
                    agent_type: agentType,
                    working_dir: projectPath,
                    cols: 120,
                    rows: 30,
                  });
                }}
              />
              {sessions.map((session) => (
                <div
                  key={session.id}
                  className={`${styles.sessionItem} ${
                    session.id === activeSessionId ? styles.sessionActive : ""
                  }`}
                  onClick={() => setActiveSessionId(session.id)}
                  role="button"
                  tabIndex={0}
                >
                  <div className={styles.sessionTop}>
                    <span className={styles.sessionName}>
                      {session.agent_name}
                    </span>
                    <span
                      className={`${styles.statusBadge} ${STATUS_STYLES[session.status] ?? ""}`}
                    >
                      {session.status}
                    </span>
                  </div>
                  <div className={styles.sessionMeta}>
                    <span>${session.cumulative_cost_usd.toFixed(4)}</span>
                    <span>{session.token_count.toLocaleString()} tokens</span>
                  </div>
                </div>
              ))}
            </div>
          )}

          <div className={styles.terminalArea}>
            <TerminalTabs
              sessions={sessions}
              activeSessionId={activeSessionId ?? ""}
              onSelectSession={setActiveSessionId}
              onCloseSession={handleCloseSession}
            />
            {showSearch && activeSessionId && (
              <SearchOverlay
                searchRef={{ current: terminalRefs.current.get(activeSessionId) ?? null }}
                onClose={() => setShowSearch(false)}
              />
            )}
            <div className={styles.terminalContainer}>
              {sessions.map((session) => (
                <div
                  key={session.id}
                  className={styles.terminalWrapper}
                  style={{
                    display:
                      session.id === activeSessionId ? "flex" : "none",
                  }}
                >
                  {viewMode === "terminal" ? (
                    <TerminalPane
                      ref={(handle) => {
                        if (handle) terminalRefs.current.set(session.id, handle);
                      }}
                      sessionId={session.id}
                      onExit={handleExit}
                    />
                  ) : (
                    <AIConversationView sessionId={session.id} />
                  )}
                </div>
              ))}
            </div>
            {activeSession && inputMode === "smart" && (
              <PromptInput
                sessionId={activeSession.id}
                disabled={activeSession.status !== "Running"}
                mode={inputMode}
                onModeChange={setInputMode}
              />
            )}
            <BottomBar
              onNewAgent={() => setShowLaunchWizard(true)}
              onWizard={() => setShowLaunchWizard(true)}
              viewMode={viewMode}
              onViewModeChange={setViewMode}
              inputMode={inputMode}
              onInputModeChange={setInputMode}
            />
          </div>

          <RightPanel
            visible={rightPanelVisible}
            activeTab={rightPanelTab}
            onTabChange={setRightPanelTab}
            sessionWorkingDir={activeSession?.working_dir}
            sessionId={activeSession?.id}
          />
        </div>
      )}

      <LaunchWizard
        isOpen={showLaunchWizard}
        onClose={() => setShowLaunchWizard(false)}
        onLaunch={handleLaunch}
        agents={agents ?? []}
        preSelectedAgent={preSelectedAgent}
      />

      <CommandPalette
        isOpen={showPalette}
        onClose={() => setShowPalette(false)}
        onNavigate={navigate}
        onLaunchAgent={() => setShowLaunchWizard(true)}
        onToggleSidebar={() => setSidebarVisible((v) => !v)}
        onToggleInfoPanel={() => setInfoPanelCollapsed((v) => !v)}
        sessions={sessions}
        onSwitchSession={setActiveSessionId}
        onTerminateSession={handleCloseSession}
      />

      <RemoteControlModal
        isOpen={showRemote}
        onClose={() => setShowRemote(false)}
      />
    </div>
  );
}
