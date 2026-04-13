use chrono::Utc;
use iii_sdk::{III, RegisterFunctionMessage};
use serde_json::{Value, json};
use tracing::{info, warn};

use super::sysutil::{api_response, extract_input, kv_err, require_str};
use crate::adapters::{
    AgentAdapter, ClaudeCodeAdapter, CodexAdapter, CopilotAdapter, CursorAdapter, GeminiCliAdapter,
    GooseAdapter, OpenCodeAdapter,
};
use crate::models::{Agent, AgentConfig, AgentStatus, AgentType, CostRecord, SessionStatus};
use crate::state::StateKV;

pub fn register(iii: &III, kv: &StateKV) {
    register_list(iii, kv);
    register_get(iii, kv);
    register_create(iii, kv);
    register_update(iii, kv);
    register_delete(iii, kv);
    register_status(iii, kv);
    register_detect(iii, kv);
    register_connect(iii, kv);
    register_disconnect(iii, kv);
    register_sync(iii, kv);
}

fn register_list(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.agents.list".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                let input = extract_input(input);
                let agents: Vec<Agent> = kv.list("agents").await.map_err(kv_err)?;

                let agent_type_filter = input
                    .get("agent_type")
                    .and_then(|v| v.as_str())
                    .and_then(|s| serde_json::from_value::<AgentType>(json!(s)).ok());

                let status_filter = input
                    .get("status")
                    .and_then(|v| v.as_str())
                    .and_then(|s| serde_json::from_value::<AgentStatus>(json!(s)).ok());

                let filtered: Vec<&Agent> = agents
                    .iter()
                    .filter(|a| {
                        agent_type_filter
                            .as_ref()
                            .is_none_or(|t| a.agent_type == *t)
                    })
                    .filter(|a| status_filter.as_ref().is_none_or(|s| a.status == *s))
                    .collect();

                Ok(api_response(json!({
                    "agents": filtered,
                    "total": filtered.len()
                })))
            }
        },
    );
}

fn register_get(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.agents.get".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                let input = extract_input(input);
                let agent_id = require_str(&input, "agent_id")?;

                let agent: Option<Agent> = kv.get("agents", &agent_id).await.map_err(kv_err)?;

                match agent {
                    Some(a) => {
                        let config: Option<AgentConfig> =
                            kv.get("agent_config", &agent_id).await.map_err(kv_err)?;

                        Ok(api_response(json!({
                            "agent": a,
                            "config": config
                        })))
                    }
                    None => Err(iii_sdk::IIIError::Handler(format!(
                        "agent not found: {}",
                        agent_id
                    ))),
                }
            }
        },
    );
}

fn register_create(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.agents.create".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                let input = extract_input(input);
                let agent_type: AgentType =
                    serde_json::from_value(input.get("agent_type").cloned().ok_or_else(|| {
                        iii_sdk::IIIError::Handler("agent_type is required".into())
                    })?)
                    .map_err(|e| {
                        iii_sdk::IIIError::Handler(format!("invalid agent_type: {}", e))
                    })?;

                let name = input
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or_else(|| agent_type.display_name())
                    .to_string();

                let mut agent = Agent::new(agent_type, name);

                if let Some(version) = input.get("version").and_then(|v| v.as_str()) {
                    agent.version = Some(version.to_string());
                }
                if let Some(config_path) = input.get("config_path").and_then(|v| v.as_str()) {
                    agent.config_path = Some(config_path.to_string());
                }
                if let Some(metadata) = input.get("metadata") {
                    agent.metadata = metadata.clone();
                }

                let agent_id = agent.id.to_string();

                kv.set("agents", &agent_id, &agent).await.map_err(kv_err)?;

                let config = AgentConfig {
                    agent_id: agent.id,
                    ..AgentConfig::default()
                };
                kv.set("agent_config", &agent_id, &config)
                    .await
                    .map_err(kv_err)?;

                Ok(api_response(json!({
                    "agent": agent,
                    "config": config
                })))
            }
        },
    );
}

fn register_update(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.agents.update".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                let input = extract_input(input);
                let agent_id = require_str(&input, "agent_id")?;

                let mut agent: Agent = kv
                    .get("agents", &agent_id)
                    .await
                    .map_err(kv_err)?
                    .ok_or_else(|| {
                        iii_sdk::IIIError::Handler(format!("agent not found: {}", agent_id))
                    })?;

                if let Some(name) = input.get("name").and_then(|v| v.as_str()) {
                    agent.name = name.to_string();
                }
                if let Some(version) = input.get("version").and_then(|v| v.as_str()) {
                    agent.version = Some(version.to_string());
                }
                if let Some(config_path) = input.get("config_path").and_then(|v| v.as_str()) {
                    agent.config_path = Some(config_path.to_string());
                }
                if let Some(status) = input.get("status") {
                    agent.status = serde_json::from_value(status.clone()).map_err(|e| {
                        iii_sdk::IIIError::Handler(format!("invalid status: {}", e))
                    })?;
                }
                if let Some(metadata) = input.get("metadata") {
                    agent.metadata = metadata.clone();
                }

                agent.last_seen = Some(Utc::now());

                kv.set("agents", &agent_id, &agent).await.map_err(kv_err)?;

                Ok(api_response(json!({"agent": agent})))
            }
        },
    );
}

