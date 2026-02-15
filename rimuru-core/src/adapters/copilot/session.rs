use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};
use uuid::Uuid;

use crate::adapters::types::{ActiveSession, SessionHistory};
use crate::error::RimuruResult;
use crate::models::AgentType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotSessionData {
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub editor: Option<String>,
    #[serde(default)]
    pub workspace: Option<String>,
    #[serde(default)]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub last_activity: Option<DateTime<Utc>>,
    #[serde(default)]
    pub suggestions_shown: i64,
    #[serde(default)]
    pub suggestions_accepted: i64,
    #[serde(default)]
    pub characters_inserted: i64,
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotTelemetryEntry {
    #[serde(default)]
    pub event_type: Option<String>,
    #[serde(default)]
    pub timestamp: Option<DateTime<Utc>>,
    #[serde(default)]
    pub editor: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub suggestion_id: Option<String>,
    #[serde(default)]
    pub accepted: bool,
    #[serde(default)]
    pub characters: i64,
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotUsageEntry {
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub editor: Option<String>,
    #[serde(default)]
    pub total_suggestions: i64,
    #[serde(default)]
    pub accepted_suggestions: i64,
    #[serde(default)]
    pub total_characters: i64,
    #[serde(default)]
    pub languages: Vec<String>,
}

pub struct SessionParser {
    config_dir: PathBuf,
    vscode_extensions_dir: PathBuf,
}

impl SessionParser {
    pub fn new(config_dir: PathBuf, vscode_extensions_dir: PathBuf) -> Self {
        Self {
            config_dir,
            vscode_extensions_dir,
        }
    }

    pub fn parse_sessions(&self) -> RimuruResult<Vec<SessionHistory>> {
        let mut sessions = Vec::new();

        if let Ok(usage_sessions) = self.parse_usage_cache() {
            sessions.extend(usage_sessions);
        }

        if let Ok(telemetry_sessions) = self.parse_telemetry_data() {
            for session in telemetry_sessions {
                if !sessions.iter().any(|s| s.session_id == session.session_id) {
                    sessions.push(session);
                }
            }
        }

        if let Ok(vscode_sessions) = self.parse_vscode_telemetry() {
            for session in vscode_sessions {
                if !sessions.iter().any(|s| s.session_id == session.session_id) {
                    sessions.push(session);
                }
            }
        }

        sessions.sort_by(|a, b| b.started_at.cmp(&a.started_at));

        Ok(sessions)
    }

    fn parse_usage_cache(&self) -> RimuruResult<Vec<SessionHistory>> {
        let mut sessions = Vec::new();

        let usage_dir = self.config_dir.join("usage");
        if !usage_dir.exists() {
            debug!("Usage cache directory does not exist: {:?}", usage_dir);
            return Ok(sessions);
        }

        let entries = match fs::read_dir(&usage_dir) {
            Ok(entries) => entries,
            Err(e) => {
                warn!("Failed to read usage directory: {}", e);
                return Ok(sessions);
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(usage_data) = serde_json::from_str::<CopilotUsageEntry>(&content) {
                        if let Some(session) = self.convert_usage_to_session(&usage_data) {
                            sessions.push(session);
                        }
                    }
                }
            }
        }

        Ok(sessions)
    }

    fn parse_telemetry_data(&self) -> RimuruResult<Vec<SessionHistory>> {
        let mut sessions = Vec::new();

        let telemetry_file = self.config_dir.join("telemetry.json");
        if !telemetry_file.exists() {
            return Ok(sessions);
        }

        let content = match fs::read_to_string(&telemetry_file) {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to read telemetry file: {}", e);
                return Ok(sessions);
            }
        };

        if let Ok(entries) = serde_json::from_str::<Vec<CopilotTelemetryEntry>>(&content) {
            let grouped = self.group_telemetry_by_session(&entries);
            for (session_id, group) in grouped {
                if let Some(session) = self.convert_telemetry_group_to_session(session_id, &group) {
                    sessions.push(session);
                }
            }
        }

        Ok(sessions)
    }

    fn parse_vscode_telemetry(&self) -> RimuruResult<Vec<SessionHistory>> {
        let mut sessions = Vec::new();

        if !self.vscode_extensions_dir.exists() {
            return Ok(sessions);
        }

        let copilot_ext = self.find_copilot_extension(&self.vscode_extensions_dir);
        if let Some(ext_path) = copilot_ext {
            let telemetry_dir = ext_path.join("telemetry");
            if telemetry_dir.exists() {
                if let Ok(dir_sessions) = self.parse_telemetry_directory(&telemetry_dir) {
                    sessions.extend(dir_sessions);
                }
            }
        }

        Ok(sessions)
    }

    fn find_copilot_extension(&self, vscode_dir: &Path) -> Option<PathBuf> {
        let entries = fs::read_dir(vscode_dir).ok()?;
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("github.copilot-") && !name.contains("chat") {
                return Some(entry.path());
            }
        }
        None
    }

    fn parse_telemetry_directory(&self, dir: &Path) -> RimuruResult<Vec<SessionHistory>> {
        let mut sessions = Vec::new();

        if !dir.exists() {
            return Ok(sessions);
        }

        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => return Ok(sessions),
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(session_data) = serde_json::from_str::<CopilotSessionData>(&content) {
                        if let Some(session) = self.convert_session_data_to_history(&session_data) {
                            sessions.push(session);
                        }
                    }
                }
            }
        }

        Ok(sessions)
    }

    fn group_telemetry_by_session<'a>(
        &self,
        entries: &'a [CopilotTelemetryEntry],
    ) -> std::collections::HashMap<String, Vec<&'a CopilotTelemetryEntry>> {
        let mut grouped: std::collections::HashMap<String, Vec<&'a CopilotTelemetryEntry>> =
            std::collections::HashMap::new();

        for entry in entries {
            let session_key = entry
                .timestamp
                .map(|t| t.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "unknown".to_string());

            grouped.entry(session_key).or_default().push(entry);
        }

        grouped
    }

    fn convert_usage_to_session(&self, usage: &CopilotUsageEntry) -> Option<SessionHistory> {
        let session_id = Uuid::new_v4();

        let started_at = usage
            .date
            .as_ref()
            .and_then(|d| {
                chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                    .ok()
                    .and_then(|date| date.and_hms_opt(0, 0, 0))
                    .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
            })
            .unwrap_or_else(Utc::now);

        let estimated_tokens = usage.total_characters / 4;

        Some(SessionHistory {
            session_id,
            agent_type: AgentType::Copilot,
            started_at,
            ended_at: Some(started_at + chrono::Duration::hours(8)),
            total_input_tokens: estimated_tokens / 2,
            total_output_tokens: estimated_tokens / 2,
            model_name: Some("copilot-gpt-4".to_string()),
            cost_usd: None,
            project_path: None,
        })
    }

    fn convert_telemetry_group_to_session(
        &self,
        _session_key: String,
        entries: &[&CopilotTelemetryEntry],
    ) -> Option<SessionHistory> {
        if entries.is_empty() {
            return None;
        }

        let session_id = Uuid::new_v4();

        let first_entry = entries.iter().filter_map(|e| e.timestamp).min();
        let last_entry = entries.iter().filter_map(|e| e.timestamp).max();

        let total_characters: i64 = entries.iter().map(|e| e.characters).sum();
        let estimated_tokens = total_characters / 4;

        let model_name = entries
            .iter()
            .find_map(|e| e.model.clone())
            .unwrap_or_else(|| "copilot-gpt-4".to_string());

        Some(SessionHistory {
            session_id,
            agent_type: AgentType::Copilot,
            started_at: first_entry.unwrap_or_else(Utc::now),
            ended_at: last_entry,
            total_input_tokens: estimated_tokens / 2,
            total_output_tokens: estimated_tokens / 2,
            model_name: Some(model_name),
            cost_usd: None,
            project_path: None,
        })
    }

    fn convert_session_data_to_history(&self, data: &CopilotSessionData) -> Option<SessionHistory> {
        let session_id = data
            .session_id
            .as_ref()
            .and_then(|id| Uuid::parse_str(id).ok())
            .unwrap_or_else(Uuid::new_v4);

        let started_at = data.started_at.unwrap_or_else(Utc::now);

        let estimated_tokens = data.characters_inserted / 4;

        Some(SessionHistory {
            session_id,
            agent_type: AgentType::Copilot,
            started_at,
            ended_at: data.last_activity,
            total_input_tokens: estimated_tokens / 2,
            total_output_tokens: estimated_tokens / 2,
            model_name: data
                .model
                .clone()
                .or_else(|| Some("copilot-gpt-4".to_string())),
            cost_usd: None,
            project_path: data.workspace.clone(),
        })
    }

    pub fn get_active_session(&self) -> RimuruResult<Option<ActiveSession>> {
        let usage_dir = self.config_dir.join("usage");
        if !usage_dir.exists() {
            return Ok(None);
        }

        let entries = match fs::read_dir(&usage_dir) {
            Ok(entries) => entries,
            Err(_) => return Ok(None),
        };

        let mut most_recent: Option<(DateTime<Utc>, ActiveSession)> = None;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(data) = serde_json::from_str::<CopilotSessionData>(&content) {
                        if let Some(last_activity) = data.last_activity {
                            let now = Utc::now();
                            let diff = now.signed_duration_since(last_activity);

                            if diff.num_minutes() < 30 {
                                let session_id = data
                                    .session_id
                                    .as_ref()
                                    .and_then(|id| Uuid::parse_str(id).ok())
                                    .unwrap_or_else(Uuid::new_v4);

                                let estimated_tokens = data.characters_inserted / 4;

                                let active = ActiveSession {
                                    session_id,
                                    agent_type: AgentType::Copilot,
                                    started_at: data.started_at.unwrap_or(last_activity),
                                    current_tokens: estimated_tokens,
                                    model_name: data.model.clone(),
                                    project_path: data.workspace.clone(),
                                };

                                if most_recent.is_none()
                                    || last_activity > most_recent.as_ref().unwrap().0
                                {
                                    most_recent = Some((last_activity, active));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(most_recent.map(|(_, session)| session))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_session_parser_empty_dir() {
        let temp_dir = tempdir().unwrap();
        let vscode_dir = tempdir().unwrap();
        let parser = SessionParser::new(
            temp_dir.path().to_path_buf(),
            vscode_dir.path().to_path_buf(),
        );

        let sessions = parser.parse_sessions().unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_session_parser_with_usage_file() {
        let temp_dir = tempdir().unwrap();
        let vscode_dir = tempdir().unwrap();

        let usage_dir = temp_dir.path().join("usage");
        fs::create_dir_all(&usage_dir).unwrap();

        let usage_data = r#"{
            "date": "2024-01-15",
            "editor": "vscode",
            "total_suggestions": 100,
            "accepted_suggestions": 75,
            "total_characters": 5000,
            "languages": ["rust", "python"]
        }"#;

        fs::write(usage_dir.join("2024-01-15.json"), usage_data).unwrap();

        let parser = SessionParser::new(
            temp_dir.path().to_path_buf(),
            vscode_dir.path().to_path_buf(),
        );
        let sessions = parser.parse_sessions().unwrap();

        assert_eq!(sessions.len(), 1);
        assert!(sessions[0].total_input_tokens > 0);
    }

    #[test]
    fn test_get_active_session_none() {
        let temp_dir = tempdir().unwrap();
        let vscode_dir = tempdir().unwrap();
        let parser = SessionParser::new(
            temp_dir.path().to_path_buf(),
            vscode_dir.path().to_path_buf(),
        );

        let active = parser.get_active_session().unwrap();
        assert!(active.is_none());
    }

    #[test]
    fn test_convert_usage_to_session() {
        let temp_dir = tempdir().unwrap();
        let vscode_dir = tempdir().unwrap();
        let parser = SessionParser::new(
            temp_dir.path().to_path_buf(),
            vscode_dir.path().to_path_buf(),
        );

        let usage = CopilotUsageEntry {
            date: Some("2024-01-15".to_string()),
            editor: Some("vscode".to_string()),
            total_suggestions: 100,
            accepted_suggestions: 75,
            total_characters: 4000,
            languages: vec!["rust".to_string()],
        };

        let session = parser.convert_usage_to_session(&usage);
        assert!(session.is_some());

        let session = session.unwrap();
        assert_eq!(session.agent_type, AgentType::Copilot);
        assert!(session.total_input_tokens > 0);
    }
}
