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
pub struct OpenCodeSessionData {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub project_path: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub ended_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub input_tokens: i64,
    #[serde(default)]
    pub output_tokens: i64,
    #[serde(default)]
    pub cost_usd: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCodeState {
    #[serde(default)]
    pub current_session_id: Option<String>,
    #[serde(default)]
    pub project_path: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub last_active: Option<DateTime<Utc>>,
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

        let sessions_dir = self.data_dir.join("sessions");
        if !sessions_dir.exists() {
            debug!("Sessions directory does not exist: {:?}", sessions_dir);
            return Ok(sessions);
        }

        let entries = match fs::read_dir(&sessions_dir) {
            Ok(entries) => entries,
            Err(e) => {
                warn!("Failed to read sessions directory: {}", e);
                return Ok(sessions);
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                if let Ok(session) = self.parse_session_file(&path) {
                    sessions.push(session);
                }
            } else if path.is_dir() {
                if let Ok(project_sessions) = self.parse_project_sessions(&path) {
                    sessions.extend(project_sessions);
                }
            }
        }

        sessions.sort_by(|a, b| b.started_at.cmp(&a.started_at));

        Ok(sessions)
    }

    fn parse_session_file(&self, file_path: &Path) -> RimuruResult<SessionHistory> {
        let content = fs::read_to_string(file_path)?;
        let data: OpenCodeSessionData = serde_json::from_str(&content)?;
        self.convert_to_session_history(&data).ok_or_else(|| {
            crate::error::RimuruError::Internal("Failed to convert session".to_string())
        })
    }

    fn parse_project_sessions(&self, project_dir: &Path) -> RimuruResult<Vec<SessionHistory>> {
        let mut sessions = Vec::new();

        let sessions_file = project_dir.join("sessions.json");
        if sessions_file.exists() {
            if let Ok(content) = fs::read_to_string(&sessions_file) {
                if let Ok(data) = serde_json::from_str::<Vec<OpenCodeSessionData>>(&content) {
                    for session_data in data {
                        if let Some(session) = self.convert_to_session_history(&session_data) {
                            sessions.push(session);
                        }
                    }
                }
            }
        }

        let state_file = project_dir.join("state.json");
        if state_file.exists() && sessions.is_empty() {
            if let Ok(content) = fs::read_to_string(&state_file) {
                if let Ok(state) = serde_json::from_str::<OpenCodeState>(&content) {
                    if let Some(session) = self.convert_state_to_history(&state, project_dir) {
                        sessions.push(session);
                    }
                }
            }
        }

        Ok(sessions)
    }

    fn convert_to_session_history(&self, data: &OpenCodeSessionData) -> Option<SessionHistory> {
        let session_id = data
            .id
            .as_ref()
            .and_then(|id| Uuid::parse_str(id).ok())
            .unwrap_or_else(Uuid::new_v4);

        let started_at = data.started_at.unwrap_or_else(Utc::now);

        let model_name = match (&data.provider, &data.model) {
            (Some(provider), Some(model)) => Some(format!("{}/{}", provider, model)),
            (None, Some(model)) => Some(model.clone()),
            (Some(provider), None) => Some(provider.clone()),
            _ => None,
        };

        Some(SessionHistory {
            session_id,
            agent_type: AgentType::OpenCode,
            started_at,
            ended_at: data.ended_at,
            total_input_tokens: data.input_tokens,
            total_output_tokens: data.output_tokens,
            model_name,
            cost_usd: data.cost_usd,
            project_path: data.project_path.clone(),
        })
    }

    fn convert_state_to_history(
        &self,
        state: &OpenCodeState,
        project_dir: &Path,
    ) -> Option<SessionHistory> {
        let session_id = state
            .current_session_id
            .as_ref()
            .and_then(|id| Uuid::parse_str(id).ok())
            .unwrap_or_else(Uuid::new_v4);

        let started_at = state.last_active.unwrap_or_else(Utc::now);

        let project_path = state
            .project_path
            .clone()
            .unwrap_or_else(|| project_dir.to_string_lossy().to_string());

        let model_name = match (&state.provider, &state.model) {
            (Some(provider), Some(model)) => Some(format!("{}/{}", provider, model)),
            (None, Some(model)) => Some(model.clone()),
            (Some(provider), None) => Some(provider.clone()),
            _ => None,
        };

        Some(SessionHistory {
            session_id,
            agent_type: AgentType::OpenCode,
            started_at,
            ended_at: None,
            total_input_tokens: state.total_input_tokens,
            total_output_tokens: state.total_output_tokens,
            model_name,
            cost_usd: Some(state.total_cost_usd),
            project_path: Some(project_path),
        })
    }

