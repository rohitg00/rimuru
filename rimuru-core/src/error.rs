//! Error types for the Rimuru core library.
//!
//! This module provides a unified error handling system for all Rimuru operations,
//! including database operations, configuration, agent management, and API sync.
//!
//! # Error Codes Reference
//!
//! | Code Range | Category | Description |
//! |------------|----------|-------------|
//! | E1001-E1099 | Database | Database connection, query, migration errors |
//! | E2001-E2099 | Config | Environment, config file, and validation errors |
//! | E3001-E3099 | Agent | Agent discovery, connection, and operation errors |
//! | E4001-E4099 | Session | Session lifecycle and status errors |
//! | E5001-E5099 | Sync/API | External API, rate limiting, and timeout errors |
//! | E6001-E6099 | Metrics | Metrics collection and storage errors |
//! | E7001-E7099 | Cost | Pricing and cost calculation errors |
//! | E8001-E8099 | SkillKit | Skill installation, translation, and sync errors |
//! | E9001-E9099 | General | Internal, IO, serialization, and validation errors |
//! | E10001-E10099 | Plugin | Plugin loading, config, and permission errors |
//! | E11001-E11099 | Hook | Hook registration, execution, and timeout errors |

use std::fmt;
use std::time::Duration;
use thiserror::Error;
use tracing::{error, info, warn};

/// Context information for error tracking and debugging.
///
/// Captures the location where an error occurred along with optional
/// operation description for detailed error reporting.
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// File where the error occurred
    pub file: &'static str,
    /// Line number where the error occurred
    pub line: u32,
    /// Column number where the error occurred
    pub column: u32,
    /// Optional description of the operation being performed
    pub operation: Option<String>,
}

impl ErrorContext {
    /// Create a new error context with the given location.
    pub fn new(file: &'static str, line: u32, column: u32) -> Self {
        Self {
            file,
            line,
            column,
            operation: None,
        }
    }

    /// Add an operation description to the context.
    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        self.operation = Some(operation.into());
        self
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)?;
        if let Some(ref op) = self.operation {
            write!(f, " ({})", op)?;
        }
        Ok(())
    }
}

/// Macro to create an ErrorContext at the current source location.
#[macro_export]
macro_rules! error_context {
    () => {
        $crate::error::ErrorContext::new(file!(), line!(), column!())
    };
    ($op:expr) => {
        $crate::error::ErrorContext::new(file!(), line!(), column!()).with_operation($op)
    };
}

/// Configuration for retry behavior with exponential backoff.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Multiplier for exponential backoff (e.g., 2.0 for doubling)
    pub backoff_multiplier: f64,
    /// Whether to add jitter to prevent thundering herd
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryConfig {
    /// Create a new retry config for database operations.
    pub fn for_database() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }

    /// Create a new retry config for API operations.
    pub fn for_api() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }

    /// Create a new retry config for agent reconnection.
    pub fn for_agent_reconnection() -> Self {
        Self {
            max_attempts: 10,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(300), // 5 minutes max
            backoff_multiplier: 1.5,
            jitter: true,
        }
    }

    /// Calculate the delay for a given attempt number (0-indexed).
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let base_delay =
            self.initial_delay.as_millis() as f64 * self.backoff_multiplier.powi(attempt as i32);
        let capped_delay = base_delay.min(self.max_delay.as_millis() as f64);

        let final_delay = if self.jitter {
            // Add random jitter up to 25% of the delay
            let jitter_factor = 1.0 + (rand_jitter() * 0.25);
            capped_delay * jitter_factor
        } else {
            capped_delay
        };

        Duration::from_millis(final_delay as u64)
    }
}

/// Simple deterministic jitter based on current timestamp.
/// Returns a value between 0.0 and 1.0.
fn rand_jitter() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (nanos % 1000) as f64 / 1000.0
}

