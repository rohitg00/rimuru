use chrono::Utc;
use iii_sdk::III;
use serde_json::{json, Value};

use crate::models::{Agent, AgentStatus, PluginState, PluginStatus, Session, SessionStatus, SystemMetrics};
use crate::state::StateKV;

pub fn register(iii: &III, kv: &StateKV) {
    register_check(iii, kv);
}

fn register_check(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    let boot_time = Utc::now();

    iii.register_function("rimuru.health.check", move |_input: Value| {
        let kv = kv.clone();
        let boot_time = boot_time;
        async move {
            let mut checks: Vec<Value> = Vec::new();
            let mut overall_healthy = true;

            let state_check = kv.get::<Value>("config", "__health_probe").await;
            match state_check {
                Ok(_) => {
                    checks.push(json!({
                        "component": "state",
                        "status": "healthy",
                        "message": "state KV is accessible"
                    }));
                }
                Err(e) => {
                    overall_healthy = false;
                    checks.push(json!({
                        "component": "state",
                        "status": "unhealthy",
                        "message": format!("state KV error: {}", e)
                    }));
                }
            }

            let agents: Vec<Agent> = kv.list("agents").await.unwrap_or_default();
            let connected_agents = agents
                .iter()
                .filter(|a| {
                    a.status == AgentStatus::Connected || a.status == AgentStatus::Active
                })
                .count();
            let total_agents = agents.len();

            checks.push(json!({
                "component": "agents",
                "status": if total_agents > 0 { "healthy" } else { "warning" },
                "message": format!("{}/{} agents connected", connected_agents, total_agents),
                "details": {
                    "total": total_agents,
                    "connected": connected_agents
                }
            }));

            let sessions: Vec<Session> = kv.list("sessions").await.unwrap_or_default();
            let active_sessions = sessions
                .iter()
                .filter(|s| s.status == SessionStatus::Active)
                .count();
            let error_sessions = sessions
                .iter()
                .filter(|s| s.status == SessionStatus::Error)
                .count();

            let session_status = if error_sessions > active_sessions && active_sessions > 0 {
                overall_healthy = false;
                "degraded"
            } else {
                "healthy"
            };

            checks.push(json!({
                "component": "sessions",
                "status": session_status,
                "message": format!("{} active, {} errored", active_sessions, error_sessions),
                "details": {
                    "active": active_sessions,
                    "errored": error_sessions,
                    "total": sessions.len()
                }
            }));

            let plugins: Vec<PluginState> = kv.list("plugin_state").await.unwrap_or_default();
            let running_plugins = plugins
                .iter()
                .filter(|p| p.status == PluginStatus::Running)
                .count();
            let errored_plugins = plugins
                .iter()
                .filter(|p| p.status == PluginStatus::Error)
                .count();

            let plugin_status = if errored_plugins > 0 {
                "degraded"
            } else {
                "healthy"
            };

            checks.push(json!({
                "component": "plugins",
                "status": plugin_status,
                "message": format!("{} running, {} errored", running_plugins, errored_plugins),
                "details": {
                    "running": running_plugins,
                    "errored": errored_plugins,
                    "total": plugins.len()
                }
            }));

            let metrics: Option<SystemMetrics> = kv
                .get("system_metrics", "latest")
                .await
                .unwrap_or(None);

            let metrics_status = if let Some(ref m) = metrics {
                let age_secs = (Utc::now() - m.timestamp).num_seconds();
                if age_secs > 300 {
                    "stale"
                } else if m.cpu_usage_percent > 90.0 || m.error_rate > 0.5 {
                    overall_healthy = false;
                    "degraded"
                } else {
                    "healthy"
                }
            } else {
                "unknown"
            };

            checks.push(json!({
                "component": "metrics",
                "status": metrics_status,
                "message": match &metrics {
                    Some(m) => format!(
                        "cpu={:.1}%, mem={:.0}/{:.0}MB, err_rate={:.2}",
                        m.cpu_usage_percent, m.memory_used_mb, m.memory_total_mb, m.error_rate
                    ),
                    None => "no metrics collected".into()
                }
            }));

            let uptime_secs = (Utc::now() - boot_time).num_seconds().max(0) as u64;
            let uptime_display = if uptime_secs >= 86400 {
                format!("{}d {}h", uptime_secs / 86400, (uptime_secs % 86400) / 3600)
            } else if uptime_secs >= 3600 {
                format!("{}h {}m", uptime_secs / 3600, (uptime_secs % 3600) / 60)
            } else {
                format!("{}m {}s", uptime_secs / 60, uptime_secs % 60)
            };

            let overall_status = if overall_healthy { "healthy" } else { "degraded" };

            Ok(json!({
                "status": overall_status,
                "uptime_secs": uptime_secs,
                "uptime": uptime_display,
                "boot_time": boot_time.to_rfc3339(),
                "timestamp": Utc::now().to_rfc3339(),
                "version": env!("CARGO_PKG_VERSION"),
                "checks": checks
            }))
        }
    });
}
