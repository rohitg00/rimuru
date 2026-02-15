import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import type { PtySessionInfo, LaunchRequest, PtyOutputPayload, PtyExitPayload } from "@/types/pty";

export interface ChatRoom {
  id: string;
  name: string;
  agents: ChatAgent[];
  created_at: string;
}

export interface ChatAgent {
  agent_type: string;
  name: string;
  role: string;
  session_id: string | null;
}

export interface ChatMessage {
  id: string;
  room_id: string;
  sender: string;
  content: string;
  timestamp: string;
  message_type: string;
}

export interface Agent {
  id: string;
  name: string;
  agent_type: string;
  config_path: string | null;
  created_at: string;
  updated_at: string;
  last_active?: string;
}

export interface AgentWithStatus {
  agent: Agent;
  is_active: boolean;
  session_count: number;
  total_cost: number;
}

export interface AddAgentRequest {
  name: string;
  agent_type: string;
  config?: Record<string, unknown>;
}

export interface Session {
  id: string;
  agent_id: string;
  agent_name?: string;
  status: string;
  started_at: string;
  ended_at?: string;
  duration_secs?: number;
  total_tokens: number;
  total_cost: number;
}

export interface SessionFilters {
  agent_id?: string;
  status?: string;
  from_date?: string;
  to_date?: string;
  limit?: number;
  offset?: number;
}

export interface CostSummary {
  total_cost: number;
  total_tokens: number;
  input_tokens: number;
  output_tokens: number;
  session_count: number;
  period: string;
}

export interface CostBreakdownItem {
  name: string;
  cost: number;
  tokens: number;
  percentage: number;
}

export interface CostBreakdown {
  by_agent: CostBreakdownItem[];
  by_model: CostBreakdownItem[];
}

export interface CostHistoryPoint {
  date: string;
  cost: number;
  tokens: number;
}

export interface TimeRangeRequest {
  range: string;
  from_date?: string;
  to_date?: string;
}

export interface SystemMetrics {
  cpu_usage: number;
  memory_used_mb: number;
  memory_total_mb: number;
  memory_usage_percent: number;
  active_sessions: number;
  timestamp: string;
}

export interface MetricsHistoryPoint {
  timestamp: string;
  cpu_usage: number;
  memory_usage_percent: number;
  active_sessions: number;
}

export interface SyncStatus {
  last_sync?: string;
  is_syncing: boolean;
  provider_statuses: ProviderStatus[];
}

export interface ProviderStatus {
  name: string;
  enabled: boolean;
  last_sync?: string;
  model_count: number;
  error?: string;
}

export interface SyncResult {
  success: boolean;
  models_synced: number;
  providers_synced: string[];
  errors: string[];
}

export interface MetricsUpdatePayload {
  cpu_usage: number;
  memory_usage_percent: number;
  memory_used_mb: number;
  active_sessions: number;
  timestamp: string;
}

export interface Skill {
  id: string;
  name: string;
  slug: string;
  description: string;
  author: string;
  version: string;
  tags: string[];
  source: string;
  downloads: number;
  created_at: string;
  updated_at: string;
}

export interface InstalledSkill extends Skill {
  installed_at: string;
  enabled: boolean;
  agents: string[];
  install_path: string;
}

export interface SkillSearchResult {
  skills: Skill[];
  total: number;
  page: number;
  per_page: number;
}

export interface SkillRecommendation {
  skill: Skill;
  confidence: number;
  reason: string;
}

export interface SkillSearchFilters {
  query?: string;
  agent?: string;
  tags?: string[];
  limit?: number;
  page?: number;
}

export interface SkillInstallRequest {
  skill_id: string;
  agents: string[];
  install_all?: boolean;
}

export interface SkillTranslateRequest {
  skill_id: string;
  from_agent: string;
  to_agent: string;
}

export interface TranslationResult {
  success: boolean;
  original_agent: string;
  target_agent: string;
  output_path: string;
  warnings: string[];
}

export const SKILLKIT_AGENTS = [
  { value: "claude-code", label: "Claude Code", icon: "⟁" },
  { value: "cursor", label: "Cursor", icon: "◫" },
  { value: "codex", label: "Codex", icon: "◎" },
  { value: "gemini-cli", label: "Gemini CLI", icon: "✦" },
  { value: "opencode", label: "OpenCode", icon: "◇" },
  { value: "github-copilot", label: "GitHub Copilot", icon: "◈" },
  { value: "goose", label: "Goose", icon: "⬡" },
  { value: "cline", label: "Cline", icon: "◉" },
  { value: "roo", label: "Roo", icon: "◆" },
  { value: "windsurf", label: "Windsurf", icon: "◊" },
  { value: "kilo", label: "Kilo", icon: "▣" },
  { value: "amp", label: "Amp", icon: "▲" },
  { value: "universal", label: "Universal", icon: "●" },
] as const;

