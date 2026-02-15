use crate::state::AppState;
use rimuru_core::{
    Agent, AgentRepository, AgentScanner, AgentType, CostRepository, Repository, SessionRepository,
};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize)]
pub struct AgentData {
    pub id: String,
    pub name: String,
    pub agent_type: String,
    pub config_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub last_active: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AgentWithStatusResponse {
    pub agent: AgentData,
    pub is_active: bool,
    pub session_count: i64,
    pub total_cost: f64,
}

impl From<Agent> for AgentWithStatusResponse {
    fn from(a: Agent) -> Self {
        let config_path = a
            .config
            .get("config_dir")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        AgentWithStatusResponse {
            agent: AgentData {
                id: a.id.to_string(),
                name: a.name,
                agent_type: a.agent_type.to_string(),
                config_path,
                created_at: a.created_at.to_rfc3339(),
                updated_at: a.updated_at.to_rfc3339(),
                last_active: None,
            },
            is_active: false,
            session_count: 0,
            total_cost: 0.0,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DiscoveredAgentResponse {
    pub name: String,
    pub agent_type: String,
    pub config_dir: String,
    pub executable_path: Option<String>,
    pub is_configured: bool,
}

#[derive(Debug, Deserialize)]
pub struct AddAgentRequest {
    pub name: String,
    pub agent_type: String,
    pub config: Option<serde_json::Value>,
}

async fn enrich_agent(
    agent: Agent,
    session_repo: &SessionRepository,
    cost_repo: &CostRepository,
) -> AgentWithStatusResponse {
    let sessions = session_repo
        .get_by_agent(agent.id)
        .await
        .unwrap_or_default();
    let session_count = sessions.len() as i64;
    let last_active = sessions.first().map(|s| s.started_at.to_rfc3339());
    let is_active = session_repo
        .get_active_by_agent(agent.id)
        .await
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    let total_cost = cost_repo
        .get_total_by_agent(agent.id)
        .await
        .map(|s| s.total_cost_usd)
        .unwrap_or(0.0);

    let mut resp = AgentWithStatusResponse::from(agent);
    resp.session_count = session_count;
    resp.is_active = is_active;
    resp.agent.last_active = last_active;
    resp.total_cost = total_cost;
    resp
}

#[tauri::command]
pub async fn get_agents(
    state: State<'_, AppState>,
) -> Result<Vec<AgentWithStatusResponse>, String> {
    let repo = AgentRepository::new(state.db.pool().clone());
    let session_repo = SessionRepository::new(state.db.pool().clone());
    let cost_repo = CostRepository::new(state.db.pool().clone());
    let agents = repo.get_all().await.map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for agent in agents {
        results.push(enrich_agent(agent, &session_repo, &cost_repo).await);
    }

    Ok(results)
}

#[tauri::command]
pub async fn get_agent_details(
    state: State<'_, AppState>,
    agent_id: String,
) -> Result<Option<AgentWithStatusResponse>, String> {
    let repo = AgentRepository::new(state.db.pool().clone());
    let session_repo = SessionRepository::new(state.db.pool().clone());
    let cost_repo = CostRepository::new(state.db.pool().clone());
    let uuid = uuid::Uuid::parse_str(&agent_id).map_err(|e| e.to_string())?;

    match repo.get_by_id(uuid).await.map_err(|e| e.to_string())? {
        Some(agent) => Ok(Some(enrich_agent(agent, &session_repo, &cost_repo).await)),
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn scan_agents(
    state: State<'_, AppState>,
) -> Result<Vec<DiscoveredAgentResponse>, String> {
    let scanner = AgentScanner::new();
    let scan_result = scanner.scan_all().await.map_err(|e| e.to_string())?;
    let repo = AgentRepository::new(state.db.pool().clone());

    let mut discovered_responses = Vec::new();

    for d in scan_result.discovered {
        let name = d.installation.name.clone();
        let agent_type_str = d.installation.agent_type.to_string();
        let config_dir = d.installation.config_dir.to_string_lossy().to_string();
        let executable_path = d
            .installation
            .executable_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string());
        let is_configured = d.installation.is_configured;

        if repo
            .get_by_name(&name)
            .await
            .map_err(|e| e.to_string())?
            .is_none()
        {
            let config = serde_json::json!({
                "config_dir": &config_dir,
                "is_configured": is_configured,
                "executable_path": &executable_path,
            });
            let agent = Agent::new(name.clone(), d.installation.agent_type, config);
            let _ = repo.create(&agent).await;
        }

        discovered_responses.push(DiscoveredAgentResponse {
            name,
            agent_type: agent_type_str,
            config_dir,
            executable_path,
            is_configured,
        });
    }

    Ok(discovered_responses)
}

#[tauri::command]
pub async fn add_agent(
    state: State<'_, AppState>,
    request: AddAgentRequest,
) -> Result<AgentWithStatusResponse, String> {
    let repo = AgentRepository::new(state.db.pool().clone());

    let agent_type = match request.agent_type.to_lowercase().as_str() {
        "claudecode" | "claude_code" | "claude-code" => AgentType::ClaudeCode,
        "opencode" | "open_code" | "open-code" => AgentType::OpenCode,
        "codex" => AgentType::Codex,
        "copilot" | "github_copilot" | "github-copilot" => AgentType::Copilot,
        "cursor" => AgentType::Cursor,
        "goose" => AgentType::Goose,
        _ => return Err(format!("Unknown agent type: {}", request.agent_type)),
    };

    let config = request.config.unwrap_or_else(|| serde_json::json!({}));
    let agent = Agent::new(request.name, agent_type, config);

    repo.create(&agent).await.map_err(|e| e.to_string())?;

    Ok(AgentWithStatusResponse::from(agent))
}