/// The main error type for the Rimuru core library.
///
/// This enum covers all possible error conditions that can occur during
/// Rimuru operations, providing detailed context for debugging and user feedback.
#[derive(Debug, Error)]
pub enum RimuruError {
    // ========================================================================
    // Database Errors (E1001-E1099)
    // ========================================================================
    /// Failed to establish database connection
    #[error("[E1001] Database connection failed: {message}")]
    DatabaseConnectionFailed {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Database query execution failed
    #[error("[E1002] Database query failed: {0}")]
    DatabaseQueryFailed(String),

    /// Database migration failed
    #[error("[E1003] Database migration failed: {0}")]
    DatabaseMigrationFailed(String),

    /// Database pool exhausted or unavailable
    #[error("[E1004] Database pool unavailable: {0}")]
    DatabasePoolUnavailable(String),

    /// Database transaction failed
    #[error("[E1005] Database transaction failed: {0}")]
    DatabaseTransactionFailed(String),

    // ========================================================================
    // Configuration Errors (E2001-E2099)
    // ========================================================================
    /// Required environment variable is missing
    #[error("[E2001] Missing environment variable: {0}")]
    MissingEnvVar(String),

    /// Environment variable has invalid value
    #[error("[E2002] Invalid environment variable '{name}': {message}")]
    InvalidEnvVar { name: String, message: String },

    /// Configuration file not found
    #[error("[E2003] Configuration file not found: {0}")]
    ConfigFileNotFound(String),

    /// Configuration file parse error
    #[error("[E2004] Failed to parse configuration: {0}")]
    ConfigParseError(String),

    /// Invalid configuration value
    #[error("[E2005] Invalid configuration value for '{key}': {message}")]
    InvalidConfigValue { key: String, message: String },

    /// Configuration error (generic)
    #[error("[E2006] Configuration error: {0}")]
    Config(String),

    // ========================================================================
    // Agent Errors (E3001-E3099)
    // ========================================================================
    /// Agent not found in the database
    #[error("[E3001] Agent not found: {0}")]
    AgentNotFound(String),

    /// Agent connection failed
    #[error("[E3002] Failed to connect to agent '{agent}': {message}")]
    AgentConnectionFailed { agent: String, message: String },

    /// Agent already exists
    #[error("[E3003] Agent already exists: {0}")]
    AgentAlreadyExists(String),

    /// Agent operation failed
    #[error("[E3004] Agent operation failed for '{agent}': {message}")]
    AgentOperationFailed { agent: String, message: String },

    /// Invalid agent type
    #[error("[E3005] Invalid agent type: {0}")]
    InvalidAgentType(String),

    /// Agent is not responding
    #[error("[E3006] Agent '{0}' is not responding")]
    AgentNotResponding(String),

    // ========================================================================
    // Session Errors (E4001-E4099)
    // ========================================================================
    /// Session not found
    #[error("[E4001] Session not found: {0}")]
    SessionNotFound(String),

    /// Session already exists
    #[error("[E4002] Session already active for agent: {0}")]
    SessionAlreadyActive(String),

    /// Session has already ended
    #[error("[E4003] Session has already ended: {0}")]
    SessionAlreadyEnded(String),

    /// Invalid session status transition
    #[error("[E4004] Invalid session status transition from {from} to {to}")]
    InvalidSessionStatusTransition { from: String, to: String },

    // ========================================================================
    // Sync Errors (API/External Services) (E5001-E5099)
    // ========================================================================
    /// API request failed
    #[error("[E5001] API request failed: {0}")]
    ApiRequestFailed(String),

    /// API response parse error
    #[error("[E5002] Failed to parse API response: {0}")]
    ApiParseError(String),

    /// API rate limit exceeded
    #[error(
        "[E5003] API rate limit exceeded for {service}, retry after {retry_after_secs} seconds"
    )]
    ApiRateLimitExceeded {
        service: String,
        retry_after_secs: u64,
    },

    /// API authentication failed
    #[error("[E5004] API authentication failed for {service}: {message}")]
    ApiAuthenticationFailed { service: String, message: String },

    /// API service unavailable
    #[error("[E5005] API service unavailable: {0}")]
    ApiServiceUnavailable(String),

    /// Sync operation timed out
    #[error("[E5006] Sync operation timed out after {0} seconds")]
    SyncTimeout(u64),

    // ========================================================================
    // Metrics Errors (E6001-E6099)
    // ========================================================================
    /// Metrics collection failed
    #[error("[E6001] Metrics collection failed: {0}")]
    MetricsCollectionFailed(String),

    /// Metrics collector is not running
    #[error("[E6002] Metrics collector is not running")]
    MetricsCollectorNotRunning,

    /// Metrics storage failed
    #[error("[E6003] Failed to store metrics: {0}")]
    MetricsStorageFailed(String),

    // ========================================================================
    // Cost Tracking Errors (E7001-E7099)
    // ========================================================================
    /// Model pricing not found
    #[error("[E7001] Pricing not found for model '{model}' from provider '{provider}'")]
    ModelPricingNotFound { model: String, provider: String },

    /// Invalid cost calculation
    #[error("[E7002] Invalid cost calculation: {0}")]
    InvalidCostCalculation(String),

    // ========================================================================
    // SkillKit Errors (E8001-E8099)
    // ========================================================================
    /// SkillKit operation failed
    #[error("[E8001] SkillKit error: {0}")]
    SkillKit(String),

    /// SkillKit not installed
    #[error("[E8002] SkillKit is not installed: {0}")]
    SkillKitNotInstalled(String),

    /// Skill not found
    #[error("[E8003] Skill not found: {0}")]
    SkillNotFound(String),

    /// Skill already installed
    #[error("[E8004] Skill already installed: {0}")]
    SkillAlreadyInstalled(String),

    /// Skill translation failed
    #[error("[E8005] Skill translation failed from {from} to {to}: {message}")]
    SkillTranslationFailed {
        from: String,
        to: String,
        message: String,
    },

    // ========================================================================
    // General Errors (E9001-E9099)
    // ========================================================================
    /// Internal error (catch-all for unexpected conditions)
    #[error("[E9001] Internal error: {0}")]
    Internal(String),

    /// Operation not supported
    #[error("[E9002] Operation not supported: {0}")]
    NotSupported(String),

    /// Resource already exists
    #[error("[E9003] Resource already exists: {0}")]
    AlreadyExists(String),

    /// Validation error
    #[error("[E9004] Validation error: {0}")]
    ValidationError(String),

    /// IO error
    #[error("[E9005] IO error: {0}")]
    IoError(String),

    /// Serialization/deserialization error
    #[error("[E9006] Serialization error: {0}")]
    SerializationError(String),

    // ========================================================================
    // Plugin Errors (E10001-E10099)
    // ========================================================================
    /// Plugin error (generic)
    #[error("[E10001] Plugin error: {0}")]
    PluginError(String),

    /// Plugin not found
    #[error("[E10002] Plugin not found: {0}")]
    PluginNotFound(String),

    /// Plugin already exists/loaded
    #[error("[E10003] Plugin already loaded: {0}")]
    PluginAlreadyLoaded(String),

    /// Plugin load failed
    #[error("[E10004] Failed to load plugin '{name}': {message}")]
    PluginLoadFailed { name: String, message: String },

    /// Plugin initialization failed
    #[error("[E10005] Failed to initialize plugin '{name}': {message}")]
    PluginInitFailed { name: String, message: String },

    /// Plugin configuration error
    #[error("[E10006] Plugin configuration error for '{name}': {message}")]
    PluginConfigError { name: String, message: String },

    /// Plugin dependency error
    #[error("[E10007] Plugin dependency error for '{plugin}': missing {dependency}")]
    PluginDependencyError { plugin: String, dependency: String },

    /// Plugin permission denied
    #[error("[E10008] Plugin '{name}' requires permission: {permission}")]
    PluginPermissionDenied { name: String, permission: String },

    /// Plugin conflict
    #[error("[E10009] Plugin conflict: {0} and {1} both provide capability {2}")]
    PluginConflict(String, String, String),

    // ========================================================================
    // Hook Errors (E11001-E11099)
    // ========================================================================
    /// Hook error (generic)
    #[error("[E11001] Hook error: {0}")]
    HookError(String),

    /// Hook handler not found
    #[error("[E11002] Hook handler not found: {0}")]
    HookHandlerNotFound(String),

    /// Hook execution failed
    #[error("[E11003] Hook execution failed for '{hook}': {message}")]
    HookExecutionFailed { hook: String, message: String },

    /// Hook timeout
    #[error("[E11004] Hook '{0}' timed out after {1} seconds")]
    HookTimeout(String, u64),

    /// Hook aborted
    #[error("[E11005] Hook '{0}' was aborted: {1}")]
    HookAborted(String, String),
}

