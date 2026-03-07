use chrono::Utc;
use iii_sdk::III;
use serde_json::{json, Value};

use super::sysutil::{collect_cpu_usage, collect_memory_info, kv_err};
use crate::models::{Agent, AgentStatus, MetricsHistory, Session, SessionStatus, SystemMetrics};
use crate::state::StateKV;

pub fn register(iii: &III, kv: &StateKV) {
    register_current(iii, kv);
    register_history(iii, kv);
    register_collect(iii, kv);
}

fn register_current(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function("rimuru.metrics.current", move |_input: Value| {
        let kv = kv.clone();
        async move {
            let metrics: Option<SystemMetrics> =
                kv.get("system_metrics", "latest").await.map_err(kv_err)?;

            match metrics {
                Some(m) => Ok(json!({"metrics": m})),
                None => {
                    let default = SystemMetrics::default();
                    Ok(json!({"metrics": default, "note": "no metrics collected yet"}))
                }
            }
        }
    });
}

fn register_history(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function("rimuru.metrics.history", move |input: Value| {
        let kv = kv.clone();
        async move {
            let limit = input.get("limit").and_then(|v| v.as_u64()).unwrap_or(60) as usize;

            let interval = input
                .get("interval_secs")
                .and_then(|v| v.as_u64())
                .unwrap_or(60);

            let history: Option<MetricsHistory> =
                kv.get("system_metrics", "history").await.map_err(kv_err)?;

            match history {
                Some(mut h) => {
                    if h.entries.len() > limit {
                        let start = h.entries.len() - limit;
                        h.entries = h.entries[start..].to_vec();
                    }
                    h.total_entries = h.entries.len();
                    h.interval_secs = interval;
                    Ok(json!({"history": h}))
                }
                None => {
                    let empty = MetricsHistory {
                        entries: vec![],
                        interval_secs: interval,
                        total_entries: 0,
                    };
                    Ok(json!({"history": empty}))
                }
            }
        }
    });
}

fn register_collect(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    let start_time = std::time::Instant::now();

    iii.register_function("rimuru.metrics.collect", move |_input: Value| {
        let kv = kv.clone();
        let start_time = start_time;
        async move {
            let agents: Vec<Agent> = kv.list("agents").await.map_err(kv_err)?;

            let sessions: Vec<Session> = kv.list("sessions").await.map_err(kv_err)?;

            let active_agents = agents
                .iter()
                .filter(|a| a.status == AgentStatus::Active || a.status == AgentStatus::Connected)
                .count() as u32;

            let active_sessions = sessions
                .iter()
                .filter(|s| s.status == SessionStatus::Active)
                .count() as u32;

            let today = Utc::now().date_naive();
            let today_cost: f64 = sessions
                .iter()
                .filter(|s| s.started_at.date_naive() == today)
                .map(|s| s.total_cost)
                .sum();

            let (memory_used_mb, memory_total_mb) = collect_memory_info().await;
            let cpu_usage = collect_cpu_usage().await;

            let uptime_secs = start_time.elapsed().as_secs();

            let total_sessions_today = sessions
                .iter()
                .filter(|s| s.started_at.date_naive() == today)
                .count() as f64;

            let errored_today = sessions
                .iter()
                .filter(|s| s.started_at.date_naive() == today && s.status == SessionStatus::Error)
                .count() as f64;

            let error_rate = if total_sessions_today > 0.0 {
                errored_today / total_sessions_today
            } else {
                0.0
            };

            let metrics = SystemMetrics {
                timestamp: Utc::now(),
                cpu_usage_percent: cpu_usage,
                memory_used_mb,
                memory_total_mb,
                active_agents,
                active_sessions,
                total_cost_today: today_cost,
                requests_per_minute: 0.0,
                avg_response_time_ms: 0.0,
                error_rate,
                uptime_secs,
            };

            kv.set("system_metrics", "latest", &metrics)
                .await
                .map_err(kv_err)?;

            let mut history: MetricsHistory = kv
                .get("system_metrics", "history")
                .await
                .map_err(kv_err)?
                .unwrap_or(MetricsHistory {
                    entries: vec![],
                    interval_secs: 60,
                    total_entries: 0,
                });

            history.entries.push(metrics.clone());

            let max_entries = 1440;
            if history.entries.len() > max_entries {
                let drain = history.entries.len() - max_entries;
                history.entries.drain(..drain);
            }
            history.total_entries = history.entries.len();

            kv.set("system_metrics", "history", &history)
                .await
                .map_err(kv_err)?;

            Ok(json!({
                "metrics": metrics,
                "history_size": history.total_entries
            }))
        }
    });
}
