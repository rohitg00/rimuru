---
type: reference
title: API Reference
created: 2026-02-05
tags:
  - api
  - reference
  - developer-guide
related:
  - "[[architecture]]"
  - "[[creating-adapters]]"
  - "[[creating-plugins]]"
---

# API Reference

This document provides a reference for the core Rimuru library APIs.

## Core Types

### AgentType

Enum representing supported AI coding agents.

```rust
pub enum AgentType {
    ClaudeCode,
    OpenCode,
    Codex,
    Copilot,
    Goose,
    Cursor,
}

impl AgentType {
    pub fn display_name(&self) -> &str;
    pub fn icon(&self) -> &str;
    pub fn type_id(&self) -> &str;
}
```

### Session

Represents a coding session with an agent.

```rust
pub struct Session {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub status: SessionStatus,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

impl Session {
    pub fn new(agent_id: Uuid) -> Self;
    pub fn duration(&self) -> Option<Duration>;
    pub fn is_active(&self) -> bool;
    pub fn complete(&mut self);
    pub fn cancel(&mut self);
}
```

### SessionStatus

```rust
pub enum SessionStatus {
    Active,
    Completed,
    Cancelled,
    Failed,
}
```

### CostRecord

Token usage and cost for a session.

```rust
pub struct CostRecord {
    pub id: Uuid,
    pub session_id: Uuid,
    pub agent_id: Uuid,
    pub model_name: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cost_usd: f64,
    pub recorded_at: DateTime<Utc>,
}

impl CostRecord {
    pub fn new(
        session_id: Uuid,
        agent_id: Uuid,
        model_name: String,
        input_tokens: i64,
        output_tokens: i64,
        cost_usd: f64,
    ) -> Self;

    pub fn total_tokens(&self) -> i64;
}
```

### ModelInfo

Model pricing and capabilities.

```rust
pub struct ModelInfo {
    pub provider: String,
    pub model_name: String,
    pub input_cost_per_1k: f64,
    pub output_cost_per_1k: f64,
    pub context_window: i32,
    pub capabilities: Vec<String>,
    pub updated_at: DateTime<Utc>,
}

impl ModelInfo {
    pub fn new(
        provider: String,
        model_name: String,
        input_cost: f64,
        output_cost: f64,
        context_window: i32,
    ) -> Self;

    pub fn calculate_cost(&self, input_tokens: i64, output_tokens: i64) -> f64;
}
```

### SystemMetrics

System resource metrics.

```rust
pub struct SystemMetrics {
    pub timestamp: DateTime<Utc>,
    pub cpu_percent: f32,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub active_sessions: i32,
}

impl SystemMetrics {
    pub fn memory_percent(&self) -> f32;
}
```

## Adapter Traits

### AgentAdapter

Core trait for agent adapters.

```rust
#[async_trait]
pub trait AgentAdapter: Send + Sync {
    /// Returns the type of agent this adapter supports
    fn agent_type(&self) -> AgentType;

    /// Returns the adapter's name
    fn name(&self) -> &str;

    /// Establish connection to the agent
    async fn connect(&mut self) -> RimuruResult<()>;

    /// Disconnect from the agent
    async fn disconnect(&mut self) -> RimuruResult<()>;

    /// Get current adapter status
    async fn get_status(&self) -> AdapterStatus;

    /// Get adapter information
    async fn get_info(&self) -> RimuruResult<AdapterInfo>;

    /// Get all sessions from the agent
    async fn get_sessions(&self) -> RimuruResult<Vec<Session>>;

    /// Get the currently active session, if any
    async fn get_active_session(&self) -> RimuruResult<Option<ActiveSession>>;

    /// Check if the agent is installed
    async fn is_installed(&self) -> bool;

    /// Perform a health check
    async fn health_check(&self) -> RimuruResult<bool>;
}
```

### CostTracker

Track token usage and calculate costs.

```rust
#[async_trait]
pub trait CostTracker: Send + Sync {
    /// Get usage statistics since a given time
    async fn get_usage(&self, since: Option<DateTime<Utc>>) -> RimuruResult<UsageStats>;

    /// Calculate cost for given token usage
    async fn calculate_cost(
        &self,
        input_tokens: i64,
        output_tokens: i64,
        model_name: &str,
    ) -> RimuruResult<f64>;

    /// Get information about a model
    async fn get_model_info(&self, model_name: &str) -> RimuruResult<Option<ModelInfo>>;

    /// List supported models
    async fn get_supported_models(&self) -> RimuruResult<Vec<String>>;

    /// Get total cost since a given time
    async fn get_total_cost(&self, since: Option<DateTime<Utc>>) -> RimuruResult<f64>;
}
```

