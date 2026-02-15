mod config;
mod cost;
mod session;

pub use config::{
    GooseConfig, GooseExtensionConfig, GooseProfile, GooseProviderConfig, GooseSettings,
};
pub use cost::GooseCostCalculator;
pub use session::{GooseHistoryEntry, GooseMessage, GooseSessionData, SessionParser};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use std::sync::{Arc, Mutex};
use tracing::{debug, info};
use uuid::Uuid;

use crate::error::{RimuruError, RimuruResult};
use crate::models::{AgentType, ModelInfo, Session, SessionStatus};

use super::traits::{AgentAdapter, CostTracker, SessionEventCallback, SessionMonitor};
use super::types::{
    ActiveSession, AdapterInfo, AdapterStatus, SessionEvent, SessionHistory, UsageStats,
};

pub struct GooseAdapter {
    name: String,
    config: GooseConfig,
    status: Arc<Mutex<AdapterStatus>>,
    last_connected: Arc<Mutex<Option<DateTime<Utc>>>>,
    error_message: Arc<Mutex<Option<String>>>,
    cost_calculator: GooseCostCalculator,
    session_parser: SessionParser,
    subscriptions: Arc<Mutex<HashMap<Uuid, SessionEventCallback>>>,
}

impl GooseAdapter {
    pub fn new(name: &str, config: GooseConfig) -> Self {
        let session_parser =
            SessionParser::new(config.sessions_dir.clone(), config.data_dir.clone());

        Self {
            name: name.to_string(),
            config,
            status: Arc::new(Mutex::new(AdapterStatus::Unknown)),
            last_connected: Arc::new(Mutex::new(None)),
            error_message: Arc::new(Mutex::new(None)),
            cost_calculator: GooseCostCalculator::new(),
            session_parser,
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_default_config(name: &str) -> Self {
        Self::new(name, GooseConfig::default())
    }

    fn detect_installation(&self) -> bool {
        if self.config.config_dir.exists() && self.config.config_file_path().exists() {
            return true;
        }

        if let Ok(output) = Command::new("which").arg("goose").output() {
            if output.status.success() {
                return true;
            }
        }

        if let Ok(output) = Command::new("where").arg("goose").output() {
            if output.status.success() {
                return true;
            }
        }

        false
    }

    #[allow(dead_code)]
    fn read_settings(&self) -> Option<GooseSettings> {
        let path = self.config.config_file_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(settings) = serde_yaml::from_str::<GooseSettings>(&content) {
                    return Some(settings);
                }

                if let Ok(settings) = serde_json::from_str::<GooseSettings>(&content) {
                    return Some(settings);
                }
            }
        }
        None
    }

    fn get_version(&self) -> Option<String> {
        if let Ok(output) = Command::new("goose").arg("--version").output() {
            if output.status.success() {
                let version_str = String::from_utf8_lossy(&output.stdout);
                let version = version_str.trim().to_string();
                if !version.is_empty() {
                    return Some(version);
                }
            }
        }

        if let Ok(output) = Command::new("goose").arg("version").output() {
            if output.status.success() {
                let version_str = String::from_utf8_lossy(&output.stdout);
                let version = version_str.trim().to_string();
                if !version.is_empty() {
                    return Some(version);
                }
            }
        }

        None
    }

    #[allow(dead_code)]
    fn check_process_running(&self) -> bool {
        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = Command::new("pgrep").arg("-f").arg("goose").output() {
                return output.status.success();
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Ok(output) = Command::new("pgrep").arg("-f").arg("goose").output() {
                return output.status.success();
            }
        }

        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = Command::new("tasklist")
                .args(["/FI", "IMAGENAME eq goose.exe"])
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                return output_str.contains("goose.exe");
            }
        }

        false
    }

    #[allow(dead_code)]
    fn emit_event(&self, event: SessionEvent) {
        let subscriptions = self.subscriptions.lock().unwrap();
        for callback in subscriptions.values() {
            callback(event.clone());
        }
    }

    fn get_default_model(&self) -> String {
        self.read_settings()
            .and_then(|s| s.provider)
            .and_then(|p| p.model)
            .unwrap_or_else(|| "claude-3-5-sonnet".to_string())
    }
}

