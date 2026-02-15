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
pub struct CodexSessionData {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub input_tokens: i64,
    #[serde(default)]
    pub output_tokens: i64,
    #[serde(default)]
    pub reasoning_tokens: i64,
    #[serde(default)]
    pub cost_usd: Option<f64>,
    #[serde(default)]
    pub instructions: Option<String>,
    #[serde(default)]
    pub approval_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexHistoryEntry {
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub ended_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub total_input_tokens: i64,
    #[serde(default)]
    pub total_output_tokens: i64,
    #[serde(default)]
    pub total_cost_usd: f64,
}

pub struct SessionParser {
    data_dir: PathBuf,
}

impl SessionParser {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    pub fn parse_sessions(&self) -> RimuruResult<Vec<SessionHistory>> {
        let mut sessions = Vec::new();

        let history_file = self.data_dir.join("history.json");
        if history_file.exists() {
            if let Ok(history_sessions) = self.parse_history_file(&history_file) {
                sessions.extend(history_sessions);
            }
        }

        let sessions_dir = self.data_dir.join("sessions");
        if sessions_dir.exists() {
            if let Ok(session_sessions) = self.parse_sessions_directory(&sessions_dir) {
                for session in session_sessions {
                    if !sessions.iter().any(|s| s.session_id == session.session_id) {
                        sessions.push(session);
                    }
                }
            }
        }

        sessions.sort_by(|a, b| b.started_at.cmp(&a.started_at));

        Ok(sessions)
    }

    fn parse_history_file(&self, path: &Path) -> RimuruResult<Vec<SessionHistory>> {
        let mut sessions = Vec::new();

        if !path.exists() {
            return Ok(sessions);
        }

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to read history file: {}", e);
                return Ok(sessions);
            }
        };

        if let Ok(entries) = serde_json::from_str::<Vec<CodexHistoryEntry>>(&content) {
            for entry in entries {
                if let Some(session) = self.convert_history_entry_to_session(&entry) {
                    sessions.push(session);
                }
            }
        }

