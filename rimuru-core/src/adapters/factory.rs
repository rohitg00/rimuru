use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

use crate::error::{RimuruError, RimuruResult};
use crate::models::AgentType;

use super::claude_code::{ClaudeCodeAdapter, ClaudeCodeConfig};
use super::codex::{CodexAdapter, CodexConfig};
use super::copilot::{CopilotAdapter, CopilotConfig};
use super::cursor::{CursorAdapter, CursorConfig};
use super::goose::{GooseAdapter, GooseConfig};
use super::opencode::{OpenCodeAdapter, OpenCodeConfig};
use super::traits::FullAdapter;
use super::types::AdapterStatus;

type BoxedAdapter = Box<dyn FullAdapter>;

#[derive(Debug, Clone)]
pub struct AdapterFactoryConfig {
    pub health_check_interval_secs: u64,
    pub max_retry_attempts: u32,
    pub retry_delay_secs: u64,
    pub lazy_initialization: bool,
}

impl Default for AdapterFactoryConfig {
    fn default() -> Self {
        Self {
            health_check_interval_secs: 60,
            max_retry_attempts: 3,
            retry_delay_secs: 5,
            lazy_initialization: true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum AdapterConfig {
    ClaudeCode(ClaudeCodeConfig),
    Codex(CodexConfig),
    Copilot(CopilotConfig),
    Cursor(CursorConfig),
    Goose(GooseConfig),
    OpenCode(OpenCodeConfig),
}

impl AdapterConfig {
    pub fn agent_type(&self) -> AgentType {
        match self {
            AdapterConfig::ClaudeCode(_) => AgentType::ClaudeCode,
            AdapterConfig::Codex(_) => AgentType::Codex,
            AdapterConfig::Copilot(_) => AgentType::Copilot,
            AdapterConfig::Cursor(_) => AgentType::Cursor,
            AdapterConfig::Goose(_) => AgentType::Goose,
            AdapterConfig::OpenCode(_) => AgentType::OpenCode,
        }
    }

    pub fn validate(&self) -> RimuruResult<()> {
        match self {
            AdapterConfig::ClaudeCode(config) => {
                if !config
                    .config_dir
                    .parent()
                    .is_none_or(|p| p.exists() || p == std::path::Path::new(""))
                {
                    return Err(RimuruError::ValidationError(
                        "Claude Code config directory parent does not exist".to_string(),
                    ));
                }
                Ok(())
            }
            AdapterConfig::Codex(config) => {
                if !config
                    .config_dir
                    .parent()
                    .is_none_or(|p| p.exists() || p == std::path::Path::new(""))
                {
                    return Err(RimuruError::ValidationError(
                        "Codex config directory parent does not exist".to_string(),
                    ));
                }
                Ok(())
            }
            AdapterConfig::Copilot(config) => {
                if !config
                    .github_copilot_dir
                    .parent()
                    .is_none_or(|p| p.exists() || p == std::path::Path::new(""))
                {
                    return Err(RimuruError::ValidationError(
                        "Copilot config directory parent does not exist".to_string(),
                    ));
                }
                Ok(())
            }
            AdapterConfig::Cursor(config) => {
                if !config
                    .config_dir
                    .parent()
                    .is_none_or(|p| p.exists() || p == std::path::Path::new(""))
                {
                    return Err(RimuruError::ValidationError(
                        "Cursor config directory parent does not exist".to_string(),
                    ));
                }
                Ok(())
            }
            AdapterConfig::Goose(config) => {
                if !config
                    .config_dir
                    .parent()
                    .is_none_or(|p| p.exists() || p == std::path::Path::new(""))
                {
                    return Err(RimuruError::ValidationError(
                        "Goose config directory parent does not exist".to_string(),
                    ));
                }
                Ok(())
            }
            AdapterConfig::OpenCode(config) => {
                if !config
                    .config_dir
                    .parent()
                    .is_none_or(|p| p.exists() || p == std::path::Path::new(""))
                {
                    return Err(RimuruError::ValidationError(
                        "OpenCode config directory parent does not exist".to_string(),
                    ));
                }
                Ok(())
            }
        }
    }

    pub fn default_for_type(agent_type: AgentType) -> Self {
        match agent_type {
            AgentType::ClaudeCode => AdapterConfig::ClaudeCode(ClaudeCodeConfig::default()),
            AgentType::Codex => AdapterConfig::Codex(CodexConfig::default()),
            AgentType::Copilot => AdapterConfig::Copilot(CopilotConfig::default()),
            AgentType::Cursor => AdapterConfig::Cursor(CursorConfig::default()),
            AgentType::Goose => AdapterConfig::Goose(GooseConfig::default()),
            AgentType::OpenCode => AdapterConfig::OpenCode(OpenCodeConfig::default()),
        }
    }
}

struct LazyAdapter {
    config: AdapterConfig,
    adapter: Option<BoxedAdapter>,
    name: String,
}

impl LazyAdapter {
    fn new(name: String, config: AdapterConfig) -> Self {
        Self {
            config,
            adapter: None,
            name,
        }
    }

    fn initialize(&mut self) -> RimuruResult<&mut BoxedAdapter> {
        if self.adapter.is_none() {
            let adapter = create_adapter_from_config(&self.name, &self.config)?;
            self.adapter = Some(adapter);
        }
        Ok(self.adapter.as_mut().unwrap())
    }

    fn get(&self) -> Option<&BoxedAdapter> {
        self.adapter.as_ref()
    }

    fn get_mut(&mut self) -> Option<&mut BoxedAdapter> {
        self.adapter.as_mut()
    }

    fn is_initialized(&self) -> bool {
        self.adapter.is_some()
    }
}

fn create_adapter_from_config(name: &str, config: &AdapterConfig) -> RimuruResult<BoxedAdapter> {
    match config {
        AdapterConfig::ClaudeCode(c) => Ok(Box::new(ClaudeCodeAdapter::new(name, c.clone()))),
        AdapterConfig::Codex(c) => Ok(Box::new(CodexAdapter::new(name, c.clone()))),
        AdapterConfig::Copilot(c) => Ok(Box::new(CopilotAdapter::new(name, c.clone()))),
        AdapterConfig::Cursor(c) => Ok(Box::new(CursorAdapter::new(name, c.clone()))),
        AdapterConfig::Goose(c) => Ok(Box::new(GooseAdapter::new(name, c.clone()))),
        AdapterConfig::OpenCode(c) => Ok(Box::new(OpenCodeAdapter::new(name, c.clone()))),
    }
}

pub struct AdapterFactory {
    config: AdapterFactoryConfig,
    adapters: RwLock<HashMap<String, LazyAdapter>>,
    type_index: RwLock<HashMap<AgentType, Vec<String>>>,
}

impl AdapterFactory {
    pub fn new(config: AdapterFactoryConfig) -> Self {
        Self {
            config,
            adapters: RwLock::new(HashMap::new()),
            type_index: RwLock::new(HashMap::new()),
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(AdapterFactoryConfig::default())
    }

    pub async fn register(&self, name: String, adapter_config: AdapterConfig) -> RimuruResult<()> {
        adapter_config.validate()?;

        let agent_type = adapter_config.agent_type();

        let mut adapters = self.adapters.write().await;
        if adapters.contains_key(&name) {
            return Err(RimuruError::AgentAlreadyExists(name));
        }

        let mut lazy_adapter = LazyAdapter::new(name.clone(), adapter_config);

        if !self.config.lazy_initialization {
            lazy_adapter.initialize()?;
            info!("Adapter '{}' initialized eagerly", name);
        } else {
            debug!("Adapter '{}' registered for lazy initialization", name);
        }

        adapters.insert(name.clone(), lazy_adapter);

        let mut type_index = self.type_index.write().await;
        type_index.entry(agent_type).or_default().push(name);

        Ok(())
    }

    pub async fn register_with_default(
        &self,
        name: String,
        agent_type: AgentType,
    ) -> RimuruResult<()> {
        let config = AdapterConfig::default_for_type(agent_type);
        self.register(name, config).await
    }

    pub async fn unregister(&self, name: &str) -> RimuruResult<()> {
        let mut adapters = self.adapters.write().await;
        let lazy_adapter = adapters
            .remove(name)
            .ok_or_else(|| RimuruError::AgentNotFound(name.to_string()))?;

        let agent_type = lazy_adapter.config.agent_type();

        let mut type_index = self.type_index.write().await;
        if let Some(names) = type_index.get_mut(&agent_type) {
            names.retain(|n| n != name);
            if names.is_empty() {
                type_index.remove(&agent_type);
            }
        }

        info!("Adapter '{}' unregistered", name);
        Ok(())
    }

    pub async fn get_or_create(&self, name: &str) -> RimuruResult<Arc<RwLock<BoxedAdapter>>> {
        let mut adapters = self.adapters.write().await;

        let lazy_adapter = adapters
            .get_mut(name)
            .ok_or_else(|| RimuruError::AgentNotFound(name.to_string()))?;

        if !lazy_adapter.is_initialized() {
            lazy_adapter.initialize()?;
            info!("Adapter '{}' initialized on first access", name);
        }

        let adapter = lazy_adapter.get_mut().unwrap();

        Ok(Arc::new(RwLock::new(unsafe {
            std::ptr::read(adapter as *const BoxedAdapter)
        })))
    }

    pub async fn get_adapter(&self, name: &str) -> Option<Arc<RwLock<BoxedAdapter>>> {
        let mut adapters = self.adapters.write().await;

        if let Some(lazy_adapter) = adapters.get_mut(name) {
            if !lazy_adapter.is_initialized() {
                if let Err(e) = lazy_adapter.initialize() {
                    error!("Failed to initialize adapter '{}': {}", name, e);
                    return None;
                }
            }

            if lazy_adapter.get().is_some() {
                return Some(Arc::new(RwLock::new(
                    create_adapter_from_config(&lazy_adapter.name, &lazy_adapter.config).ok()?,
                )));
            }
        }

        None
    }

    pub async fn get_adapters_by_type(&self, agent_type: AgentType) -> Vec<String> {
        let type_index = self.type_index.read().await;
        type_index.get(&agent_type).cloned().unwrap_or_default()
    }

    pub async fn list_registered(&self) -> Vec<(String, AgentType, bool)> {
        let adapters = self.adapters.read().await;
        adapters
            .iter()
            .map(|(name, lazy)| {
                (
                    name.clone(),
                    lazy.config.agent_type(),
                    lazy.is_initialized(),
                )
            })
            .collect()
    }

    pub async fn is_initialized(&self, name: &str) -> bool {
        let adapters = self.adapters.read().await;
        adapters.get(name).is_some_and(|a| a.is_initialized())
    }

    pub async fn initialize_all(&self) -> Vec<(String, RimuruResult<()>)> {
        let mut adapters = self.adapters.write().await;
        let mut results = Vec::new();

        for (name, lazy_adapter) in adapters.iter_mut() {
            if !lazy_adapter.is_initialized() {
                let result = lazy_adapter.initialize().map(|_| ());
                results.push((name.clone(), result));
            } else {
                results.push((name.clone(), Ok(())));
            }
        }

        results
    }

    pub async fn connect_with_retry(&self, name: &str) -> RimuruResult<()> {
        let mut adapters = self.adapters.write().await;

        let lazy_adapter = adapters
            .get_mut(name)
            .ok_or_else(|| RimuruError::AgentNotFound(name.to_string()))?;

        if !lazy_adapter.is_initialized() {
            lazy_adapter.initialize()?;
        }

        let adapter = lazy_adapter.get_mut().unwrap();

        for attempt in 1..=self.config.max_retry_attempts {
            match adapter.connect().await {
                Ok(()) => {
                    info!(
                        "Adapter '{}' connected successfully on attempt {}",
                        name, attempt
                    );
                    return Ok(());
                }
                Err(e) => {
                    warn!(
                        "Adapter '{}' connection attempt {} failed: {}",
                        name, attempt, e
                    );

                    if attempt < self.config.max_retry_attempts {
                        sleep(Duration::from_secs(self.config.retry_delay_secs)).await;
                    }
                }
            }
        }

        Err(RimuruError::AgentConnectionFailed {
            agent: name.to_string(),
            message: format!(
                "Failed to connect after {} attempts",
                self.config.max_retry_attempts
            ),
        })
    }

    pub async fn health_check_with_reconnect(&self, name: &str) -> RimuruResult<bool> {
        let mut adapters = self.adapters.write().await;

        let lazy_adapter = adapters
            .get_mut(name)
            .ok_or_else(|| RimuruError::AgentNotFound(name.to_string()))?;

        if !lazy_adapter.is_initialized() {
            return Ok(false);
        }

        let adapter = lazy_adapter.get_mut().unwrap();

        match adapter.health_check().await {
            Ok(true) => {
                debug!("Adapter '{}' health check passed", name);
                return Ok(true);
            }
            Ok(false) => {
                warn!("Adapter '{}' health check returned unhealthy", name);
            }
            Err(e) => {
                warn!("Adapter '{}' health check failed: {}", name, e);
            }
        }

        let status = adapter.get_status().await;
        if status == AdapterStatus::Error || status == AdapterStatus::Disconnected {
            info!("Attempting to reconnect adapter '{}'", name);

            for attempt in 1..=self.config.max_retry_attempts {
                match adapter.connect().await {
                    Ok(()) => {
                        info!(
                            "Adapter '{}' reconnected successfully on attempt {}",
                            name, attempt
                        );

                        if let Ok(true) = adapter.health_check().await {
                            return Ok(true);
                        }
                        return Ok(false);
                    }
                    Err(e) => {
                        warn!(
                            "Adapter '{}' reconnect attempt {} failed: {}",
                            name, attempt, e
                        );

                        if attempt < self.config.max_retry_attempts {
                            sleep(Duration::from_secs(self.config.retry_delay_secs)).await;
                        }
                    }
                }
            }
        }

        Ok(false)
    }

    pub async fn health_check_all(&self) -> HashMap<String, bool> {
        let adapters = self.adapters.read().await;
        let mut results = HashMap::new();

        for (name, lazy_adapter) in adapters.iter() {
            if lazy_adapter.is_initialized() {
                if let Some(adapter) = lazy_adapter.get() {
                    let healthy = adapter.health_check().await.unwrap_or(false);
                    results.insert(name.clone(), healthy);
                } else {
                    results.insert(name.clone(), false);
                }
            } else {
                results.insert(name.clone(), false);
            }
        }

        results
    }

    pub async fn connect_all_with_retry(&self) -> Vec<(String, RimuruResult<()>)> {
        let names: Vec<String> = {
            let adapters = self.adapters.read().await;
            adapters.keys().cloned().collect()
        };

        let mut results = Vec::new();

        for name in names {
            let result = self.connect_with_retry(&name).await;
            results.push((name, result));
        }

        results
    }

    pub async fn disconnect_all(&self) -> Vec<(String, RimuruResult<()>)> {
        let mut adapters = self.adapters.write().await;
        let mut results = Vec::new();

        for (name, lazy_adapter) in adapters.iter_mut() {
            if lazy_adapter.is_initialized() {
                if let Some(adapter) = lazy_adapter.get_mut() {
                    let result = adapter.disconnect().await;
                    results.push((name.clone(), result));
                }
            }
        }

        results
    }

    pub fn factory_config(&self) -> &AdapterFactoryConfig {
        &self.config
    }
}

impl Default for AdapterFactory {
    fn default() -> Self {
        Self::with_default_config()
    }
}

pub fn create_adapter(name: &str, agent_type: AgentType) -> BoxedAdapter {
    match agent_type {
        AgentType::ClaudeCode => Box::new(ClaudeCodeAdapter::with_default_config(name)),
        AgentType::Codex => Box::new(CodexAdapter::with_default_config(name)),
        AgentType::Copilot => Box::new(CopilotAdapter::with_default_config(name)),
        AgentType::Cursor => Box::new(CursorAdapter::with_default_config(name)),
        AgentType::Goose => Box::new(GooseAdapter::with_default_config(name)),
        AgentType::OpenCode => Box::new(OpenCodeAdapter::with_default_config(name)),
    }
}

pub fn create_adapter_with_config(name: &str, config: AdapterConfig) -> RimuruResult<BoxedAdapter> {
    config.validate()?;
    create_adapter_from_config(name, &config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_config_agent_type() {
        assert_eq!(
            AdapterConfig::ClaudeCode(ClaudeCodeConfig::default()).agent_type(),
            AgentType::ClaudeCode
        );
        assert_eq!(
            AdapterConfig::Codex(CodexConfig::default()).agent_type(),
            AgentType::Codex
        );
        assert_eq!(
            AdapterConfig::Copilot(CopilotConfig::default()).agent_type(),
            AgentType::Copilot
        );
        assert_eq!(
            AdapterConfig::Cursor(CursorConfig::default()).agent_type(),
            AgentType::Cursor
        );
        assert_eq!(
            AdapterConfig::Goose(GooseConfig::default()).agent_type(),
            AgentType::Goose
        );
        assert_eq!(
            AdapterConfig::OpenCode(OpenCodeConfig::default()).agent_type(),
            AgentType::OpenCode
        );
    }

    #[test]
    fn test_adapter_config_default_for_type() {
        for agent_type in [
            AgentType::ClaudeCode,
            AgentType::Codex,
            AgentType::Copilot,
            AgentType::Cursor,
            AgentType::Goose,
            AgentType::OpenCode,
        ] {
            let config = AdapterConfig::default_for_type(agent_type);
            assert_eq!(config.agent_type(), agent_type);
        }
    }

    #[test]
    fn test_adapter_config_validate() {
        let config = AdapterConfig::ClaudeCode(ClaudeCodeConfig::default());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_create_adapter() {
        let adapter = create_adapter("test", AgentType::ClaudeCode);
        assert_eq!(adapter.name(), "test");
        assert_eq!(adapter.agent_type(), AgentType::ClaudeCode);
    }

    #[test]
    fn test_create_all_adapter_types() {
        for agent_type in [
            AgentType::ClaudeCode,
            AgentType::Codex,
            AgentType::Copilot,
            AgentType::Cursor,
            AgentType::Goose,
            AgentType::OpenCode,
        ] {
            let adapter = create_adapter("test", agent_type);
            assert_eq!(adapter.agent_type(), agent_type);
        }
    }

    #[tokio::test]
    async fn test_factory_creation() {
        let factory = AdapterFactory::with_default_config();
        assert!(factory.factory_config().lazy_initialization);
        assert_eq!(factory.factory_config().max_retry_attempts, 3);
    }

    #[tokio::test]
    async fn test_factory_register_adapter() {
        let factory = AdapterFactory::with_default_config();

        factory
            .register(
                "test-claude".to_string(),
                AdapterConfig::ClaudeCode(ClaudeCodeConfig::default()),
            )
            .await
            .unwrap();

        let registered = factory.list_registered().await;
        assert_eq!(registered.len(), 1);
        assert_eq!(registered[0].0, "test-claude");
        assert_eq!(registered[0].1, AgentType::ClaudeCode);
        assert!(!registered[0].2);
    }

    #[tokio::test]
    async fn test_factory_register_with_default() {
        let factory = AdapterFactory::with_default_config();

        factory
            .register_with_default("test-opencode".to_string(), AgentType::OpenCode)
            .await
            .unwrap();

        let registered = factory.list_registered().await;
        assert_eq!(registered.len(), 1);
        assert_eq!(registered[0].1, AgentType::OpenCode);
    }

    #[tokio::test]
    async fn test_factory_register_duplicate_fails() {
        let factory = AdapterFactory::with_default_config();

        factory
            .register_with_default("test".to_string(), AgentType::ClaudeCode)
            .await
            .unwrap();

        let result = factory
            .register_with_default("test".to_string(), AgentType::OpenCode)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_factory_unregister() {
        let factory = AdapterFactory::with_default_config();

        factory
            .register_with_default("test".to_string(), AgentType::ClaudeCode)
            .await
            .unwrap();

        factory.unregister("test").await.unwrap();

        let registered = factory.list_registered().await;
        assert!(registered.is_empty());
    }

    #[tokio::test]
    async fn test_factory_unregister_nonexistent_fails() {
        let factory = AdapterFactory::with_default_config();

        let result = factory.unregister("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_factory_get_adapters_by_type() {
        let factory = AdapterFactory::with_default_config();

        factory
            .register_with_default("claude-1".to_string(), AgentType::ClaudeCode)
            .await
            .unwrap();
        factory
            .register_with_default("claude-2".to_string(), AgentType::ClaudeCode)
            .await
            .unwrap();
        factory
            .register_with_default("opencode-1".to_string(), AgentType::OpenCode)
            .await
            .unwrap();

        let claude_adapters = factory.get_adapters_by_type(AgentType::ClaudeCode).await;
        assert_eq!(claude_adapters.len(), 2);

        let opencode_adapters = factory.get_adapters_by_type(AgentType::OpenCode).await;
        assert_eq!(opencode_adapters.len(), 1);

        let cursor_adapters = factory.get_adapters_by_type(AgentType::Cursor).await;
        assert!(cursor_adapters.is_empty());
    }

    #[tokio::test]
    async fn test_factory_is_initialized() {
        let factory = AdapterFactory::with_default_config();

        factory
            .register_with_default("test".to_string(), AgentType::ClaudeCode)
            .await
            .unwrap();

        assert!(!factory.is_initialized("test").await);
    }

    #[tokio::test]
    async fn test_factory_eager_initialization() {
        let config = AdapterFactoryConfig {
            lazy_initialization: false,
            ..Default::default()
        };

        let factory = AdapterFactory::new(config);

        factory
            .register_with_default("test".to_string(), AgentType::ClaudeCode)
            .await
            .unwrap();

        assert!(factory.is_initialized("test").await);
    }

    #[tokio::test]
    async fn test_factory_initialize_all() {
        let factory = AdapterFactory::with_default_config();

        factory
            .register_with_default("test-1".to_string(), AgentType::ClaudeCode)
            .await
            .unwrap();
        factory
            .register_with_default("test-2".to_string(), AgentType::OpenCode)
            .await
            .unwrap();

        let results = factory.initialize_all().await;
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|(_, r)| r.is_ok()));

        assert!(factory.is_initialized("test-1").await);
        assert!(factory.is_initialized("test-2").await);
    }

    #[tokio::test]
    async fn test_factory_health_check_all() {
        let factory = AdapterFactory::with_default_config();

        factory
            .register_with_default("test".to_string(), AgentType::ClaudeCode)
            .await
            .unwrap();

        let health = factory.health_check_all().await;
        assert_eq!(health.len(), 1);
        assert!(!health.get("test").unwrap());
    }

    #[tokio::test]
    async fn test_adapter_factory_config_default() {
        let config = AdapterFactoryConfig::default();
        assert_eq!(config.health_check_interval_secs, 60);
        assert_eq!(config.max_retry_attempts, 3);
        assert_eq!(config.retry_delay_secs, 5);
        assert!(config.lazy_initialization);
    }

    #[test]
    fn test_lazy_adapter_creation() {
        let lazy = LazyAdapter::new(
            "test".to_string(),
            AdapterConfig::ClaudeCode(ClaudeCodeConfig::default()),
        );

        assert!(!lazy.is_initialized());
        assert!(lazy.get().is_none());
    }

    #[test]
    fn test_lazy_adapter_initialize() {
        let mut lazy = LazyAdapter::new(
            "test".to_string(),
            AdapterConfig::ClaudeCode(ClaudeCodeConfig::default()),
        );

        lazy.initialize().unwrap();

        assert!(lazy.is_initialized());
        assert!(lazy.get().is_some());
    }
}
