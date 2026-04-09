use chrono::Utc;
use iii_sdk::{III, RegisterFunctionMessage};
use serde_json::{Value, json};

use crate::models::{
    Agent, AgentStatus, PluginState, PluginStatus, Session, SessionStatus, SystemMetrics,
};
use crate::state::StateKV;

pub fn register(iii: &III, kv: &StateKV) {
    register_check(iii, kv);
}

struct HealthCheck {
    component: &'static str,
    status: &'static str,
    message: String,
    details: Option<Value>,
    healthy: bool,
}

impl HealthCheck {
    fn to_json(&self) -> Value {
        let mut v = json!({
            "component": self.component,
            "status": self.status,
            "message": self.message,
        });
        if let Some(ref d) = self.details {
            v.as_object_mut()
                .unwrap()
                .insert("details".into(), d.clone());
        }
        v
    }
}

async fn check_state(kv: &StateKV) -> HealthCheck {
    match kv.get::<Value>("config", "__health_probe").await {
        Ok(_) => HealthCheck {
            component: "state",
            status: "healthy",
            message: "state KV is accessible".into(),
            details: None,
            healthy: true,
        },
        Err(e) => HealthCheck {
            component: "state",
            status: "unhealthy",
            message: format!("state KV error: {}", e),
            details: None,
            healthy: false,
        },
    }
}

async fn check_agents(kv: &StateKV) -> HealthCheck {
    let agents: Vec<Agent> = kv.list("agents").await.unwrap_or_default();
    let connected = agents
        .iter()
        .filter(|a| a.status == AgentStatus::Connected || a.status == AgentStatus::Active)
        .count();
    let total = agents.len();
    HealthCheck {
        component: "agents",
        status: if total > 0 { "healthy" } else { "warning" },
        message: format!("{}/{} agents connected", connected, total),
        details: Some(json!({"total": total, "connected": connected})),
        healthy: true,
    }
}

async fn check_sessions(kv: &StateKV) -> HealthCheck {
    let sessions: Vec<Session> = kv.list("sessions").await.unwrap_or_default();
    let active = sessions
        .iter()
        .filter(|s| s.status == SessionStatus::Active)
        .count();
    let errored = sessions
        .iter()
        .filter(|s| s.status == SessionStatus::Error)
        .count();
    let degraded = errored > active && active > 0;
    HealthCheck {
        component: "sessions",
        status: if degraded { "degraded" } else { "healthy" },
        message: format!("{} active, {} errored", active, errored),
        details: Some(json!({"active": active, "errored": errored, "total": sessions.len()})),
        healthy: !degraded,
    }
}

async fn check_plugins(kv: &StateKV) -> HealthCheck {
    let plugins: Vec<PluginState> = kv.list("plugin_state").await.unwrap_or_default();
    let running = plugins
        .iter()
        .filter(|p| p.status == PluginStatus::Running)
        .count();
    let errored = plugins
        .iter()
        .filter(|p| p.status == PluginStatus::Error)
        .count();
    HealthCheck {
        component: "plugins",
        status: if errored > 0 { "degraded" } else { "healthy" },
        message: format!("{} running, {} errored", running, errored),
        details: Some(json!({"running": running, "errored": errored, "total": plugins.len()})),
        healthy: true,
    }
}

async fn check_metrics(kv: &StateKV) -> HealthCheck {
    let metrics: Option<SystemMetrics> = kv.get("system_metrics", "latest").await.unwrap_or(None);
    match metrics {
        Some(ref m) => {
            let age_secs = (Utc::now() - m.timestamp).num_seconds();
            let (status, healthy) = if age_secs > 300 {
                ("stale", true)
            } else if m.cpu_usage_percent > 90.0 || m.error_rate > 0.5 {
                ("degraded", false)
            } else {
                ("healthy", true)
            };
            HealthCheck {
                component: "metrics",
                status,
                message: format!(
                    "cpu={:.1}%, mem={:.0}/{:.0}MB, err_rate={:.2}",
                    m.cpu_usage_percent, m.memory_used_mb, m.memory_total_mb, m.error_rate
                ),
                details: None,
                healthy,
            }
        }
        None => HealthCheck {
            component: "metrics",
            status: "unknown",
            message: "no metrics collected".into(),
            details: None,
            healthy: true,
        },
    }
}

fn register_check(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    let boot_time = Utc::now();

    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.health.check".to_string()),
        move |_input: Value| {
            let kv = kv.clone();
            let boot_time = boot_time;
            async move {
                let checks = [
                    check_state(&kv).await,
                    check_agents(&kv).await,
                    check_sessions(&kv).await,
                    check_plugins(&kv).await,
                    check_metrics(&kv).await,
                ];

                let overall_healthy = checks.iter().all(|c| c.healthy);
                let check_json: Vec<Value> = checks.iter().map(|c| c.to_json()).collect();

                let uptime_secs = (Utc::now() - boot_time).num_seconds().max(0) as u64;
                let uptime_display = if uptime_secs >= 86400 {
                    format!("{}d {}h", uptime_secs / 86400, (uptime_secs % 86400) / 3600)
                } else if uptime_secs >= 3600 {
                    format!("{}h {}m", uptime_secs / 3600, (uptime_secs % 3600) / 60)
                } else {
                    format!("{}m {}s", uptime_secs / 60, uptime_secs % 60)
                };

                Ok(json!({
                    "status": if overall_healthy { "healthy" } else { "degraded" },
                    "uptime_secs": uptime_secs,
                    "uptime": uptime_display,
                    "boot_time": boot_time.to_rfc3339(),
                    "timestamp": Utc::now().to_rfc3339(),
                    "version": env!("CARGO_PKG_VERSION"),
                    "checks": check_json
                }))
            }
        },
    );
}