// Legacy compatibility: Create DatabaseConnectionFailed from string
impl RimuruError {
    /// Create a database connection error from a string message.
    pub fn database_connection_failed(message: impl Into<String>) -> Self {
        RimuruError::DatabaseConnectionFailed {
            message: message.into(),
            source: None,
        }
    }

    /// Create a database connection error with a source error.
    pub fn database_connection_failed_with_source<E>(message: impl Into<String>, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        RimuruError::DatabaseConnectionFailed {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}

/// Result type alias for Rimuru operations.
pub type RimuruResult<T> = Result<T, RimuruError>;

// ============================================================================
// From trait implementations for seamless error propagation
// ============================================================================

impl From<sqlx::Error> for RimuruError {
    fn from(err: sqlx::Error) -> Self {
        match &err {
            sqlx::Error::PoolTimedOut => RimuruError::DatabasePoolUnavailable(err.to_string()),
            sqlx::Error::PoolClosed => {
                RimuruError::DatabasePoolUnavailable("Connection pool is closed".to_string())
            }
            sqlx::Error::RowNotFound => {
                RimuruError::DatabaseQueryFailed("Row not found".to_string())
            }
            sqlx::Error::Configuration(_) => {
                RimuruError::database_connection_failed(err.to_string())
            }
            sqlx::Error::Database(db_err) => RimuruError::DatabaseQueryFailed(db_err.to_string()),
            _ => RimuruError::DatabaseQueryFailed(err.to_string()),
        }
    }
}

impl From<sqlx::migrate::MigrateError> for RimuruError {
    fn from(err: sqlx::migrate::MigrateError) -> Self {
        RimuruError::DatabaseMigrationFailed(err.to_string())
    }
}

impl From<std::env::VarError> for RimuruError {
    fn from(err: std::env::VarError) -> Self {
        match err {
            std::env::VarError::NotPresent => {
                RimuruError::MissingEnvVar("(unspecified)".to_string())
            }
            std::env::VarError::NotUnicode(_) => RimuruError::InvalidEnvVar {
                name: "(unspecified)".to_string(),
                message: "Value is not valid Unicode".to_string(),
            },
        }
    }
}

impl From<reqwest::Error> for RimuruError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            RimuruError::SyncTimeout(30)
        } else if err.is_connect() {
            RimuruError::ApiServiceUnavailable(err.to_string())
        } else if err.is_status() {
            if let Some(status) = err.status() {
                if status.as_u16() == 429 {
                    return RimuruError::ApiRateLimitExceeded {
                        service: err
                            .url()
                            .map(|u| u.host_str().unwrap_or("unknown").to_string())
                            .unwrap_or_else(|| "unknown".to_string()),
                        retry_after_secs: 60,
                    };
                } else if status.as_u16() == 401 || status.as_u16() == 403 {
                    return RimuruError::ApiAuthenticationFailed {
                        service: err
                            .url()
                            .map(|u| u.host_str().unwrap_or("unknown").to_string())
                            .unwrap_or_else(|| "unknown".to_string()),
                        message: status.to_string(),
                    };
                }
            }
            RimuruError::ApiRequestFailed(err.to_string())
        } else if err.is_decode() {
            RimuruError::ApiParseError(err.to_string())
        } else {
            RimuruError::ApiRequestFailed(err.to_string())
        }
    }
}

