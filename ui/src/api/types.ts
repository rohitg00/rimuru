export type AgentStatus =
  | "connected"
  | "disconnected"
  | "idle"
  | "busy"
  | "error";

export interface Agent {
  id: string;
  name: string;
  agent_type: string;
  status: AgentStatus;
  model?: string;
  version?: string;
  provider?: string;
  session_count: number;
  total_cost: number;
  total_tokens?: number;
  last_active?: string | null;
  last_seen?: string | null;
  created_at?: string;
  metadata: Record<string, unknown>;
}

export interface Session {
  id: string;
  agent_id: string;
  agent_name?: string;
  agent_type?: string;
  status: "active" | "completed" | "failed" | "paused";
  model: string;
  input_tokens: number;
  output_tokens: number;
  cache_read_tokens?: number;
  cache_write_tokens?: number;
  cost?: number;
  total_cost?: number;
  started_at: string;
  ended_at: string | null;
  duration_ms?: number;
  messages: number;
  metadata: Record<string, unknown>;
}

export interface CostRecord {
  id?: string;
  agent_id?: string;
  agent_name?: string;
  session_id?: string;
  model?: string;
  provider?: string;
  input_tokens: number;
  output_tokens: number;
  cache_read_tokens?: number;
  cache_write_tokens?: number;
  cost: number;
  total_cost?: number;
  timestamp: string;
  recorded_at?: string;
}

export interface DailyCost {
  date: string;
  cost: number;
  total_cost?: number;
  input_tokens?: number;
  output_tokens?: number;
  tokens?: number;
  sessions?: number;
  record_count?: number;
}

export interface ModelInfo {
  id: string;
  name: string;
  provider: string;
  input_price_per_million: number;
  output_price_per_million: number;
  cache_read_price_per_million: number;
  cache_write_price_per_million: number;
  max_output_tokens: number;
  context_window: number;
  supports_tools?: boolean;
  supports_vision?: boolean;
  last_synced?: string;
}

export interface SystemMetrics {
  cpu_percent?: number;
  memory_used_mb?: number;
  memory_total_mb?: number;
  heap_used_mb?: number;
  event_loop_lag_ms?: number;
  active_connections?: number;
  requests_per_second?: number;
  uptime_seconds?: number;
  total_cost_today?: number;
  avg_response_time_ms?: number;
  timestamp: string;
  metrics?: Omit<SystemMetrics, "metrics">;
}

export interface MetricsTimeline {
  timestamps: string[];
  cpu: number[];
  memory: number[];
  requests: number[];
  connections: number[];
}

export interface PluginManifest {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  enabled: boolean;
  installed: boolean;
  hooks: string[];
  config: Record<string, unknown>;
}

export interface HookConfig {
  id: string;
  name: string;
  event: string;
  plugin_id: string | null;
  enabled: boolean;
  script: string;
  matcher?: string;
  timeout_ms: number;
  last_run: string | null;
  last_status: "success" | "failure" | "timeout" | null;
  run_count: number;
  error_count: number;
}

export interface HookExecution {
  id: string;
  hook_id: string;
  hook_name: string;
  event: string;
  status: "success" | "failure" | "timeout";
  duration_ms: number;
  output: string | null;
  error: string | null;
  timestamp: string;
}

export interface AppConfig {
  api_port: number;
  theme: string;
  auto_detect_agents: boolean;
  auto_sync_models: boolean;
  budget_monthly: number;
  budget_alert_threshold: number;
  log_level: string;
  cost_tracking_enabled: boolean;
  enable_hooks: boolean;
  enable_plugins: boolean;
  metrics_collection_enabled: boolean;
  metrics_interval_secs: number;
  poll_interval_secs: number;
  model_sync_interval_hours: number;
  max_cost_history_days: number;
  max_session_history_days: number;
  max_metrics_entries: number;
  session_monitoring_enabled: boolean;
  currency: string;
}

export function formatAgentType(agentType: string): string {
  const MAP: Record<string, string> = {
    claude_code: "Claude Code",
    cursor: "Cursor",
    codex: "Codex",
    gemini_cli: "Gemini CLI",
    opencode: "OpenCode",
    windsurf: "Windsurf",
    copilot: "Copilot",
    goose: "Goose",
  };
  return (
    MAP[agentType] ??
    agentType.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase())
  );
}

export function formatProvider(provider: string): string {
  const MAP: Record<string, string> = {
    anthropic: "Anthropic",
    open_a_i: "OpenAI",
    openai: "OpenAI",
    google: "Google",
    openrouter: "OpenRouter",
    local: "Local",
  };
  return (
    MAP[provider] ??
    provider.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase())
  );
}

export function normalizeProvider(provider: string): string {
  const MAP: Record<string, string> = {
    open_a_i: "openai",
  };
  return MAP[provider] ?? provider;
}

export interface ActivityEvent {
  id: string;
  type:
    | "agent_connected"
    | "agent_disconnected"
    | "session_started"
    | "session_ended"
    | "cost_alert"
    | "plugin_installed"
    | "hook_triggered"
    | "error";
  message: string;
  agent_id: string | null;
  timestamp: string;
  metadata: Record<string, unknown>;
}

export interface StatsOverview {
  total_cost: number;
  total_cost_today: number;
  active_agents: number;
  total_agents: number;
  active_sessions: number;
  total_sessions: number;
  total_tokens: number;
  models_used: number;
  plugins_installed: number;
  hooks_active: number;
}

export interface McpServer {
  id: string;
  name: string;
  command: string;
  args: string[];
  env: Record<string, string>;
  enabled: boolean;
}

export interface StreamEvent {
  type: string;
  data: unknown;
  timestamp: string;
}
