use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::debug;
use uuid::Uuid;

use crate::adapters::types::{ActiveSession, SessionHistory};
use crate::error::RimuruResult;
use crate::models::AgentType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorSessionData {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub workspace: Option<String>,
    #[serde(default, rename = "workspacePath")]
    pub workspace_path: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default, rename = "startedAt")]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(default, rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(default, rename = "endedAt")]
    pub ended_at: Option<DateTime<Utc>>,
    #[serde(default, rename = "inputTokens")]
    pub input_tokens: i64,
    #[serde(default, rename = "outputTokens")]
    pub output_tokens: i64,
    #[serde(default, rename = "totalCostUsd")]
    pub total_cost_usd: Option<f64>,
    #[serde(default)]
    pub messages: Option<Vec<CursorMessage>>,
    #[serde(default, rename = "sessionType")]
    pub session_type: Option<CursorSessionType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorMessage {
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub timestamp: Option<DateTime<Utc>>,
    #[serde(default)]
    pub tokens: Option<i64>,
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum CursorSessionType {
    #[default]
    Chat,
    Composer,
    Tab,
    Edit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorChatEntry {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub messages: Vec<CursorMessage>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default, rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default, rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorComposerEntry {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub messages: Vec<CursorMessage>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default, rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default, rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
}

pub struct SessionParser {
    app_data_dir: PathBuf,
    #[allow(dead_code)]
    config_dir: PathBuf,
    logs_dir: PathBuf,
}

impl SessionParser {
    pub fn new(app_data_dir: PathBuf, config_dir: PathBuf, logs_dir: PathBuf) -> Self {
        Self {
            app_data_dir,
            config_dir,
            logs_dir,
        }
    }

    pub fn parse_sessions(&self) -> RimuruResult<Vec<SessionHistory>> {
        let mut sessions = Vec::new();

        if let Ok(chat_sessions) = self.parse_chat_history() {
            sessions.extend(chat_sessions);
        }

        if let Ok(composer_sessions) = self.parse_composer_history() {
            sessions.extend(composer_sessions);
        }

        if let Ok(log_sessions) = self.parse_log_files() {
            for session in log_sessions {
                if !sessions.iter().any(|s| s.session_id == session.session_id) {
                    sessions.push(session);
                }
            }
        }

        sessions.sort_by(|a, b| b.started_at.cmp(&a.started_at));

        Ok(sessions)
    }

    fn parse_chat_history(&self) -> RimuruResult<Vec<SessionHistory>> {
        let mut sessions = Vec::new();

        let chat_dir = self
            .app_data_dir
            .join("User")
            .join("globalStorage")
            .join("cursor.chat");

        if !chat_dir.exists() {
            debug!("Cursor chat directory does not exist: {:?}", chat_dir);
            return Ok(sessions);
        }

        if let Ok(entries) = fs::read_dir(&chat_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let ext = path.extension().and_then(|e| e.to_str());
                    if ext == Some("json") {
                        if let Ok(session) = self.parse_chat_file(&path) {
                            sessions.push(session);
                        }
                    }
                }
            }
        }

        Ok(sessions)
    }

    fn parse_composer_history(&self) -> RimuruResult<Vec<SessionHistory>> {
        let mut sessions = Vec::new();

        let composer_dir = self
            .app_data_dir
            .join("User")
            .join("globalStorage")
            .join("cursor.composer");

        if !composer_dir.exists() {
            debug!(
                "Cursor composer directory does not exist: {:?}",
                composer_dir
            );
            return Ok(sessions);
        }

        if let Ok(entries) = fs::read_dir(&composer_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let ext = path.extension().and_then(|e| e.to_str());
                    if ext == Some("json") {
                        if let Ok(session) = self.parse_composer_file(&path) {
                            sessions.push(session);
                        }
                    }
                }
            }
        }

        Ok(sessions)
    }

    fn parse_log_files(&self) -> RimuruResult<Vec<SessionHistory>> {
        let mut sessions = Vec::new();

        if !self.logs_dir.exists() {
            debug!("Cursor logs directory does not exist: {:?}", self.logs_dir);
            return Ok(sessions);
        }

        let main_log = self.logs_dir.join("main.log");
        if main_log.exists() {
            if let Ok(log_sessions) = self.parse_log_file(&main_log) {
                sessions.extend(log_sessions);
            }
        }

        let renderer_log = self.logs_dir.join("renderer.log");
        if renderer_log.exists() {
            if let Ok(log_sessions) = self.parse_log_file(&renderer_log) {
                for session in log_sessions {
                    if !sessions.iter().any(|s| s.session_id == session.session_id) {
                        sessions.push(session);
                    }
                }
            }
        }

        Ok(sessions)
    }

    fn parse_chat_file(&self, path: &Path) -> RimuruResult<SessionHistory> {
        let content = fs::read_to_string(path)?;

        if let Ok(chat_entry) = serde_json::from_str::<CursorChatEntry>(&content) {
            if !chat_entry.messages.is_empty() {
                return self.convert_chat_entry_to_history(&chat_entry);
            }
        }

        if let Ok(session_data) = serde_json::from_str::<CursorSessionData>(&content) {
            return self.convert_session_data_to_history(&session_data);
        }

        Err(crate::error::RimuruError::SerializationError(
            "Failed to parse Cursor chat file".to_string(),
        ))
    }

    fn parse_composer_file(&self, path: &Path) -> RimuruResult<SessionHistory> {
        let content = fs::read_to_string(path)?;

        if let Ok(composer_entry) = serde_json::from_str::<CursorComposerEntry>(&content) {
            if !composer_entry.messages.is_empty() || !composer_entry.files.is_empty() {
                return self.convert_composer_entry_to_history(&composer_entry);
            }
        }

        if let Ok(session_data) = serde_json::from_str::<CursorSessionData>(&content) {
            return self.convert_session_data_to_history(&session_data);
        }

        Err(crate::error::RimuruError::SerializationError(
            "Failed to parse Cursor composer file".to_string(),
        ))
    }

    fn parse_log_file(&self, path: &Path) -> RimuruResult<Vec<SessionHistory>> {
        let _content = fs::read_to_string(path)?;

        debug!("Log file parsing for {:?} not fully implemented yet", path);
        Ok(vec![])
    }

    fn convert_session_data_to_history(
        &self,
        data: &CursorSessionData,
    ) -> RimuruResult<SessionHistory> {
        let session_id = data
            .id
            .as_ref()
            .and_then(|id| Uuid::parse_str(id).ok())
            .unwrap_or_else(Uuid::new_v4);

        let started_at = data.started_at.unwrap_or_else(Utc::now);

        Ok(SessionHistory {
            session_id,
            agent_type: AgentType::Cursor,
            started_at,
            ended_at: data.ended_at.or(data.updated_at),
            total_input_tokens: data.input_tokens,
            total_output_tokens: data.output_tokens,
            model_name: data.model.clone(),
            cost_usd: data.total_cost_usd,
            project_path: data.workspace_path.clone(),
        })
    }

    fn convert_chat_entry_to_history(
        &self,
        entry: &CursorChatEntry,
    ) -> RimuruResult<SessionHistory> {
        let session_id = entry
            .id
            .as_ref()
            .and_then(|id| Uuid::parse_str(id).ok())
            .unwrap_or_else(Uuid::new_v4);

        let started_at = entry.created_at.unwrap_or_else(Utc::now);

        let (input_tokens, output_tokens) = self.count_message_tokens(&entry.messages);

        Ok(SessionHistory {
            session_id,
            agent_type: AgentType::Cursor,
            started_at,
            ended_at: entry.updated_at,
            total_input_tokens: input_tokens,
            total_output_tokens: output_tokens,
            model_name: entry.model.clone(),
            cost_usd: None,
            project_path: None,
        })
    }

    fn convert_composer_entry_to_history(
        &self,
        entry: &CursorComposerEntry,
    ) -> RimuruResult<SessionHistory> {
        let session_id = entry
            .id
            .as_ref()
            .and_then(|id| Uuid::parse_str(id).ok())
            .unwrap_or_else(Uuid::new_v4);

        let started_at = entry.created_at.unwrap_or_else(Utc::now);

        let (input_tokens, output_tokens) = self.count_message_tokens(&entry.messages);

        Ok(SessionHistory {
            session_id,
            agent_type: AgentType::Cursor,
            started_at,
            ended_at: entry.updated_at,
            total_input_tokens: input_tokens,
            total_output_tokens: output_tokens,
            model_name: entry.model.clone(),
            cost_usd: None,
            project_path: None,
        })
    }

    fn count_message_tokens(&self, messages: &[CursorMessage]) -> (i64, i64) {
        let mut input_tokens = 0i64;
        let mut output_tokens = 0i64;

        for msg in messages {
            let tokens = msg.tokens.unwrap_or_else(|| {
                msg.content
                    .as_ref()
                    .map(|c| (c.len() as i64) / 4)
                    .unwrap_or(0)
            });

            match msg.role.as_str() {
                "user" | "system" => input_tokens += tokens,
                "assistant" => output_tokens += tokens,
                _ => input_tokens += tokens,
            }
        }

        (input_tokens, output_tokens)
    }

    pub fn get_active_session(&self) -> RimuruResult<Option<ActiveSession>> {
        let chat_dir = self
            .app_data_dir
            .join("User")
            .join("globalStorage")
            .join("cursor.chat");

        if !chat_dir.exists() {
            return Ok(None);
        }

        let mut most_recent: Option<(DateTime<Utc>, ActiveSession)> = None;

        if let Ok(entries) = fs::read_dir(&chat_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }

                let ext = path.extension().and_then(|e| e.to_str());
                if ext != Some("json") {
                    continue;
                }

                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(data) = serde_json::from_str::<CursorSessionData>(&content) {
                        if data.ended_at.is_some() {
                            continue;
                        }

                        let updated_at = data.updated_at.or(data.started_at);
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
                                    agent_type: AgentType::Cursor,
                                    started_at: data.started_at.unwrap_or(updated_at),
                                    current_tokens: data.input_tokens + data.output_tokens,
                                    model_name: data.model.clone(),
                                    project_path: data.workspace_path.clone(),
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
        let parser = SessionParser::new(
            temp_dir.path().to_path_buf(),
            temp_dir.path().join("config"),
            temp_dir.path().join("logs"),
        );

        let sessions = parser.parse_sessions().unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_session_parser_with_chat_file() {
        let temp_dir = tempdir().unwrap();
        let chat_dir = temp_dir
            .path()
            .join("User")
            .join("globalStorage")
            .join("cursor.chat");
        fs::create_dir_all(&chat_dir).unwrap();

        let session_data = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "model": "gpt-4o",
            "workspacePath": "/home/user/project",
            "inputTokens": 5000,
            "outputTokens": 2000,
            "totalCostUsd": 0.035
        }"#;

        fs::write(chat_dir.join("session1.json"), session_data).unwrap();

        let parser = SessionParser::new(
            temp_dir.path().to_path_buf(),
            temp_dir.path().join("config"),
            temp_dir.path().join("logs"),
        );
        let sessions = parser.parse_sessions().unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].total_input_tokens, 5000);
        assert_eq!(sessions[0].total_output_tokens, 2000);
        assert_eq!(sessions[0].model_name, Some("gpt-4o".to_string()));
    }

    #[test]
    fn test_session_parser_with_chat_entry() {
        let temp_dir = tempdir().unwrap();
        let chat_dir = temp_dir
            .path()
            .join("User")
            .join("globalStorage")
            .join("cursor.chat");
        fs::create_dir_all(&chat_dir).unwrap();

        let chat_data = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440001",
            "model": "claude-3-sonnet",
            "messages": [
                {"role": "user", "content": "Hello", "tokens": 100},
                {"role": "assistant", "content": "Hi there!", "tokens": 200}
            ]
        }"#;

        fs::write(chat_dir.join("chat.json"), chat_data).unwrap();

        let parser = SessionParser::new(
            temp_dir.path().to_path_buf(),
            temp_dir.path().join("config"),
            temp_dir.path().join("logs"),
        );
        let sessions = parser.parse_sessions().unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].total_input_tokens, 100);
        assert_eq!(sessions[0].total_output_tokens, 200);
    }

    #[test]
    fn test_get_active_session_none() {
        let temp_dir = tempdir().unwrap();
        let parser = SessionParser::new(
            temp_dir.path().to_path_buf(),
            temp_dir.path().join("config"),
            temp_dir.path().join("logs"),
        );

        let active = parser.get_active_session().unwrap();
        assert!(active.is_none());
    }

    #[test]
    fn test_get_active_session_with_ended_session() {
        let temp_dir = tempdir().unwrap();
        let chat_dir = temp_dir
            .path()
            .join("User")
            .join("globalStorage")
            .join("cursor.chat");
        fs::create_dir_all(&chat_dir).unwrap();

        let ended_session = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440003",
            "endedAt": "2025-01-01T00:00:00Z",
            "inputTokens": 1000,
            "outputTokens": 500
        }"#;

        fs::write(chat_dir.join("ended.json"), ended_session).unwrap();

        let parser = SessionParser::new(
            temp_dir.path().to_path_buf(),
            temp_dir.path().join("config"),
            temp_dir.path().join("logs"),
        );
        let active = parser.get_active_session().unwrap();
        assert!(active.is_none());
    }

    #[test]
    fn test_session_type_default() {
        let session_type = CursorSessionType::default();
        assert_eq!(session_type, CursorSessionType::Chat);
    }

    #[test]
    fn test_count_message_tokens() {
        let temp_dir = tempdir().unwrap();
        let parser = SessionParser::new(
            temp_dir.path().to_path_buf(),
            temp_dir.path().join("config"),
            temp_dir.path().join("logs"),
        );

        let messages = vec![
            CursorMessage {
                role: "user".to_string(),
                content: Some("Hello".to_string()),
                timestamp: None,
                tokens: Some(10),
                model: None,
            },
            CursorMessage {
                role: "assistant".to_string(),
                content: Some("Hi there!".to_string()),
                timestamp: None,
                tokens: Some(20),
                model: None,
            },
            CursorMessage {
                role: "system".to_string(),
                content: Some("You are helpful".to_string()),
                timestamp: None,
                tokens: Some(5),
                model: None,
            },
        ];

        let (input, output) = parser.count_message_tokens(&messages);
        assert_eq!(input, 15);
        assert_eq!(output, 20);
    }
}