### SessionMonitor

Monitor session events.

```rust
pub type SessionEventCallback = Box<dyn Fn(SessionEvent) + Send + Sync>;

#[async_trait]
pub trait SessionMonitor: Send + Sync {
    /// Subscribe to session events
    async fn subscribe_to_events(&self, callback: SessionEventCallback) -> RimuruResult<Uuid>;

    /// Unsubscribe from events
    async fn unsubscribe(&self, subscription_id: Uuid) -> RimuruResult<()>;

    /// Get session history
    async fn get_session_history(
        &self,
        limit: Option<usize>,
        since: Option<DateTime<Utc>>,
    ) -> RimuruResult<Vec<SessionHistory>>;

    /// Get details for a specific session
    async fn get_session_details(&self, session_id: Uuid) -> RimuruResult<Option<SessionHistory>>;

    /// Get all active sessions
    async fn get_active_sessions(&self) -> RimuruResult<Vec<ActiveSession>>;
}
```

### FullAdapter

Combines all adapter traits.

```rust
pub trait FullAdapter: AgentAdapter + CostTracker + SessionMonitor {}

// Automatically implemented for types that implement all three traits
impl<T> FullAdapter for T where T: AgentAdapter + CostTracker + SessionMonitor {}
```

## Repository Pattern

### Repository Trait

Generic repository interface.

```rust
#[async_trait]
pub trait Repository<T, ID> {
    async fn find_by_id(&self, id: ID) -> RimuruResult<Option<T>>;
    async fn find_all(&self) -> RimuruResult<Vec<T>>;
    async fn save(&self, entity: T) -> RimuruResult<T>;
    async fn delete(&self, id: ID) -> RimuruResult<bool>;
}
```

### AgentRepository

```rust
pub struct AgentRepository {
    pool: PgPool,
}

impl AgentRepository {
    pub fn new(pool: PgPool) -> Self;
    pub async fn find_by_type(&self, agent_type: AgentType) -> RimuruResult<Vec<Agent>>;
    pub async fn find_active(&self) -> RimuruResult<Vec<Agent>>;
    pub async fn count(&self) -> RimuruResult<i64>;
}
```

### SessionRepository

```rust
pub struct SessionRepository {
    pool: PgPool,
}

impl SessionRepository {
    pub fn new(pool: PgPool) -> Self;
    pub async fn find_by_agent(&self, agent_id: Uuid) -> RimuruResult<Vec<Session>>;
    pub async fn find_active(&self) -> RimuruResult<Vec<Session>>;
    pub async fn find_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> RimuruResult<Vec<Session>>;
    pub async fn get_active_count(&self) -> RimuruResult<i64>;
}
```

### CostRepository

```rust
pub struct CostRepository {
    pool: PgPool,
}

impl CostRepository {
    pub fn new(pool: PgPool) -> Self;
    pub async fn find_by_session(&self, session_id: Uuid) -> RimuruResult<Vec<CostRecord>>;
    pub async fn find_by_agent(&self, agent_id: Uuid) -> RimuruResult<Vec<CostRecord>>;
    pub async fn find_by_model(&self, model_name: &str) -> RimuruResult<Vec<CostRecord>>;
    pub async fn sum_costs_since(&self, since: DateTime<Utc>) -> RimuruResult<f64>;
    pub async fn aggregate_by_model(
        &self,
        since: Option<DateTime<Utc>>,
    ) -> RimuruResult<Vec<CostSummary>>;
}
```

## Services

### AdapterManager

Manages adapter lifecycle.

```rust
pub struct AdapterManager {
    adapters: HashMap<AgentType, Box<dyn FullAdapter>>,
    config: AdapterManagerConfig,
}

impl AdapterManager {
    pub fn new(config: AdapterManagerConfig) -> Self;

    /// Register an adapter
    pub fn register(&mut self, adapter: Box<dyn FullAdapter>);

    /// Get an adapter by type
    pub fn get(&self, agent_type: AgentType) -> Option<&dyn FullAdapter>;

    /// Connect all registered adapters
    pub async fn connect_all(&mut self) -> RimuruResult<()>;

    /// Disconnect all adapters
    pub async fn disconnect_all(&mut self) -> RimuruResult<()>;

    /// Get health status of all adapters
    pub async fn health_check_all(&self) -> HashMap<AgentType, AdapterHealth>;
}
```

