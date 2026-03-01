use serde::Deserialize;
use chrono::{DateTime, NaiveDate, Utc};

#[derive(Debug, Clone, Deserialize)]
pub struct Agent {
    pub id: String,
    pub agent_type: String,
    pub name: String,
    pub status: String,
    pub version: Option<String>,
    pub config_path: Option<String>,
    pub connected_at: Option<DateTime<Utc>>,
    pub last_seen: Option<DateTime<Utc>>,
    pub session_count: u64,
    pub total_cost: f64,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Session {
    pub id: String,
    pub agent_id: String,
    pub agent_type: String,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub project_path: Option<String>,
    pub total_tokens: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_cost: f64,
    pub model: Option<String>,
    pub messages: u64,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CostSummary {
    pub total_cost: f64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_records: u64,
    #[serde(default)]
    pub by_agent: Vec<AgentCostSummary>,
    #[serde(default)]
    pub by_model: Vec<ModelCostSummary>,
    pub period_start: Option<DateTime<Utc>>,
    pub period_end: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentCostSummary {
    pub agent_type: String,
    pub total_cost: f64,
    pub total_tokens: u64,
    pub record_count: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelCostSummary {
    pub model: String,
    pub total_cost: f64,
    pub total_tokens: u64,
    pub record_count: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DailyCostSummary {
    pub date: NaiveDate,
    pub total_cost: f64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub record_count: u64,
    #[serde(default)]
    pub by_agent: Vec<AgentCostSummary>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub input_price_per_million: f64,
    pub output_price_per_million: f64,
    pub cache_read_price_per_million: Option<f64>,
    pub cache_write_price_per_million: Option<f64>,
    pub context_window: u64,
    pub max_output_tokens: Option<u64>,
    pub supports_vision: bool,
    pub supports_tools: bool,
    pub last_synced: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: DateTime<Utc>,
    pub cpu_usage_percent: f64,
    pub memory_used_mb: f64,
    pub memory_total_mb: f64,
    pub active_agents: u32,
    pub active_sessions: u32,
    pub total_cost_today: f64,
    pub requests_per_minute: f64,
    pub avg_response_time_ms: f64,
    pub error_rate: f64,
    pub uptime_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MetricsHistory {
    pub entries: Vec<SystemMetrics>,
    pub interval_secs: u64,
    pub total_entries: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub language: String,
    pub binary_path: String,
    #[serde(default)]
    pub functions: Vec<String>,
    pub enabled: bool,
    pub installed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HookConfig {
    pub id: String,
    pub event_type: String,
    pub function_id: String,
    pub priority: i32,
    pub enabled: bool,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct McpServer {
    pub name: String,
    pub url: String,
    pub status: String,
    #[serde(default)]
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DashboardStats {
    #[serde(default)]
    pub active_agents: u32,
    #[serde(default)]
    pub total_agents: u32,
    #[serde(default)]
    pub total_sessions: u64,
    #[serde(default)]
    pub total_cost: f64,
    #[serde(default)]
    pub total_cost_today: f64,
    #[serde(default)]
    pub total_tokens: u64,
    #[serde(default)]
    pub active_sessions: u32,
    #[serde(default)]
    pub hooks_active: u32,
    #[serde(default)]
    pub models_used: u32,
    #[serde(default)]
    pub plugins_installed: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActivityEvent {
    #[serde(default)]
    pub event_type: String,
    #[serde(default)]
    pub description: String,
    pub timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    #[serde(default)]
    pub uptime_secs: u64,
}

use crate::app::Tab;

pub struct ApiClient {
    client: reqwest::Client,
    base_url: String,
}

pub struct RefreshResult {
    pub connected: bool,
    pub error: Option<String>,
    pub health: Option<HealthStatus>,
    pub stats: Option<DashboardStats>,
    pub activity: Option<Vec<ActivityEvent>>,
    pub metrics: Option<SystemMetrics>,
    pub metrics_history: Option<MetricsHistory>,
    pub agents: Option<Vec<Agent>>,
    pub sessions: Option<Vec<Session>>,
    pub cost_summary: Option<CostSummary>,
    pub daily_costs: Option<Vec<DailyCostSummary>>,
    pub models: Option<Vec<ModelInfo>>,
    pub plugins: Option<Vec<PluginManifest>>,
    pub hooks: Option<Vec<HookConfig>>,
    pub mcp_servers: Option<Vec<McpServer>>,
}

impl ApiClient {
    pub fn new(port: u16) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .connect_timeout(std::time::Duration::from_millis(500))
            .build()
            .unwrap_or_default();
        Self {
            client,
            base_url: format!("http://localhost:{}", port),
        }
    }

    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, String> {
        self.client
            .get(format!("{}{}", self.base_url, path))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<T>()
            .await
            .map_err(|e| e.to_string())
    }

    async fn post_empty(&self, path: &str) -> Result<serde_json::Value, String> {
        self.client
            .post(format!("{}{}", self.base_url, path))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn get_health(&self) -> Result<HealthStatus, String> {
        self.get("/api/health").await
    }

    pub async fn get_stats(&self) -> Result<DashboardStats, String> {
        self.get("/api/stats").await
    }

    pub async fn get_activity(&self) -> Result<Vec<ActivityEvent>, String> {
        self.get("/api/activity/recent").await
    }

    pub async fn get_agents(&self) -> Result<Vec<Agent>, String> {
        self.get("/api/agents").await
    }

    pub async fn connect_agent(&self, id: &str) -> Result<Agent, String> {
        let resp = self
            .client
            .post(format!("{}/api/agents/{}/connect", self.base_url, id))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        resp.json().await.map_err(|e| e.to_string())
    }

    pub async fn disconnect_agent(&self, id: &str) -> Result<Agent, String> {
        let resp = self
            .client
            .post(format!("{}/api/agents/{}/disconnect", self.base_url, id))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        resp.json().await.map_err(|e| e.to_string())
    }

    pub async fn detect_agents(&self) -> Result<serde_json::Value, String> {
        self.get("/api/agents/detect").await
    }

    pub async fn get_sessions(&self) -> Result<Vec<Session>, String> {
        self.get("/api/sessions").await
    }

    pub async fn get_active_sessions(&self) -> Result<Vec<Session>, String> {
        self.get("/api/sessions/active").await
    }

    pub async fn get_costs_summary(&self) -> Result<CostSummary, String> {
        self.get("/api/costs/summary").await
    }

    pub async fn get_costs_daily(&self) -> Result<Vec<DailyCostSummary>, String> {
        self.get("/api/costs/daily").await
    }

    pub async fn get_models(&self) -> Result<Vec<ModelInfo>, String> {
        self.get("/api/models").await
    }

    pub async fn sync_models(&self) -> Result<serde_json::Value, String> {
        self.post_empty("/api/models/sync").await
    }

    pub async fn get_metrics(&self) -> Result<SystemMetrics, String> {
        self.get("/api/metrics").await
    }

    pub async fn get_metrics_history(&self) -> Result<MetricsHistory, String> {
        self.get("/api/metrics/history").await
    }

    pub async fn get_plugins(&self) -> Result<Vec<PluginManifest>, String> {
        self.get("/api/plugins").await
    }

    pub async fn toggle_plugin(&self, id: &str, action: &str) -> Result<serde_json::Value, String> {
        self.post_empty(&format!("/api/plugins/{}/{}", id, action)).await
    }

    pub async fn get_hooks(&self) -> Result<Vec<HookConfig>, String> {
        self.get("/api/hooks").await
    }

    pub async fn get_mcp_servers(&self) -> Result<Vec<McpServer>, String> {
        self.get("/api/mcp").await
    }

    pub async fn refresh_for_tab(&self, tab: Tab) -> RefreshResult {
        let mut r = RefreshResult {
            connected: false,
            error: None,
            health: None,
            stats: None,
            activity: None,
            metrics: None,
            metrics_history: None,
            agents: None,
            sessions: None,
            cost_summary: None,
            daily_costs: None,
            models: None,
            plugins: None,
            hooks: None,
            mcp_servers: None,
        };

        match self.get_health().await {
            Ok(h) => {
                r.connected = true;
                r.health = Some(h);
            }
            Err(e) => {
                r.error = Some(e);
                return r;
            }
        }

        match tab {
            Tab::Dashboard => {
                r.stats = self.get_stats().await.ok();
                r.activity = self.get_activity().await.ok();
                r.metrics = self.get_metrics().await.ok();
                r.agents = self.get_agents().await.ok();
                r.daily_costs = self.get_costs_daily().await.ok();
            }
            Tab::Agents => {
                r.agents = self.get_agents().await.ok();
            }
            Tab::Sessions => {
                r.sessions = self.get_sessions().await.ok();
            }
            Tab::Costs => {
                r.cost_summary = self.get_costs_summary().await.ok();
                r.daily_costs = self.get_costs_daily().await.ok();
            }
            Tab::Models => {
                r.models = self.get_models().await.ok();
            }
            Tab::Metrics => {
                r.metrics = self.get_metrics().await.ok();
                r.metrics_history = self.get_metrics_history().await.ok();
            }
            Tab::Plugins => {
                r.plugins = self.get_plugins().await.ok();
            }
            Tab::Hooks => {
                r.hooks = self.get_hooks().await.ok();
            }
            Tab::Mcp => {
                r.mcp_servers = self.get_mcp_servers().await.ok();
            }
            Tab::Help => {}
        }

        r
    }
}
