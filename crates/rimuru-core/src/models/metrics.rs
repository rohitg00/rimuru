use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: DateTime<Utc>,
    pub cpu_usage_percent: f64,
    pub memory_used_mb: f64,
    pub memory_total_mb: f64,
    pub active_agents: u32,
    pub active_sessions: u32,
    pub total_cost_today: f64,
    pub requests_per_minute: f64,
    pub avg_response_time_ms: f64,
    pub error_rate: f64,
    pub uptime_secs: u64,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            cpu_usage_percent: 0.0,
            memory_used_mb: 0.0,
            memory_total_mb: 0.0,
            active_agents: 0,
            active_sessions: 0,
            total_cost_today: 0.0,
            requests_per_minute: 0.0,
            avg_response_time_ms: 0.0,
            error_rate: 0.0,
            uptime_secs: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsHistory {
    pub entries: Vec<SystemMetrics>,
    pub interval_secs: u64,
    pub total_entries: usize,
}