### CostAggregator

Aggregate costs across agents.

```rust
pub struct CostAggregator {
    repo: CostRepository,
}

impl CostAggregator {
    pub fn new(repo: CostRepository) -> Self;

    /// Get cost breakdown by filter
    pub async fn get_breakdown(&self, filter: CostFilter) -> RimuruResult<CostBreakdown>;

    /// Get aggregated cost report
    pub async fn get_report(&self, range: TimeRange) -> RimuruResult<AggregatedCostReport>;

    /// Get costs grouped by model
    pub async fn by_model(&self, range: TimeRange) -> RimuruResult<Vec<CostSummary>>;

    /// Get costs grouped by agent
    pub async fn by_agent(&self, range: TimeRange) -> RimuruResult<Vec<CostSummary>>;
}
```

## Plugin Types

### Plugin Trait

Base trait for all plugins.

```rust
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Get plugin information
    fn info(&self) -> &PluginInfo;

    /// Initialize the plugin
    async fn initialize(&mut self, ctx: &PluginContext) -> RimuruResult<()>;

    /// Shutdown the plugin
    async fn shutdown(&mut self) -> RimuruResult<()>;

    /// Get current plugin state
    fn state(&self) -> PluginState;
}
```

### ExporterPlugin

Export data to custom formats.

```rust
#[async_trait]
pub trait ExporterPlugin: Plugin {
    /// Export format name
    fn format(&self) -> &str;

    /// File extension for exports
    fn file_extension(&self) -> &str;

    /// Export sessions to bytes
    async fn export_sessions(
        &self,
        sessions: &[Session],
        options: ExportOptions,
    ) -> RimuruResult<Vec<u8>>;

    /// Export costs to bytes
    async fn export_costs(
        &self,
        costs: &[CostRecord],
        options: ExportOptions,
    ) -> RimuruResult<Vec<u8>>;
}
```

### NotifierPlugin

Send notifications.

```rust
#[async_trait]
pub trait NotifierPlugin: Plugin {
    /// Send a notification
    async fn send(&self, notification: Notification) -> RimuruResult<()>;

    /// List supported notification levels
    fn supported_levels(&self) -> Vec<NotificationLevel>;
}
```

### ViewPlugin

Custom UI views.

```rust
#[async_trait]
pub trait ViewPlugin: Plugin {
    /// View identifier
    fn view_name(&self) -> &str;

    /// Display title
    fn view_title(&self) -> &str;

    /// Render the view
    async fn render(&self, ctx: &ViewContext) -> RimuruResult<ViewOutput>;

    /// Handle user action
    async fn handle_action(&self, action: &str, input: ViewInput) -> RimuruResult<ViewOutput>;
}
```

## Error Handling

### RimuruError

Main error type.

```rust
#[derive(Debug, Error)]
pub enum RimuruError {
    #[error("[E1001] Database connection failed: {0}")]
    Database(String),

    #[error("[E1002] Database query failed: {0}")]
    DatabaseQuery(String),

    #[error("[E2001] Agent adapter error: {0}")]
    Adapter(String),

    #[error("[E2002] Agent not found: {0}")]
    AgentNotFound(String),

    #[error("[E3001] Configuration error: {0}")]
    Config(String),

    #[error("[E4001] Plugin error: {0}")]
    Plugin(String),

    #[error("[E5001] Sync error: {0}")]
    Sync(String),

    #[error("[E6001] Not found: {0} with id {1}")]
    NotFound(String, String),

    #[error("[E7001] Validation error: {0}")]
    Validation(String),

    #[error("[E8001] IO error: {0}")]
    Io(String),

    #[error("[E9001] HTTP error: {0}")]
    Http(String),
}
```

### Error Context

Add context to errors.

```rust
pub struct ErrorContext {
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
    pub operation: String,
}

// Use the macro to create context
let ctx = error_context!("loading configuration");

// Add context to error
let result = operation()
    .map_err(|e| RimuruError::with_context(e, ctx))?;
```

### Retry Support

Automatic retry for transient failures.

```rust
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
}

impl RetryConfig {
    pub fn database() -> Self;  // Preset for database operations
    pub fn api() -> Self;       // Preset for API calls
    pub fn agent() -> Self;     // Preset for agent connections
}

// Retry an async operation
let result = retry_async(RetryConfig::database(), || async {
    database.connect().await
}).await?;

// With custom config
let result = retry_async_with_config(
    RetryConfig {
        max_retries: 5,
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(10),
        multiplier: 2.0,
    },
    || async { api.call().await },
).await?;
```

