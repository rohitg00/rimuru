---
type: reference
title: Creating Agent Adapters
created: 2026-02-05
tags:
  - adapters
  - development
  - developer-guide
related:
  - "[[architecture]]"
  - "[[api-reference]]"
  - "[[creating-plugins]]"
---

# Creating Agent Adapters

This guide explains how to add support for new AI coding agents in Rimuru.

## Overview

An agent adapter provides a unified interface for Rimuru to interact with a specific AI coding agent. Each adapter implements three core traits:

1. `AgentAdapter` - Basic agent operations
2. `CostTracker` - Token usage and cost calculation
3. `SessionMonitor` - Session event monitoring

## Directory Structure

Create a new directory under `rimuru-core/src/adapters/`:

```
rimuru-core/src/adapters/
├── myagent/
│   ├── mod.rs        # Module entry, re-exports
│   ├── config.rs     # Configuration types
│   ├── cost.rs       # Cost tracking implementation
│   └── session.rs    # Session monitoring
├── mod.rs            # Add module here
└── ...
```

## Step 1: Define the Agent Type

Add your agent to the `AgentType` enum in `rimuru-core/src/models/agent.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    ClaudeCode,
    OpenCode,
    Codex,
    Copilot,
    Goose,
    Cursor,
    MyAgent,  // Add your agent
}

impl AgentType {
    pub fn display_name(&self) -> &str {
        match self {
            Self::ClaudeCode => "Claude Code",
            Self::OpenCode => "OpenCode",
            Self::Codex => "Codex",
            Self::Copilot => "GitHub Copilot",
            Self::Goose => "Goose",
            Self::Cursor => "Cursor",
            Self::MyAgent => "My Agent",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::ClaudeCode => "⟁",
            Self::OpenCode => "◇",
            Self::Codex => "◎",
            Self::Copilot => "◈",
            Self::Goose => "⬡",
            Self::Cursor => "◫",
            Self::MyAgent => "★",  // Choose an icon
        }
    }
}
```

## Step 2: Create Configuration

Create `rimuru-core/src/adapters/myagent/config.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyAgentConfig {
    /// Path to the agent's configuration directory
    pub config_path: PathBuf,

    /// API endpoint if applicable
    pub api_endpoint: Option<String>,

    /// API key for authentication
    pub api_key: Option<String>,

    /// Enable debug logging
    #[serde(default)]
    pub debug: bool,
}

impl Default for MyAgentConfig {
    fn default() -> Self {
        Self {
            config_path: dirs::home_dir()
                .unwrap_or_default()
                .join(".myagent"),
            api_endpoint: None,
            api_key: None,
            debug: false,
        }
    }
}

impl MyAgentConfig {
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            config_path,
            ..Default::default()
        }
    }

    /// Load configuration from standard locations
    pub fn load() -> Option<Self> {
        let home = dirs::home_dir()?;
        let config_path = home.join(".myagent");

        if config_path.exists() {
            Some(Self::new(config_path))
        } else {
            None
        }
    }
}
```

## Step 3: Implement the Adapter

Create `rimuru-core/src/adapters/myagent/mod.rs`:

