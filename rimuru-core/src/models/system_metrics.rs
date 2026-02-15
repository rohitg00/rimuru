use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SystemMetrics {
    pub timestamp: DateTime<Utc>,
    pub cpu_percent: f32,
    pub memory_used_mb: i64,
    pub memory_total_mb: i64,
    pub active_sessions: i32,
}

impl SystemMetrics {
    pub fn new(
        cpu_percent: f32,
        memory_used_mb: i64,
        memory_total_mb: i64,
        active_sessions: i32,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            cpu_percent,
            memory_used_mb,
            memory_total_mb,
            active_sessions,
        }
    }

    pub fn memory_percent(&self) -> f32 {
        if self.memory_total_mb > 0 {
            (self.memory_used_mb as f32 / self.memory_total_mb as f32) * 100.0
        } else {
            0.0
        }
    }

    pub fn memory_available_mb(&self) -> i64 {
        self.memory_total_mb - self.memory_used_mb
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub cpu_percent: f32,
    pub memory_used_mb: i64,
    pub memory_total_mb: i64,
    pub memory_percent: f32,
    pub active_sessions: i32,
}

impl From<SystemMetrics> for MetricsSnapshot {
    fn from(metrics: SystemMetrics) -> Self {
        Self {
            cpu_percent: metrics.cpu_percent,
            memory_used_mb: metrics.memory_used_mb,
            memory_total_mb: metrics.memory_total_mb,
            memory_percent: metrics.memory_percent(),
            active_sessions: metrics.active_sessions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_metrics_new() {
        let metrics = SystemMetrics::new(45.5, 8192, 16384, 3);

        assert_eq!(metrics.cpu_percent, 45.5);
        assert_eq!(metrics.memory_used_mb, 8192);
        assert_eq!(metrics.memory_total_mb, 16384);
        assert_eq!(metrics.active_sessions, 3);
    }

    #[test]
    fn test_memory_percent() {
        let metrics = SystemMetrics::new(50.0, 8192, 16384, 0);

        assert_eq!(metrics.memory_percent(), 50.0);
    }

    #[test]
    fn test_memory_percent_zero_total() {
        let metrics = SystemMetrics::new(50.0, 0, 0, 0);

        assert_eq!(metrics.memory_percent(), 0.0);
    }

    #[test]
    fn test_memory_available() {
        let metrics = SystemMetrics::new(50.0, 8192, 16384, 0);

        assert_eq!(metrics.memory_available_mb(), 8192);
    }

    #[test]
    fn test_metrics_snapshot_from() {
        let metrics = SystemMetrics::new(25.0, 4096, 16384, 2);
        let snapshot: MetricsSnapshot = metrics.into();

        assert_eq!(snapshot.cpu_percent, 25.0);
        assert_eq!(snapshot.memory_used_mb, 4096);
        assert_eq!(snapshot.memory_total_mb, 16384);
        assert_eq!(snapshot.memory_percent, 25.0);
        assert_eq!(snapshot.active_sessions, 2);
    }
}