fn register_delete(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.agents.delete".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                let input = extract_input(input);
                let agent_id = require_str(&input, "agent_id")?;

                let _agent: Agent = kv
                    .get("agents", &agent_id)
                    .await
                    .map_err(kv_err)?
                    .ok_or_else(|| {
                        iii_sdk::IIIError::Handler(format!("agent not found: {}", agent_id))
                    })?;

                kv.delete("agents", &agent_id).await.map_err(kv_err)?;

                kv.delete("agent_config", &agent_id).await.map_err(kv_err)?;

                Ok(api_response(json!({"deleted": agent_id})))
            }
        },
    );
}

fn register_status(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.agents.status".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                let input = extract_input(input);
                let agent_id = require_str(&input, "agent_id")?;

                let new_status: AgentStatus = serde_json::from_value(
                    input
                        .get("status")
                        .cloned()
                        .ok_or_else(|| iii_sdk::IIIError::Handler("status is required".into()))?,
                )
                .map_err(|e| iii_sdk::IIIError::Handler(format!("invalid status: {}", e)))?;

                let mut agent: Agent = kv
                    .get("agents", &agent_id)
                    .await
                    .map_err(kv_err)?
                    .ok_or_else(|| {
                        iii_sdk::IIIError::Handler(format!("agent not found: {}", agent_id))
                    })?;

                let old_status = agent.status;
                agent.status = new_status;
                agent.last_seen = Some(Utc::now());

                if new_status == AgentStatus::Connected && old_status == AgentStatus::Disconnected {
                    agent.connected_at = Some(Utc::now());
                }

                kv.set("agents", &agent_id, &agent).await.map_err(kv_err)?;

                Ok(api_response(json!({
                    "agent_id": agent_id,
                    "old_status": old_status,
                    "new_status": new_status
                })))
            }
        },
    );
}

fn agent_checks() -> Vec<(AgentType, Vec<std::path::PathBuf>)> {
    let home = dirs::home_dir().unwrap_or_default();
    vec![
        (AgentType::ClaudeCode, vec![home.join(".claude")]),
        (
            AgentType::Cursor,
            vec![
                home.join(".cursor"),
                home.join("Library/Application Support/Cursor"),
            ],
        ),
        (
            AgentType::Copilot,
            vec![
                home.join(".config/github-copilot"),
                home.join(".vscode/extensions"),
            ],
        ),
        (
            AgentType::Codex,
            vec![home.join(".codex"), home.join(".config/codex")],
        ),
        (AgentType::Goose, vec![home.join(".config/goose")]),
        (AgentType::OpenCode, vec![home.join(".opencode")]),
        (
            AgentType::GeminiCli,
            vec![home.join(".gemini"), home.join(".config/gemini")],
        ),
    ]
}

fn register_detect(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(RegisterFunctionMessage::with_id("rimuru.agents.detect".to_string()), move |input: Value| {
        let kv = kv.clone();
        async move {
            let input = extract_input(input);
            let auto_register = input
                .get("auto_register")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let checks = agent_checks();
            let mut detected: Vec<Value> = Vec::new();

            let existing_agents: Vec<Agent> = kv.list("agents").await.unwrap_or_default();

            for (agent_type, paths) in &checks {
                let found = paths.iter().any(|p| p.exists());
                let already_registered = existing_agents
                    .iter()
                    .any(|a| a.agent_type == *agent_type);

                let mut registered = already_registered;

                if found && !already_registered && auto_register {
                    let config_path = paths.iter().find(|p| p.exists()).map(|p| p.display().to_string());
                    let mut agent = Agent::new(*agent_type, agent_type.display_name().to_string());
                    if let Some(cp) = &config_path {
                        agent.config_path = Some(cp.clone());
                    }
                    let agent_id = agent.id.to_string();
                    if kv.set("agents", &agent_id, &agent).await.is_ok() {
                        let config = AgentConfig {
                            agent_id: agent.id,
                            ..AgentConfig::default()
                        };
                        if let Err(e) = kv.set("agent_config", &agent_id, &config).await {
                            warn!("Failed to store agent config {}: {}", agent_id, e);
                        }
                        registered = true;
                    }
                }

                detected.push(json!({
                    "agent_type": agent_type,
                    "display_name": agent_type.display_name(),
                    "installed": found,
                    "registered": registered
                }));
            }

            Ok(api_response(json!({
                "detected": detected,
                "total": detected.iter().filter(|d| d["installed"].as_bool().unwrap_or(false)).count()
            })))
        }
    });
}