## Database

### Database Connection

```rust
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Connect to database
    pub async fn connect(config: &DatabaseConfig) -> RimuruResult<Self>;

    /// Get the connection pool
    pub fn pool(&self) -> &PgPool;

    /// Run migrations
    pub async fn run_migrations(&self) -> RimuruResult<()>;

    /// Health check
    pub async fn health_check(&self) -> RimuruResult<()>;

    /// Close all connections
    pub async fn close(&self);
}
```

### DatabaseConfig

```rust
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: Duration,
    pub idle_timeout: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgres://localhost/rimuru".to_string(),
            max_connections: 10,
            min_connections: 1,
            connect_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
        }
    }
}
```

## Configuration

### RimuruConfig

Main configuration structure.

```rust
pub struct RimuruConfig {
    pub database: DatabaseConfig,
    pub agents: AgentsConfig,
    pub sync: SyncConfig,
    pub logging: LoggingConfig,
    pub metrics: MetricsConfig,
}

impl RimuruConfig {
    /// Load from default locations
    pub fn load() -> RimuruResult<Self>;

    /// Load from specific path
    pub fn load_from(path: &Path) -> RimuruResult<Self>;
}
```

### Directory Helpers

```rust
/// Get config directory (~/.config/rimuru or equivalent)
pub fn get_config_dir() -> PathBuf;

/// Get data directory (~/.local/share/rimuru or equivalent)
pub fn get_data_dir() -> PathBuf;

/// Get cache directory (~/.cache/rimuru or equivalent)
pub fn get_cache_dir() -> PathBuf;

/// Ensure directories exist
pub fn ensure_config_dir() -> RimuruResult<PathBuf>;
pub fn ensure_data_dir() -> RimuruResult<PathBuf>;
pub fn ensure_cache_dir() -> RimuruResult<PathBuf>;
```

## Hooks

### Hook Enum

Available hook points.

```rust
pub enum Hook {
    SessionStart,
    SessionEnd,
    CostThreshold,
    MetricsExport,
    Custom(String),
}
```

### HookManager

Register and trigger hooks.

```rust
pub struct HookManager {
    handlers: HashMap<Hook, Vec<HookHandlerInfo>>,
}

impl HookManager {
    pub fn new() -> Self;

    /// Register a hook handler
    pub fn register(
        &mut self,
        hook: Hook,
        config: HookConfig,
        handler: Box<dyn HookHandler>,
    ) -> Uuid;

    /// Unregister a handler
    pub fn unregister(&mut self, handler_id: Uuid);

    /// Trigger a hook
    pub async fn trigger(&self, hook: Hook, data: HookData) -> Vec<HookResult>;
}
```

### HookHandler Trait

```rust
#[async_trait]
pub trait HookHandler: Send + Sync {
    async fn handle(&self, ctx: &HookContext, data: &HookData) -> HookResult;
}
```

## Sync Providers

### ModelSyncProvider Trait

```rust
#[async_trait]
pub trait ModelSyncProvider: Send + Sync {
    /// Provider name
    fn name(&self) -> &str;

    /// Fetch model information
    async fn fetch_models(&self) -> RimuruResult<Vec<ModelInfo>>;

    /// Check if provider is available
    async fn is_available(&self) -> bool;
}
```

### Built-in Providers

```rust
pub struct AnthropicSyncProvider;
pub struct OpenAISyncProvider;
pub struct GoogleSyncProvider;
pub struct OpenRouterSyncProvider;
pub struct LiteLLMSyncProvider;
```

## Discovery

### AgentDiscovery Trait

```rust
#[async_trait]
pub trait AgentDiscovery: Send + Sync {
    fn agent_type(&self) -> AgentType;
    async fn discover(&self) -> Option<DiscoveredAgent>;
    async fn verify_installation(&self, agent: &DiscoveredAgent) -> bool;
}
```

### AgentScanner

```rust
pub struct AgentScanner {
    discoveries: Vec<Box<dyn AgentDiscovery>>,
}

impl AgentScanner {
    pub fn new() -> Self;
    pub fn with_all_agents() -> Self;
    pub async fn scan(&self, options: ScanOptions) -> ScanResult;
}
```

## See Also

- [[architecture]] - System architecture overview
- [[creating-adapters]] - Adding new agent adapters
- [[creating-plugins]] - Plugin development guide
- [[building]] - Build from source

---

For complete API documentation with all types and methods, run:

```bash
cargo doc --no-deps --document-private-items --open
```