    pub fn get_active_session(&self) -> RimuruResult<Option<ActiveSession>> {
        let state_file = self.data_dir.join("state.json");
        if !state_file.exists() {
            return Ok(None);
        }

        let content = match fs::read_to_string(&state_file) {
            Ok(c) => c,
            Err(_) => return Ok(None),
        };

        let state: OpenCodeState = match serde_json::from_str(&content) {
            Ok(s) => s,
            Err(_) => return Ok(None),
        };

        if let Some(last_active) = state.last_active {
            let now = Utc::now();
            let diff = now.signed_duration_since(last_active);
            if diff.num_minutes() < 30 {
                let session_id = state
                    .current_session_id
                    .as_ref()
                    .and_then(|id| Uuid::parse_str(id).ok())
                    .unwrap_or_else(Uuid::new_v4);

                let model_name = match (&state.provider, &state.model) {
                    (Some(provider), Some(model)) => Some(format!("{}/{}", provider, model)),
                    (None, Some(model)) => Some(model.clone()),
                    (Some(provider), None) => Some(provider.clone()),
                    _ => None,
                };

                return Ok(Some(ActiveSession {
                    session_id,
                    agent_type: AgentType::OpenCode,
                    started_at: last_active,
                    current_tokens: state.total_input_tokens + state.total_output_tokens,
                    model_name,
                    project_path: state.project_path,
                }));
            }
        }

        let sessions_dir = self.data_dir.join("sessions");
        if !sessions_dir.exists() {
            return Ok(None);
        }

        let entries = match fs::read_dir(&sessions_dir) {
            Ok(entries) => entries,
            Err(_) => return Ok(None),
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let state_file = path.join("state.json");
                if state_file.exists() {
                    if let Ok(content) = fs::read_to_string(&state_file) {
                        if let Ok(state) = serde_json::from_str::<OpenCodeState>(&content) {
                            if let Some(last_active) = state.last_active {
                                let now = Utc::now();
                                let diff = now.signed_duration_since(last_active);
                                if diff.num_minutes() < 30 {
                                    let session_id = state
                                        .current_session_id
                                        .as_ref()
                                        .and_then(|id| Uuid::parse_str(id).ok())
                                        .unwrap_or_else(Uuid::new_v4);

                                    let model_name = match (&state.provider, &state.model) {
                                        (Some(provider), Some(model)) => {
                                            Some(format!("{}/{}", provider, model))
                                        }
                                        (None, Some(model)) => Some(model.clone()),
                                        (Some(provider), None) => Some(provider.clone()),
                                        _ => None,
                                    };

                                    return Ok(Some(ActiveSession {
                                        session_id,
                                        agent_type: AgentType::OpenCode,
                                        started_at: last_active,
                                        current_tokens: state.total_input_tokens
                                            + state.total_output_tokens,
                                        model_name,
                                        project_path: state.project_path,
                                    }));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
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
    fn test_session_parser_with_sessions_file() {
        let temp_dir = tempdir().unwrap();
        let sessions_dir = temp_dir.path().join("sessions").join("test-project");
        fs::create_dir_all(&sessions_dir).unwrap();

        let sessions_data = r#"[
            {
                "id": "550e8400-e29b-41d4-a716-446655440000",
                "project_path": "/home/user/project",
                "provider": "openai",
                "model": "gpt-4o",
                "input_tokens": 5000,
                "output_tokens": 2000
            }
        ]"#;

        fs::write(sessions_dir.join("sessions.json"), sessions_data).unwrap();

        let parser = SessionParser::new(temp_dir.path().to_path_buf());
        let sessions = parser.parse_sessions().unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].total_input_tokens, 5000);
        assert_eq!(sessions[0].total_output_tokens, 2000);
        assert_eq!(sessions[0].model_name, Some("openai/gpt-4o".to_string()));
    }

    #[test]
    fn test_session_parser_with_state_file() {
        let temp_dir = tempdir().unwrap();
        let sessions_dir = temp_dir.path().join("sessions").join("test-project");
        fs::create_dir_all(&sessions_dir).unwrap();

        let state_data = r#"{
            "current_session_id": "550e8400-e29b-41d4-a716-446655440001",
            "project_path": "/home/user/project2",
            "provider": "anthropic",
            "model": "claude-3-opus",
            "total_input_tokens": 10000,
            "total_output_tokens": 5000,
            "total_cost_usd": 0.25
        }"#;

        fs::write(sessions_dir.join("state.json"), state_data).unwrap();

        let parser = SessionParser::new(temp_dir.path().to_path_buf());
        let sessions = parser.parse_sessions().unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].total_input_tokens, 10000);
        assert_eq!(sessions[0].total_output_tokens, 5000);
        assert_eq!(sessions[0].cost_usd, Some(0.25));
        assert_eq!(
            sessions[0].model_name,
            Some("anthropic/claude-3-opus".to_string())
        );
    }

    #[test]
    fn test_get_active_session_none() {
        let temp_dir = tempdir().unwrap();
        let parser = SessionParser::new(temp_dir.path().to_path_buf());

        let active = parser.get_active_session().unwrap();
        assert!(active.is_none());
    }

    #[test]
    fn test_single_session_file() {
        let temp_dir = tempdir().unwrap();
        let sessions_dir = temp_dir.path().join("sessions");
        fs::create_dir_all(&sessions_dir).unwrap();

        let session_data = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440002",
            "project_path": "/home/user/project3",
            "provider": "google",
            "model": "gemini-2.0-flash",
            "input_tokens": 3000,
            "output_tokens": 1500,
            "cost_usd": 0.05
        }"#;

        fs::write(sessions_dir.join("session_001.json"), session_data).unwrap();

        let parser = SessionParser::new(temp_dir.path().to_path_buf());
        let sessions = parser.parse_sessions().unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].total_input_tokens, 3000);
        assert_eq!(
            sessions[0].model_name,
            Some("google/gemini-2.0-flash".to_string())
        );
    }
}