impl From<serde_json::Error> for RimuruError {
    fn from(err: serde_json::Error) -> Self {
        RimuruError::SerializationError(err.to_string())
    }
}

impl From<std::io::Error> for RimuruError {
    fn from(err: std::io::Error) -> Self {
        RimuruError::IoError(err.to_string())
    }
}

impl From<config::ConfigError> for RimuruError {
    fn from(err: config::ConfigError) -> Self {
        match err {
            config::ConfigError::NotFound(key) => RimuruError::InvalidConfigValue {
                key,
                message: "Key not found".to_string(),
            },
            config::ConfigError::FileParse { uri, cause } => RimuruError::ConfigParseError(
                format!("Failed to parse {}: {}", uri.unwrap_or_default(), cause),
            ),
            config::ConfigError::Type {
                origin,
                unexpected,
                expected,
                key,
            } => RimuruError::InvalidConfigValue {
                key: key.unwrap_or_else(|| origin.map(|o| o.to_string()).unwrap_or_default()),
                message: format!("Expected {}, got {}", expected, unexpected),
            },
            _ => RimuruError::ConfigParseError(err.to_string()),
        }
    }
}

impl From<crate::db::DatabaseError> for RimuruError {
    fn from(err: crate::db::DatabaseError) -> Self {
        match err {
            crate::db::DatabaseError::MissingEnvVar(name) => RimuruError::MissingEnvVar(name),
            crate::db::DatabaseError::ConnectionFailed(e) => {
                RimuruError::database_connection_failed(e.to_string())
            }
            crate::db::DatabaseError::MigrationFailed(e) => {
                RimuruError::DatabaseMigrationFailed(e.to_string())
            }
            crate::db::DatabaseError::InvalidConfig(msg) => RimuruError::InvalidConfigValue {
                key: "database".to_string(),
                message: msg,
            },
        }
    }
}

// ============================================================================
// Error categorization helpers
// ============================================================================

impl RimuruError {
    /// Returns true if this error is related to database operations.
    pub fn is_database_error(&self) -> bool {
        matches!(
            self,
            RimuruError::DatabaseConnectionFailed { .. }
                | RimuruError::DatabaseQueryFailed(_)
                | RimuruError::DatabaseMigrationFailed(_)
                | RimuruError::DatabasePoolUnavailable(_)
                | RimuruError::DatabaseTransactionFailed(_)
        )
    }

    /// Returns true if this error is related to configuration.
    pub fn is_config_error(&self) -> bool {
        matches!(
            self,
            RimuruError::MissingEnvVar(_)
                | RimuruError::InvalidEnvVar { .. }
                | RimuruError::ConfigFileNotFound(_)
                | RimuruError::ConfigParseError(_)
                | RimuruError::InvalidConfigValue { .. }
                | RimuruError::Config(_)
        )
    }

    /// Returns true if this error is related to agent operations.
    pub fn is_agent_error(&self) -> bool {
        matches!(
            self,
            RimuruError::AgentNotFound(_)
                | RimuruError::AgentConnectionFailed { .. }
                | RimuruError::AgentAlreadyExists(_)
                | RimuruError::AgentOperationFailed { .. }
                | RimuruError::InvalidAgentType(_)
                | RimuruError::AgentNotResponding(_)
        )
    }

