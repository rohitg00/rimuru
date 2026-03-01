use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginLanguage {
    Rust,
    TypeScript,
}

impl std::fmt::Display for PluginLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rust => write!(f, "Rust"),
            Self::TypeScript => write!(f, "TypeScript"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub language: PluginLanguage,
    pub binary_path: String,
    pub functions: Vec<String>,
    pub hooks: Vec<HookRegistration>,
    pub enabled: bool,
    pub installed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookRegistration {
    pub event_type: String,
    pub function_id: String,
    pub priority: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginStatus {
    Running,
    Stopped,
    Error,
    Installing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginState {
    pub plugin_id: String,
    pub status: PluginStatus,
    pub pid: Option<u32>,
    pub started_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub restart_count: u32,
}
