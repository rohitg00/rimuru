use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use chrono::{Duration as ChronoDuration, Utc};
use sysinfo::{Disks, System};
use tokio::sync::RwLock;
use uuid::Uuid;

use super::state::{
    AgentCostSummary, AgentData, AppData, CostData, CostTrend, DailyCost, MetricsData,
    ModelCostSummary, SessionData,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadingState {
    Idle,
    Loading,
    Success,
    Error,
}

impl LoadingState {
    pub fn is_loading(&self) -> bool {
        matches!(self, LoadingState::Loading)
    }

    pub fn is_error(&self) -> bool {
        matches!(self, LoadingState::Error)
    }

    pub fn indicator(&self) -> &'static str {
        match self {
            LoadingState::Idle => "",
            LoadingState::Loading => "⟳",
            LoadingState::Success => "✓",
            LoadingState::Error => "✗",
        }
    }
}

/// Errors that can occur during data loading operations.
///
/// This integrates with the core RimuruError for proper error handling
/// while providing TUI-specific error formatting and recovery suggestions.
#[derive(Debug, Clone)]
pub enum DataLoadError {
    /// Database connection or query error
    DatabaseError(String),
    /// Network or connection error
    ConnectionError(String),
    /// Request timeout
    TimeoutError,
    /// Other errors
    Other(String),
}

impl DataLoadError {
    /// Create a DataLoadError from a RimuruError
    pub fn from_rimuru_error(err: &rimuru_core::RimuruError) -> Self {
        if err.is_database_error() {
            DataLoadError::DatabaseError(err.to_string())
        } else if err.is_sync_error() {
            if matches!(err, rimuru_core::RimuruError::SyncTimeout(_)) {
                DataLoadError::TimeoutError
            } else {
                DataLoadError::ConnectionError(err.to_string())
            }
        } else {
            DataLoadError::Other(err.to_string())
        }
    }

    /// Returns true if this error is transient and can be retried
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            DataLoadError::DatabaseError(_)
                | DataLoadError::ConnectionError(_)
                | DataLoadError::TimeoutError
        )
    }

    /// Returns a user-friendly recovery suggestion
    pub fn recovery_suggestion(&self) -> &'static str {
        match self {
            DataLoadError::DatabaseError(_) => {
                "Check database connection. Will retry automatically."
            }
            DataLoadError::ConnectionError(_) => {
                "Check network connection. Will retry automatically."
            }
            DataLoadError::TimeoutError => "Request timed out. Will retry with longer timeout.",
            DataLoadError::Other(_) => "An unexpected error occurred.",
        }
    }

    /// Returns a short status indicator for the TUI
    pub fn status_indicator(&self) -> &'static str {
        match self {
            DataLoadError::DatabaseError(_) => "DB Error",
            DataLoadError::ConnectionError(_) => "Offline",
            DataLoadError::TimeoutError => "Timeout",
            DataLoadError::Other(_) => "Error",
        }
    }
}

impl std::fmt::Display for DataLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataLoadError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            DataLoadError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            DataLoadError::TimeoutError => write!(f, "Request timed out"),
            DataLoadError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for DataLoadError {}

impl From<rimuru_core::RimuruError> for DataLoadError {
    fn from(err: rimuru_core::RimuruError) -> Self {
        DataLoadError::from_rimuru_error(&err)
    }
}

pub struct DataLoader {
    data: Arc<RwLock<AppData>>,
    loading_state: Arc<RwLock<LoadingState>>,
    last_error: Arc<RwLock<Option<DataLoadError>>>,
    last_fetch: Arc<RwLock<Option<Instant>>>,
    min_fetch_interval: Duration,
    system: Arc<RwLock<System>>,
}