export type SkillKitAgent = typeof SKILLKIT_AGENTS[number]["value"];

export interface DirEntry {
  name: string;
  is_dir: boolean;
  size: number;
  modified: number;
}

export interface DirectoryStats {
  file_count: number;
  folder_count: number;
  total_size: number;
}

export interface GitInfo {
  branch: string;
  is_clean: boolean;
  remote_url: string | null;
  status_summary: string;
}

export interface SessionEventPayload {
  session_id: string;
  agent_id: string;
  event_type: string;
  timestamp: string;
}

export interface CostRecordedPayload {
  session_id: string;
  model: string;
  cost: number;
  tokens: number;
  timestamp: string;
}

export interface Plugin {
  id: string;
  name: string;
  version: string;
  author: string;
  description: string;
  capabilities: string[];
  status: string;
  enabled: boolean;
  homepage?: string;
  repository?: string;
  license?: string;
  loaded_at?: string;
  error?: string;
  is_builtin: boolean;
}

export interface PluginConfig {
  plugin_id: string;
  enabled: boolean;
  settings: Record<string, unknown>;
  priority: number;
  schema?: Record<string, unknown>;
}

export interface PluginEvent {
  event: string;
  plugin_id: string;
  timestamp: string;
  error?: string;
}

export interface InstallPluginRequest {
  source: string;
  auto_enable: boolean;
}

export interface ConfigurePluginRequest {
  plugin_id: string;
  key: string;
  value: unknown;
}

export interface HookType {
  name: string;
  description: string;
  data_type: string;
  handler_count: number;
  enabled: boolean;
  last_triggered?: string;
}

export interface HookHandler {
  id: string;
  name: string;
  hook_type: string;
  priority: number;
  enabled: boolean;
  plugin_id?: string;
  description?: string;
}

export interface HookExecution {
  id: string;
  hook_type: string;
  handler_name: string;
  status: string;
  started_at: string;
  completed_at?: string;
  duration_ms?: number;
  error?: string;
}

export interface TriggerHookRequest {
  hook_name: string;
  data?: unknown;
  source?: string;
}

export interface TriggerHookResponse {
  success: boolean;
  handlers_executed: number;
  aborted: boolean;
  abort_reason?: string;
  execution_id: string;
}

export interface HookStats {
  total_hook_types: number;
  active_handlers: number;
  total_handlers: number;
  total_executions: number;
  successful_executions: number;
  failed_executions: number;
  aborted_executions: number;
}

export interface AppSettings {
  sync_interval: string;
  default_model: string;
  session_timeout: string;
}

export interface DbStats {
  db_size_bytes: number;
  db_size_display: string;
  total_sessions: number;
  total_agents: number;
  db_path: string;
}

