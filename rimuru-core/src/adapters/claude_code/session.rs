use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use tracing::{debug, warn};
use uuid::Uuid;

use crate::adapters::types::{ActiveSession, SessionHistory};
use crate::error::RimuruResult;
use crate::models::AgentType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeCodeSessionData {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub project_path: Option<String>,
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
pub struct ClaudeCodeProjectState {
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub project_path: String,
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
    projects_dir: PathBuf,
}

impl SessionParser {
    pub fn new(projects_dir: PathBuf) -> Self {
        Self { projects_dir }
    }

    pub fn parse_sessions(&self) -> RimuruResult<Vec<SessionHistory>> {
        let mut sessions = Vec::new();

        if !self.projects_dir.exists() {
            debug!("Projects directory does not exist: {:?}", self.projects_dir);
            return Ok(sessions);
        }

        let entries = match fs::read_dir(&self.projects_dir) {
            Ok(entries) => entries,
            Err(e) => {
                warn!("Failed to read projects directory: {}", e);
                return Ok(sessions);
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(project_sessions) = self.parse_project_sessions(&path) {
                    sessions.extend(project_sessions);
                }
            }
        }

        sessions.sort_by(|a, b| b.started_at.cmp(&a.started_at));

        Ok(sessions)
    }

    fn parse_project_sessions(&self, project_dir: &Path) -> RimuruResult<Vec<SessionHistory>> {
        let mut sessions = Vec::new();

        let sessions_file = project_dir.join("sessions.json");
        if sessions_file.exists() {
            if let Ok(content) = fs::read_to_string(&sessions_file) {
                if let Ok(data) = serde_json::from_str::<Vec<ClaudeCodeSessionData>>(&content) {
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
                if let Ok(state) = serde_json::from_str::<ClaudeCodeProjectState>(&content) {
                    if let Some(session) =
                        self.convert_project_state_to_history(&state, project_dir)
                    {
                        sessions.push(session);
                    }
                }
            }
        }

        if sessions.is_empty() {
            if let Ok(entries) = fs::read_dir(project_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                        if let Some(session) = self.parse_jsonl_session(&path) {
                            sessions.push(session);
                        }
                    }
                }
            }
        }

        Ok(sessions)
    }

    fn convert_to_session_history(&self, data: &ClaudeCodeSessionData) -> Option<SessionHistory> {
        let session_id = data
            .id
            .as_ref()
            .and_then(|id| Uuid::parse_str(id).ok())
            .unwrap_or_else(Uuid::new_v4);

        let started_at = data.started_at.unwrap_or_else(Utc::now);

        Some(SessionHistory {
            session_id,
            agent_type: AgentType::ClaudeCode,
            started_at,
            ended_at: data.ended_at,
            total_input_tokens: data.input_tokens,
            total_output_tokens: data.output_tokens,
            model_name: data.model.clone(),
            cost_usd: data.cost_usd,
            project_path: data.project_path.clone(),
        })
    }

    fn convert_project_state_to_history(
        &self,
        state: &ClaudeCodeProjectState,
        project_dir: &Path,
    ) -> Option<SessionHistory> {
        let session_id = state
            .session_id
            .as_ref()
            .and_then(|id| Uuid::parse_str(id).ok())
            .unwrap_or_else(Uuid::new_v4);

        let started_at = state.last_active.unwrap_or_else(Utc::now);

        let project_path = if state.project_path.is_empty() {
            project_dir.to_string_lossy().to_string()
        } else {
            state.project_path.clone()
        };

        Some(SessionHistory {
            session_id,
            agent_type: AgentType::ClaudeCode,
            started_at,
            ended_at: None,
            total_input_tokens: state.total_input_tokens,
            total_output_tokens: state.total_output_tokens,
            model_name: None,
            cost_usd: Some(state.total_cost_usd),
            project_path: Some(project_path),
        })
    }

    fn parse_jsonl_session(&self, jsonl_path: &Path) -> Option<SessionHistory> {
        let session_id_str = jsonl_path.file_stem()?.to_str()?;
        let session_id = Uuid::parse_str(session_id_str).ok()?;

        let file = fs::File::open(jsonl_path).ok()?;
        let reader = BufReader::new(file);

        let mut first_timestamp: Option<DateTime<Utc>> = None;
        let mut last_timestamp: Option<DateTime<Utc>> = None;
        let mut total_input_tokens: i64 = 0;
        let mut total_output_tokens: i64 = 0;
        let mut model_name: Option<String> = None;
        let mut project_path: Option<String> = None;

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => continue,
            };
            let entry: serde_json::Value = match serde_json::from_str(&line) {
                Ok(v) => v,
                Err(_) => continue,
            };

            if let Some(snapshot) = entry.get("snapshot") {
                if let Some(ts_str) = snapshot.get("timestamp").and_then(|v| v.as_str()) {
                    if let Ok(ts) = ts_str.parse::<DateTime<Utc>>() {
                        if first_timestamp.is_none() {
                            first_timestamp = Some(ts);
                        }
                        last_timestamp = Some(ts);
                    }
                }
            }

            if let Some(cwd) = entry.get("cwd").and_then(|v| v.as_str()) {
                if project_path.is_none() {
                    project_path = Some(cwd.to_string());
                }
            }

            if entry.get("type").and_then(|v| v.as_str()) == Some("assistant") {
                if let Some(msg) = entry.get("message") {
                    if let Some(m) = msg.get("model").and_then(|v| v.as_str()) {
                        model_name = Some(m.to_string());
                    }
                    if let Some(usage) = msg.get("usage") {
                        total_input_tokens += usage
                            .get("input_tokens")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0);
                        total_input_tokens += usage
                            .get("cache_read_input_tokens")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0);
                        total_input_tokens += usage
                            .get("cache_creation_input_tokens")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0);
                        total_output_tokens += usage
                            .get("output_tokens")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0);
                    }
                }
            }
        }

        let started_at = first_timestamp.unwrap_or_else(Utc::now);
        let ended_at = last_timestamp;
        let has_ended = ended_at.map(|e| e != started_at).unwrap_or(false);

        Some(SessionHistory {
            session_id,
            agent_type: AgentType::ClaudeCode,
            started_at,
            ended_at: if has_ended { ended_at } else { None },
            total_input_tokens,
            total_output_tokens,
            model_name,
            cost_usd: None,
            project_path,
        })
    }

    pub fn get_active_session(&self) -> RimuruResult<Option<ActiveSession>> {
        if !self.projects_dir.exists() {
            return Ok(None);
        }

        let entries = match fs::read_dir(&self.projects_dir) {
            Ok(entries) => entries,
            Err(_) => return Ok(None),
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let state_file = path.join("state.json");
                if state_file.exists() {
                    if let Ok(content) = fs::read_to_string(&state_file) {
                        if let Ok(state) = serde_json::from_str::<ClaudeCodeProjectState>(&content)
                        {
                            if let Some(last_active) = state.last_active {
                                let now = Utc::now();
                                let diff = now.signed_duration_since(last_active);
                                if diff.num_minutes() < 30 {
                                    let session_id = state
                                        .session_id
                                        .as_ref()
                                        .and_then(|id| Uuid::parse_str(id).ok())
                                        .unwrap_or_else(Uuid::new_v4);

                                    return Ok(Some(ActiveSession {
                                        session_id,
                                        agent_type: AgentType::ClaudeCode,
                                        started_at: last_active,
                                        current_tokens: state.total_input_tokens
                                            + state.total_output_tokens,
                                        model_name: None,
                                        project_path: Some(state.project_path),
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
        let project_dir = temp_dir.path().join("test-project");
        fs::create_dir_all(&project_dir).unwrap();

        let sessions_data = r#"[
            {
                "id": "550e8400-e29b-41d4-a716-446655440000",
                "project_path": "/home/user/project",
                "model": "claude-3-opus",
                "input_tokens": 5000,
                "output_tokens": 2000
            }
        ]"#;

        fs::write(project_dir.join("sessions.json"), sessions_data).unwrap();

        let parser = SessionParser::new(temp_dir.path().to_path_buf());
        let sessions = parser.parse_sessions().unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].total_input_tokens, 5000);
        assert_eq!(sessions[0].total_output_tokens, 2000);
        assert_eq!(sessions[0].model_name, Some("claude-3-opus".to_string()));
    }

    #[test]
    fn test_session_parser_with_state_file() {
        let temp_dir = tempdir().unwrap();
        let project_dir = temp_dir.path().join("test-project");
        fs::create_dir_all(&project_dir).unwrap();

        let state_data = r#"{
            "session_id": "550e8400-e29b-41d4-a716-446655440001",
            "project_path": "/home/user/project2",
            "total_input_tokens": 10000,
            "total_output_tokens": 5000,
            "total_cost_usd": 0.25
        }"#;

        fs::write(project_dir.join("state.json"), state_data).unwrap();

        let parser = SessionParser::new(temp_dir.path().to_path_buf());
        let sessions = parser.parse_sessions().unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].total_input_tokens, 10000);
        assert_eq!(sessions[0].total_output_tokens, 5000);
        assert_eq!(sessions[0].cost_usd, Some(0.25));
    }

    #[test]
    fn test_get_active_session_none() {
        let temp_dir = tempdir().unwrap();
        let parser = SessionParser::new(temp_dir.path().to_path_buf());

        let active = parser.get_active_session().unwrap();
        assert!(active.is_none());
    }
}