impl DataLoader {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(AppData::new())),
            loading_state: Arc::new(RwLock::new(LoadingState::Idle)),
            last_error: Arc::new(RwLock::new(None)),
            last_fetch: Arc::new(RwLock::new(None)),
            min_fetch_interval: Duration::from_millis(500),
            system: Arc::new(RwLock::new(System::new_all())),
        }
    }

    pub fn with_min_fetch_interval(mut self, interval: Duration) -> Self {
        self.min_fetch_interval = interval;
        self
    }

    pub async fn loading_state(&self) -> LoadingState {
        *self.loading_state.read().await
    }

    pub async fn last_error(&self) -> Option<DataLoadError> {
        self.last_error.read().await.clone()
    }

    pub async fn data(&self) -> AppData {
        self.data.read().await.clone()
    }

    pub async fn should_refresh(&self) -> bool {
        let last_fetch = self.last_fetch.read().await;
        match *last_fetch {
            Some(instant) => instant.elapsed() >= self.min_fetch_interval,
            None => true,
        }
    }

    pub async fn refresh(&self) -> Result<(), DataLoadError> {
        if !self.should_refresh().await {
            return Ok(());
        }

        *self.loading_state.write().await = LoadingState::Loading;
        *self.last_error.write().await = None;

        let result = self.fetch_all_data().await;

        match result {
            Ok(new_data) => {
                let mut data = self.data.write().await;
                *data = new_data;
                *self.loading_state.write().await = LoadingState::Success;
                *self.last_fetch.write().await = Some(Instant::now());
                Ok(())
            }
            Err(e) => {
                *self.loading_state.write().await = LoadingState::Error;
                *self.last_error.write().await = Some(e.clone());
                Err(e)
            }
        }
    }

    pub async fn force_refresh(&self) -> Result<(), DataLoadError> {
        *self.last_fetch.write().await = None;
        self.refresh().await
    }

    async fn fetch_all_data(&self) -> Result<AppData, DataLoadError> {
        let agents = self.fetch_agents().await;
        let sessions = self.fetch_sessions().await;
        let costs = self.fetch_costs().await;
        let metrics = self.fetch_metrics().await;

        Ok(AppData {
            agents,
            sessions,
            costs,
            metrics,
            last_updated: Some(Utc::now()),
        })
    }

    async fn fetch_agents(&self) -> Vec<AgentData> {
        vec![
            AgentData {
                id: Uuid::new_v4(),
                name: "Claude Code".to_string(),
                agent_type: "claude_code".to_string(),
                is_connected: true,
                active_sessions: 2,
                tokens_today: 45_230,
                cost_today: 1.23,
                last_activity: Some(Utc::now()),
                created_at: Utc::now() - ChronoDuration::days(30),
            },
            AgentData {
                id: Uuid::new_v4(),
                name: "Cursor".to_string(),
                agent_type: "cursor".to_string(),
                is_connected: true,
                active_sessions: 0,
                tokens_today: 12_500,
                cost_today: 0.35,
                last_activity: Some(Utc::now() - ChronoDuration::hours(2)),
                created_at: Utc::now() - ChronoDuration::days(15),
            },
            AgentData {
                id: Uuid::new_v4(),
                name: "OpenCode".to_string(),
                agent_type: "open_code".to_string(),
                is_connected: true,
                active_sessions: 1,
                tokens_today: 8_750,
                cost_today: 0.28,
                last_activity: Some(Utc::now() - ChronoDuration::minutes(30)),
                created_at: Utc::now() - ChronoDuration::days(7),
            },
            AgentData {
                id: Uuid::new_v4(),
                name: "Codex".to_string(),
                agent_type: "codex".to_string(),
                is_connected: false,
                active_sessions: 0,
                tokens_today: 0,
                cost_today: 0.0,
                last_activity: Some(Utc::now() - ChronoDuration::days(2)),
                created_at: Utc::now() - ChronoDuration::days(60),
            },
            AgentData {
                id: Uuid::new_v4(),
                name: "Copilot".to_string(),
                agent_type: "copilot".to_string(),
                is_connected: true,
                active_sessions: 0,
                tokens_today: 3_200,
                cost_today: 0.08,
                last_activity: Some(Utc::now() - ChronoDuration::hours(5)),
                created_at: Utc::now() - ChronoDuration::days(90),
            },
            AgentData {
                id: Uuid::new_v4(),
                name: "Goose".to_string(),
                agent_type: "goose".to_string(),
                is_connected: false,
                active_sessions: 0,
                tokens_today: 0,
                cost_today: 0.0,
                last_activity: None,
                created_at: Utc::now() - ChronoDuration::days(5),
            },
        ]
    }

    async fn fetch_sessions(&self) -> Vec<SessionData> {
        let agent_id = Uuid::new_v4();
        vec![
            SessionData {
                id: Uuid::new_v4(),
                agent_id,
                agent_name: "Claude Code".to_string(),
                is_active: true,
                status: "active".to_string(),
                started_at: Utc::now() - ChronoDuration::minutes(45),
                ended_at: None,
                duration_seconds: None,
                input_tokens: 12_500,
                output_tokens: 8_300,
                cost: 0.52,
                model_name: Some("claude-opus-4-5".to_string()),
                metadata: serde_json::json!({"project": "rimuru-tui"}),
            },
            SessionData {
                id: Uuid::new_v4(),
                agent_id,
                agent_name: "Claude Code".to_string(),
                is_active: true,
                status: "active".to_string(),
                started_at: Utc::now() - ChronoDuration::minutes(15),
                ended_at: None,
                duration_seconds: None,
                input_tokens: 3_200,
                output_tokens: 1_800,
                cost: 0.12,
                model_name: Some("claude-sonnet-4".to_string()),
                metadata: serde_json::json!({"project": "rimuru-core"}),
            },
            SessionData {
                id: Uuid::new_v4(),
                agent_id: Uuid::new_v4(),
                agent_name: "OpenCode".to_string(),
                is_active: true,
                status: "active".to_string(),
                started_at: Utc::now() - ChronoDuration::minutes(30),
                ended_at: None,
                duration_seconds: None,
                input_tokens: 5_100,
                output_tokens: 3_650,
                cost: 0.28,
                model_name: Some("gpt-4-turbo".to_string()),
                metadata: serde_json::json!({"project": "api-server"}),
            },
            SessionData {
                id: Uuid::new_v4(),
                agent_id: Uuid::new_v4(),
                agent_name: "Cursor".to_string(),
                is_active: false,
                status: "completed".to_string(),
                started_at: Utc::now() - ChronoDuration::hours(3),
                ended_at: Some(Utc::now() - ChronoDuration::hours(2)),
                duration_seconds: Some(3600),
                input_tokens: 8_500,
                output_tokens: 4_000,
                cost: 0.35,
                model_name: Some("claude-sonnet-4".to_string()),
                metadata: serde_json::json!({"project": "frontend"}),
            },
            SessionData {
                id: Uuid::new_v4(),
                agent_id: Uuid::new_v4(),
                agent_name: "Claude Code".to_string(),
                is_active: false,
                status: "completed".to_string(),
                started_at: Utc::now() - ChronoDuration::hours(6),
                ended_at: Some(Utc::now() - ChronoDuration::hours(5)),
                duration_seconds: Some(4500),
                input_tokens: 15_200,
                output_tokens: 9_800,
                cost: 0.75,
                model_name: Some("claude-opus-4-5".to_string()),
                metadata: serde_json::json!({"project": "database"}),
            },
            SessionData {
                id: Uuid::new_v4(),
                agent_id: Uuid::new_v4(),
                agent_name: "Copilot".to_string(),
                is_active: false,
                status: "completed".to_string(),
                started_at: Utc::now() - ChronoDuration::hours(6),
                ended_at: Some(Utc::now() - ChronoDuration::hours(5)),
                duration_seconds: Some(2700),
                input_tokens: 2_100,
                output_tokens: 1_100,
                cost: 0.08,
                model_name: Some("gpt-4".to_string()),
                metadata: serde_json::json!({"project": "tests"}),
            },
            SessionData {
                id: Uuid::new_v4(),
                agent_id: Uuid::new_v4(),
                agent_name: "Claude Code".to_string(),
                is_active: false,
                status: "failed".to_string(),
                started_at: Utc::now() - ChronoDuration::hours(8),
                ended_at: Some(Utc::now() - ChronoDuration::hours(8)),
                duration_seconds: Some(180),
                input_tokens: 500,
                output_tokens: 100,
                cost: 0.02,
                model_name: Some("claude-sonnet-4".to_string()),
                metadata: serde_json::json!({"project": "ci-cd", "error": "context limit exceeded"}),
            },
        ]
    }

    async fn fetch_costs(&self) -> CostData {
        let daily_costs: Vec<DailyCost> = (0..7)
            .map(|i| DailyCost {
                date: Utc::now() - ChronoDuration::days(i),
                cost: match i {
                    0 => 1.94,
                    1 => 2.15,
                    2 => 1.78,
                    3 => 2.45,
                    4 => 1.92,
                    5 => 1.65,
                    6 => 2.08,
                    _ => 0.0,
                },
            })
            .collect();

        CostData {
            today_total: 1.94,
            week_total: 13.97,
            month_total: 58.42,
            all_time_total: 234.18,
            today_tokens: 69_680,
            today_sessions: 7,
            by_agent: vec![
                AgentCostSummary {
                    agent_id: Uuid::new_v4(),
                    agent_name: "Claude Code".to_string(),
                    cost: 1.41,
                    tokens: 50_230,
                    sessions: 4,
                    trend: CostTrend::Up,
                },
                AgentCostSummary {
                    agent_id: Uuid::new_v4(),
                    agent_name: "Cursor".to_string(),
                    cost: 0.35,
                    tokens: 12_500,
                    sessions: 1,
                    trend: CostTrend::Down,
                },
                AgentCostSummary {
                    agent_id: Uuid::new_v4(),
                    agent_name: "OpenCode".to_string(),
                    cost: 0.28,
                    tokens: 8_750,
                    sessions: 1,
                    trend: CostTrend::Stable,
                },
                AgentCostSummary {
                    agent_id: Uuid::new_v4(),
                    agent_name: "Copilot".to_string(),
                    cost: 0.08,
                    tokens: 3_200,
                    sessions: 1,
                    trend: CostTrend::Down,
                },
            ],
            by_model: vec![
                ModelCostSummary {
                    model_name: "claude-opus-4-5".to_string(),
                    cost: 1.27,
                    tokens: 45_900,
                    sessions: 3,
                },
                ModelCostSummary {
                    model_name: "claude-sonnet-4".to_string(),
                    cost: 0.49,
                    tokens: 14_600,
                    sessions: 3,
                },
                ModelCostSummary {
                    model_name: "gpt-4-turbo".to_string(),
                    cost: 0.28,
                    tokens: 8_750,
                    sessions: 1,
                },
                ModelCostSummary {
                    model_name: "gpt-4".to_string(),
                    cost: 0.08,
                    tokens: 3_200,
                    sessions: 1,
                },
            ],
            daily_costs,
        }
    }

    async fn fetch_metrics(&self) -> MetricsData {
        let mut system = self.system.write().await;
        system.refresh_all();

        let cpu_percent = system.global_cpu_usage();
        let memory_used_mb = system.used_memory() as i64 / 1024 / 1024;
        let memory_total_mb = system.total_memory() as i64 / 1024 / 1024;

        let disks = Disks::new_with_refreshed_list();
        let (disk_used, disk_total) = disks
            .iter()
            .filter(|d| d.mount_point().to_string_lossy().starts_with('/'))
            .fold((0u64, 0u64), |(used, total), disk| {
                (
                    used + disk.total_space() - disk.available_space(),
                    total + disk.total_space(),
                )
            });

        let disk_used_mb = disk_used as i64 / 1024 / 1024;
        let disk_total_mb = disk_total as i64 / 1024 / 1024;

        let cpu_history: Vec<f32> = (0..20)
            .map(|i| {
                let base = cpu_percent;
                let variation = (i as f32 * 0.5).sin() * 10.0;
                (base + variation).max(0.0).min(100.0)
            })
            .collect();

        let peak_cpu = cpu_history.iter().cloned().fold(0.0f32, f32::max);
        let average_cpu = cpu_history.iter().sum::<f32>() / cpu_history.len() as f32;

        MetricsData {
            cpu_percent,
            memory_used_mb,
            memory_total_mb,
            disk_used_mb,
            disk_total_mb,
            active_sessions: 3,
            cpu_history,
            memory_history: vec![memory_used_mb as f32 / memory_total_mb as f32 * 100.0; 20],
            peak_cpu,
            average_cpu,
        }
    }

    pub fn clear_error(&self) {
        if let Ok(mut error) = self.last_error.try_write() {
            *error = None;
        }
    }

    pub fn set_loading_idle(&self) {
        if let Ok(mut state) = self.loading_state.try_write() {
            *state = LoadingState::Idle;
        }
    }
}