#[async_trait]
impl AgentAdapter for GooseAdapter {
    fn agent_type(&self) -> AgentType {
        AgentType::Goose
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn connect(&mut self) -> RimuruResult<()> {
        info!("Connecting to Goose adapter: {}", self.name);

        if !self.detect_installation() {
            let msg = "Goose is not installed or not configured".to_string();
            *self.error_message.lock().unwrap() = Some(msg.clone());
            *self.status.lock().unwrap() = AdapterStatus::Error;
            return Err(RimuruError::AgentConnectionFailed {
                agent: self.name.clone(),
                message: msg,
            });
        }

        *self.status.lock().unwrap() = AdapterStatus::Connected;
        *self.last_connected.lock().unwrap() = Some(Utc::now());
        *self.error_message.lock().unwrap() = None;

        debug!("Goose adapter connected successfully");
        Ok(())
    }

    async fn disconnect(&mut self) -> RimuruResult<()> {
        info!("Disconnecting Goose adapter: {}", self.name);
        *self.status.lock().unwrap() = AdapterStatus::Disconnected;
        Ok(())
    }

    async fn get_status(&self) -> AdapterStatus {
        *self.status.lock().unwrap()
    }

    async fn get_info(&self) -> RimuruResult<AdapterInfo> {
        Ok(AdapterInfo {
            name: self.name.clone(),
            agent_type: AgentType::Goose,
            version: self.get_version(),
            status: *self.status.lock().unwrap(),
            config_path: Some(self.config.config_dir.to_string_lossy().to_string()),
            last_connected: *self.last_connected.lock().unwrap(),
            error_message: self.error_message.lock().unwrap().clone(),
        })
    }

    async fn get_sessions(&self) -> RimuruResult<Vec<Session>> {
        let history = self.session_parser.parse_sessions()?;

        let sessions: Vec<Session> = history
            .into_iter()
            .map(|h| Session {
                id: h.session_id,
                agent_id: Uuid::nil(),
                status: if h.ended_at.is_some() {
                    SessionStatus::Completed
                } else {
                    SessionStatus::Active
                },
                started_at: h.started_at,
                ended_at: h.ended_at,
                metadata: serde_json::json!({
                    "model": h.model_name,
                    "project_path": h.project_path,
                    "input_tokens": h.total_input_tokens,
                    "output_tokens": h.total_output_tokens,
                    "cost_usd": h.cost_usd,
                }),
            })
            .collect();

        Ok(sessions)
    }

    async fn get_active_session(&self) -> RimuruResult<Option<ActiveSession>> {
        self.session_parser.get_active_session()
    }

    async fn is_installed(&self) -> bool {
        self.detect_installation()
    }

    async fn health_check(&self) -> RimuruResult<bool> {
        let status = *self.status.lock().unwrap();
        if status != AdapterStatus::Connected {
            return Ok(false);
        }

        let installed = self.detect_installation();
        if !installed {
            *self.status.lock().unwrap() = AdapterStatus::Error;
            *self.error_message.lock().unwrap() = Some("Goose installation not found".to_string());
            return Ok(false);
        }

        Ok(true)
    }
}

#[async_trait]
impl CostTracker for GooseAdapter {
    async fn get_usage(&self, since: Option<DateTime<Utc>>) -> RimuruResult<UsageStats> {
        let sessions = self.session_parser.parse_sessions()?;

        let filtered_sessions: Vec<_> = if let Some(since_time) = since {
            sessions
                .into_iter()
                .filter(|s| s.started_at >= since_time)
                .collect()
        } else {
            sessions
        };

        let mut total_input = 0i64;
        let mut total_output = 0i64;
        let mut request_count = 0i64;

        for session in &filtered_sessions {
            total_input += session.total_input_tokens;
            total_output += session.total_output_tokens;
            request_count += 1;
        }

        let model_name = filtered_sessions.first().and_then(|s| s.model_name.clone());

        Ok(UsageStats {
            input_tokens: total_input,
            output_tokens: total_output,
            requests: request_count,
            model_name,
            period_start: since,
            period_end: Some(Utc::now()),
        })
    }