```rust
mod config;
mod cost;
mod session;

pub use config::MyAgentConfig;
pub use cost::MyAgentCostCalculator;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::RimuruResult;
use crate::models::{AgentType, ModelInfo, Session};
use super::{
    ActiveSession, AdapterInfo, AdapterStatus, AgentAdapter, CostTracker,
    SessionEvent, SessionEventCallback, SessionHistory, SessionMonitor, UsageStats,
};

pub struct MyAgentAdapter {
    config: MyAgentConfig,
    status: Arc<RwLock<AdapterStatus>>,
    // Add fields for managing state
}

impl MyAgentAdapter {
    pub fn new(config: MyAgentConfig) -> Self {
        Self {
            config,
            status: Arc::new(RwLock::new(AdapterStatus::Unknown)),
        }
    }

    pub fn with_default_config() -> Option<Self> {
        MyAgentConfig::load().map(Self::new)
    }

    /// Check if the agent binary is installed
    async fn check_installation(&self) -> bool {
        // Check for agent binary or config files
        self.config.config_path.exists()
    }

    /// Read session data from agent's storage
    async fn read_sessions(&self) -> RimuruResult<Vec<Session>> {
        // Implementation depends on how your agent stores sessions
        // Common patterns:
        // 1. SQLite database
        // 2. JSON files
        // 3. Log files
        // 4. API calls
        todo!("Implement session reading")
    }
}

#[async_trait]
impl AgentAdapter for MyAgentAdapter {
    fn agent_type(&self) -> AgentType {
        AgentType::MyAgent
    }

    fn name(&self) -> &str {
        "myagent"
    }

    async fn connect(&mut self) -> RimuruResult<()> {
        // Initialize connection to agent
        // This might involve:
        // - Opening database connection
        // - Starting file watcher
        // - Connecting to API

        if self.check_installation().await {
            *self.status.write().await = AdapterStatus::Connected;
            Ok(())
        } else {
            *self.status.write().await = AdapterStatus::Error(
                "Agent not installed".to_string()
            );
            Err(crate::error::RimuruError::Adapter(
                "MyAgent is not installed".to_string()
            ))
        }
    }

    async fn disconnect(&mut self) -> RimuruResult<()> {
        // Clean up resources
        *self.status.write().await = AdapterStatus::Disconnected;
        Ok(())
    }

    async fn get_status(&self) -> AdapterStatus {
        *self.status.read().await
    }

    async fn get_info(&self) -> RimuruResult<AdapterInfo> {
        Ok(AdapterInfo {
            name: self.name().to_string(),
            agent_type: self.agent_type(),
            version: self.get_version().await,
            status: *self.status.read().await,
            config_path: Some(self.config.config_path.to_string_lossy().to_string()),
            last_connected: None,
            error_message: None,
        })
    }

    async fn get_sessions(&self) -> RimuruResult<Vec<Session>> {
        self.read_sessions().await
    }

    async fn get_active_session(&self) -> RimuruResult<Option<ActiveSession>> {
        // Check for currently active session
        // This might involve:
        // - Checking process status
        // - Reading lock files
        // - API calls
        Ok(None)
    }

    async fn is_installed(&self) -> bool {
        self.check_installation().await
    }

    async fn health_check(&self) -> RimuruResult<bool> {
        // Verify adapter is functioning correctly
        Ok(self.check_installation().await)
    }
}

impl MyAgentAdapter {
    async fn get_version(&self) -> Option<String> {
        // Try to detect installed version
        // This might involve:
        // - Running `myagent --version`
        // - Reading package.json
        // - Checking binary metadata
        None
    }
}
```

## Step 4: Implement Cost Tracking

Create `rimuru-core/src/adapters/myagent/cost.rs`:

```rust
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::error::RimuruResult;
use crate::models::ModelInfo;
use super::super::{CostTracker, UsageStats};

pub struct MyAgentCostCalculator {
    // Add fields for cost calculation
    // Might include:
    // - Model pricing data
    // - API client for real-time pricing
}

impl MyAgentCostCalculator {
    pub fn new() -> Self {
        Self {}
    }

    /// Get pricing for a specific model
    fn get_model_pricing(&self, model_name: &str) -> Option<(f64, f64)> {
        // Return (input_cost_per_1k, output_cost_per_1k)
        match model_name {
            "myagent-small" => Some((0.001, 0.002)),
            "myagent-large" => Some((0.01, 0.03)),
            _ => None,
        }
    }
}

#[async_trait]
impl CostTracker for MyAgentCostCalculator {
    async fn get_usage(&self, since: Option<DateTime<Utc>>) -> RimuruResult<UsageStats> {
        // Aggregate token usage from sessions
        // Filter by `since` if provided
        Ok(UsageStats {
            input_tokens: 0,
            output_tokens: 0,
            requests: 0,
            model_name: None,
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
        let (input_rate, output_rate) = self
            .get_model_pricing(model_name)
            .unwrap_or((0.01, 0.03));  // Default pricing

        let input_cost = (input_tokens as f64 / 1000.0) * input_rate;
        let output_cost = (output_tokens as f64 / 1000.0) * output_rate;

        Ok(input_cost + output_cost)
    }

    async fn get_model_info(&self, model_name: &str) -> RimuruResult<Option<ModelInfo>> {
        let pricing = self.get_model_pricing(model_name);

        Ok(pricing.map(|(input_rate, output_rate)| {
            ModelInfo::new(
                "myagent".to_string(),
                model_name.to_string(),
                input_rate,
                output_rate,
                128000,  // Context window
            )
        }))
    }

    async fn get_supported_models(&self) -> RimuruResult<Vec<String>> {
        Ok(vec![
            "myagent-small".to_string(),
            "myagent-large".to_string(),
        ])
    }

    async fn get_total_cost(&self, since: Option<DateTime<Utc>>) -> RimuruResult<f64> {
        let usage = self.get_usage(since).await?;
        let model = usage.model_name.as_deref().unwrap_or("myagent-small");
        self.calculate_cost(usage.input_tokens, usage.output_tokens, model).await
    }
}
```