fn register_connect(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.agents.connect".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                let input = extract_input(input);
                let agent_type_str = require_str(&input, "agent_type")?;

                let agent_type: AgentType =
                    serde_json::from_value(json!(agent_type_str)).map_err(|e| {
                        iii_sdk::IIIError::Handler(format!("invalid agent_type: {}", e))
                    })?;

                let existing_agents: Vec<Agent> = kv.list("agents").await.unwrap_or_default();

                if let Some(existing) = existing_agents.iter().find(|a| a.agent_type == agent_type)
                {
                    let agent_id = existing.id.to_string();
                    let mut agent = existing.clone();
                    agent.status = AgentStatus::Connected;
                    agent.connected_at = Some(Utc::now());
                    agent.last_seen = Some(Utc::now());
                    kv.set("agents", &agent_id, &agent).await.map_err(kv_err)?;
                    return Ok(api_response(json!({"agent": agent, "action": "connected"})));
                }

                let checks = agent_checks();
                let config_path = checks
                    .iter()
                    .find(|(t, _)| *t == agent_type)
                    .and_then(|(_, paths)| paths.iter().find(|p| p.exists()))
                    .map(|p| p.display().to_string());

                let mut agent = Agent::new(agent_type, agent_type.display_name().to_string());
                agent.status = AgentStatus::Connected;
                agent.connected_at = Some(Utc::now());
                agent.last_seen = Some(Utc::now());
                if let Some(cp) = &config_path {
                    agent.config_path = Some(cp.clone());
                }

                let agent_id = agent.id.to_string();
                kv.set("agents", &agent_id, &agent).await.map_err(kv_err)?;
                let config = AgentConfig {
                    agent_id: agent.id,
                    ..AgentConfig::default()
                };
                kv.set("agent_config", &agent_id, &config)
                    .await
                    .map_err(kv_err)?;

                Ok(api_response(
                    json!({"agent": agent, "action": "created_and_connected"}),
                ))
            }
        },
    );
}

fn register_disconnect(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.agents.disconnect".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                let input = extract_input(input);
                let agent_id = require_str(&input, "agent_id")?;

                let mut agent: Agent = kv
                    .get("agents", &agent_id)
                    .await
                    .map_err(kv_err)?
                    .ok_or_else(|| {
                        iii_sdk::IIIError::Handler(format!("agent not found: {}", agent_id))
                    })?;

                agent.status = AgentStatus::Disconnected;
                agent.last_seen = Some(Utc::now());

                kv.set("agents", &agent_id, &agent).await.map_err(kv_err)?;

                Ok(api_response(
                    json!({"agent": agent, "action": "disconnected"}),
                ))
            }
        },
    );
}

fn get_adapter(agent_type: &AgentType) -> Option<Box<dyn AgentAdapter>> {
    match agent_type {
        AgentType::ClaudeCode => Some(Box::new(ClaudeCodeAdapter::new())),
        AgentType::Cursor => Some(Box::new(CursorAdapter::new())),
        AgentType::Copilot => Some(Box::new(CopilotAdapter::new())),
        AgentType::Codex => Some(Box::new(CodexAdapter::new())),
        AgentType::Goose => Some(Box::new(GooseAdapter::new())),
        AgentType::OpenCode => Some(Box::new(OpenCodeAdapter::new())),
        AgentType::GeminiCli => Some(Box::new(GeminiCliAdapter::new())),
    }
}

struct SyncSessionResult {
    sessions_stored: u64,
    costs_stored: u64,
    total_cost: f64,
    total_tokens: u64,
    session_count: u64,
    has_active: bool,
}

