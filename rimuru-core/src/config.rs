use config::{Config as ConfigBuilder, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigLoadError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Missing required configuration: {0}")]
    MissingRequired(String),

    #[error("Invalid configuration value for {key}: {message}")]
    InvalidValue { key: String, message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RimuruConfig {
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub metrics: MetricsConfig,
    pub agents: AgentsConfig,
    pub sync: SyncConfig,
    pub display: DisplayConfig,
    pub tui: TuiConfig,
    pub desktop: DesktopConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    #[serde(default = "default_database_url")]
    pub url: String,

    #[serde(default = "default_pool_min")]
    pub pool_min_connections: u32,

    #[serde(default = "default_pool_max")]
    pub pool_max_connections: u32,

    #[serde(default = "default_acquire_timeout")]
    pub pool_acquire_timeout_secs: u64,

    #[serde(default = "default_idle_timeout")]
    pub pool_idle_timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,

    #[serde(default)]
    pub json_format: bool,

    #[serde(default)]
    pub file_path: String,

    #[serde(default = "default_max_file_size")]
    pub max_file_size_mb: u64,

    #[serde(default = "default_max_backup_count")]
    pub max_backup_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    #[serde(default = "default_metrics_interval")]
    pub interval_secs: u64,

    #[serde(default = "default_true")]
    pub store_to_db: bool,

    #[serde(default = "default_retention_days")]
    pub retention_days: u32,

    #[serde(default = "default_true")]
    pub collect_cpu: bool,

    #[serde(default = "default_true")]
    pub collect_memory: bool,

    #[serde(default = "default_true")]
    pub track_sessions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentsConfig {
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout_secs: u64,

    #[serde(default = "default_true")]
    pub auto_discover: bool,

    #[serde(default = "default_health_check_interval")]
    pub health_check_interval_secs: u64,

    #[serde(default)]
    pub supported: SupportedAgents,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportedAgents {
    #[serde(default = "default_true")]
    pub claude_code: bool,

    #[serde(default = "default_true")]
    pub codex: bool,

    #[serde(default = "default_true")]
    pub copilot: bool,

    #[serde(default = "default_true")]
    pub goose: bool,

    #[serde(default = "default_true")]
    pub opencode: bool,

    #[serde(default = "default_true")]
    pub cursor: bool,
}

impl Default for SupportedAgents {
    fn default() -> Self {
        Self {
            claude_code: true,
            codex: true,
            copilot: true,
            goose: true,
            opencode: true,
            cursor: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default = "default_sync_interval")]
    pub interval_hours: u32,

    #[serde(default = "default_retry_count")]
    pub retry_count: u32,

    #[serde(default = "default_retry_delay")]
    pub retry_delay_secs: u64,

    #[serde(default)]
    pub providers: SyncProviders,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncProviders {
    #[serde(default = "default_true")]
    pub anthropic: bool,

    #[serde(default = "default_true")]
    pub openai: bool,

    #[serde(default = "default_true")]
    pub google: bool,
}

impl Default for SyncProviders {
    fn default() -> Self {
        Self {
            anthropic: true,
            openai: true,
            google: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_true")]
    pub color: bool,

    #[serde(default = "default_datetime_format")]
    pub datetime_format: String,

    #[serde(default = "default_currency")]
    pub currency: String,

    #[serde(default = "default_cost_precision")]
    pub cost_precision: u32,

    #[serde(default)]
    pub compact: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiConfig {
    #[serde(default = "default_refresh_rate")]
    pub refresh_rate_ms: u64,

    #[serde(default = "default_true")]
    pub show_metrics: bool,

    #[serde(default = "default_true")]
    pub show_costs: bool,

    #[serde(default = "default_theme")]
    pub theme: String,

    #[serde(default = "default_true")]
    pub mouse_enabled: bool,

    #[serde(default = "default_true")]
    pub unicode_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopConfig {
    #[serde(default = "default_width")]
    pub default_width: u32,

    #[serde(default = "default_height")]
    pub default_height: u32,

    #[serde(default)]
    pub start_minimized: bool,

    #[serde(default = "default_true")]
    pub notifications_enabled: bool,

    #[serde(default = "default_warning_threshold")]
    pub cost_warning_threshold: f64,

    #[serde(default = "default_alert_threshold")]
    pub cost_alert_threshold: f64,
}

fn default_database_url() -> String {
    "postgres://localhost/rimuru_dev".to_string()
}

fn default_pool_min() -> u32 {
    1
}

fn default_pool_max() -> u32 {
    10
}

fn default_acquire_timeout() -> u64 {
    30
}

fn default_idle_timeout() -> u64 {
    600
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_max_file_size() -> u64 {
    100
}

fn default_max_backup_count() -> u32 {
    5
}

fn default_metrics_interval() -> u64 {
    5
}

fn default_true() -> bool {
    true
}

fn default_retention_days() -> u32 {
    30
}

fn default_connection_timeout() -> u64 {
    30
}

fn default_health_check_interval() -> u64 {
    60
}

fn default_sync_interval() -> u32 {
    24
}

fn default_retry_count() -> u32 {
    3
}

fn default_retry_delay() -> u64 {
    60
}

fn default_datetime_format() -> String {
    "%Y-%m-%d %H:%M:%S".to_string()
}

fn default_currency() -> String {
    "USD".to_string()
}

fn default_cost_precision() -> u32 {
    6
}

fn default_refresh_rate() -> u64 {
    250
}

fn default_theme() -> String {
    "auto".to_string()
}

fn default_width() -> u32 {
    1200
}

fn default_height() -> u32 {
    800
}

fn default_warning_threshold() -> f64 {
    1.0
}

fn default_alert_threshold() -> f64 {
    5.0
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: default_database_url(),
            pool_min_connections: default_pool_min(),
            pool_max_connections: default_pool_max(),
            pool_acquire_timeout_secs: default_acquire_timeout(),
            pool_idle_timeout_secs: default_idle_timeout(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            json_format: false,
            file_path: String::new(),
            max_file_size_mb: default_max_file_size(),
            max_backup_count: default_max_backup_count(),
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            interval_secs: default_metrics_interval(),
            store_to_db: true,
            retention_days: default_retention_days(),
            collect_cpu: true,
            collect_memory: true,
            track_sessions: true,
        }
    }
}

impl Default for AgentsConfig {
    fn default() -> Self {
        Self {
            connection_timeout_secs: default_connection_timeout(),
            auto_discover: true,
            health_check_interval_secs: default_health_check_interval(),
            supported: SupportedAgents::default(),
        }
    }
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_hours: default_sync_interval(),
            retry_count: default_retry_count(),
            retry_delay_secs: default_retry_delay(),
            providers: SyncProviders::default(),
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            color: true,
            datetime_format: default_datetime_format(),
            currency: default_currency(),
            cost_precision: default_cost_precision(),
            compact: false,
        }
    }
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            refresh_rate_ms: default_refresh_rate(),
            show_metrics: true,
            show_costs: true,
            theme: default_theme(),
            mouse_enabled: true,
            unicode_enabled: true,
        }
    }
}

impl Default for DesktopConfig {
    fn default() -> Self {
        Self {
            default_width: default_width(),
            default_height: default_height(),
            start_minimized: false,
            notifications_enabled: true,
            cost_warning_threshold: default_warning_threshold(),
            cost_alert_threshold: default_alert_threshold(),
        }
    }
}

impl RimuruConfig {
    pub fn load() -> Result<Self, ConfigLoadError> {
        Self::load_from_paths(get_config_paths())
    }

    pub fn load_from_paths(paths: Vec<PathBuf>) -> Result<Self, ConfigLoadError> {
        load_dotenv_files();

        let mut builder = ConfigBuilder::builder();

        for path in paths {
            if path.exists() {
                builder = builder.add_source(File::from(path).required(false));
            }
        }

        builder = builder
            .add_source(
                Environment::with_prefix("RIMURU")
                    .separator("_")
                    .try_parsing(true),
            )
            .add_source(Environment::default().try_parsing(true));

        let config = builder.build()?;

        let mut rimuru_config: RimuruConfig = config.try_deserialize().unwrap_or_default();

        if let Ok(url) = std::env::var("DATABASE_URL") {
            rimuru_config.database.url = url;
        } else if let Ok(url) = std::env::var("RIMURU_DATABASE_URL") {
            rimuru_config.database.url = url;
        }

        if let Ok(level) = std::env::var("RIMURU_LOG_LEVEL") {
            rimuru_config.logging.level = level;
        } else if let Ok(level) = std::env::var("RUST_LOG") {
            rimuru_config.logging.level = level;
        }

        if let Ok(interval) = std::env::var("RIMURU_METRICS_INTERVAL") {
            if let Ok(secs) = interval.parse() {
                rimuru_config.metrics.interval_secs = secs;
            }
        }

        rimuru_config.validate()?;

        Ok(rimuru_config)
    }

    pub fn validate(&self) -> Result<(), ConfigLoadError> {
        if self.database.url.is_empty() {
            return Err(ConfigLoadError::MissingRequired("database.url".to_string()));
        }

        if !self.database.url.starts_with("postgres://")
            && !self.database.url.starts_with("postgresql://")
        {
            return Err(ConfigLoadError::InvalidValue {
                key: "database.url".to_string(),
                message:
                    "Must be a valid PostgreSQL URL starting with postgres:// or postgresql://"
                        .to_string(),
            });
        }

        if self.database.pool_min_connections > self.database.pool_max_connections {
            return Err(ConfigLoadError::InvalidValue {
                key: "database.pool_min_connections".to_string(),
                message: "Cannot be greater than pool_max_connections".to_string(),
            });
        }

        if self.metrics.interval_secs == 0 {
            return Err(ConfigLoadError::InvalidValue {
                key: "metrics.interval_secs".to_string(),
                message: "Must be greater than 0".to_string(),
            });
        }

        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        let level_lower = self.logging.level.to_lowercase();
        if !valid_levels.contains(&level_lower.as_str()) && !level_lower.contains('=') {
            return Err(ConfigLoadError::InvalidValue {
                key: "logging.level".to_string(),
                message: format!(
                    "Invalid log level '{}'. Must be one of: {:?}",
                    self.logging.level, valid_levels
                ),
            });
        }

        Ok(())
    }

    pub fn database_url(&self) -> &str {
        &self.database.url
    }

    pub fn log_level(&self) -> &str {
        &self.logging.level
    }

    pub fn metrics_interval(&self) -> u64 {
        self.metrics.interval_secs
    }
}

fn get_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(cwd) = std::env::current_dir() {
        paths.push(cwd.join("config").join("default.toml"));
        paths.push(cwd.join("config").join("local.toml"));
        paths.push(cwd.join("rimuru.toml"));
    }

    if let Some(config_dir) = dirs::config_dir() {
        paths.push(config_dir.join("rimuru").join("config.toml"));
    }

    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".rimuru").join("config.toml"));
        paths.push(home.join(".config").join("rimuru").join("config.toml"));
    }

    paths
}

fn load_dotenv_files() {
    let env_paths = get_dotenv_paths();

    for path in env_paths {
        if path.exists() {
            let _ = dotenvy::from_path(&path);
        }
    }
}

fn get_dotenv_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(cwd) = std::env::current_dir() {
        paths.push(cwd.join(".env"));
        paths.push(cwd.join(".env.local"));
    }

    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".rimuru").join(".env"));
    }

    if let Some(config_dir) = dirs::config_dir() {
        paths.push(config_dir.join("rimuru").join(".env"));
    }

    paths
}

