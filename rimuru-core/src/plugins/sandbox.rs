use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, warn};

use crate::error::{RimuruError, RimuruResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    Filesystem,
    FilesystemRead,
    FilesystemWrite,
    Network,
    NetworkOutbound,
    NetworkInbound,
    Database,
    DatabaseRead,
    DatabaseWrite,
    SystemMetrics,
    ProcessSpawn,
    Environment,
    All,
}

impl Permission {
    pub fn implies(&self, other: &Permission) -> bool {
        match self {
            Permission::All => true,
            Permission::Filesystem => matches!(
                other,
                Permission::Filesystem | Permission::FilesystemRead | Permission::FilesystemWrite
            ),
            Permission::Network => matches!(
                other,
                Permission::Network | Permission::NetworkOutbound | Permission::NetworkInbound
            ),
            Permission::Database => matches!(
                other,
                Permission::Database | Permission::DatabaseRead | Permission::DatabaseWrite
            ),
            _ => self == other,
        }
    }
}

impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Permission::Filesystem => write!(f, "filesystem"),
            Permission::FilesystemRead => write!(f, "filesystem:read"),
            Permission::FilesystemWrite => write!(f, "filesystem:write"),
            Permission::Network => write!(f, "network"),
            Permission::NetworkOutbound => write!(f, "network:outbound"),
            Permission::NetworkInbound => write!(f, "network:inbound"),
            Permission::Database => write!(f, "database"),
            Permission::DatabaseRead => write!(f, "database:read"),
            Permission::DatabaseWrite => write!(f, "database:write"),
            Permission::SystemMetrics => write!(f, "system_metrics"),
            Permission::ProcessSpawn => write!(f, "process_spawn"),
            Permission::Environment => write!(f, "environment"),
            Permission::All => write!(f, "all"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: Option<u64>,
    pub max_cpu_time_ms: Option<u64>,
    pub max_execution_time_ms: Option<u64>,
    pub max_file_size_mb: Option<u64>,
    pub max_open_files: Option<u32>,
    pub max_network_connections: Option<u32>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: Some(256),
            max_cpu_time_ms: Some(30_000),
            max_execution_time_ms: Some(60_000),
            max_file_size_mb: Some(100),
            max_open_files: Some(100),
            max_network_connections: Some(10),
        }
    }
}

impl ResourceLimits {
    pub fn unlimited() -> Self {
        Self {
            max_memory_mb: None,
            max_cpu_time_ms: None,
            max_execution_time_ms: None,
            max_file_size_mb: None,
            max_open_files: None,
            max_network_connections: None,
        }
    }