    async fn calculate_cost(
        &self,
        input_tokens: i64,
        output_tokens: i64,
        model_name: &str,
    ) -> RimuruResult<f64> {
        self.cost_calculator
            .calculate_cost(input_tokens, output_tokens, model_name)
    }

    async fn get_model_info(&self, model_name: &str) -> RimuruResult<Option<ModelInfo>> {
        Ok(self.cost_calculator.get_model(model_name).cloned())
    }

    async fn get_supported_models(&self) -> RimuruResult<Vec<String>> {
        Ok(self.cost_calculator.get_supported_models())
    }

    async fn get_total_cost(&self, since: Option<DateTime<Utc>>) -> RimuruResult<f64> {
        let sessions = self.session_parser.parse_sessions()?;

        let filtered_sessions: Vec<_> = if let Some(since_time) = since {
            sessions
                .into_iter()
                .filter(|s| s.started_at >= since_time)
                .collect()
        } else {
            sessions
        };

        let mut total_cost = 0.0f64;
        let default_model = self.get_default_model();

        for session in filtered_sessions {
            if let Some(cost) = session.cost_usd {
                total_cost += cost;
            } else {
                let model = session.model_name.as_deref().unwrap_or(&default_model);
                let cost = self.cost_calculator.calculate_cost(
                    session.total_input_tokens,
                    session.total_output_tokens,
                    model,
                )?;
                total_cost += cost;
            }
        }

        Ok(total_cost)
    }
}

#[async_trait]
impl SessionMonitor for GooseAdapter {
    async fn subscribe_to_events(&self, callback: SessionEventCallback) -> RimuruResult<Uuid> {
        let subscription_id = Uuid::new_v4();
        let mut subscriptions = self.subscriptions.lock().unwrap();
        subscriptions.insert(subscription_id, callback);
        Ok(subscription_id)
    }

    async fn unsubscribe(&self, subscription_id: Uuid) -> RimuruResult<()> {
        let mut subscriptions = self.subscriptions.lock().unwrap();
        subscriptions.remove(&subscription_id);
        Ok(())
    }

    async fn get_session_history(
        &self,
        limit: Option<usize>,
        since: Option<DateTime<Utc>>,
    ) -> RimuruResult<Vec<SessionHistory>> {
        let mut sessions = self.session_parser.parse_sessions()?;

        if let Some(since_time) = since {
            sessions.retain(|s| s.started_at >= since_time);
        }

        if let Some(limit_count) = limit {
            sessions.truncate(limit_count);
        }

        Ok(sessions)
    }

    async fn get_session_details(&self, session_id: Uuid) -> RimuruResult<Option<SessionHistory>> {
        let sessions = self.session_parser.parse_sessions()?;
        Ok(sessions.into_iter().find(|s| s.session_id == session_id))
    }