## Step 5: Implement Session Monitoring

Create `rimuru-core/src/adapters/myagent/session.rs`:

```rust
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::RimuruResult;
use super::super::{
    ActiveSession, SessionEvent, SessionEventCallback, SessionHistory, SessionMonitor,
};

pub struct MyAgentSessionMonitor {
    subscriptions: Arc<RwLock<HashMap<Uuid, SessionEventCallback>>>,
}

impl MyAgentSessionMonitor {
    pub fn new() -> Self {
        Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Emit an event to all subscribers
    async fn emit_event(&self, event: SessionEvent) {
        let subs = self.subscriptions.read().await;
        for callback in subs.values() {
            callback(event.clone());
        }
    }
}

#[async_trait]
impl SessionMonitor for MyAgentSessionMonitor {
    async fn subscribe_to_events(&self, callback: SessionEventCallback) -> RimuruResult<Uuid> {
        let id = Uuid::new_v4();
        self.subscriptions.write().await.insert(id, callback);
        Ok(id)
    }

    async fn unsubscribe(&self, subscription_id: Uuid) -> RimuruResult<()> {
        self.subscriptions.write().await.remove(&subscription_id);
        Ok(())
    }

    async fn get_session_history(
        &self,
        limit: Option<usize>,
        since: Option<DateTime<Utc>>,
    ) -> RimuruResult<Vec<SessionHistory>> {
        // Read session history from agent's storage
        // Apply limit and since filters
        Ok(vec![])
    }

    async fn get_session_details(&self, session_id: Uuid) -> RimuruResult<Option<SessionHistory>> {
        // Get detailed information about a specific session
        Ok(None)
    }

    async fn get_active_sessions(&self) -> RimuruResult<Vec<ActiveSession>> {
        // Return currently active sessions
        Ok(vec![])
    }
}
```

## Step 6: Register the Adapter

Update `rimuru-core/src/adapters/mod.rs`:

```rust
mod myagent;

pub use myagent::{MyAgentAdapter, MyAgentConfig, MyAgentCostCalculator};
```

Add to the adapter factory in `rimuru-core/src/adapters/factory.rs`:

```rust
pub fn create_adapter(agent_type: AgentType) -> Option<Box<dyn FullAdapter>> {
    match agent_type {
        AgentType::ClaudeCode => ClaudeCodeAdapter::with_default_config()
            .map(|a| Box::new(a) as Box<dyn FullAdapter>),
        // ... other adapters ...
        AgentType::MyAgent => MyAgentAdapter::with_default_config()
            .map(|a| Box::new(a) as Box<dyn FullAdapter>),
    }
}
```

## Step 7: Add Discovery Support

Create `rimuru-core/src/discovery/myagent.rs`:

```rust
use async_trait::async_trait;
use std::path::PathBuf;

use crate::models::AgentType;
use super::{AgentDiscovery, DiscoveredAgent, DetectionMethod};

pub struct MyAgentDiscovery;

impl MyAgentDiscovery {
    pub fn new() -> Self {
        Self
    }

    fn find_config_path() -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        let config_path = home.join(".myagent");

        if config_path.exists() {
            Some(config_path)
        } else {
            None
        }
    }

    async fn detect_version(config_path: &PathBuf) -> Option<String> {
        // Try to detect version from config files or binary
        None
    }
}

#[async_trait]
impl AgentDiscovery for MyAgentDiscovery {
    fn agent_type(&self) -> AgentType {
        AgentType::MyAgent
    }

    async fn discover(&self) -> Option<DiscoveredAgent> {
        let config_path = Self::find_config_path()?;
        let version = Self::detect_version(&config_path).await;

        Some(DiscoveredAgent {
            agent_type: self.agent_type(),
            name: "My Agent".to_string(),
            config_path: Some(config_path),
            version,
            detection_method: DetectionMethod::ConfigFile,
            is_active: false,
        })
    }

    async fn verify_installation(&self, agent: &DiscoveredAgent) -> bool {
        agent.config_path
            .as_ref()
            .map(|p| p.exists())
            .unwrap_or(false)
    }
}
```

