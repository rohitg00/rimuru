use chrono::Utc;
use iii_sdk::III;
use serde_json::{json, Value};

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
            let metrics: Option<SystemMetrics> = kv
                .get("system_metrics", "latest")
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

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
            let limit = input
                .get("limit")
                .and_then(|v| v.as_u64())
                .unwrap_or(60) as usize;

            let interval = input
                .get("interval_secs")
                .and_then(|v| v.as_u64())
                .unwrap_or(60);

            let history: Option<MetricsHistory> = kv
                .get("system_metrics", "history")
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

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
            let agents: Vec<Agent> = kv
                .list("agents")
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

            let sessions: Vec<Session> = kv
                .list("sessions")
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

            let active_agents = agents
                .iter()
                .filter(|a| {
                    a.status == AgentStatus::Active || a.status == AgentStatus::Connected
                })
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
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

            let mut history: MetricsHistory = kv
                .get("system_metrics", "history")
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?
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
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

            Ok(json!({
                "metrics": metrics,
                "history_size": history.total_entries
            }))
        }
    });
}

async fn collect_memory_info() -> (f64, f64) {
    if cfg!(target_os = "macos") {
        let output = tokio::process::Command::new("sysctl")
            .args(["-n", "hw.memsize"])
            .output()
            .await;

        let total_bytes: u64 = match output {
            Ok(o) => String::from_utf8_lossy(&o.stdout)
                .trim()
                .parse()
                .unwrap_or(0),
            Err(_) => 0,
        };
        let total_mb = total_bytes as f64 / (1024.0 * 1024.0);

        let vm_output = tokio::process::Command::new("vm_stat").output().await;

        let used_mb = match vm_output {
            Ok(o) => {
                let text = String::from_utf8_lossy(&o.stdout);
                let page_size: u64 = 16384;
                let mut active: u64 = 0;
                let mut wired: u64 = 0;
                let mut compressed: u64 = 0;

                for line in text.lines() {
                    if line.contains("Pages active:") {
                        active = parse_vm_stat_line(line);
                    } else if line.contains("Pages wired down:") {
                        wired = parse_vm_stat_line(line);
                    } else if line.contains("Pages occupied by compressor:") {
                        compressed = parse_vm_stat_line(line);
                    }
                }

                ((active + wired + compressed) * page_size) as f64 / (1024.0 * 1024.0)
            }
            Err(_) => 0.0,
        };

        (used_mb, total_mb)
    } else if cfg!(target_os = "linux") {
        let output = tokio::fs::read_to_string("/proc/meminfo").await;

        match output {
            Ok(content) => {
                let mut total_kb: u64 = 0;
                let mut available_kb: u64 = 0;

                for line in content.lines() {
                    if line.starts_with("MemTotal:") {
                        total_kb = parse_meminfo_value(line);
                    } else if line.starts_with("MemAvailable:") {
                        available_kb = parse_meminfo_value(line);
                    }
                }

                let total_mb = total_kb as f64 / 1024.0;
                let used_mb = (total_kb - available_kb) as f64 / 1024.0;
                (used_mb, total_mb)
            }
            Err(_) => (0.0, 0.0),
        }
    } else {
        (0.0, 0.0)
    }
}

fn parse_vm_stat_line(line: &str) -> u64 {
    line.split(':')
        .nth(1)
        .unwrap_or("")
        .trim()
        .trim_end_matches('.')
        .parse()
        .unwrap_or(0)
}

fn parse_meminfo_value(line: &str) -> u64 {
    line.split_whitespace()
        .nth(1)
        .unwrap_or("0")
        .parse()
        .unwrap_or(0)
}

async fn collect_cpu_usage() -> f64 {
    if cfg!(target_os = "macos") {
        let output = tokio::process::Command::new("ps")
            .args(["-A", "-o", "%cpu"])
            .output()
            .await;

        match output {
            Ok(o) => {
                let text = String::from_utf8_lossy(&o.stdout);
                let total: f64 = text
                    .lines()
                    .skip(1)
                    .filter_map(|line| line.trim().parse::<f64>().ok())
                    .sum();

                let num_cpus = std::thread::available_parallelism()
                    .map(|n| n.get() as f64)
                    .unwrap_or(1.0);

                (total / num_cpus).min(100.0)
            }
            Err(_) => 0.0,
        }
    } else if cfg!(target_os = "linux") {
        let output = tokio::process::Command::new("grep")
            .args(["cpu ", "/proc/stat"])
            .output()
            .await;

        match output {
            Ok(o) => {
                let text = String::from_utf8_lossy(&o.stdout);
                let parts: Vec<u64> = text
                    .split_whitespace()
                    .skip(1)
                    .filter_map(|s| s.parse().ok())
                    .collect();

                if parts.len() >= 4 {
                    let total: u64 = parts.iter().sum();
                    let idle = parts[3];
                    if total > 0 {
                        ((total - idle) as f64 / total as f64) * 100.0
                    } else {
                        0.0
                    }
                } else {
                    0.0
                }
            }
            Err(_) => 0.0,
        }
    } else {
        0.0
    }
}