export const commands = {
  async getAgents(): Promise<AgentWithStatus[]> {
    return invoke("get_agents");
  },

  async getAgentDetails(agentId: string): Promise<AgentWithStatus | null> {
    return invoke("get_agent_details", { agentId });
  },

  async scanAgents(): Promise<Agent[]> {
    return invoke("scan_agents");
  },

  async addAgent(request: AddAgentRequest): Promise<AgentWithStatus> {
    return invoke("add_agent", { request });
  },

  async getSessions(filters?: SessionFilters): Promise<Session[]> {
    return invoke("get_sessions", { filters });
  },

  async getSessionDetails(sessionId: string): Promise<Session | null> {
    return invoke("get_session_details", { sessionId });
  },

  async getActiveSessions(): Promise<Session[]> {
    return invoke("get_active_sessions");
  },

  async getCostSummary(timeRange?: TimeRangeRequest): Promise<CostSummary> {
    return invoke("get_cost_summary", { timeRange });
  },

  async getCostBreakdown(timeRange?: TimeRangeRequest): Promise<CostBreakdown> {
    return invoke("get_cost_breakdown", { timeRange });
  },

  async getCostHistory(days?: number): Promise<CostHistoryPoint[]> {
    return invoke("get_cost_history", { days });
  },

  async getSystemMetrics(): Promise<SystemMetrics> {
    return invoke("get_system_metrics");
  },

  async getMetricsHistory(hours?: number): Promise<MetricsHistoryPoint[]> {
    return invoke("get_metrics_history", { hours });
  },

  async triggerSync(): Promise<SyncResult> {
    return invoke("trigger_sync");
  },

  async getSyncStatus(): Promise<SyncStatus> {
    return invoke("get_sync_status");
  },

  async searchSkills(filters?: SkillSearchFilters): Promise<SkillSearchResult> {
    return invoke("search_skills", { filters });
  },

  async getInstalledSkills(agent?: string): Promise<InstalledSkill[]> {
    return invoke("get_installed_skills", { agent });
  },

  async getSkillDetails(skillId: string): Promise<Skill | null> {
    return invoke("get_skill_details", { skillId });
  },

  async installSkill(request: SkillInstallRequest): Promise<InstalledSkill> {
    return invoke("install_skill", { request });
  },

  async uninstallSkill(skillId: string, agent?: string): Promise<boolean> {
    return invoke("uninstall_skill", { skillId, agent });
  },

  async translateSkill(request: SkillTranslateRequest): Promise<TranslationResult> {
    return invoke("translate_skill", { request });
  },

  async getSkillRecommendations(workflow?: string): Promise<SkillRecommendation[]> {
    return invoke("get_skill_recommendations", { workflow });
  },

  async enableSkill(skillId: string, agent?: string): Promise<boolean> {
    return invoke("enable_skill", { skillId, agent });
  },

  async disableSkill(skillId: string, agent?: string): Promise<boolean> {
    return invoke("disable_skill", { skillId, agent });
  },

  async getPlugins(showAvailable?: boolean, capability?: string): Promise<Plugin[]> {
    return invoke("get_plugins", { showAvailable, capability });
  },

  async getPluginDetails(pluginId: string): Promise<Plugin | null> {
    return invoke("get_plugin_details", { pluginId });
  },

  async installPlugin(request: InstallPluginRequest): Promise<Plugin> {
    return invoke("install_plugin", { request });
  },

  async enablePlugin(pluginId: string): Promise<boolean> {
    return invoke("enable_plugin", { pluginId });
  },

  async disablePlugin(pluginId: string): Promise<boolean> {
    return invoke("disable_plugin", { pluginId });
  },

  async uninstallPlugin(pluginId: string, force?: boolean): Promise<boolean> {
    return invoke("uninstall_plugin", { pluginId, force });
  },

  async getPluginConfig(pluginId: string): Promise<PluginConfig> {
    return invoke("get_plugin_config", { pluginId });
  },

  async configurePlugin(request: ConfigurePluginRequest): Promise<boolean> {
    return invoke("configure_plugin", { request });
  },

  async getPluginEvents(pluginId?: string, limit?: number): Promise<PluginEvent[]> {
    return invoke("get_plugin_events", { pluginId, limit });
  },

  async getHooks(): Promise<HookType[]> {
    return invoke("get_hooks");
  },

  async getHookHandlers(hookType?: string): Promise<HookHandler[]> {
    return invoke("get_hook_handlers", { hookType });
  },

  async getHookExecutions(hookType?: string, limit?: number): Promise<HookExecution[]> {
    return invoke("get_hook_executions", { hookType, limit });
  },

  async triggerHook(request: TriggerHookRequest): Promise<TriggerHookResponse> {
    return invoke("trigger_hook", { request });
  },

  async enableHookHandler(handlerId: string): Promise<boolean> {
    return invoke("enable_hook_handler", { handlerId });
  },

  async disableHookHandler(handlerId: string): Promise<boolean> {
    return invoke("disable_hook_handler", { handlerId });
  },

  async getHookStats(): Promise<HookStats> {
    return invoke("get_hook_stats");
  },

  async getSettings(): Promise<AppSettings> {
    return invoke("get_settings");
  },

  async saveSettings(settings: AppSettings): Promise<boolean> {
    return invoke("save_settings", { settings });
  },

  async getDbStats(): Promise<DbStats> {
    return invoke("get_db_stats");
  },

  async exportSessions(format: string, filters?: SessionFilters): Promise<string> {
    return invoke("export_sessions", { format, filters });
  },

  async exportCosts(format: string, timeRange?: TimeRangeRequest): Promise<string> {
    return invoke("export_costs", { format, timeRange });
  },

  async launchSession(request: LaunchRequest): Promise<string> {
    return invoke("launch_session", { request });
  },

  async writeToSession(sessionId: string, dataBase64: string): Promise<void> {
    return invoke("write_to_session", { sessionId, dataBase64 });
  },

  async resizeSession(sessionId: string, cols: number, rows: number): Promise<void> {
    return invoke("resize_session", { sessionId, cols, rows });
  },

  async terminateSession(sessionId: string): Promise<void> {
    return invoke("terminate_session", { sessionId });
  },

  async listLiveSessions(): Promise<PtySessionInfo[]> {
    return invoke("list_live_sessions");
  },

  async getLiveSession(sessionId: string): Promise<PtySessionInfo | null> {
    return invoke("get_live_session", { sessionId });
  },

  async createWorktree(repoPath: string, branchName: string): Promise<string> {
    return invoke("create_git_worktree", { repoPath, branchName });
  },

  async cleanupWorktree(repoPath: string, worktreePath: string): Promise<void> {
    return invoke("cleanup_git_worktree", { repoPath, worktreePath });
  },

  async listWorktrees(repoPath: string): Promise<Array<{ path: string; branch: string; head: string }>> {
    return invoke("list_git_worktrees", { repoPath });
  },

  async discoverSessions(): Promise<Array<{
    provider: string;
    project_name: string;
    project_path: string;
    last_active: string | null;
    session_count: number;
  }>> {
    return invoke("discover_agent_sessions");
  },

  async listPlaybooks(): Promise<Array<{
    id: string;
    name: string;
    description: string;
    steps: Array<{
      name: string;
      prompt: string;
      agent_type: string;
      working_dir: string | null;
      gate: string;
      timeout_secs: number | null;
    }>;
    file_path: string;
  }>> {
    return invoke("list_playbooks");
  },

  async loadPlaybook(path: string): Promise<{
    id: string;
    name: string;
    description: string;
    steps: Array<{
      name: string;
      prompt: string;
      agent_type: string;
      working_dir: string | null;
      gate: string;
      timeout_secs: number | null;
    }>;
    file_path: string;
  }> {
    return invoke("load_playbook", { path });
  },

  async createChatRoom(name: string, agents: Array<{ agent_type: string; name: string; role: string }>): Promise<ChatRoom> {
    return invoke("create_chat_room", { name, agents });
  },

  async sendChatMessage(roomId: string, content: string): Promise<ChatMessage> {
    return invoke("send_chat_message", { roomId, content });
  },

  async getChatMessages(roomId: string): Promise<ChatMessage[]> {
    return invoke("get_chat_messages", { roomId });
  },

  async listChatRooms(): Promise<ChatRoom[]> {
    return invoke("list_chat_rooms");
  },

  async closeChatRoom(roomId: string): Promise<void> {
    return invoke("close_chat_room", { roomId });
  },

  async startRemoteServer(port?: number): Promise<{ running: boolean; url: string | null; qr_svg: string | null }> {
    return invoke("start_remote_server", { port });
  },

  async stopRemoteServer(): Promise<void> {
    return invoke("stop_remote_server");
  },

  async getRemoteStatus(): Promise<{ running: boolean; url: string | null; qr_svg: string | null }> {
    return invoke("get_remote_status");
  },

  async readDirectory(path: string): Promise<DirEntry[]> {
    return invoke("read_directory", { path });
  },

  async getDirectoryStats(path: string): Promise<DirectoryStats> {
    return invoke("get_directory_stats", { path });
  },

  async getGitInfo(path: string): Promise<GitInfo> {
    return invoke("get_git_info", { path });
  },

  async readFilePreview(path: string, maxLines?: number): Promise<string> {
    return invoke("read_file_preview", { path, maxLines: maxLines ?? 50 });
  },
};

export const events = {
  onMetricsUpdate(callback: (payload: MetricsUpdatePayload) => void): Promise<UnlistenFn> {
    return listen<MetricsUpdatePayload>("metrics-update", (event) => callback(event.payload));
  },

  onSessionStarted(callback: (payload: SessionEventPayload) => void): Promise<UnlistenFn> {
    return listen<SessionEventPayload>("session-started", (event) => callback(event.payload));
  },

  onSessionEnded(callback: (payload: SessionEventPayload) => void): Promise<UnlistenFn> {
    return listen<SessionEventPayload>("session-ended", (event) => callback(event.payload));
  },

  onCostRecorded(callback: (payload: CostRecordedPayload) => void): Promise<UnlistenFn> {
    return listen<CostRecordedPayload>("cost-recorded", (event) => callback(event.payload));
  },

  onPtyOutput(sessionId: string, callback: (payload: PtyOutputPayload) => void): Promise<UnlistenFn> {
    return listen<PtyOutputPayload>(`pty-output-${sessionId}`, (event) => callback(event.payload));
  },

  onPtyExit(callback: (payload: PtyExitPayload) => void): Promise<UnlistenFn> {
    return listen<PtyExitPayload>("pty-exit", (event) => callback(event.payload));
  },
};
