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
pub struct GooseSessionData {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub profile: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub working_directory: Option<String>,
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub ended_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub input_tokens: i64,
    #[serde(default)]
    pub output_tokens: i64,
    #[serde(default)]
    pub total_cost_usd: Option<f64>,
    #[serde(default)]
    pub messages: Option<Vec<GooseMessage>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GooseMessage {
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub timestamp: Option<DateTime<Utc>>,
    #[serde(default)]
    pub tokens: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GooseHistoryEntry {
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub profile: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub working_directory: Option<String>,
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
    sessions_dir: PathBuf,
    data_dir: PathBuf,
}

impl SessionParser {
    pub fn new(sessions_dir: PathBuf, data_dir: PathBuf) -> Self {
        Self {
            sessions_dir,
            data_dir,
        }
    }

    pub fn parse_sessions(&self) -> RimuruResult<Vec<SessionHistory>> {
        let mut sessions = Vec::new();

        if let Ok(file_sessions) = self.parse_session_files() {
            sessions.extend(file_sessions);
        }

        if let Ok(db_sessions) = self.parse_database_sessions() {
            for session in db_sessions {
                if !sessions.iter().any(|s| s.session_id == session.session_id) {
                    sessions.push(session);
                }
            }
        }

        sessions.sort_by(|a, b| b.started_at.cmp(&a.started_at));

        Ok(sessions)
    }

    fn parse_session_files(&self) -> RimuruResult<Vec<SessionHistory>> {
        let mut sessions = Vec::new();

        if !self.sessions_dir.exists() {
            debug!("Sessions directory does not exist: {:?}", self.sessions_dir);
            return Ok(sessions);
        }

        let entries = match fs::read_dir(&self.sessions_dir) {
            Ok(entries) => entries,
            Err(e) => {
                warn!("Failed to read sessions directory: {}", e);
                return Ok(sessions);
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_file() {
                let ext = path.extension().and_then(|e| e.to_str());
                if ext == Some("json") || ext == Some("jsonl") {
                    if let Ok(session) = self.parse_session_file(&path) {
                        sessions.push(session);
                    }
                }
            }

            if path.is_dir() {
                if let Ok(dir_sessions) = self.parse_session_directory(&path) {
                    sessions.extend(dir_sessions);
                }
            }
        }

        Ok(sessions)
    }

    fn parse_session_file(&self, path: &Path) -> RimuruResult<SessionHistory> {
        let content = fs::read_to_string(path)?;

        if let Ok(session_data) = serde_json::from_str::<GooseSessionData>(&content) {
            return self.convert_session_data_to_history(&session_data);
        }

        if let Ok(history_entry) = serde_json::from_str::<GooseHistoryEntry>(&content) {
            return self.convert_history_entry_to_session(&history_entry);
        }

        Err(crate::error::RimuruError::SerializationError(
            "Failed to parse Goose session file".to_string(),
        ))
    }

    fn parse_session_directory(&self, dir: &Path) -> RimuruResult<Vec<SessionHistory>> {
        let mut sessions = Vec::new();

        let session_file = dir.join("session.json");
        if session_file.exists() {
            if let Ok(session) = self.parse_session_file(&session_file) {
                sessions.push(session);
            }
        }

        let metadata_file = dir.join("metadata.json");
        if metadata_file.exists() && sessions.is_empty() {
            if let Ok(session) = self.parse_session_file(&metadata_file) {
                sessions.push(session);
            }
        }

        Ok(sessions)
    }

    fn parse_database_sessions(&self) -> RimuruResult<Vec<SessionHistory>> {
        let db_path = self.data_dir.join("goose.db");
        if !db_path.exists() {
            return Ok(vec![]);
        }

        debug!(
            "SQLite database found at {:?}, but parsing not implemented yet",
            db_path
        );
        Ok(vec![])
    }

    fn convert_session_data_to_history(
        &self,
        data: &GooseSessionData,
    ) -> RimuruResult<SessionHistory> {
        let session_id = data
            .id
            .as_ref()
            .and_then(|id| Uuid::parse_str(id).ok())
            .unwrap_or_else(Uuid::new_v4);

        let started_at = data.created_at.unwrap_or_else(Utc::now);

        let model_name = data
            .model
            .clone()
            .or_else(|| data.provider.as_ref().map(|p| format!("{}/default", p)));

        Ok(SessionHistory {
            session_id,
            agent_type: AgentType::Goose,
            started_at,
            ended_at: data.ended_at.or(data.updated_at),
            total_input_tokens: data.input_tokens,
            total_output_tokens: data.output_tokens,
            model_name,
            cost_usd: data.total_cost_usd,
            project_path: data.working_directory.clone(),
        })
    }

    fn convert_history_entry_to_session(
        &self,
        entry: &GooseHistoryEntry,
    ) -> RimuruResult<SessionHistory> {
        let session_id = entry
            .session_id
            .as_ref()
            .and_then(|id| Uuid::parse_str(id).ok())
            .unwrap_or_else(Uuid::new_v4);

        let started_at = entry.started_at.unwrap_or_else(Utc::now);

        let model_name = entry
            .model
            .clone()
            .or_else(|| entry.provider.as_ref().map(|p| format!("{}/default", p)));

        Ok(SessionHistory {
            session_id,
            agent_type: AgentType::Goose,
            started_at,
            ended_at: entry.ended_at,
            total_input_tokens: entry.total_input_tokens,
            total_output_tokens: entry.total_output_tokens,
            model_name,
            cost_usd: Some(entry.total_cost_usd),
            project_path: entry.working_directory.clone(),
        })
    }

    pub fn get_active_session(&self) -> RimuruResult<Option<ActiveSession>> {
        if !self.sessions_dir.exists() {
            return Ok(None);
        }

        let entries = match fs::read_dir(&self.sessions_dir) {
            Ok(entries) => entries,
            Err(_) => return Ok(None),
        };

        let mut most_recent: Option<(DateTime<Utc>, ActiveSession)> = None;

        for entry in entries.flatten() {
            let path = entry.path();

            let session_file = if path.is_dir() {
                path.join("session.json")
            } else if path.extension().is_some_and(|e| e == "json") {
                path.clone()
            } else {
                continue;
            };

            if !session_file.exists() {
                continue;
            }

            if let Ok(content) = fs::read_to_string(&session_file) {
                if let Ok(data) = serde_json::from_str::<GooseSessionData>(&content) {
                    if data.ended_at.is_some() {
                        continue;
                    }

                    let updated_at = data.updated_at.or(data.created_at);
                    if let Some(updated_at) = updated_at {
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
                                agent_type: AgentType::Goose,
                                started_at: data.created_at.unwrap_or(updated_at),
                                current_tokens: data.input_tokens + data.output_tokens,
                                model_name: data.model.clone(),
                                project_path: data.working_directory.clone(),
                            };

                            if most_recent.is_none() || updated_at > most_recent.as_ref().unwrap().0
                            {
                                most_recent = Some((updated_at, active));
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
        let parser = SessionParser::new(
            temp_dir.path().join("sessions"),
            temp_dir.path().to_path_buf(),
        );

        let sessions = parser.parse_sessions().unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_session_parser_with_session_file() {
        let temp_dir = tempdir().unwrap();
        let sessions_dir = temp_dir.path().join("sessions");
        fs::create_dir_all(&sessions_dir).unwrap();

        let session_data = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "provider": "anthropic",
            "model": "claude-3-sonnet",
            "working_directory": "/home/user/project",
            "input_tokens": 5000,
            "output_tokens": 2000,
            "total_cost_usd": 0.035
        }"#;

        fs::write(sessions_dir.join("session1.json"), session_data).unwrap();

        let parser = SessionParser::new(sessions_dir, temp_dir.path().to_path_buf());
        let sessions = parser.parse_sessions().unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].total_input_tokens, 5000);
        assert_eq!(sessions[0].total_output_tokens, 2000);
        assert_eq!(sessions[0].model_name, Some("claude-3-sonnet".to_string()));
    }

    #[test]
    fn test_session_parser_with_history_entry() {
        let temp_dir = tempdir().unwrap();
        let sessions_dir = temp_dir.path().join("sessions");
        fs::create_dir_all(&sessions_dir).unwrap();

        let history_data = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440001",
            "profile": "default",
            "provider": "openai",
            "model": "gpt-4o",
            "input_tokens": 10000,
            "output_tokens": 5000,
            "total_cost_usd": 0.075
        }"#;

        fs::write(sessions_dir.join("history.json"), history_data).unwrap();

        let parser = SessionParser::new(sessions_dir, temp_dir.path().to_path_buf());
        let sessions = parser.parse_sessions().unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].total_input_tokens, 10000);
        assert_eq!(sessions[0].total_output_tokens, 5000);
        assert_eq!(sessions[0].cost_usd, Some(0.075));
    }

    #[test]
    fn test_session_parser_with_directory_structure() {
        let temp_dir = tempdir().unwrap();
        let sessions_dir = temp_dir.path().join("sessions");
        let session_subdir = sessions_dir.join("session-abc123");
        fs::create_dir_all(&session_subdir).unwrap();

        let session_data = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440002",
            "provider": "anthropic",
            "model": "claude-3-opus",
            "input_tokens": 15000,
            "output_tokens": 7500
        }"#;

        fs::write(session_subdir.join("session.json"), session_data).unwrap();

        let parser = SessionParser::new(sessions_dir, temp_dir.path().to_path_buf());
        let sessions = parser.parse_sessions().unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].total_input_tokens, 15000);
        assert_eq!(sessions[0].total_output_tokens, 7500);
    }

    #[test]
    fn test_get_active_session_none() {
        let temp_dir = tempdir().unwrap();
        let parser = SessionParser::new(
            temp_dir.path().join("sessions"),
            temp_dir.path().to_path_buf(),
        );

        let active = parser.get_active_session().unwrap();
        assert!(active.is_none());
    }

    #[test]
    fn test_get_active_session_with_ended_session() {
        let temp_dir = tempdir().unwrap();
        let sessions_dir = temp_dir.path().join("sessions");
        fs::create_dir_all(&sessions_dir).unwrap();

        let ended_session = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440003",
            "ended_at": "2025-01-01T00:00:00Z",
            "input_tokens": 1000,
            "output_tokens": 500
        }"#;

        fs::write(sessions_dir.join("ended.json"), ended_session).unwrap();

        let parser = SessionParser::new(sessions_dir, temp_dir.path().to_path_buf());
        let active = parser.get_active_session().unwrap();
        assert!(active.is_none());
    }
}
