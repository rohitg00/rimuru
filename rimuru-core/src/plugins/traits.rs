use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::error::RimuruResult;
use crate::models::{CostRecord, Session};

use super::types::{PluginConfig, PluginContext, PluginInfo};

#[async_trait]
pub trait Plugin: Send + Sync {
    fn info(&self) -> &PluginInfo;

    async fn init(&mut self, ctx: &PluginContext) -> RimuruResult<()>;

    async fn shutdown(&mut self) -> RimuruResult<()>;

    fn is_initialized(&self) -> bool;

    fn configure(&mut self, config: PluginConfig) -> RimuruResult<()>;

    fn config_schema(&self) -> Option<serde_json::Value> {
        None
    }
}

#[async_trait]
pub trait AgentPlugin: Plugin {
    fn agent_type(&self) -> &str;

    async fn connect(&mut self) -> RimuruResult<()>;

    async fn disconnect(&mut self) -> RimuruResult<()>;

    fn is_connected(&self) -> bool;

    async fn get_sessions(&self) -> RimuruResult<Vec<Session>>;

    async fn get_costs(&self) -> RimuruResult<Vec<CostRecord>>;

    async fn watch_sessions(&self, callback: SessionCallback) -> RimuruResult<()>;
}

pub type SessionCallback = Arc<dyn Fn(Session) + Send + Sync>;

#[async_trait]
pub trait ExporterPlugin: Plugin {
    fn format(&self) -> &str;

    fn file_extension(&self) -> &str;

    async fn export_sessions(
        &self,
        sessions: &[Session],
        options: ExportOptions,
    ) -> RimuruResult<Vec<u8>>;

    async fn export_costs(
        &self,
        costs: &[CostRecord],
        options: ExportOptions,
    ) -> RimuruResult<Vec<u8>>;

    async fn export_to_file(&self, data: ExportData, path: &std::path::Path) -> RimuruResult<()> {
        let bytes = match data {
            ExportData::Sessions(sessions) => {
                self.export_sessions(&sessions, ExportOptions::default())
                    .await?
            }
            ExportData::Costs(costs) => self.export_costs(&costs, ExportOptions::default()).await?,
            ExportData::Combined { sessions, costs } => {
                let mut result = self
                    .export_sessions(&sessions, ExportOptions::default())
                    .await?;
                result.extend(b"\n");
                result.extend(self.export_costs(&costs, ExportOptions::default()).await?);
                result
            }
        };
        tokio::fs::write(path, bytes).await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExportOptions {
    pub include_headers: bool,
    pub date_format: Option<String>,
    pub timezone: Option<String>,
    pub fields: Option<Vec<String>>,
    pub pretty: bool,
}

pub enum ExportData {
    Sessions(Vec<Session>),
    Costs(Vec<CostRecord>),
    Combined {
        sessions: Vec<Session>,
        costs: Vec<CostRecord>,
    },
}

#[async_trait]
pub trait NotifierPlugin: Plugin {
    fn notification_type(&self) -> &str;

    async fn send(&self, notification: Notification) -> RimuruResult<()>;

    async fn send_batch(&self, notifications: Vec<Notification>) -> RimuruResult<()> {
        for notification in notifications {
            self.send(notification).await?;
        }
        Ok(())
    }

    async fn test_connection(&self) -> RimuruResult<bool>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub title: String,
    pub message: String,
    pub level: NotificationLevel,
    pub data: Option<serde_json::Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Notification {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            level: NotificationLevel::Info,
            data: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn info(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title, message).with_level(NotificationLevel::Info)
    }

    pub fn warning(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title, message).with_level(NotificationLevel::Warning)
    }

    pub fn error(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title, message).with_level(NotificationLevel::Error)
    }

    pub fn critical(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title, message).with_level(NotificationLevel::Critical)
    }

    pub fn with_level(mut self, level: NotificationLevel) -> Self {
        self.level = level;
        self
    }

    pub fn with_data<T: Serialize>(mut self, data: T) -> Self {
        self.data = serde_json::to_value(data).ok();
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum NotificationLevel {
    Debug,
    #[default]
    Info,
    Warning,
    Error,
    Critical,
}

#[async_trait]
pub trait ViewPlugin: Plugin {
    fn view_name(&self) -> &str;

    fn view_title(&self) -> &str;

    fn keybind(&self) -> Option<char>;

    async fn render(&self, ctx: &ViewContext) -> RimuruResult<ViewOutput>;

    async fn handle_input(&mut self, input: ViewInput) -> RimuruResult<ViewAction>;
}

#[derive(Debug, Clone)]
pub struct ViewContext {
    pub width: u16,
    pub height: u16,
    pub focused: bool,
    pub data: HashMap<String, serde_json::Value>,
}

impl ViewContext {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            focused: false,
            data: HashMap::new(),
        }
    }

    pub fn set_data<T: Serialize>(&mut self, key: &str, value: T) {
        if let Ok(v) = serde_json::to_value(value) {
            self.data.insert(key.to_string(), v);
        }
    }

    pub fn get_data<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.data
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }
}

#[derive(Debug, Clone)]
pub enum ViewOutput {
    Text(String),
    Lines(Vec<String>),
    Widget(WidgetData),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetData {
    pub widget_type: String,
    pub props: serde_json::Value,
}

#[derive(Debug, Clone)]
pub enum ViewInput {
    Key(char),
    Enter,
    Escape,
    Tab,
    BackTab,
    Up,
    Down,
    Left,
    Right,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum ViewAction {
    None,
    Refresh,
    Close,
    Navigate(String),
    Custom(String, serde_json::Value),
}

pub trait PluginFactory: Send + Sync {
    fn create(&self) -> Box<dyn Plugin>;
    fn plugin_type(&self) -> &'static str;
}

pub type DynPlugin = Box<dyn Plugin>;
pub type DynAgentPlugin = Box<dyn AgentPlugin>;
pub type DynExporterPlugin = Box<dyn ExporterPlugin>;
pub type DynNotifierPlugin = Box<dyn NotifierPlugin>;
pub type DynViewPlugin = Box<dyn ViewPlugin>;
