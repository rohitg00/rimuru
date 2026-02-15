---
type: reference
title: System Architecture Overview
created: 2026-02-05
tags:
  - architecture
  - design
  - developer-guide
related:
  - "[[building]]"
  - "[[creating-adapters]]"
  - "[[creating-plugins]]"
---

# System Architecture Overview

Rimuru is a unified AI agent orchestration and cost tracking platform built with a modular, layered architecture in Rust.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        User Interfaces                               │
├─────────────────┬─────────────────┬─────────────────────────────────┤
│   rimuru-cli    │   rimuru-tui    │         rimuru-desktop          │
│   (Terminal)    │   (Interactive) │         (Tauri + React)         │
└────────┬────────┴────────┬────────┴────────────────┬────────────────┘
         │                 │                         │
         └─────────────────┼─────────────────────────┘
                           │
┌──────────────────────────┴──────────────────────────────────────────┐
│                         rimuru-core                                  │
├─────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │
│  │  Adapters   │  │   Models    │  │   Services  │  │   Plugins   │ │
│  │             │  │             │  │             │  │             │ │
│  │ ClaudeCode  │  │   Agent     │  │ AdapterMgr  │  │  Registry   │ │
│  │ OpenCode    │  │   Session   │  │ CostAggr    │  │  Loader     │ │
│  │ Codex       │  │   Cost      │  │ SessionAggr │  │  Sandbox    │ │
│  │ Copilot     │  │   Model     │  │             │  │             │ │
│  │ Goose       │  │   Metrics   │  │             │  │             │ │
│  │ Cursor      │  │             │  │             │  │             │ │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘ │
│                                                                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │
│  │  Discovery  │  │    Sync     │  │    Hooks    │  │  SkillKit   │ │
│  │             │  │             │  │             │  │             │ │
│  │  Scanner    │  │  Providers  │  │  Manager    │  │  Bridge     │ │
│  │  Detection  │  │  Scheduler  │  │  Handlers   │  │  Installer  │ │
│  │             │  │  Aggregator │  │             │  │  Translator │ │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘ │
│                                                                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                  │
│  │ Repository  │  │   Config    │  │   Metrics   │                  │
│  │             │  │             │  │             │                  │
│  │ Agent       │  │  Database   │  │  Collector  │                  │
│  │ Session     │  │  Logging    │  │  System     │                  │
│  │ Cost        │  │  Agents     │  │  Aggregator │                  │
│  │ Model       │  │  Sync       │  │             │                  │
│  └──────┬──────┘  └─────────────┘  └─────────────┘                  │
│         │                                                            │
└─────────┼────────────────────────────────────────────────────────────┘
          │
