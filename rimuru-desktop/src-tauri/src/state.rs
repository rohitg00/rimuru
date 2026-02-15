use rimuru_core::{
    create_builtin_exporter, create_builtin_notifier, init_database_with_url, list_builtin_plugins,
    skillkit::SkillKitBridge, ConfigLoadError, CostAlertConfig, CostAlertHandler, Database,
    DatabaseError, Hook, HookManager, HttpMethod, MetricsExportConfig, MetricsExportHandler,
    PluginLoader, PluginRegistry, RimuruConfig, SessionLogConfig, SessionLogFormat,
    SessionLogHandler, SessionStartLogHandler, WebhookConfig, WebhookHandler,
};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::groupchat::ChatRoomManager;
use crate::pty::manager::PtySessionManager;
use crate::remote::server::RemoteServer;

use crate::commands::settings::AppSettings;

#[derive(Debug, Error)]
pub enum AppStateError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigLoadError),

    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
}

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub config: Arc<RimuruConfig>,
    pub last_sync: Arc<RwLock<Option<chrono::DateTime<chrono::Utc>>>>,
    pub skillkit_bridge: Arc<RwLock<Option<SkillKitBridge>>>,
    pub plugin_registry: Arc<PluginRegistry>,
    pub hook_manager: Arc<HookManager>,
    pub settings: Arc<RwLock<AppSettings>>,
    pub hook_last_triggered: Arc<RwLock<HashMap<String, chrono::DateTime<chrono::Utc>>>>,
    pub plugin_events: Arc<RwLock<Vec<PluginEventRecord>>>,
    pub pty_manager: Arc<PtySessionManager>,
    pub chat_manager: Arc<ChatRoomManager>,
    pub remote_server: Arc<RwLock<Option<RemoteServer>>>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PluginEventRecord {
    pub event: String,
    pub plugin_id: String,
    pub timestamp: String,
    pub error: Option<String>,
}

impl AppState {
    pub async fn new() -> Result<Self, AppStateError> {
        let config = RimuruConfig::load()?;
        let db = init_database_with_url(&config.database.url).await?;

        let skillkit_bridge = match SkillKitBridge::new().await {
            Ok(bridge) => {
                if bridge.is_available() {
                    info!("SkillKit bridge initialized successfully");
                    Some(bridge)
                } else {
                    warn!("SkillKit is not available - install with: npm i -g skillkit");
                    Some(bridge)
                }
            }
            Err(e) => {
                warn!("Failed to initialize SkillKit bridge: {}. Skills features will be unavailable.", e);
                None
            }
        };

        let plugin_loader = match PluginLoader::with_default_dir() {
            Ok(loader) => {
                let _ = loader.ensure_plugins_dir().await;
                Arc::new(loader)
            }
            Err(e) => {
                warn!("Failed to create plugin loader: {}. Using temp dir.", e);
                let temp_dir = std::env::temp_dir().join("rimuru-plugins");
                let loader = PluginLoader::new(temp_dir);
                let _ = loader.ensure_plugins_dir().await;
                Arc::new(loader)
            }
        };

        let plugin_registry = Arc::new(PluginRegistry::new(plugin_loader));

        if let Some(csv) = create_builtin_exporter("csv-exporter") {
            if let Err(e) = plugin_registry
                .register_exporter_plugin("csv-exporter", csv)
                .await
            {
                warn!("Failed to register CSV exporter: {}", e);
            }
        }
        if let Some(json) = create_builtin_exporter("json-exporter") {
            if let Err(e) = plugin_registry
                .register_exporter_plugin("json-exporter", json)
                .await
            {
                warn!("Failed to register JSON exporter: {}", e);
            }
        }
        if let Some(slack) = create_builtin_notifier("slack-notifier") {
            if let Err(e) = plugin_registry
                .register_notifier_plugin("slack-notifier", slack)
                .await
            {
                warn!("Failed to register Slack notifier: {}", e);
            }
        }
        if let Some(discord) = create_builtin_notifier("discord-notifier") {
            if let Err(e) = plugin_registry
                .register_notifier_plugin("discord-notifier", discord)
                .await
            {
                warn!("Failed to register Discord notifier: {}", e);
            }
        }
        if let Some(webhook) = create_builtin_notifier("webhook-notifier") {
            if let Err(e) = plugin_registry
                .register_notifier_plugin("webhook-notifier", webhook)
                .await
            {
                warn!("Failed to register Webhook notifier: {}", e);
            }
        }

        let hook_manager = Arc::new(HookManager::new().with_max_history(500));

        let cost_alert = Arc::new(CostAlertHandler::new(CostAlertConfig::default()));
        if let Err(e) = hook_manager.register(cost_alert).await {
            warn!("Failed to register cost alert handler: {}", e);
        }

        let session_log = Arc::new(SessionLogHandler::new(SessionLogConfig::default()));
        if let Err(e) = hook_manager.register(session_log).await {
            warn!("Failed to register session log handler: {}", e);
        }

        let session_start_log = Arc::new(SessionStartLogHandler::new(SessionLogConfig::default()));
        if let Err(e) = hook_manager.register(session_start_log).await {
            warn!("Failed to register session start log handler: {}", e);
        }

        let metrics_export = Arc::new(MetricsExportHandler::new(MetricsExportConfig::default()));
        if let Err(e) = hook_manager.register(metrics_export).await {
            warn!("Failed to register metrics export handler: {}", e);
        }

        info!(
            "Plugin system initialized: {} builtin plugins registered",
            list_builtin_plugins().len()
        );

        Ok(Self {
            db: Arc::new(db),
            config: Arc::new(config),
            last_sync: Arc::new(RwLock::new(None)),
            skillkit_bridge: Arc::new(RwLock::new(skillkit_bridge)),
            plugin_registry,
            hook_manager,
            settings: Arc::new(RwLock::new(AppSettings::default())),
            hook_last_triggered: Arc::new(RwLock::new(HashMap::new())),
            plugin_events: Arc::new(RwLock::new(Vec::new())),
            pty_manager: Arc::new(PtySessionManager::new()),
            chat_manager: Arc::new(ChatRoomManager::new()),
            remote_server: Arc::new(RwLock::new(None)),
        })
    }
}