    /// Returns true if this error is related to API/sync operations.
    pub fn is_sync_error(&self) -> bool {
        matches!(
            self,
            RimuruError::ApiRequestFailed(_)
                | RimuruError::ApiParseError(_)
                | RimuruError::ApiRateLimitExceeded { .. }
                | RimuruError::ApiAuthenticationFailed { .. }
                | RimuruError::ApiServiceUnavailable(_)
                | RimuruError::SyncTimeout(_)
        )
    }

    /// Returns true if this error is transient and the operation might succeed on retry.
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            RimuruError::DatabasePoolUnavailable(_)
                | RimuruError::ApiRateLimitExceeded { .. }
                | RimuruError::ApiServiceUnavailable(_)
                | RimuruError::SyncTimeout(_)
                | RimuruError::AgentNotResponding(_)
                | RimuruError::DatabaseConnectionFailed { .. }
        )
    }

    /// Returns a suggested retry delay in seconds if the error is transient.
    pub fn suggested_retry_delay(&self) -> Option<u64> {
        match self {
            RimuruError::ApiRateLimitExceeded {
                retry_after_secs, ..
            } => Some(*retry_after_secs),
            RimuruError::DatabasePoolUnavailable(_) => Some(1),
            RimuruError::DatabaseConnectionFailed { .. } => Some(2),
            RimuruError::ApiServiceUnavailable(_) => Some(5),
            RimuruError::SyncTimeout(_) => Some(10),
            RimuruError::AgentNotResponding(_) => Some(2),
            _ => None,
        }
    }

    /// Returns an error code suitable for logging or external reporting.
    pub fn error_code(&self) -> &'static str {
        match self {
            RimuruError::DatabaseConnectionFailed { .. } => "E1001",
            RimuruError::DatabaseQueryFailed(_) => "E1002",
            RimuruError::DatabaseMigrationFailed(_) => "E1003",
            RimuruError::DatabasePoolUnavailable(_) => "E1004",
            RimuruError::DatabaseTransactionFailed(_) => "E1005",
            RimuruError::MissingEnvVar(_) => "E2001",
            RimuruError::InvalidEnvVar { .. } => "E2002",
            RimuruError::ConfigFileNotFound(_) => "E2003",
            RimuruError::ConfigParseError(_) => "E2004",
            RimuruError::InvalidConfigValue { .. } => "E2005",
            RimuruError::Config(_) => "E2006",
            RimuruError::AgentNotFound(_) => "E3001",
            RimuruError::AgentConnectionFailed { .. } => "E3002",
            RimuruError::AgentAlreadyExists(_) => "E3003",
            RimuruError::AgentOperationFailed { .. } => "E3004",
            RimuruError::InvalidAgentType(_) => "E3005",
            RimuruError::AgentNotResponding(_) => "E3006",
            RimuruError::SessionNotFound(_) => "E4001",
            RimuruError::SessionAlreadyActive(_) => "E4002",
            RimuruError::SessionAlreadyEnded(_) => "E4003",
            RimuruError::InvalidSessionStatusTransition { .. } => "E4004",
            RimuruError::ApiRequestFailed(_) => "E5001",
            RimuruError::ApiParseError(_) => "E5002",
            RimuruError::ApiRateLimitExceeded { .. } => "E5003",
            RimuruError::ApiAuthenticationFailed { .. } => "E5004",
            RimuruError::ApiServiceUnavailable(_) => "E5005",
            RimuruError::SyncTimeout(_) => "E5006",
            RimuruError::MetricsCollectionFailed(_) => "E6001",
            RimuruError::MetricsCollectorNotRunning => "E6002",
            RimuruError::MetricsStorageFailed(_) => "E6003",
            RimuruError::ModelPricingNotFound { .. } => "E7001",
            RimuruError::InvalidCostCalculation(_) => "E7002",
            RimuruError::SkillKit(_) => "E8001",
            RimuruError::SkillKitNotInstalled(_) => "E8002",
            RimuruError::SkillNotFound(_) => "E8003",
            RimuruError::SkillAlreadyInstalled(_) => "E8004",
            RimuruError::SkillTranslationFailed { .. } => "E8005",
            RimuruError::Internal(_) => "E9001",
            RimuruError::NotSupported(_) => "E9002",
            RimuruError::AlreadyExists(_) => "E9003",
            RimuruError::ValidationError(_) => "E9004",
            RimuruError::IoError(_) => "E9005",
            RimuruError::SerializationError(_) => "E9006",
            RimuruError::PluginError(_) => "E10001",
            RimuruError::PluginNotFound(_) => "E10002",
            RimuruError::PluginAlreadyLoaded(_) => "E10003",
            RimuruError::PluginLoadFailed { .. } => "E10004",
            RimuruError::PluginInitFailed { .. } => "E10005",
            RimuruError::PluginConfigError { .. } => "E10006",
            RimuruError::PluginDependencyError { .. } => "E10007",
            RimuruError::PluginPermissionDenied { .. } => "E10008",
            RimuruError::PluginConflict(_, _, _) => "E10009",
            RimuruError::HookError(_) => "E11001",
            RimuruError::HookHandlerNotFound(_) => "E11002",
            RimuruError::HookExecutionFailed { .. } => "E11003",
            RimuruError::HookTimeout(_, _) => "E11004",
            RimuruError::HookAborted(_, _) => "E11005",
        }
    }

    /// Returns a user-friendly suggestion for how to resolve this error.
    pub fn user_suggestion(&self) -> Option<&'static str> {
        match self {
            RimuruError::DatabaseConnectionFailed { .. } => {
                Some("Check that PostgreSQL is running and DATABASE_URL is correct")
            }
            RimuruError::DatabasePoolUnavailable(_) => {
                Some("The database is busy. Try again in a few seconds")
            }
            RimuruError::MissingEnvVar(_) => {
                Some("Create a .env file or set the environment variable")
            }
            RimuruError::ConfigFileNotFound(_) => {
                Some("Run 'rimuru init' to create the configuration file")
            }
            RimuruError::AgentNotFound(_) => {
                Some("Run 'rimuru agents scan' to discover installed agents")
            }
            RimuruError::AgentNotResponding(_) => {
                Some("Check that the agent is running and try 'rimuru agents reconnect'")
            }
            RimuruError::ApiRateLimitExceeded { .. } => {
                Some("Wait for the rate limit to reset or use a different API key")
            }
            RimuruError::ApiAuthenticationFailed { .. } => {
                Some("Check your API key in the configuration")
            }
            RimuruError::SkillKitNotInstalled(_) => {
                Some("Install SkillKit: npm install -g skillkit")
            }
            RimuruError::PluginNotFound(_) => {
                Some("Run 'rimuru plugins list --available' to see available plugins")
            }
            _ => None,
        }
    }

    /// Returns the recommended retry configuration for this error type.
    pub fn retry_config(&self) -> Option<RetryConfig> {
        if !self.is_transient() {
            return None;
        }

        Some(match self {
            RimuruError::DatabaseConnectionFailed { .. }
            | RimuruError::DatabasePoolUnavailable(_) => RetryConfig::for_database(),
            RimuruError::ApiRateLimitExceeded { .. }
            | RimuruError::ApiServiceUnavailable(_)
            | RimuruError::SyncTimeout(_) => RetryConfig::for_api(),
            RimuruError::AgentNotResponding(_) => RetryConfig::for_agent_reconnection(),
            _ => RetryConfig::default(),
        })
    }

    pub fn is_skillkit_error(&self) -> bool {
        matches!(
            self,
            RimuruError::SkillKit(_)
                | RimuruError::SkillKitNotInstalled(_)
                | RimuruError::SkillNotFound(_)
                | RimuruError::SkillAlreadyInstalled(_)
                | RimuruError::SkillTranslationFailed { .. }
        )
    }

    /// Returns true if this error is related to plugin operations.
    pub fn is_plugin_error(&self) -> bool {
        matches!(
            self,
            RimuruError::PluginError(_)
                | RimuruError::PluginNotFound(_)
                | RimuruError::PluginAlreadyLoaded(_)
                | RimuruError::PluginLoadFailed { .. }
                | RimuruError::PluginInitFailed { .. }
                | RimuruError::PluginConfigError { .. }
                | RimuruError::PluginDependencyError { .. }
                | RimuruError::PluginPermissionDenied { .. }
                | RimuruError::PluginConflict(_, _, _)
        )
    }

    /// Returns true if this error is related to hook operations.
    pub fn is_hook_error(&self) -> bool {
        matches!(
            self,
            RimuruError::HookError(_)
                | RimuruError::HookHandlerNotFound(_)
                | RimuruError::HookExecutionFailed { .. }
                | RimuruError::HookTimeout(_, _)
                | RimuruError::HookAborted(_, _)
        )
    }

    /// Create a plugin error
    pub fn plugin(message: impl Into<String>) -> Self {
        RimuruError::PluginError(message.into())
    }

    /// Create a hook error
    pub fn hook(message: impl Into<String>) -> Self {
        RimuruError::HookError(message.into())
    }

    /// Log this error with appropriate severity level.
    pub fn log(&self) {
        let code = self.error_code();
        let suggestion = self.user_suggestion();

        if self.is_transient() {
            warn!(
                error_code = %code,
                suggestion = suggestion,
                "Transient error occurred: {}",
                self
            );
        } else {
            error!(
                error_code = %code,
                suggestion = suggestion,
                "Error occurred: {}",
                self
            );
        }
    }

    /// Log this error with context information.
    pub fn log_with_context(&self, context: &ErrorContext) {
        let code = self.error_code();
        let suggestion = self.user_suggestion();

        if self.is_transient() {
            warn!(
                error_code = %code,
                location = %context,
                suggestion = suggestion,
                "Transient error at {}: {}",
                context,
                self
            );
        } else {
            error!(
                error_code = %code,
                location = %context,
                suggestion = suggestion,
                "Error at {}: {}",
                context,
                self
            );
        }
    }
}

