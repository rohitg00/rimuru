use chrono::Utc;
use iii_sdk::III;
use serde_json::{json, Value};
use tracing::{info, warn};

use crate::adapters::{AgentAdapter, ClaudeCodeAdapter, CursorAdapter, CopilotAdapter, CodexAdapter, GooseAdapter, OpenCodeAdapter};
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
    iii.register_function("rimuru.agents.list", move |input: Value| {
        let kv = kv.clone();
        async move {
            let agents: Vec<Agent> = kv
                .list("agents")
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

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
                        .map_or(true, |t| a.agent_type == *t)
                })
                .filter(|a| status_filter.as_ref().map_or(true, |s| a.status == *s))
                .collect();

            Ok(json!({
                "agents": filtered,
                "total": filtered.len()
            }))
        }
    });
}

fn register_get(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function("rimuru.agents.get", move |input: Value| {
        let kv = kv.clone();
        async move {
            let agent_id = input
                .get("agent_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| iii_sdk::IIIError::Handler("agent_id is required".into()))?;

            let agent: Option<Agent> = kv
                .get("agents", agent_id)
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

            match agent {
                Some(a) => {
                    let config: Option<AgentConfig> = kv
                        .get("agent_config", agent_id)
                        .await
                        .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

                    Ok(json!({
                        "agent": a,
                        "config": config
                    }))
                }
                None => Err(iii_sdk::IIIError::Handler(format!(
                    "agent not found: {}",
                    agent_id
                ))),
            }
        }
    });
}

fn register_create(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function("rimuru.agents.create", move |input: Value| {
        let kv = kv.clone();
        async move {
            let agent_type: AgentType = serde_json::from_value(
                input
                    .get("agent_type")
                    .cloned()
                    .ok_or_else(|| iii_sdk::IIIError::Handler("agent_type is required".into()))?,
            )
            .map_err(|e| iii_sdk::IIIError::Handler(format!("invalid agent_type: {}", e)))?;

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

            kv.set("agents", &agent_id, &agent)
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

            let config = AgentConfig {
                agent_id: agent.id,
                ..AgentConfig::default()
            };
            kv.set("agent_config", &agent_id, &config)
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

            Ok(json!({
                "agent": agent,
                "config": config
            }))
        }
    });
}

fn register_update(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function("rimuru.agents.update", move |input: Value| {
        let kv = kv.clone();
        async move {
            let agent_id = input
                .get("agent_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| iii_sdk::IIIError::Handler("agent_id is required".into()))?;

            let mut agent: Agent = kv
                .get("agents", agent_id)
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?
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
                agent.status = serde_json::from_value(status.clone())
                    .map_err(|e| iii_sdk::IIIError::Handler(format!("invalid status: {}", e)))?;
            }
            if let Some(metadata) = input.get("metadata") {
                agent.metadata = metadata.clone();
            }

            agent.last_seen = Some(Utc::now());

            kv.set("agents", agent_id, &agent)
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

            Ok(json!({"agent": agent}))
        }
    });
}

fn register_delete(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function("rimuru.agents.delete", move |input: Value| {
        let kv = kv.clone();
        async move {
            let agent_id = input
                .get("agent_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| iii_sdk::IIIError::Handler("agent_id is required".into()))?;

            let _agent: Agent = kv
                .get("agents", agent_id)
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?
                .ok_or_else(|| {
                    iii_sdk::IIIError::Handler(format!("agent not found: {}", agent_id))
                })?;

            kv.delete("agents", agent_id)
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

            kv.delete("agent_config", agent_id)
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

            Ok(json!({"deleted": agent_id}))
        }
    });
}

fn register_status(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function("rimuru.agents.status", move |input: Value| {
        let kv = kv.clone();
        async move {
            let agent_id = input
                .get("agent_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| iii_sdk::IIIError::Handler("agent_id is required".into()))?;

            let new_status: AgentStatus = serde_json::from_value(
                input
                    .get("status")
                    .cloned()
                    .ok_or_else(|| iii_sdk::IIIError::Handler("status is required".into()))?,
            )
            .map_err(|e| iii_sdk::IIIError::Handler(format!("invalid status: {}", e)))?;

            let mut agent: Agent = kv
                .get("agents", agent_id)
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?
                .ok_or_else(|| {
                    iii_sdk::IIIError::Handler(format!("agent not found: {}", agent_id))
                })?;

            let old_status = agent.status;
            agent.status = new_status;
            agent.last_seen = Some(Utc::now());

            if new_status == AgentStatus::Connected && old_status == AgentStatus::Disconnected {
                agent.connected_at = Some(Utc::now());
            }

            kv.set("agents", agent_id, &agent)
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

            Ok(json!({
                "agent_id": agent_id,
                "old_status": old_status,
                "new_status": new_status
            }))
        }
    });
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
    ]
}

fn register_detect(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function("rimuru.agents.detect", move |input: Value| {
        let kv = kv.clone();
        async move {
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
                        let _ = kv.set("agent_config", &agent_id, &config).await;
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

            Ok(json!({
                "detected": detected,
                "total": detected.iter().filter(|d| d["installed"].as_bool().unwrap_or(false)).count()
            }))
        }
    });
}

fn register_connect(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function("rimuru.agents.connect", move |input: Value| {
        let kv = kv.clone();
        async move {
            let agent_type_str = input
                .get("agent_type")
                .and_then(|v| v.as_str())
                .ok_or_else(|| iii_sdk::IIIError::Handler("agent_type is required".into()))?;

            let agent_type: AgentType = serde_json::from_value(json!(agent_type_str))
                .map_err(|e| iii_sdk::IIIError::Handler(format!("invalid agent_type: {}", e)))?;

            let existing_agents: Vec<Agent> = kv.list("agents").await.unwrap_or_default();

            if let Some(existing) = existing_agents.iter().find(|a| a.agent_type == agent_type) {
                let agent_id = existing.id.to_string();
                let mut agent = existing.clone();
                agent.status = AgentStatus::Connected;
                agent.connected_at = Some(Utc::now());
                agent.last_seen = Some(Utc::now());
                kv.set("agents", &agent_id, &agent)
                    .await
                    .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;
                return Ok(json!({"agent": agent, "action": "connected"}));
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
            kv.set("agents", &agent_id, &agent)
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;
            let config = AgentConfig {
                agent_id: agent.id,
                ..AgentConfig::default()
            };
            kv.set("agent_config", &agent_id, &config)
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

            Ok(json!({"agent": agent, "action": "created_and_connected"}))
        }
    });
}

fn register_disconnect(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function("rimuru.agents.disconnect", move |input: Value| {
        let kv = kv.clone();
        async move {
            let agent_id = input
                .get("agent_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| iii_sdk::IIIError::Handler("agent_id is required".into()))?;

            let mut agent: Agent = kv
                .get("agents", agent_id)
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?
                .ok_or_else(|| {
                    iii_sdk::IIIError::Handler(format!("agent not found: {}", agent_id))
                })?;

            agent.status = AgentStatus::Disconnected;
            agent.last_seen = Some(Utc::now());

            kv.set("agents", agent_id, &agent)
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

            Ok(json!({"agent": agent, "action": "disconnected"}))
        }
    });
}

fn get_adapter(agent_type: &AgentType) -> Option<Box<dyn AgentAdapter>> {
    match agent_type {
        AgentType::ClaudeCode => Some(Box::new(ClaudeCodeAdapter::new())),
        AgentType::Cursor => Some(Box::new(CursorAdapter::new())),
        AgentType::Copilot => Some(Box::new(CopilotAdapter::new())),
        AgentType::Codex => Some(Box::new(CodexAdapter::new())),
        AgentType::Goose => Some(Box::new(GooseAdapter::new())),
        AgentType::OpenCode => Some(Box::new(OpenCodeAdapter::new())),
    }
}

fn register_sync(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function("rimuru.agents.sync", move |_input: Value| {
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

                if !adapter.is_installed() {
                    let agent_id = agent.id.to_string();
                    let _ = kv.delete("agents", &agent_id).await;
                    let _ = kv.delete("agent_config", &agent_id).await;
                    continue;
                }

                let sessions = match adapter.get_sessions().await {
                    Ok(s) => s,
                    Err(e) => {
                        warn!("Failed to get sessions for {}: {}", agent.name, e);
                        continue;
                    }
                };

                let mut agent_total_cost = 0.0f64;
                let mut agent_session_count = 0u64;
                let mut agent_total_tokens = 0u64;
                let mut has_active_session = false;

                for mut session in sessions {
                    session.agent_id = agent.id;
                    let session_id = session.id.to_string();

                    agent_total_cost += session.total_cost;
                    agent_total_tokens += session.total_tokens;
                    agent_session_count += 1;
                    if matches!(session.status, SessionStatus::Active) {
                        has_active_session = true;
                    }

                    if session.total_cost > 0.0 {
                        if let Some(ref model) = session.model {
                            let mut cost_record = CostRecord::new(
                                agent.id,
                                agent.agent_type,
                                model.clone(),
                                "anthropic".to_string(),
                                session.input_tokens,
                                session.output_tokens,
                                session.total_cost * 0.3,
                                session.total_cost * 0.7,
                            );
                            cost_record.recorded_at = session.started_at;
                            let record_id = cost_record.id.to_string();
                            let _ = kv.set("cost_records", &record_id, &cost_record).await;
                            synced_costs += 1;
                        }
                    }

                    let _ = kv.set("sessions", &session_id, &session).await;
                    synced_sessions += 1;
                }

                let agent_id = agent.id.to_string();
                if agent_session_count == 0 || (agent_total_cost == 0.0 && agent_total_tokens == 0) {
                    let _ = kv.delete("agents", &agent_id).await;
                    let _ = kv.delete("agent_config", &agent_id).await;
                    continue;
                }
                let mut updated_agent = agent.clone();
                updated_agent.session_count = agent_session_count;
                updated_agent.total_cost = agent_total_cost;
                updated_agent.last_seen = Some(Utc::now());
                updated_agent.version = adapter.detect_version();
                updated_agent.status = if has_active_session {
                    AgentStatus::Connected
                } else {
                    AgentStatus::Disconnected
                };
                let _ = kv.set("agents", &agent_id, &updated_agent).await;
                synced_agents += 1;
            }

            info!(
                "Synced {} sessions, {} costs from {} agents",
                synced_sessions, synced_costs, synced_agents
            );

            Ok(json!({
                "synced_agents": synced_agents,
                "synced_sessions": synced_sessions,
                "synced_costs": synced_costs
            }))
        }
    });
}
