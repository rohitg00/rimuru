use crate::models::SystemMetrics;
use std::sync::Arc;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};
use tokio::sync::RwLock;
use tracing::trace;

pub struct SystemCollector {
    system: Arc<RwLock<System>>,
}

impl SystemCollector {
    pub fn new() -> Self {
        let system = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );

        Self {
            system: Arc::new(RwLock::new(system)),
        }
    }

    pub async fn collect(&self, active_sessions: i32) -> SystemMetrics {
        let mut system = self.system.write().await;

        system.refresh_cpu_all();
        system.refresh_memory();

        let cpu_percent = system.global_cpu_usage();
        let memory_used_mb = (system.used_memory() / 1024 / 1024) as i64;
        let memory_total_mb = (system.total_memory() / 1024 / 1024) as i64;

        trace!(
            cpu_percent = cpu_percent,
            memory_used_mb = memory_used_mb,
            memory_total_mb = memory_total_mb,
            active_sessions = active_sessions,
            "System metrics collected"
        );

        SystemMetrics::new(
            cpu_percent,
            memory_used_mb,
            memory_total_mb,
            active_sessions,
        )
    }

    pub async fn get_cpu_usage(&self) -> f32 {
        let mut system = self.system.write().await;
        system.refresh_cpu_all();
        system.global_cpu_usage()
    }

    pub async fn get_memory_info(&self) -> (i64, i64) {
        let mut system = self.system.write().await;
        system.refresh_memory();
        let used = (system.used_memory() / 1024 / 1024) as i64;
        let total = (system.total_memory() / 1024 / 1024) as i64;
        (used, total)
    }

    pub async fn get_cpu_count(&self) -> usize {
        let system = self.system.read().await;
        system.cpus().len()
    }

    pub async fn get_system_name(&self) -> Option<String> {
        System::name()
    }

    pub async fn get_os_version(&self) -> Option<String> {
        System::os_version()
    }

    pub async fn get_host_name(&self) -> Option<String> {
        System::host_name()
    }
}

impl Default for SystemCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for SystemCollector {
    fn clone(&self) -> Self {
        Self {
            system: Arc::clone(&self.system),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_collector_creation() {
        let collector = SystemCollector::new();
        let metrics = collector.collect(0).await;

        assert!(metrics.cpu_percent >= 0.0);
        assert!(metrics.memory_total_mb > 0);
        assert!(metrics.memory_used_mb >= 0);
        assert_eq!(metrics.active_sessions, 0);
    }

    #[tokio::test]
    async fn test_cpu_usage() {
        let collector = SystemCollector::new();
        let cpu = collector.get_cpu_usage().await;
        assert!(cpu >= 0.0 && cpu <= 100.0);
    }

    #[tokio::test]
    async fn test_memory_info() {
        let collector = SystemCollector::new();
        let (used, total) = collector.get_memory_info().await;
        assert!(total > 0);
        assert!(used >= 0);
        assert!(used <= total);
    }

    #[tokio::test]
    async fn test_cpu_count() {
        let collector = SystemCollector::new();
        let count = collector.get_cpu_count().await;
        assert!(count > 0);
    }

    #[tokio::test]
    async fn test_collector_clone() {
        let collector = SystemCollector::new();
        let cloned = collector.clone();
        let metrics = cloned.collect(5).await;
        assert_eq!(metrics.active_sessions, 5);
    }

    #[test]
    fn test_collector_default() {
        let collector = SystemCollector::default();
        assert!(Arc::strong_count(&collector.system) == 1);
    }
}
