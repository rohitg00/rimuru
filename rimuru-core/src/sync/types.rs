use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub provider: String,
    pub models_updated: usize,
    pub models_added: usize,
    pub models_unchanged: usize,
    pub last_sync: DateTime<Utc>,
    pub duration_ms: u64,
    pub errors: Vec<SyncError>,
    pub success: bool,
}

impl SyncResult {
    pub fn new(provider: &str) -> Self {
        Self {
            provider: provider.to_string(),
            models_updated: 0,
            models_added: 0,
            models_unchanged: 0,
            last_sync: Utc::now(),
            duration_ms: 0,
            errors: Vec::new(),
            success: true,
        }
    }

    pub fn with_error(mut self, error: SyncError) -> Self {
        self.errors.push(error);
        self.success = false;
        self
    }

    pub fn total_models(&self) -> usize {
        self.models_updated + self.models_added + self.models_unchanged
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncError {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
    pub timestamp: DateTime<Utc>,
}

impl SyncError {
    pub fn new(code: &str, message: &str, recoverable: bool) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            recoverable,
            timestamp: Utc::now(),
        }
    }

    pub fn api_error(message: &str) -> Self {
        Self::new("API_ERROR", message, true)
    }

    pub fn parse_error(message: &str) -> Self {
        Self::new("PARSE_ERROR", message, false)
    }

    pub fn rate_limit(message: &str) -> Self {
        Self::new("RATE_LIMIT", message, true)
    }