// ============================================================================
// Retry utilities
// ============================================================================

/// Execute an async operation with retry logic based on error configuration.
///
/// # Example
/// ```ignore
/// use rimuru_core::error::retry_async;
///
/// let result = retry_async(|| async {
///     database.connect().await
/// }).await;
/// ```
pub async fn retry_async<F, Fut, T>(operation: F) -> RimuruResult<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = RimuruResult<T>>,
{
    retry_async_with_config(operation, RetryConfig::default()).await
}

/// Execute an async operation with custom retry configuration.
pub async fn retry_async_with_config<F, Fut, T>(
    operation: F,
    config: RetryConfig,
) -> RimuruResult<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = RimuruResult<T>>,
{
    let mut last_error = None;

    for attempt in 0..config.max_attempts {
        match operation().await {
            Ok(result) => {
                if attempt > 0 {
                    info!(
                        "Operation succeeded on attempt {} after {} retries",
                        attempt + 1,
                        attempt
                    );
                }
                return Ok(result);
            }
            Err(e) => {
                if !e.is_transient() || attempt == config.max_attempts - 1 {
                    e.log();
                    return Err(e);
                }

                let delay = config.delay_for_attempt(attempt);
                warn!(
                    "Attempt {} failed ({}), retrying in {:?}",
                    attempt + 1,
                    e,
                    delay
                );

                tokio::time::sleep(delay).await;
                last_error = Some(e);
            }
        }
    }

    Err(last_error
        .unwrap_or_else(|| RimuruError::Internal("Retry loop exhausted without error".to_string())))
}