pub fn get_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("rimuru"))
}

pub fn get_data_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("rimuru"))
}

pub fn get_cache_dir() -> Option<PathBuf> {
    dirs::cache_dir().map(|d| d.join("rimuru"))
}

pub fn ensure_config_dir() -> Result<PathBuf, std::io::Error> {
    let config_dir = get_config_dir().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine config directory",
        )
    })?;

    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)?;
    }

    Ok(config_dir)
}

pub fn ensure_data_dir() -> Result<PathBuf, std::io::Error> {
    let data_dir = get_data_dir().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine data directory",
        )
    })?;

    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir)?;
    }

    Ok(data_dir)
}

pub fn ensure_cache_dir() -> Result<PathBuf, std::io::Error> {
    let cache_dir = get_cache_dir().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine cache directory",
        )
    })?;

    if !cache_dir.exists() {
        std::fs::create_dir_all(&cache_dir)?;
    }

    Ok(cache_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RimuruConfig::default();

        assert_eq!(config.database.url, "postgres://localhost/rimuru_dev");
        assert_eq!(config.database.pool_min_connections, 1);
        assert_eq!(config.database.pool_max_connections, 10);
        assert_eq!(config.logging.level, "info");
        assert!(!config.logging.json_format);
        assert_eq!(config.metrics.interval_secs, 5);
        assert!(config.metrics.store_to_db);
        assert_eq!(config.metrics.retention_days, 30);
        assert!(config.agents.auto_discover);
        assert_eq!(config.agents.connection_timeout_secs, 30);
        assert!(config.sync.enabled);
        assert_eq!(config.sync.interval_hours, 24);
        assert!(config.display.color);
        assert_eq!(config.display.currency, "USD");
        assert_eq!(config.tui.refresh_rate_ms, 250);
        assert_eq!(config.tui.theme, "auto");
        assert_eq!(config.desktop.default_width, 1200);
        assert_eq!(config.desktop.default_height, 800);
    }

    #[test]
    fn test_validation_valid_config() {
        let config = RimuruConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validation_empty_database_url() {
        let mut config = RimuruConfig::default();
        config.database.url = String::new();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_database_url() {
        let mut config = RimuruConfig::default();
        config.database.url = "mysql://localhost/test".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_pool_config() {
        let mut config = RimuruConfig::default();
        config.database.pool_min_connections = 20;
        config.database.pool_max_connections = 10;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_zero_metrics_interval() {
        let mut config = RimuruConfig::default();
        config.metrics.interval_secs = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_log_level() {
        let mut config = RimuruConfig::default();
        config.logging.level = "invalid".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_complex_log_level() {
        let mut config = RimuruConfig::default();
        config.logging.level = "rimuru=debug,sqlx=warn".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_helper_methods() {
        let config = RimuruConfig::default();
        assert_eq!(config.database_url(), "postgres://localhost/rimuru_dev");
        assert_eq!(config.log_level(), "info");
        assert_eq!(config.metrics_interval(), 5);
    }

    #[test]
    fn test_directory_helpers() {
        let config_dir = get_config_dir();
        assert!(config_dir.is_some());

        let data_dir = get_data_dir();
        assert!(data_dir.is_some());

        let cache_dir = get_cache_dir();
        assert!(cache_dir.is_some());
    }

    #[test]
    fn test_supported_agents_default() {
        let agents = SupportedAgents::default();
        assert!(agents.claude_code);
        assert!(agents.codex);
        assert!(agents.copilot);
        assert!(agents.goose);
        assert!(agents.opencode);
        assert!(agents.cursor);
    }

    #[test]
    fn test_sync_providers_default() {
        let providers = SyncProviders::default();
        assert!(providers.anthropic);
        assert!(providers.openai);
        assert!(providers.google);
    }
}
