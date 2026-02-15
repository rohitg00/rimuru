use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Playbook {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<PlaybookStep>,
    pub file_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaybookStep {
    pub name: String,
    pub prompt: String,
    pub agent_type: String,
    pub working_dir: Option<String>,
    pub gate: String,
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Serialize, Clone)]
pub struct PlaybookExecution {
    pub id: String,
    pub playbook_id: String,
    pub current_step: usize,
    pub status: String,
    pub step_results: Vec<StepResult>,
    pub started_at: String,
    pub variables: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct StepResult {
    pub step_index: usize,
    pub status: String,
    pub session_id: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
}