impl Default for DataLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loading_state() {
        assert!(!LoadingState::Idle.is_loading());
        assert!(LoadingState::Loading.is_loading());
        assert!(!LoadingState::Success.is_loading());
        assert!(!LoadingState::Error.is_loading());

        assert!(!LoadingState::Idle.is_error());
        assert!(!LoadingState::Loading.is_error());
        assert!(!LoadingState::Success.is_error());
        assert!(LoadingState::Error.is_error());
    }

    #[test]
    fn test_loading_state_indicators() {
        assert_eq!(LoadingState::Idle.indicator(), "");
        assert_eq!(LoadingState::Loading.indicator(), "⟳");
        assert_eq!(LoadingState::Success.indicator(), "✓");
        assert_eq!(LoadingState::Error.indicator(), "✗");
    }

    #[test]
    fn test_data_load_error_display() {
        let db_error = DataLoadError::DatabaseError("connection failed".to_string());
        assert_eq!(db_error.to_string(), "Database error: connection failed");

        let conn_error = DataLoadError::ConnectionError("timeout".to_string());
        assert_eq!(conn_error.to_string(), "Connection error: timeout");

        let timeout_error = DataLoadError::TimeoutError;
        assert_eq!(timeout_error.to_string(), "Request timed out");

        let other_error = DataLoadError::Other("unknown".to_string());
        assert_eq!(other_error.to_string(), "unknown");
    }

    #[tokio::test]
    async fn test_data_loader_new() {
        let loader = DataLoader::new();

        assert_eq!(loader.loading_state().await, LoadingState::Idle);
        assert!(loader.last_error().await.is_none());
        assert!(loader.data().await.is_empty());
        assert!(loader.should_refresh().await);
    }

    #[tokio::test]
    async fn test_data_loader_with_interval() {
        let loader = DataLoader::new().with_min_fetch_interval(Duration::from_secs(10));

        assert!(loader.should_refresh().await);
    }

    #[tokio::test]
    async fn test_data_loader_refresh() {
        let loader = DataLoader::new().with_min_fetch_interval(Duration::from_millis(100));

        let result = loader.refresh().await;
        assert!(result.is_ok());

        let state = loader.loading_state().await;
        assert_eq!(state, LoadingState::Success);

        let data = loader.data().await;
        assert!(!data.is_empty());
        assert!(data.last_updated.is_some());
    }

    #[tokio::test]
    async fn test_data_loader_debounce() {
        let loader = DataLoader::new().with_min_fetch_interval(Duration::from_secs(60));

        loader.refresh().await.unwrap();
        assert!(!loader.should_refresh().await);

        loader.force_refresh().await.unwrap();
        assert!(loader.data().await.last_updated.is_some());
    }

    #[tokio::test]
    async fn test_data_loader_agents() {
        let loader = DataLoader::new();
        loader.refresh().await.unwrap();

        let data = loader.data().await;
        assert!(!data.agents.is_empty());

        let connected = data.connected_agents_count();
        assert!(connected > 0);
    }

    #[tokio::test]
    async fn test_data_loader_sessions() {
        let loader = DataLoader::new();
        loader.refresh().await.unwrap();

        let data = loader.data().await;
        assert!(!data.sessions.is_empty());

        let active = data.active_sessions_count();
        assert!(active > 0);
    }

    #[tokio::test]
    async fn test_data_loader_costs() {
        let loader = DataLoader::new();
        loader.refresh().await.unwrap();

        let data = loader.data().await;
        assert!(data.costs.today_total > 0.0);
        assert!(!data.costs.by_agent.is_empty());
        assert!(!data.costs.by_model.is_empty());
        assert!(!data.costs.daily_costs.is_empty());
    }

    #[tokio::test]
    async fn test_data_loader_metrics() {
        let loader = DataLoader::new();
        loader.refresh().await.unwrap();

        let data = loader.data().await;
        assert!(data.metrics.memory_total_mb > 0);
        assert!(!data.metrics.cpu_history.is_empty());
    }

    #[test]
    fn test_data_loader_clear_error() {
        let loader = DataLoader::new();
        loader.clear_error();
        loader.set_loading_idle();
    }
}