    pub fn restricted() -> Self {
        Self {
            max_memory_mb: Some(64),
            max_cpu_time_ms: Some(5_000),
            max_execution_time_ms: Some(10_000),
            max_file_size_mb: Some(10),
            max_open_files: Some(10),
            max_network_connections: Some(2),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub granted_permissions: HashSet<Permission>,
    pub allowed_paths: Vec<PathBuf>,
    pub denied_paths: Vec<PathBuf>,
    pub allowed_hosts: Vec<String>,
    pub denied_hosts: Vec<String>,
    pub resource_limits: ResourceLimits,
    pub plugin_data_dir: PathBuf,
}

impl SandboxConfig {
    pub fn new(plugin_data_dir: PathBuf) -> Self {
        Self {
            granted_permissions: HashSet::new(),
            allowed_paths: vec![plugin_data_dir.clone()],
            denied_paths: Vec::new(),
            allowed_hosts: Vec::new(),
            denied_hosts: Vec::new(),
            resource_limits: ResourceLimits::default(),
            plugin_data_dir,
        }
    }

    pub fn with_permission(mut self, permission: Permission) -> Self {
        self.granted_permissions.insert(permission);
        self
    }

    pub fn with_permissions(mut self, permissions: impl IntoIterator<Item = Permission>) -> Self {
        self.granted_permissions.extend(permissions);
        self
    }

    pub fn with_allowed_path(mut self, path: PathBuf) -> Self {
        self.allowed_paths.push(path);
        self
    }

    pub fn with_denied_path(mut self, path: PathBuf) -> Self {
        self.denied_paths.push(path);
        self
    }

    pub fn with_allowed_host(mut self, host: String) -> Self {
        self.allowed_hosts.push(host);
        self
    }

    pub fn with_denied_host(mut self, host: String) -> Self {
        self.denied_hosts.push(host);
        self
    }

    pub fn with_resource_limits(mut self, limits: ResourceLimits) -> Self {
        self.resource_limits = limits;
        self
    }

    pub fn trusted() -> Self {
        Self {
            granted_permissions: HashSet::from([Permission::All]),
            allowed_paths: Vec::new(),
            denied_paths: Vec::new(),
            allowed_hosts: Vec::new(),
            denied_hosts: Vec::new(),
            resource_limits: ResourceLimits::unlimited(),
            plugin_data_dir: PathBuf::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessViolation {
    pub permission: Permission,
    pub requested_resource: String,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl AccessViolation {
    pub fn new(permission: Permission, requested_resource: String, message: String) -> Self {
        Self {
            permission,
            requested_resource,
            message,
            timestamp: chrono::Utc::now(),
        }
    }
}

pub struct Sandbox {
    plugin_id: String,
    config: SandboxConfig,
    violations: Arc<RwLock<Vec<AccessViolation>>>,
    resource_usage: Arc<RwLock<ResourceUsage>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub current_memory_mb: u64,
    pub total_cpu_time_ms: u64,
    pub open_files: u32,
    pub active_connections: u32,
}

impl Sandbox {
    pub fn new(plugin_id: String, config: SandboxConfig) -> Self {
        Self {
            plugin_id,
            config,
            violations: Arc::new(RwLock::new(Vec::new())),
            resource_usage: Arc::new(RwLock::new(ResourceUsage::default())),
        }
    }

    pub fn plugin_id(&self) -> &str {
        &self.plugin_id
    }

    pub fn config(&self) -> &SandboxConfig {
        &self.config
    }

    pub fn has_permission(&self, permission: Permission) -> bool {
        for granted in &self.config.granted_permissions {
            if granted.implies(&permission) {
                return true;
            }
        }
        false
    }

    pub fn check_permission(&self, permission: Permission) -> RimuruResult<()> {
        if self.has_permission(permission) {
            Ok(())
        } else {
            Err(RimuruError::PluginPermissionDenied {
                name: self.plugin_id.clone(),
                permission: permission.to_string(),
            })
        }
    }

    pub fn check_path_access(&self, path: &Path, write: bool) -> RimuruResult<()> {
        let permission = if write {
            Permission::FilesystemWrite
        } else {
            Permission::FilesystemRead
        };

        self.check_permission(permission)?;

        let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        for denied in &self.config.denied_paths {
            if canonical_path.starts_with(denied) {
                return Err(RimuruError::PluginPermissionDenied {
                    name: self.plugin_id.clone(),
                    permission: format!("path access denied: {:?}", path),
                });
            }
        }

        if self.config.allowed_paths.is_empty() {
            return Ok(());
        }

        for allowed in &self.config.allowed_paths {
            if canonical_path.starts_with(allowed) {
                return Ok(());
            }
        }

        Err(RimuruError::PluginPermissionDenied {
            name: self.plugin_id.clone(),
            permission: format!("path not in allowed list: {:?}", path),
        })
    }

    pub fn check_network_access(&self, host: &str, outbound: bool) -> RimuruResult<()> {
        let permission = if outbound {
            Permission::NetworkOutbound
        } else {
            Permission::NetworkInbound
        };

        self.check_permission(permission)?;

        for denied in &self.config.denied_hosts {
            if host.ends_with(denied) || host == denied {
                return Err(RimuruError::PluginPermissionDenied {
                    name: self.plugin_id.clone(),
                    permission: format!("network access denied to host: {}", host),
                });
            }
        }

        if self.config.allowed_hosts.is_empty() {
            return Ok(());
        }

        for allowed in &self.config.allowed_hosts {
            if host.ends_with(allowed) || host == allowed {
                return Ok(());
            }
        }

        Err(RimuruError::PluginPermissionDenied {
            name: self.plugin_id.clone(),
            permission: format!("network access denied to host: {}", host),
        })
    }

    pub fn check_database_access(&self, write: bool) -> RimuruResult<()> {
        let permission = if write {
            Permission::DatabaseWrite
        } else {
            Permission::DatabaseRead
        };
        self.check_permission(permission)
    }

    pub async fn check_resource_limits(&self) -> RimuruResult<()> {
        let usage = self.resource_usage.read().await;
        let limits = &self.config.resource_limits;

        if let Some(max_memory) = limits.max_memory_mb {
            if usage.current_memory_mb > max_memory {
                return Err(RimuruError::plugin(format!(
                    "Memory limit exceeded: {} MB > {} MB",
                    usage.current_memory_mb, max_memory
                )));
            }
        }

        if let Some(max_cpu) = limits.max_cpu_time_ms {
            if usage.total_cpu_time_ms > max_cpu {
                return Err(RimuruError::plugin(format!(
                    "CPU time limit exceeded: {} ms > {} ms",
                    usage.total_cpu_time_ms, max_cpu
                )));
            }
        }

        if let Some(max_files) = limits.max_open_files {
            if usage.open_files > max_files {
                return Err(RimuruError::plugin(format!(
                    "Open files limit exceeded: {} > {}",
                    usage.open_files, max_files
                )));
            }
        }

        if let Some(max_connections) = limits.max_network_connections {
            if usage.active_connections > max_connections {
                return Err(RimuruError::plugin(format!(
                    "Network connections limit exceeded: {} > {}",
                    usage.active_connections, max_connections
                )));
            }
        }

        Ok(())
    }

    pub async fn update_memory_usage(&self, mb: u64) {
        let mut usage = self.resource_usage.write().await;
        usage.current_memory_mb = mb;
    }

    pub async fn add_cpu_time(&self, ms: u64) {
        let mut usage = self.resource_usage.write().await;
        usage.total_cpu_time_ms += ms;
    }

    pub async fn increment_open_files(&self) -> RimuruResult<()> {
        let mut usage = self.resource_usage.write().await;
        if let Some(max) = self.config.resource_limits.max_open_files {
            if usage.open_files >= max {
                return Err(RimuruError::plugin("Open files limit reached"));
            }
        }
        usage.open_files += 1;
        Ok(())
    }

    pub async fn decrement_open_files(&self) {
        let mut usage = self.resource_usage.write().await;
        if usage.open_files > 0 {
            usage.open_files -= 1;
        }
    }

    pub async fn increment_connections(&self) -> RimuruResult<()> {
        let mut usage = self.resource_usage.write().await;
        if let Some(max) = self.config.resource_limits.max_network_connections {
            if usage.active_connections >= max {
                return Err(RimuruError::plugin("Network connections limit reached"));
            }
        }
        usage.active_connections += 1;
        Ok(())
    }

    pub async fn decrement_connections(&self) {
        let mut usage = self.resource_usage.write().await;
        if usage.active_connections > 0 {
            usage.active_connections -= 1;
        }
    }

    pub async fn get_resource_usage(&self) -> ResourceUsage {
        self.resource_usage.read().await.clone()
    }

    pub async fn record_violation(&self, violation: AccessViolation) {
        warn!(
            "Plugin {} access violation: {} - {}",
            self.plugin_id, violation.permission, violation.message
        );
        let mut violations = self.violations.write().await;
        violations.push(violation);
    }

    pub async fn get_violations(&self) -> Vec<AccessViolation> {
        self.violations.read().await.clone()
    }

    pub async fn clear_violations(&self) {
        let mut violations = self.violations.write().await;
        violations.clear();
    }
}

pub struct SandboxManager {
    sandboxes: Arc<RwLock<std::collections::HashMap<String, Sandbox>>>,
    default_config: SandboxConfig,
}

impl SandboxManager {
    pub fn new() -> Self {
        Self {
            sandboxes: Arc::new(RwLock::new(std::collections::HashMap::new())),
            default_config: SandboxConfig::new(PathBuf::new()),
        }
    }

    pub fn with_default_config(mut self, config: SandboxConfig) -> Self {
        self.default_config = config;
        self
    }

    pub async fn create_sandbox(
        &self,
        plugin_id: &str,
        config: Option<SandboxConfig>,
    ) -> RimuruResult<()> {
        let config = config.unwrap_or_else(|| self.default_config.clone());
        let sandbox = Sandbox::new(plugin_id.to_string(), config);

        let mut sandboxes = self.sandboxes.write().await;
        if sandboxes.contains_key(plugin_id) {
            return Err(RimuruError::PluginAlreadyLoaded(plugin_id.to_string()));
        }

        sandboxes.insert(plugin_id.to_string(), sandbox);
        debug!("Created sandbox for plugin: {}", plugin_id);
        Ok(())
    }

    pub async fn remove_sandbox(&self, plugin_id: &str) -> RimuruResult<()> {
        let mut sandboxes = self.sandboxes.write().await;
        sandboxes
            .remove(plugin_id)
            .ok_or_else(|| RimuruError::PluginNotFound(plugin_id.to_string()))?;
        debug!("Removed sandbox for plugin: {}", plugin_id);
        Ok(())
    }

    pub async fn check_permission(
        &self,
        plugin_id: &str,
        permission: Permission,
    ) -> RimuruResult<()> {
        let sandboxes = self.sandboxes.read().await;
        let sandbox = sandboxes
            .get(plugin_id)
            .ok_or_else(|| RimuruError::PluginNotFound(plugin_id.to_string()))?;
        sandbox.check_permission(permission)
    }

    pub async fn check_path_access(
        &self,
        plugin_id: &str,
        path: &Path,
        write: bool,
    ) -> RimuruResult<()> {
        let sandboxes = self.sandboxes.read().await;
        let sandbox = sandboxes
            .get(plugin_id)
            .ok_or_else(|| RimuruError::PluginNotFound(plugin_id.to_string()))?;
        sandbox.check_path_access(path, write)
    }

    pub async fn check_network_access(
        &self,
        plugin_id: &str,
        host: &str,
        outbound: bool,
    ) -> RimuruResult<()> {
        let sandboxes = self.sandboxes.read().await;
        let sandbox = sandboxes
            .get(plugin_id)
            .ok_or_else(|| RimuruError::PluginNotFound(plugin_id.to_string()))?;
        sandbox.check_network_access(host, outbound)
    }

    pub async fn get_violations(&self, plugin_id: &str) -> RimuruResult<Vec<AccessViolation>> {
        let sandboxes = self.sandboxes.read().await;
        let sandbox = sandboxes
            .get(plugin_id)
            .ok_or_else(|| RimuruError::PluginNotFound(plugin_id.to_string()))?;
        Ok(sandbox.get_violations().await)
    }

    pub async fn get_all_violations(&self) -> Vec<(String, Vec<AccessViolation>)> {
        let sandboxes = self.sandboxes.read().await;
        let mut all_violations = Vec::new();

        for (plugin_id, sandbox) in sandboxes.iter() {
            let violations = sandbox.get_violations().await;
            if !violations.is_empty() {
                all_violations.push((plugin_id.clone(), violations));
            }
        }

        all_violations
    }
}

impl Default for SandboxManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_permission_implies() {
        assert!(Permission::All.implies(&Permission::Filesystem));
        assert!(Permission::All.implies(&Permission::Network));
        assert!(Permission::Filesystem.implies(&Permission::FilesystemRead));
        assert!(Permission::Filesystem.implies(&Permission::FilesystemWrite));
        assert!(!Permission::FilesystemRead.implies(&Permission::FilesystemWrite));
        assert!(Permission::Network.implies(&Permission::NetworkOutbound));
        assert!(Permission::Network.implies(&Permission::NetworkInbound));
    }

    #[test]
    fn test_sandbox_config_builder() {
        let config = SandboxConfig::new(PathBuf::from("/data"))
            .with_permission(Permission::FilesystemRead)
            .with_permission(Permission::NetworkOutbound)
            .with_allowed_path(PathBuf::from("/tmp"))
            .with_allowed_host("api.example.com".to_string());

        assert!(config
            .granted_permissions
            .contains(&Permission::FilesystemRead));
        assert!(config
            .granted_permissions
            .contains(&Permission::NetworkOutbound));
        assert!(config.allowed_paths.contains(&PathBuf::from("/tmp")));
        assert!(config
            .allowed_hosts
            .contains(&"api.example.com".to_string()));
    }

    #[test]
    fn test_sandbox_permission_check() {
        let config =
            SandboxConfig::new(PathBuf::from("/data")).with_permission(Permission::FilesystemRead);

        let sandbox = Sandbox::new("test-plugin".to_string(), config);

        assert!(sandbox.check_permission(Permission::FilesystemRead).is_ok());
        assert!(sandbox
            .check_permission(Permission::FilesystemWrite)
            .is_err());
    }

    #[test]
    fn test_sandbox_implied_permission() {
        let config =
            SandboxConfig::new(PathBuf::from("/data")).with_permission(Permission::Filesystem);

        let sandbox = Sandbox::new("test-plugin".to_string(), config);

        assert!(sandbox.check_permission(Permission::Filesystem).is_ok());
        assert!(sandbox.check_permission(Permission::FilesystemRead).is_ok());
        assert!(sandbox
            .check_permission(Permission::FilesystemWrite)
            .is_ok());
    }

    #[tokio::test]
    async fn test_sandbox_path_access() {
        let temp_dir = TempDir::new().unwrap();
        let allowed_path = temp_dir.path().to_path_buf();

        let config = SandboxConfig::new(allowed_path.clone())
            .with_permission(Permission::Filesystem)
            .with_allowed_path(allowed_path.clone());

        let sandbox = Sandbox::new("test-plugin".to_string(), config);

        let allowed_file = allowed_path.join("test.txt");
        assert!(sandbox.check_path_access(&allowed_file, false).is_ok());
        assert!(sandbox.check_path_access(&allowed_file, true).is_ok());

        let denied_path = PathBuf::from("/etc/passwd");
        assert!(sandbox.check_path_access(&denied_path, false).is_err());
    }

    #[test]
    fn test_sandbox_network_access() {
        let config = SandboxConfig::new(PathBuf::from("/data"))
            .with_permission(Permission::Network)
            .with_allowed_host("api.example.com".to_string())
            .with_denied_host("malicious.com".to_string());

        let sandbox = Sandbox::new("test-plugin".to_string(), config);

        assert!(sandbox
            .check_network_access("api.example.com", true)
            .is_ok());
        assert!(sandbox.check_network_access("malicious.com", true).is_err());
        assert!(sandbox.check_network_access("other.com", true).is_err());
    }

    #[tokio::test]
    async fn test_resource_limits() {
        let config =
            SandboxConfig::new(PathBuf::from("/data")).with_resource_limits(ResourceLimits {
                max_memory_mb: Some(100),
                max_open_files: Some(5),
                ..Default::default()
            });

        let sandbox = Sandbox::new("test-plugin".to_string(), config);

        sandbox.update_memory_usage(50).await;
        assert!(sandbox.check_resource_limits().await.is_ok());

        sandbox.update_memory_usage(150).await;
        assert!(sandbox.check_resource_limits().await.is_err());
    }

    #[tokio::test]
    async fn test_sandbox_manager() {
        let manager = SandboxManager::new();

        let config =
            SandboxConfig::new(PathBuf::from("/data")).with_permission(Permission::FilesystemRead);

        manager
            .create_sandbox("plugin-1", Some(config))
            .await
            .unwrap();

        assert!(manager
            .check_permission("plugin-1", Permission::FilesystemRead)
            .await
            .is_ok());

        assert!(manager
            .check_permission("plugin-1", Permission::FilesystemWrite)
            .await
            .is_err());

        manager.remove_sandbox("plugin-1").await.unwrap();

        assert!(manager
            .check_permission("plugin-1", Permission::FilesystemRead)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_violation_recording() {
        let config = SandboxConfig::new(PathBuf::from("/data"));
        let sandbox = Sandbox::new("test-plugin".to_string(), config);

        let violation = AccessViolation::new(
            Permission::Filesystem,
            "/etc/passwd".to_string(),
            "Attempted to access restricted path".to_string(),
        );

        sandbox.record_violation(violation).await;

        let violations = sandbox.get_violations().await;
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].permission, Permission::Filesystem);

        sandbox.clear_violations().await;
        let violations = sandbox.get_violations().await;
        assert!(violations.is_empty());
    }
}