// ============================================================================
// User-friendly error formatting for CLI
// ============================================================================

/// Format an error for CLI display with colors and suggestions.
pub struct CliErrorDisplay<'a> {
    error: &'a RimuruError,
    show_code: bool,
    show_suggestion: bool,
}

impl<'a> CliErrorDisplay<'a> {
    pub fn new(error: &'a RimuruError) -> Self {
        Self {
            error,
            show_code: true,
            show_suggestion: true,
        }
    }

    pub fn without_code(mut self) -> Self {
        self.show_code = false;
        self
    }

    pub fn without_suggestion(mut self) -> Self {
        self.show_suggestion = false;
        self
    }
}

impl<'a> fmt::Display for CliErrorDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Main error message (already includes code)
        writeln!(f, "{}", self.error)?;

        // Add suggestion if available
        if self.show_suggestion {
            if let Some(suggestion) = self.error.user_suggestion() {
                writeln!(f)?;
                writeln!(f, "  Suggestion: {}", suggestion)?;
            }
        }

        // Add retry hint for transient errors
        if self.error.is_transient() {
            if let Some(delay) = self.error.suggested_retry_delay() {
                writeln!(f)?;
                writeln!(
                    f,
                    "  This error may be temporary. Try again in {} seconds.",
                    delay
                )?;
            }
        }

        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = RimuruError::MissingEnvVar("DATABASE_URL".to_string());
        assert!(err.to_string().contains("E2001"));
        assert!(err.to_string().contains("DATABASE_URL"));

        let err = RimuruError::AgentConnectionFailed {
            agent: "ClaudeCode".to_string(),
            message: "Connection refused".to_string(),
        };
        assert!(err.to_string().contains("E3002"));
        assert!(err.to_string().contains("ClaudeCode"));
    }

    #[test]
    fn test_error_categorization() {
        let db_err = RimuruError::database_connection_failed("timeout");
        assert!(db_err.is_database_error());
        assert!(!db_err.is_config_error());
        assert!(!db_err.is_agent_error());
        assert!(!db_err.is_sync_error());

        let config_err = RimuruError::MissingEnvVar("API_KEY".to_string());
        assert!(!config_err.is_database_error());
        assert!(config_err.is_config_error());

        let agent_err = RimuruError::AgentNotFound("test-agent".to_string());
        assert!(agent_err.is_agent_error());

        let sync_err = RimuruError::ApiRequestFailed("network error".to_string());
        assert!(sync_err.is_sync_error());
    }

    #[test]
    fn test_is_transient() {
        assert!(RimuruError::DatabasePoolUnavailable("timeout".to_string()).is_transient());
        assert!(RimuruError::database_connection_failed("connection refused").is_transient());
        assert!(RimuruError::ApiRateLimitExceeded {
            service: "api".to_string(),
            retry_after_secs: 60,
        }
        .is_transient());
        assert!(RimuruError::ApiServiceUnavailable("503".to_string()).is_transient());
        assert!(RimuruError::SyncTimeout(30).is_transient());
        assert!(RimuruError::AgentNotResponding("agent".to_string()).is_transient());

        assert!(!RimuruError::MissingEnvVar("KEY".to_string()).is_transient());
        assert!(!RimuruError::AgentNotFound("agent".to_string()).is_transient());
    }

    #[test]
    fn test_suggested_retry_delay() {
        let err = RimuruError::ApiRateLimitExceeded {
            service: "api".to_string(),
            retry_after_secs: 120,
        };
        assert_eq!(err.suggested_retry_delay(), Some(120));

        let err = RimuruError::DatabasePoolUnavailable("timeout".to_string());
        assert_eq!(err.suggested_retry_delay(), Some(1));

        let err = RimuruError::database_connection_failed("refused");
        assert_eq!(err.suggested_retry_delay(), Some(2));

        let err = RimuruError::MissingEnvVar("KEY".to_string());
        assert_eq!(err.suggested_retry_delay(), None);
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(
            RimuruError::database_connection_failed("err").error_code(),
            "E1001"
        );
        assert_eq!(
            RimuruError::MissingEnvVar("KEY".to_string()).error_code(),
            "E2001"
        );
        assert_eq!(
            RimuruError::AgentNotFound("agent".to_string()).error_code(),
            "E3001"
        );
        assert_eq!(
            RimuruError::SessionNotFound("session".to_string()).error_code(),
            "E4001"
        );
        assert_eq!(
            RimuruError::ApiRequestFailed("err".to_string()).error_code(),
            "E5001"
        );
        assert_eq!(
            RimuruError::MetricsCollectionFailed("err".to_string()).error_code(),
            "E6001"
        );
        assert_eq!(
            RimuruError::ModelPricingNotFound {
                model: "gpt-4".to_string(),
                provider: "openai".to_string()
            }
            .error_code(),
            "E7001"
        );
        assert_eq!(
            RimuruError::Internal("err".to_string()).error_code(),
            "E9001"
        );
    }

    #[test]
    fn test_user_suggestions() {
        assert!(RimuruError::database_connection_failed("err")
            .user_suggestion()
            .is_some());
        assert!(RimuruError::MissingEnvVar("KEY".to_string())
            .user_suggestion()
            .is_some());
        assert!(RimuruError::AgentNotFound("agent".to_string())
            .user_suggestion()
            .is_some());
        assert!(RimuruError::SkillKitNotInstalled("err".to_string())
            .user_suggestion()
            .is_some());

        // Some errors may not have suggestions
        assert!(RimuruError::Internal("err".to_string())
            .user_suggestion()
            .is_none());
    }

    #[test]
    fn test_retry_config() {
        let db_config = RetryConfig::for_database();
        assert_eq!(db_config.max_attempts, 5);

        let api_config = RetryConfig::for_api();
        assert_eq!(api_config.max_attempts, 3);

        let agent_config = RetryConfig::for_agent_reconnection();
        assert_eq!(agent_config.max_attempts, 10);
    }

    #[test]
    fn test_retry_delay_calculation() {
        let config = RetryConfig {
            max_attempts: 5,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            jitter: false,
        };

        let delay0 = config.delay_for_attempt(0);
        assert_eq!(delay0, Duration::from_millis(100));

        let delay1 = config.delay_for_attempt(1);
        assert_eq!(delay1, Duration::from_millis(200));

        let delay2 = config.delay_for_attempt(2);
        assert_eq!(delay2, Duration::from_millis(400));
    }

    #[test]
    fn test_error_context() {
        let ctx = ErrorContext::new("src/main.rs", 42, 10);
        assert_eq!(ctx.to_string(), "src/main.rs:42:10");

        let ctx_with_op = ctx.with_operation("database_connect");
        assert_eq!(
            ctx_with_op.to_string(),
            "src/main.rs:42:10 (database_connect)"
        );
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let rimuru_err: RimuruError = io_err.into();
        assert!(matches!(rimuru_err, RimuruError::IoError(_)));
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_result: Result<serde_json::Value, _> = serde_json::from_str("invalid json");
        let json_err = json_result.unwrap_err();
        let rimuru_err: RimuruError = json_err.into();
        assert!(matches!(rimuru_err, RimuruError::SerializationError(_)));
    }

    #[test]
    fn test_cli_error_display() {
        let err = RimuruError::MissingEnvVar("DATABASE_URL".to_string());
        let display = CliErrorDisplay::new(&err);
        let output = display.to_string();

        assert!(output.contains("DATABASE_URL"));
        assert!(output.contains("Suggestion"));
    }
}