┌─────────┴────────────────────────────────────────────────────────────┐
│                        PostgreSQL Database                           │
│  ┌─────────┐  ┌──────────┐  ┌───────┐  ┌────────┐  ┌─────────────┐  │
│  │ agents  │  │ sessions │  │ costs │  │ models │  │ metrics_... │  │
│  └─────────┘  └──────────┘  └───────┘  └────────┘  └─────────────┘  │
└──────────────────────────────────────────────────────────────────────┘
```

## Workspace Structure

The project is organized as a Cargo workspace with the following crates:

| Crate | Description |
|-------|-------------|
| `rimuru-core` | Core library with models, adapters, repositories, services |
| `rimuru-cli` | Command-line interface |
| `rimuru-tui` | Terminal UI using Ratatui |
| `rimuru-desktop` | Desktop application using Tauri + React |
| `rimuru-plugin-sdk` | SDK for developing plugins |

## Core Modules

### Adapters (`rimuru-core/src/adapters/`)

Adapters provide a unified interface to interact with different AI coding agents:

```rust
#[async_trait]
pub trait AgentAdapter: Send + Sync {
    fn agent_type(&self) -> AgentType;
    fn name(&self) -> &str;
    async fn connect(&mut self) -> RimuruResult<()>;
    async fn disconnect(&mut self) -> RimuruResult<()>;
    async fn get_status(&self) -> AdapterStatus;
    async fn get_info(&self) -> RimuruResult<AdapterInfo>;
    async fn get_sessions(&self) -> RimuruResult<Vec<Session>>;
    async fn get_active_session(&self) -> RimuruResult<Option<ActiveSession>>;
    async fn is_installed(&self) -> bool;
    async fn health_check(&self) -> RimuruResult<bool>;
}
```

Additional traits:
- `CostTracker` - Track token usage and calculate costs
- `SessionMonitor` - Monitor session events and history
- `FullAdapter` - Combines all three traits

### Models (`rimuru-core/src/models/`)

Core data types:

| Model | Description |
|-------|-------------|
| `Agent` | Represents a registered AI agent |
| `AgentType` | Enum: ClaudeCode, OpenCode, Codex, Copilot, Goose, Cursor |
| `Session` | A coding session with an agent |
| `SessionStatus` | Enum: Active, Completed, Cancelled, Failed |
| `CostRecord` | Token usage and cost for a session |
| `ModelInfo` | Model pricing and capabilities |
| `SystemMetrics` | CPU, memory, active sessions |

### Repository Layer (`rimuru-core/src/repo/`)

Async repository pattern for database operations:

```rust
#[async_trait]
pub trait Repository<T, ID> {
    async fn find_by_id(&self, id: ID) -> RimuruResult<Option<T>>;
    async fn find_all(&self) -> RimuruResult<Vec<T>>;
    async fn save(&self, entity: T) -> RimuruResult<T>;
    async fn delete(&self, id: ID) -> RimuruResult<bool>;
}
```

Implementations:
- `AgentRepository` - CRUD for agents
- `SessionRepository` - Sessions with filtering
- `CostRepository` - Cost records with aggregation
- `ModelRepository` - Model pricing data
- `MetricsRepository` - System metrics history

### Services (`rimuru-core/src/services/`)

Business logic layer:

- `AdapterManager` - Manages adapter lifecycle, health checks
- `CostAggregator` - Aggregates costs across agents/models
- `SessionAggregator` - Unified session view across agents

### Discovery (`rimuru-core/src/discovery/`)

Automatic agent detection:

```rust
pub trait AgentDiscovery: Send + Sync {
    fn agent_type(&self) -> AgentType;
    async fn discover(&self) -> Option<DiscoveredAgent>;
    async fn verify_installation(&self, agent: &DiscoveredAgent) -> bool;
}
```

`AgentScanner` orchestrates discovery across all supported agents.

### Sync (`rimuru-core/src/sync/`)

Model pricing synchronization:

- `ModelSyncProvider` trait - Interface for sync providers
- Providers: Anthropic, OpenAI, Google, OpenRouter, LiteLLM
- `ModelAggregator` - Combines data from multiple providers
- `BackgroundSyncScheduler` - Periodic sync scheduling

### Plugins (`rimuru-core/src/plugins/`)

Extensibility system:

- `Plugin` trait - Base for all plugins
- `ExporterPlugin` - Export sessions/costs to formats (CSV, JSON, XML)
- `NotifierPlugin` - Send notifications (Slack, Discord, Webhook)
- `AgentPlugin` - Add support for new agents
- `ViewPlugin` - Custom TUI/desktop views
- `PluginRegistry` - Manages plugin lifecycle
- `Sandbox` - Security isolation for plugins

### Hooks (`rimuru-core/src/hooks/`)

Event-driven automation:

```rust
pub enum Hook {
    SessionStart,
    SessionEnd,
    CostThreshold,
    MetricsExport,
    Custom(String),
}
```

Built-in handlers:
- `SessionLogHandler` - Log session events
- `CostAlertHandler` - Alert on cost thresholds
- `WebhookHandler` - Call external webhooks
- `MetricsExportHandler` - Export metrics periodically

### SkillKit Integration (`rimuru-core/src/skillkit/`)

Bridge to the SkillKit marketplace:

- `SkillKitBridge` - Main interface
- `SkillSearcher` - Search marketplace
- `SkillInstaller` - Install skills to agents
- `SkillTranslator` - Translate skills between agent formats
- `SkillRecommender` - AI-powered skill recommendations

## Data Flow

### Session Tracking Flow

```
Agent Activity → Adapter.get_sessions() → SessionRepository.save()
                                                    │
                                                    ▼
                        CostTracker.calculate_cost() → CostRepository.save()
                                                    │
                                                    ▼
                        HookManager.trigger(SessionEnd) → Notifications
```

### Model Sync Flow

```
Scheduler Tick → SyncProvider.fetch_models() → ModelAggregator.aggregate()
                                                        │
                                                        ▼
                                    ModelRepository.save() → CostTracker.get_model_info()
```

## Error Handling

All async operations return `RimuruResult<T>`:

```rust
pub type RimuruResult<T> = Result<T, RimuruError>;

#[derive(Debug, Error)]
pub enum RimuruError {
    #[error("[E1001] Database connection failed: {0}")]
    Database(String),

    #[error("[E2001] Agent adapter error: {0}")]
    Adapter(String),

    // ... more variants
}
```

Error context with file/line tracking:

```rust
let result = operation().map_err(|e| {
    RimuruError::with_context(e, error_context!("operation failed"))
})?;
```

Retry support:

```rust
let result = retry_async(RetryConfig::database(), || {
    async { database.connect().await }
}).await?;
```

## Configuration

Configuration sources (in priority order):
1. Environment variables
2. `.env` file
3. `local.toml` in config directory
4. Default values

Key configuration:

```toml
[database]
url = "postgres://user:pass@localhost/rimuru"
max_connections = 10

[agents]
claude_code.enabled = true
claude_code.config_path = "~/.claude"

[sync]
enabled = true
interval_hours = 24
providers = ["anthropic", "openai"]
```

## Security Considerations

1. **Plugin Sandbox** - Plugins run in isolated environments with:
   - Filesystem access restrictions
   - Network access controls
   - Resource limits (memory, CPU)

2. **Credential Handling** - API keys stored in:
   - Environment variables
   - OS keychain (desktop)
   - Encrypted config files

3. **Database** - Uses parameterized queries via sqlx to prevent SQL injection

## See Also

- [[building]] - Build from source
- [[creating-adapters]] - Add new agent support
- [[creating-plugins]] - Plugin development guide
- [[api-reference]] - API documentation