    pub fn auth_error(message: &str) -> Self {
        Self::new("AUTH_ERROR", message, false)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub enabled: bool,
    pub interval_secs: u64,
    pub retry_max_attempts: u32,
    pub retry_base_delay_secs: u64,
    pub retry_max_delay_secs: u64,
    pub providers: SyncProviderConfig,
    pub rate_limits: RateLimitConfig,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 6 * 60 * 60,
            retry_max_attempts: 3,
            retry_base_delay_secs: 60,
            retry_max_delay_secs: 3600,
            providers: SyncProviderConfig::default(),
            rate_limits: RateLimitConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncProviderConfig {
    pub anthropic: bool,
    pub openai: bool,
    pub google: bool,
    pub openrouter: bool,
    pub litellm: bool,
}

impl Default for SyncProviderConfig {
    fn default() -> Self {
        Self {
            anthropic: true,
            openai: true,
            google: true,
            openrouter: true,
            litellm: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub anthropic_rpm: u32,
    pub openai_rpm: u32,
    pub google_rpm: u32,
    pub openrouter_rpm: u32,
    pub litellm_rpm: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            anthropic_rpm: 60,
            openai_rpm: 60,
            google_rpm: 60,
            openrouter_rpm: 100,
            litellm_rpm: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncStatus {
    pub is_running: bool,
    pub last_full_sync: Option<DateTime<Utc>>,
    pub next_scheduled_sync: Option<DateTime<Utc>>,
    pub provider_status: HashMap<String, ProviderSyncStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSyncStatus {
    pub provider: String,
    pub enabled: bool,
    pub last_sync: Option<DateTime<Utc>>,
    pub last_success: bool,
    pub models_count: usize,
    pub consecutive_failures: u32,
    pub last_error: Option<String>,
}

impl ProviderSyncStatus {
    pub fn new(provider: &str, enabled: bool) -> Self {
        Self {
            provider: provider.to_string(),
            enabled,
            last_sync: None,
            last_success: true,
            models_count: 0,
            consecutive_failures: 0,
            last_error: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncHistory {
    pub entries: Vec<SyncHistoryEntry>,
}

impl SyncHistory {
    pub fn add_entry(&mut self, entry: SyncHistoryEntry) {
        self.entries.push(entry);
        if self.entries.len() > 1000 {
            self.entries.remove(0);
        }
    }

    pub fn recent(&self, limit: usize) -> &[SyncHistoryEntry] {
        let start = self.entries.len().saturating_sub(limit);
        &self.entries[start..]
    }

    pub fn success_count(&self) -> usize {
        self.entries.iter().filter(|e| e.success).count()
    }

    pub fn failure_count(&self) -> usize {
        self.entries.iter().filter(|e| !e.success).count()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncHistoryEntry {
    pub timestamp: DateTime<Utc>,
    pub provider: String,
    pub success: bool,
    pub models_synced: usize,
    pub duration_ms: u64,
    pub error_message: Option<String>,
}

impl SyncHistoryEntry {
    pub fn success(provider: &str, models_synced: usize, duration_ms: u64) -> Self {
        Self {
            timestamp: Utc::now(),
            provider: provider.to_string(),
            success: true,
            models_synced,
            duration_ms,
            error_message: None,
        }
    }

    pub fn failure(provider: &str, error: &str, duration_ms: u64) -> Self {
        Self {
            timestamp: Utc::now(),
            provider: provider.to_string(),
            success: false,
            models_synced: 0,
            duration_ms,
            error_message: Some(error.to_string()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelCapability {
    TextGeneration,
    Vision,
    FunctionCalling,
    JsonMode,
    Embedding,
    CodeGeneration,
    MultiModal,
    Streaming,
    FineTuning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedModelInfo {
    pub provider: String,
    pub model_name: String,
    pub model_id: String,
    pub input_price_per_1k: f64,
    pub output_price_per_1k: f64,
    pub context_window: i32,
    pub max_output_tokens: Option<i32>,
    pub capabilities: Vec<ModelCapability>,
    pub deprecation_date: Option<DateTime<Utc>>,
    pub source: ModelSource,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelSource {
    OfficialApi,
    OfficialDocs,
    OpenRouter,
    LiteLLM,
    Manual,
}

impl ModelSource {
    pub fn priority(&self) -> u8 {
        match self {
            ModelSource::OfficialApi => 1,
            ModelSource::OfficialDocs => 2,
            ModelSource::OpenRouter => 3,
            ModelSource::LiteLLM => 4,
            ModelSource::Manual => 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_result_new() {
        let result = SyncResult::new("anthropic");
        assert_eq!(result.provider, "anthropic");
        assert!(result.success);
        assert!(result.errors.is_empty());
        assert_eq!(result.total_models(), 0);
    }

    #[test]
    fn test_sync_result_with_error() {
        let result =
            SyncResult::new("openai").with_error(SyncError::api_error("Connection failed"));

        assert!(!result.success);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].code, "API_ERROR");
    }

    #[test]
    fn test_sync_error_variants() {
        let api_err = SyncError::api_error("test");
        assert_eq!(api_err.code, "API_ERROR");
        assert!(api_err.recoverable);

        let parse_err = SyncError::parse_error("test");
        assert_eq!(parse_err.code, "PARSE_ERROR");
        assert!(!parse_err.recoverable);

        let rate_err = SyncError::rate_limit("test");
        assert_eq!(rate_err.code, "RATE_LIMIT");
        assert!(rate_err.recoverable);

        let auth_err = SyncError::auth_error("test");
        assert_eq!(auth_err.code, "AUTH_ERROR");
        assert!(!auth_err.recoverable);
    }

    #[test]
    fn test_sync_config_default() {
        let config = SyncConfig::default();
        assert!(config.enabled);
        assert_eq!(config.interval_secs, 6 * 60 * 60);
        assert_eq!(config.retry_max_attempts, 3);
        assert!(config.providers.anthropic);
        assert!(config.providers.openai);
        assert!(config.providers.google);
        assert!(config.providers.openrouter);
        assert!(config.providers.litellm);
    }

    #[test]
    fn test_provider_sync_status() {
        let status = ProviderSyncStatus::new("anthropic", true);
        assert_eq!(status.provider, "anthropic");
        assert!(status.enabled);
        assert!(status.last_success);
        assert_eq!(status.consecutive_failures, 0);
    }

    #[test]
    fn test_sync_history() {
        let mut history = SyncHistory::default();

        history.add_entry(SyncHistoryEntry::success("anthropic", 10, 100));
        history.add_entry(SyncHistoryEntry::failure("openai", "timeout", 50));
        history.add_entry(SyncHistoryEntry::success("google", 5, 80));

        assert_eq!(history.entries.len(), 3);
        assert_eq!(history.success_count(), 2);
        assert_eq!(history.failure_count(), 1);

        let recent = history.recent(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].provider, "openai");
        assert_eq!(recent[1].provider, "google");
    }

    #[test]
    fn test_model_source_priority() {
        assert!(ModelSource::OfficialApi.priority() < ModelSource::OfficialDocs.priority());
        assert!(ModelSource::OfficialDocs.priority() < ModelSource::OpenRouter.priority());
        assert!(ModelSource::OpenRouter.priority() < ModelSource::LiteLLM.priority());
        assert!(ModelSource::LiteLLM.priority() < ModelSource::Manual.priority());
    }

    #[test]
    fn test_sync_history_entry_constructors() {
        let success = SyncHistoryEntry::success("test", 5, 100);
        assert!(success.success);
        assert_eq!(success.models_synced, 5);
        assert!(success.error_message.is_none());

        let failure = SyncHistoryEntry::failure("test", "error msg", 50);
        assert!(!failure.success);
        assert_eq!(failure.models_synced, 0);
        assert_eq!(failure.error_message, Some("error msg".to_string()));
    }
}