    async fn get_active_sessions(&self) -> RimuruResult<Vec<ActiveSession>> {
        if let Some(active) = self.session_parser.get_active_session()? {
            Ok(vec![active])
        } else {
            Ok(vec![])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_adapter_creation() {
        let adapter = GooseAdapter::with_default_config("test-goose");

        assert_eq!(adapter.name(), "test-goose");
        assert_eq!(adapter.agent_type(), AgentType::Goose);
    }

    #[tokio::test]
    async fn test_adapter_initial_status() {
        let adapter = GooseAdapter::with_default_config("test");

        let status = adapter.get_status().await;
        assert_eq!(status, AdapterStatus::Unknown);
    }

    #[tokio::test]
    async fn test_adapter_info() {
        let temp_dir = tempdir().unwrap();
        let config = GooseConfig::new().with_config_dir(temp_dir.path().to_path_buf());

        let adapter = GooseAdapter::new("test-adapter", config);
        let info = adapter.get_info().await.unwrap();

        assert_eq!(info.name, "test-adapter");
        assert_eq!(info.agent_type, AgentType::Goose);
        assert_eq!(info.status, AdapterStatus::Unknown);
    }

    #[tokio::test]
    async fn test_adapter_disconnect() {
        let adapter = GooseAdapter::with_default_config("test");
        let mut adapter = adapter;

        adapter.disconnect().await.unwrap();
        assert_eq!(adapter.get_status().await, AdapterStatus::Disconnected);
    }

    #[tokio::test]
    async fn test_get_supported_models() {
        let adapter = GooseAdapter::with_default_config("test");

        let models = adapter.get_supported_models().await.unwrap();
        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.contains("claude")));
        assert!(models.iter().any(|m| m.contains("gpt")));
        assert!(models.iter().any(|m| m.contains("gemini")));
    }

    #[tokio::test]
    async fn test_calculate_cost_anthropic() {
        let adapter = GooseAdapter::with_default_config("test");

        let cost = adapter
            .calculate_cost(10000, 5000, "claude-3-5-sonnet")
            .await
            .unwrap();

        assert!(cost > 0.0);
    }

    #[tokio::test]
    async fn test_calculate_cost_openai() {
        let adapter = GooseAdapter::with_default_config("test");

        let cost = adapter.calculate_cost(10000, 5000, "gpt-4o").await.unwrap();

        assert!(cost > 0.0);
    }

    #[tokio::test]
    async fn test_calculate_cost_ollama_free() {
        let adapter = GooseAdapter::with_default_config("test");

        let cost = adapter
            .calculate_cost(10000, 5000, "ollama/local")
            .await
            .unwrap();

        assert_eq!(cost, 0.0);
    }

    #[tokio::test]
    async fn test_subscription_management() {
        let adapter = GooseAdapter::with_default_config("test");

        let callback = Box::new(|_event: SessionEvent| {});
        let sub_id = adapter.subscribe_to_events(callback).await.unwrap();

        adapter.unsubscribe(sub_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_get_session_history_empty() {
        let temp_dir = tempdir().unwrap();
        let config = GooseConfig::new()
            .with_config_dir(temp_dir.path().to_path_buf())
            .with_data_dir(temp_dir.path().to_path_buf())
            .with_sessions_dir(temp_dir.path().join("sessions"));

        let adapter = GooseAdapter::new("test", config);

        let history = adapter.get_session_history(None, None).await.unwrap();
        assert!(history.is_empty());
    }

    #[tokio::test]
    async fn test_get_active_sessions_empty() {
        let temp_dir = tempdir().unwrap();
        let config = GooseConfig::new()
            .with_config_dir(temp_dir.path().to_path_buf())
            .with_data_dir(temp_dir.path().to_path_buf())
            .with_sessions_dir(temp_dir.path().join("sessions"));

        let adapter = GooseAdapter::new("test", config);

        let active = adapter.get_active_sessions().await.unwrap();
        assert!(active.is_empty());
    }

    #[tokio::test]
    async fn test_get_usage_empty() {
        let temp_dir = tempdir().unwrap();
        let config = GooseConfig::new()
            .with_config_dir(temp_dir.path().to_path_buf())
            .with_data_dir(temp_dir.path().to_path_buf())
            .with_sessions_dir(temp_dir.path().join("sessions"));

        let adapter = GooseAdapter::new("test", config);

        let usage = adapter.get_usage(None).await.unwrap();
        assert_eq!(usage.input_tokens, 0);
        assert_eq!(usage.output_tokens, 0);
        assert_eq!(usage.requests, 0);
    }
}