        Ok(sessions)
    }

    fn parse_sessions_directory(&self, sessions_dir: &Path) -> RimuruResult<Vec<SessionHistory>> {
        let mut sessions = Vec::new();

        if !sessions_dir.exists() {
            debug!("Sessions directory does not exist: {:?}", sessions_dir);
            return Ok(sessions);
        }

        let entries = match fs::read_dir(sessions_dir) {
            Ok(entries) => entries,
            Err(e) => {
                warn!("Failed to read sessions directory: {}", e);
                return Ok(sessions);
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(session_data) = serde_json::from_str::<CodexSessionData>(&content) {
                        if let Some(session) = self.convert_session_data_to_history(&session_data) {
                            sessions.push(session);
                        }
                    }
                }
            }
        }

        Ok(sessions)
    }

    fn convert_history_entry_to_session(
        &self,
        entry: &CodexHistoryEntry,
    ) -> Option<SessionHistory> {
        let session_id = entry
            .session_id
            .as_ref()
            .and_then(|id| Uuid::parse_str(id).ok())
            .unwrap_or_else(Uuid::new_v4);

        let started_at = entry.started_at.unwrap_or_else(Utc::now);

        Some(SessionHistory {
            session_id,
            agent_type: AgentType::Codex,
            started_at,
            ended_at: entry.ended_at,
            total_input_tokens: entry.total_input_tokens,
            total_output_tokens: entry.total_output_tokens,
            model_name: entry.model.clone(),
            cost_usd: Some(entry.total_cost_usd),
            project_path: entry.cwd.clone(),
        })
    }

    fn convert_session_data_to_history(&self, data: &CodexSessionData) -> Option<SessionHistory> {
        let session_id = data
            .id
            .as_ref()
            .and_then(|id| Uuid::parse_str(id).ok())
            .unwrap_or_else(Uuid::new_v4);

        let started_at = data.created_at.unwrap_or_else(Utc::now);

        Some(SessionHistory {
            session_id,
            agent_type: AgentType::Codex,
            started_at,
            ended_at: data.updated_at,
            total_input_tokens: data.input_tokens,
            total_output_tokens: data.output_tokens + data.reasoning_tokens,
            model_name: data.model.clone(),
            cost_usd: data.cost_usd,
            project_path: data.cwd.clone(),
        })
    }

    pub fn get_active_session(&self) -> RimuruResult<Option<ActiveSession>> {
        let sessions_dir = self.data_dir.join("sessions");
        if !sessions_dir.exists() {
            return Ok(None);
        }

        let entries = match fs::read_dir(&sessions_dir) {
            Ok(entries) => entries,
            Err(_) => return Ok(None),
        };

        let mut most_recent: Option<(DateTime<Utc>, ActiveSession)> = None;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(data) = serde_json::from_str::<CodexSessionData>(&content) {
                        if let Some(updated_at) = data.updated_at {
                            let now = Utc::now();
                            let diff = now.signed_duration_since(updated_at);

                            if diff.num_minutes() < 30 {
                                let session_id = data
                                    .id
                                    .as_ref()
                                    .and_then(|id| Uuid::parse_str(id).ok())
                                    .unwrap_or_else(Uuid::new_v4);

                                let active = ActiveSession {
                                    session_id,
                                    agent_type: AgentType::Codex,
                                    started_at: data.created_at.unwrap_or(updated_at),
                                    current_tokens: data.input_tokens
                                        + data.output_tokens
                                        + data.reasoning_tokens,
                                    model_name: data.model.clone(),
                                    project_path: data.cwd.clone(),
                                };

                                if most_recent.is_none()
                                    || updated_at > most_recent.as_ref().unwrap().0
                                {
                                    most_recent = Some((updated_at, active));
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
        let parser = SessionParser::new(temp_dir.path().to_path_buf());

        let sessions = parser.parse_sessions().unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_session_parser_with_history_file() {
        let temp_dir = tempdir().unwrap();

        let history_data = r#"[
            {
                "session_id": "550e8400-e29b-41d4-a716-446655440000",
                "cwd": "/home/user/project",
                "model": "gpt-4o",
                "total_input_tokens": 5000,
                "total_output_tokens": 2000,
                "total_cost_usd": 0.035
            }
        ]"#;

        fs::write(temp_dir.path().join("history.json"), history_data).unwrap();

        let parser = SessionParser::new(temp_dir.path().to_path_buf());
        let sessions = parser.parse_sessions().unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].total_input_tokens, 5000);
        assert_eq!(sessions[0].total_output_tokens, 2000);
        assert_eq!(sessions[0].model_name, Some("gpt-4o".to_string()));
    }

    #[test]
    fn test_session_parser_with_session_file() {
        let temp_dir = tempdir().unwrap();
        let sessions_dir = temp_dir.path().join("sessions");
        fs::create_dir_all(&sessions_dir).unwrap();

        let session_data = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440001",
            "cwd": "/home/user/project2",
            "model": "o4-mini",
            "input_tokens": 10000,
            "output_tokens": 5000,
            "reasoning_tokens": 1000,
            "cost_usd": 0.05
        }"#;

        fs::write(sessions_dir.join("session1.json"), session_data).unwrap();

        let parser = SessionParser::new(temp_dir.path().to_path_buf());
        let sessions = parser.parse_sessions().unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].total_input_tokens, 10000);
        assert_eq!(sessions[0].total_output_tokens, 6000);
        assert_eq!(sessions[0].cost_usd, Some(0.05));
    }

    #[test]
    fn test_get_active_session_none() {
        let temp_dir = tempdir().unwrap();
        let parser = SessionParser::new(temp_dir.path().to_path_buf());

        let active = parser.get_active_session().unwrap();
        assert!(active.is_none());
    }
}