async fn sync_agent_sessions(
    kv: &StateKV,
    agent: &Agent,
    sessions: Vec<crate::models::Session>,
) -> SyncSessionResult {
    let mut result = SyncSessionResult {
        sessions_stored: 0,
        costs_stored: 0,
        total_cost: 0.0,
        total_tokens: 0,
        session_count: 0,
        has_active: false,
    };

    for mut session in sessions {
        session.agent_id = agent.id;
        let session_id = session.id.to_string();

        result.total_cost += session.total_cost;
        result.total_tokens += session.total_tokens;
        result.session_count += 1;
        if matches!(session.status, SessionStatus::Active) {
            result.has_active = true;
        }

        if session.total_cost > 0.0
            && let Some(ref model) = session.model
        {
            let model_lower = model.to_lowercase();
            let provider = if model_lower.contains("claude") {
                "anthropic"
            } else if model_lower.contains("gpt")
                || model_lower.contains("openai")
                || model_lower.contains("o3")
                || model_lower.contains("o1")
            {
                "openai"
            } else if model_lower.contains("gemini") {
                "google"
            } else if model_lower.contains("github") || model_lower.contains("copilot") {
                "github"
            } else if model_lower.contains("deepseek") {
                "deepseek"
            } else if model_lower.contains("kimi") || model_lower.contains("moonshot") {
                "moonshot"
            } else if model_lower.contains("glm") || model_lower.contains("zhipu") {
                "zhipu"
            } else if model_lower.contains("mistral") || model_lower.contains("codestral") {
                "mistral"
            } else if model_lower.contains("llama") {
                "meta"
            } else if model_lower.contains("qwen") {
                "alibaba"
            } else {
                "unknown"
            };
            let input_ratio = match provider {
                "anthropic" => 0.25,
                "openai" => 0.33,
                "google" => 0.30,
                "deepseek" => 0.15,
                "moonshot" => 0.20,
                "zhipu" => 0.20,
                "mistral" => 0.25,
                "meta" => 0.25,
                _ => 0.30,
            };
            let mut cost_record = CostRecord::new(
                agent.id,
                agent.agent_type,
                model.clone(),
                provider.to_string(),
                session.input_tokens,
                session.output_tokens,
                session.total_cost * input_ratio,
                session.total_cost * (1.0 - input_ratio),
            );
            cost_record.recorded_at = session.started_at;
            let record_id = cost_record.id.to_string();
            if let Err(e) = kv.set("cost_records", &record_id, &cost_record).await {
                warn!("Failed to store cost record {}: {}", record_id, e);
            }
            result.costs_stored += 1;
        }

        if let Err(e) = kv.set("sessions", &session_id, &session).await {
            warn!("Failed to store session {}: {}", session_id, e);
        }
        result.sessions_stored += 1;
    }

    result
}

async fn remove_agent(kv: &StateKV, agent_id: &str) {
    if let Err(e) = kv.delete("agents", agent_id).await {
        warn!("Failed to delete agent {}: {}", agent_id, e);
    }
    if let Err(e) = kv.delete("agent_config", agent_id).await {
        warn!("Failed to delete agent config {}: {}", agent_id, e);
    }
}

async fn update_agent_after_sync(
    kv: &StateKV,
    agent: &Agent,
    adapter: &dyn AgentAdapter,
    sync_result: &SyncSessionResult,
) {
    let agent_id = agent.id.to_string();
    let mut updated = agent.clone();
    updated.session_count = sync_result.session_count;
    updated.total_cost = sync_result.total_cost;
    updated.last_seen = Some(Utc::now());
    updated.version = adapter.detect_version();
    updated.status = if sync_result.has_active {
        AgentStatus::Connected
    } else {
        AgentStatus::Disconnected
    };
    if let Err(e) = kv.set("agents", &agent_id, &updated).await {
        warn!("Failed to update agent {}: {}", agent_id, e);
    }
}

fn register_sync(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.agents.sync".to_string()),
        move |_input: Value| {
            let kv = kv.clone();
            async move {
                let agents: Vec<Agent> = kv.list("agents").await.unwrap_or_default();
                let mut synced_sessions = 0u64;
                let mut synced_costs = 0u64;
                let mut synced_agents = 0u64;

                for agent in &agents {
                    let adapter = match get_adapter(&agent.agent_type) {
                        Some(a) => a,
                        None => continue,
                    };

                    let agent_id = agent.id.to_string();

                    if !adapter.is_installed() {
                        remove_agent(&kv, &agent_id).await;
                        continue;
                    }

                    let sessions = match adapter.get_sessions().await {
                        Ok(s) => s,
                        Err(e) => {
                            warn!("Failed to get sessions for {}: {}", agent.name, e);
                            continue;
                        }
                    };

                    let result = sync_agent_sessions(&kv, agent, sessions).await;

                    if result.session_count == 0
                        || (result.total_cost == 0.0 && result.total_tokens == 0)
                    {
                        remove_agent(&kv, &agent_id).await;
                        continue;
                    }

                    update_agent_after_sync(&kv, agent, adapter.as_ref(), &result).await;

                    synced_sessions += result.sessions_stored;
                    synced_costs += result.costs_stored;
                    synced_agents += 1;
                }

                info!(
                    "Synced {} sessions, {} costs from {} agents",
                    synced_sessions, synced_costs, synced_agents
                );

                Ok(api_response(json!({
                    "synced_agents": synced_agents,
                    "synced_sessions": synced_sessions,
                    "synced_costs": synced_costs
                })))
            }
        },
    );
}