Register in `rimuru-core/src/discovery/mod.rs`:

```rust
mod myagent;
pub use myagent::MyAgentDiscovery;
```

## Step 8: Update Configuration

Add agent configuration to `rimuru-core/src/config.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentsConfig {
    pub claude_code: AgentInstanceConfig,
    pub opencode: AgentInstanceConfig,
    pub codex: AgentInstanceConfig,
    pub copilot: AgentInstanceConfig,
    pub goose: AgentInstanceConfig,
    pub cursor: AgentInstanceConfig,
    pub myagent: AgentInstanceConfig,  // Add this
}
```

## Step 9: Write Tests

Create `rimuru-core/src/adapters/myagent/tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_adapter_creation() {
        let config = MyAgentConfig::default();
        let adapter = MyAgentAdapter::new(config);

        assert_eq!(adapter.agent_type(), AgentType::MyAgent);
        assert_eq!(adapter.name(), "myagent");
    }

    #[tokio::test]
    async fn test_cost_calculation() {
        let calculator = MyAgentCostCalculator::new();
        let cost = calculator.calculate_cost(1000, 500, "myagent-small").await.unwrap();

        // 1000 input tokens * $0.001/1k + 500 output tokens * $0.002/1k
        // = $0.001 + $0.001 = $0.002
        assert!((cost - 0.002).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_supported_models() {
        let calculator = MyAgentCostCalculator::new();
        let models = calculator.get_supported_models().await.unwrap();

        assert!(models.contains(&"myagent-small".to_string()));
        assert!(models.contains(&"myagent-large".to_string()));
    }
}
```

## Best Practices

### Session Data Parsing

Different agents store session data in various formats. Common approaches:

```rust
// JSON files
let content = std::fs::read_to_string(path)?;
let data: SessionData = serde_json::from_str(&content)?;

// SQLite database
let conn = Connection::open(db_path)?;
let sessions = conn.query("SELECT * FROM sessions", [])?;

// Log file parsing
let log = std::fs::read_to_string(log_path)?;
for line in log.lines() {
    if let Some(session) = parse_session_line(line) {
        sessions.push(session);
    }
}
```

### Real-time Monitoring

For real-time session monitoring, consider:

1. **File watching** - Monitor config/log files for changes
2. **Process monitoring** - Check if agent process is running
3. **API polling** - If agent has an API

```rust
use notify::{Watcher, RecursiveMode};

async fn watch_sessions(&self) {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::recommended_watcher(tx)?;

    watcher.watch(&self.config.sessions_path, RecursiveMode::Recursive)?;

    for event in rx {
        if let Ok(event) = event {
            self.emit_event(SessionEvent::Updated).await;
        }
    }
}
```

### Error Handling

Handle agent-specific errors gracefully:

```rust
async fn read_sessions(&self) -> RimuruResult<Vec<Session>> {
    let path = &self.config.sessions_path;

    if !path.exists() {
        return Ok(vec![]);  // No sessions yet
    }

    let content = std::fs::read_to_string(path)
        .map_err(|e| RimuruError::Adapter(
            format!("Failed to read sessions: {}", e)
        ))?;

    serde_json::from_str(&content)
        .map_err(|e| RimuruError::Adapter(
            format!("Invalid session data: {}", e)
        ))
}
```

## Checklist

Before submitting your adapter:

- [ ] All three traits implemented (`AgentAdapter`, `CostTracker`, `SessionMonitor`)
- [ ] Agent type added to enum
- [ ] Discovery support added
- [ ] Configuration support added
- [ ] Unit tests written and passing
- [ ] Documentation added
- [ ] Manual testing completed

## See Also

- [[architecture]] - System architecture
- [[api-reference]] - API documentation
- [[creating-plugins]] - Plugin development
