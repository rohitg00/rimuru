use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::info;

use crate::error::{RimuruError, RimuruResult};

use super::loader::PluginLoader;
use super::traits::{DynAgentPlugin, DynExporterPlugin, DynNotifierPlugin, DynViewPlugin};
use super::types::{PluginCapability, PluginDependency, PluginState, PluginStatus};

pub struct CapabilityProvider {
    pub plugin_id: String,
    pub capability: PluginCapability,
}

pub struct PluginRegistry {
    loader: Arc<PluginLoader>,
    agent_plugins: Arc<RwLock<HashMap<String, DynAgentPlugin>>>,
    exporter_plugins: Arc<RwLock<HashMap<String, DynExporterPlugin>>>,
    notifier_plugins: Arc<RwLock<HashMap<String, DynNotifierPlugin>>>,
    view_plugins: Arc<RwLock<HashMap<String, DynViewPlugin>>>,
    capability_providers: Arc<RwLock<HashMap<PluginCapability, Vec<String>>>>,
}

impl PluginRegistry {
    pub fn new(loader: Arc<PluginLoader>) -> Self {
        Self {
            loader,
            agent_plugins: Arc::new(RwLock::new(HashMap::new())),
            exporter_plugins: Arc::new(RwLock::new(HashMap::new())),
            notifier_plugins: Arc::new(RwLock::new(HashMap::new())),
            view_plugins: Arc::new(RwLock::new(HashMap::new())),
            capability_providers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn loader(&self) -> &Arc<PluginLoader> {
        &self.loader
    }

    pub async fn register_agent_plugin(
        &self,
        plugin_id: &str,
        plugin: DynAgentPlugin,
    ) -> RimuruResult<()> {
        let mut plugins = self.agent_plugins.write().await;

        if plugins.contains_key(plugin_id) {
            return Err(RimuruError::PluginAlreadyLoaded(plugin_id.to_string()));
        }

        plugins.insert(plugin_id.to_string(), plugin);

        let mut providers = self.capability_providers.write().await;
        providers
            .entry(PluginCapability::Agent)
            .or_default()
            .push(plugin_id.to_string());

        info!("Registered agent plugin: {}", plugin_id);
        Ok(())
    }

    pub async fn register_exporter_plugin(
        &self,
        plugin_id: &str,
        plugin: DynExporterPlugin,
    ) -> RimuruResult<()> {
        let mut plugins = self.exporter_plugins.write().await;

        if plugins.contains_key(plugin_id) {
            return Err(RimuruError::PluginAlreadyLoaded(plugin_id.to_string()));
        }

        plugins.insert(plugin_id.to_string(), plugin);

        let mut providers = self.capability_providers.write().await;
        providers
            .entry(PluginCapability::Exporter)
            .or_default()
            .push(plugin_id.to_string());

        info!("Registered exporter plugin: {}", plugin_id);
        Ok(())
    }

    pub async fn register_notifier_plugin(
        &self,
        plugin_id: &str,
        plugin: DynNotifierPlugin,
    ) -> RimuruResult<()> {
        let mut plugins = self.notifier_plugins.write().await;

        if plugins.contains_key(plugin_id) {
            return Err(RimuruError::PluginAlreadyLoaded(plugin_id.to_string()));
        }

        plugins.insert(plugin_id.to_string(), plugin);

        let mut providers = self.capability_providers.write().await;
        providers
            .entry(PluginCapability::Notifier)
            .or_default()
            .push(plugin_id.to_string());

        info!("Registered notifier plugin: {}", plugin_id);
        Ok(())
    }

    pub async fn register_view_plugin(
        &self,
        plugin_id: &str,
        plugin: DynViewPlugin,
    ) -> RimuruResult<()> {
        let mut plugins = self.view_plugins.write().await;

        if plugins.contains_key(plugin_id) {
            return Err(RimuruError::PluginAlreadyLoaded(plugin_id.to_string()));
        }

        plugins.insert(plugin_id.to_string(), plugin);

        let mut providers = self.capability_providers.write().await;
        providers
            .entry(PluginCapability::View)
            .or_default()
            .push(plugin_id.to_string());

        info!("Registered view plugin: {}", plugin_id);
        Ok(())
    }

    pub async fn unregister_plugin(&self, plugin_id: &str) -> RimuruResult<()> {
        let mut removed = false;

        {
            let mut agents = self.agent_plugins.write().await;
            if agents.remove(plugin_id).is_some() {
                removed = true;
            }
        }

        {
            let mut exporters = self.exporter_plugins.write().await;
            if exporters.remove(plugin_id).is_some() {
                removed = true;
            }
        }

        {
            let mut notifiers = self.notifier_plugins.write().await;
            if notifiers.remove(plugin_id).is_some() {
                removed = true;
            }
        }

        {
            let mut views = self.view_plugins.write().await;
            if views.remove(plugin_id).is_some() {
                removed = true;
            }
        }

        {
            let mut providers = self.capability_providers.write().await;
            for plugins in providers.values_mut() {
                plugins.retain(|id| id != plugin_id);
            }
        }

        if removed {
            info!("Unregistered plugin: {}", plugin_id);
            Ok(())
        } else {
            Err(RimuruError::PluginNotFound(plugin_id.to_string()))
        }
    }

    pub async fn has_agent_plugin(&self, plugin_id: &str) -> bool {
        let plugins = self.agent_plugins.read().await;
        plugins.contains_key(plugin_id)
    }

    pub async fn has_exporter_plugin(&self, plugin_id: &str) -> bool {
        let plugins = self.exporter_plugins.read().await;
        plugins.contains_key(plugin_id)
    }

    pub async fn has_notifier_plugin(&self, plugin_id: &str) -> bool {
        let plugins = self.notifier_plugins.read().await;
        plugins.contains_key(plugin_id)
    }

    pub async fn has_view_plugin(&self, plugin_id: &str) -> bool {
        let plugins = self.view_plugins.read().await;
        plugins.contains_key(plugin_id)
    }

    pub async fn get_agent_plugin_ids(&self) -> Vec<String> {
        let plugins = self.agent_plugins.read().await;
        plugins.keys().cloned().collect()
    }

    pub async fn get_exporter_plugin_ids(&self) -> Vec<String> {
        let plugins = self.exporter_plugins.read().await;
        plugins.keys().cloned().collect()
    }

    pub async fn get_notifier_plugin_ids(&self) -> Vec<String> {
        let plugins = self.notifier_plugins.read().await;
        plugins.keys().cloned().collect()
    }

    pub async fn get_view_plugin_ids(&self) -> Vec<String> {
        let plugins = self.view_plugins.read().await;
        plugins.keys().cloned().collect()
    }

    pub async fn get_providers_for_capability(&self, capability: PluginCapability) -> Vec<String> {
        let providers = self.capability_providers.read().await;
        providers.get(&capability).cloned().unwrap_or_default()
    }

    pub async fn check_conflicts(&self) -> Vec<PluginConflict> {
        let providers = self.capability_providers.read().await;
        let mut conflicts = Vec::new();

        for (capability, plugin_ids) in providers.iter() {
            if plugin_ids.len() > 1 {
                conflicts.push(PluginConflict {
                    capability: *capability,
                    plugins: plugin_ids.clone(),
                });
            }
        }

        conflicts
    }

    pub async fn resolve_dependencies(
        &self,
        plugin_id: &str,
    ) -> RimuruResult<DependencyResolution> {
        let manifest = self.loader.get_plugin_manifest(plugin_id).await?;
        let mut resolution = DependencyResolution::new(plugin_id.to_string());

        for dep in &manifest.dependencies {
            let dep_resolved = self.find_dependency(dep).await;
            match dep_resolved {
                Some(resolved_id) => {
                    resolution.resolved.push(ResolvedDependency {
                        name: dep.name.clone(),
                        version_requirement: dep.version_requirement.clone(),
                        resolved_id,
                    });
                }
                None => {
                    if dep.optional {
                        resolution.optional_missing.push(dep.clone());
                    } else {
                        resolution.missing.push(dep.clone());
                    }
                }
            }
        }

        Ok(resolution)
    }

    async fn find_dependency(&self, dep: &PluginDependency) -> Option<String> {
        let all_plugins = self.loader.get_loaded_plugin_ids().await;

        for plugin_id in all_plugins {
            if let Ok(manifest) = self.loader.get_plugin_manifest(&plugin_id).await {
                if manifest.plugin.name == dep.name
                    && self.version_satisfies(&manifest.plugin.version, &dep.version_requirement)
                {
                    return Some(plugin_id);
                }
            }
        }

        None
    }

    fn version_satisfies(&self, version: &str, requirement: &str) -> bool {
        if requirement == "*" {
            return true;
        }

        let req = requirement.trim();

        if req.starts_with(">=") {
            let min_version = req.trim_start_matches(">=").trim();
            return self.compare_versions(version, min_version) >= 0;
        }

        if req.starts_with("<=") {
            let max_version = req.trim_start_matches("<=").trim();
            return self.compare_versions(version, max_version) <= 0;
        }

        if req.starts_with('>') {
            let min_version = req.trim_start_matches('>').trim();
            return self.compare_versions(version, min_version) > 0;
        }

        if req.starts_with('<') {
            let max_version = req.trim_start_matches('<').trim();
            return self.compare_versions(version, max_version) < 0;
        }

        if req.starts_with('^') {
            let base_version = req.trim_start_matches('^').trim();
            let base_parts: Vec<u32> = base_version
                .split('.')
                .filter_map(|p| p.parse().ok())
                .collect();
            let version_parts: Vec<u32> =
                version.split('.').filter_map(|p| p.parse().ok()).collect();

            if version_parts.is_empty() || base_parts.is_empty() {
                return false;
            }

            if version_parts[0] != base_parts[0] {
                return false;
            }

            return self.compare_versions(version, base_version) >= 0;
        }

        version == req
    }

    fn compare_versions(&self, a: &str, b: &str) -> i32 {
        let a_parts: Vec<u32> = a.split('.').filter_map(|p| p.parse().ok()).collect();
        let b_parts: Vec<u32> = b.split('.').filter_map(|p| p.parse().ok()).collect();

        let max_len = a_parts.len().max(b_parts.len());

        for i in 0..max_len {
            let a_val = a_parts.get(i).copied().unwrap_or(0);
            let b_val = b_parts.get(i).copied().unwrap_or(0);

            if a_val > b_val {
                return 1;
            } else if a_val < b_val {
                return -1;
            }
        }

        0
    }

    pub async fn get_load_order(&self) -> RimuruResult<Vec<String>> {
        let all_plugins = self.loader.get_loaded_plugin_ids().await;
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        let mut in_degree: HashMap<String, usize> = HashMap::new();

        for plugin_id in &all_plugins {
            graph.entry(plugin_id.clone()).or_default();
            in_degree.entry(plugin_id.clone()).or_insert(0);

            let resolution = self.resolve_dependencies(plugin_id).await?;
            for resolved_dep in &resolution.resolved {
                graph
                    .entry(resolved_dep.resolved_id.clone())
                    .or_default()
                    .push(plugin_id.clone());
                *in_degree.entry(plugin_id.clone()).or_insert(0) += 1;
            }
        }

        let mut result = Vec::new();
        let mut queue: Vec<String> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(id, _)| id.clone())
            .collect();

        while let Some(plugin_id) = queue.pop() {
            result.push(plugin_id.clone());

            if let Some(dependents) = graph.get(&plugin_id) {
                for dependent in dependents {
                    if let Some(degree) = in_degree.get_mut(dependent) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push(dependent.clone());
                        }
                    }
                }
            }
        }

        if result.len() != all_plugins.len() {
            return Err(RimuruError::plugin("Circular dependency detected"));
        }

        Ok(result)
    }

    pub async fn enable_plugin(&self, plugin_id: &str) -> RimuruResult<()> {
        let resolution = self.resolve_dependencies(plugin_id).await?;

        if !resolution.missing.is_empty() {
            let missing_names: Vec<String> = resolution
                .missing
                .iter()
                .map(|d| format!("{} ({})", d.name, d.version_requirement))
                .collect();
            return Err(RimuruError::PluginDependencyError {
                plugin: plugin_id.to_string(),
                dependency: missing_names.join(", "),
            });
        }

        for dep in &resolution.resolved {
            let dep_state = self.loader.get_plugin_state(&dep.resolved_id).await?;
            if dep_state.status != PluginStatus::Enabled {
                self.loader.enable_plugin(&dep.resolved_id).await?;
            }
        }

        self.loader.enable_plugin(plugin_id).await
    }

    pub async fn disable_plugin(&self, plugin_id: &str) -> RimuruResult<()> {
        let all_plugins = self.loader.get_loaded_plugin_ids().await;

        for other_plugin_id in &all_plugins {
            if other_plugin_id == plugin_id {
                continue;
            }

            let other_state = self.loader.get_plugin_state(other_plugin_id).await?;
            if other_state.status != PluginStatus::Enabled {
                continue;
            }

            let resolution = self.resolve_dependencies(other_plugin_id).await?;
            for dep in &resolution.resolved {
                if dep.resolved_id == plugin_id {
                    return Err(RimuruError::plugin(format!(
                        "Cannot disable {}: {} depends on it",
                        plugin_id, other_plugin_id
                    )));
                }
            }
        }

        self.loader.disable_plugin(plugin_id).await
    }

    pub async fn get_all_plugin_states(&self) -> Vec<PluginState> {
        self.loader.get_all_plugins().await
    }

    pub async fn get_enabled_plugins(&self) -> Vec<PluginState> {
        self.loader
            .get_all_plugins()
            .await
            .into_iter()
            .filter(|p| p.status == PluginStatus::Enabled)
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct PluginConflict {
    pub capability: PluginCapability,
    pub plugins: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DependencyResolution {
    pub plugin_id: String,
    pub resolved: Vec<ResolvedDependency>,
    pub missing: Vec<PluginDependency>,
    pub optional_missing: Vec<PluginDependency>,
}

impl DependencyResolution {
    pub fn new(plugin_id: String) -> Self {
        Self {
            plugin_id,
            resolved: Vec::new(),
            missing: Vec::new(),
            optional_missing: Vec::new(),
        }
    }

    pub fn is_satisfied(&self) -> bool {
        self.missing.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedDependency {
    pub name: String,
    pub version_requirement: String,
    pub resolved_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn setup_test_loader() -> (TempDir, Arc<PluginLoader>) {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path().to_path_buf();
        let loader = Arc::new(PluginLoader::new(plugins_dir));
        loader.ensure_plugins_dir().await.unwrap();
        (temp_dir, loader)
    }

    async fn create_test_plugin(
        dir: &std::path::Path,
        name: &str,
        deps: &[(&str, &str)],
    ) -> std::path::PathBuf {
        let plugin_dir = dir.join(name);
        tokio::fs::create_dir_all(&plugin_dir).await.unwrap();

        let deps_toml = if deps.is_empty() {
            String::new()
        } else {
            let deps_entries: Vec<String> = deps
                .iter()
                .map(|(name, version)| {
                    format!(
                        r#"[[dependencies]]
name = "{}"
version_requirement = "{}"
optional = false"#,
                        name, version
                    )
                })
                .collect();
            deps_entries.join("\n\n")
        };

        let manifest_content = format!(
            r#"
[plugin]
name = "{}"
version = "1.0.0"
author = "Test"
description = "Test plugin"

[capabilities.exporter]
format = "json"
file_extension = "json"
supports_sessions = true
supports_costs = true

{}
"#,
            name, deps_toml
        );

        tokio::fs::write(
            plugin_dir.join(super::super::manifest::PluginManifest::FILENAME),
            manifest_content,
        )
        .await
        .unwrap();

        plugin_dir
    }

    #[tokio::test]
    async fn test_registry_creation() {
        let (_temp_dir, loader) = setup_test_loader().await;
        let _registry = PluginRegistry::new(loader);
    }

    #[tokio::test]
    async fn test_capability_providers() {
        let (temp_dir, loader) = setup_test_loader().await;
        let registry = PluginRegistry::new(loader.clone());

        let plugin_dir = create_test_plugin(
            temp_dir.path().join("plugins").as_path(),
            "test-exporter",
            &[],
        )
        .await;
        loader.load_plugin(&plugin_dir).await.unwrap();

        let providers = registry
            .get_providers_for_capability(PluginCapability::Exporter)
            .await;
        assert!(providers.is_empty());
    }

    #[tokio::test]
    async fn test_version_comparison() {
        let (_temp_dir, loader) = setup_test_loader().await;
        let registry = PluginRegistry::new(loader);

        assert!(registry.version_satisfies("1.0.0", "*"));
        assert!(registry.version_satisfies("1.0.0", "1.0.0"));
        assert!(registry.version_satisfies("1.1.0", ">=1.0.0"));
        assert!(!registry.version_satisfies("0.9.0", ">=1.0.0"));
        assert!(registry.version_satisfies("1.0.0", "^1.0.0"));
        assert!(registry.version_satisfies("1.9.9", "^1.0.0"));
        assert!(!registry.version_satisfies("2.0.0", "^1.0.0"));
    }

    #[tokio::test]
    async fn test_dependency_resolution() {
        let (temp_dir, loader) = setup_test_loader().await;
        let plugins_dir = temp_dir.path().join("plugins");
        tokio::fs::create_dir_all(&plugins_dir).await.unwrap();

        let loader = Arc::new(PluginLoader::new(plugins_dir.clone()));
        loader.ensure_plugins_dir().await.unwrap();

        let plugin_a_dir = create_test_plugin(&plugins_dir, "plugin-a", &[]).await;
        let plugin_b_dir =
            create_test_plugin(&plugins_dir, "plugin-b", &[("plugin-a", ">=1.0.0")]).await;

        loader.load_plugin(&plugin_a_dir).await.unwrap();
        loader.load_plugin(&plugin_b_dir).await.unwrap();

        let registry = PluginRegistry::new(loader);
        let resolution = registry
            .resolve_dependencies("plugin-b@1.0.0")
            .await
            .unwrap();

        assert!(resolution.is_satisfied());
        assert_eq!(resolution.resolved.len(), 1);
        assert_eq!(resolution.resolved[0].name, "plugin-a");
    }

    #[tokio::test]
    async fn test_load_order() {
        let (temp_dir, loader) = setup_test_loader().await;
        let plugins_dir = temp_dir.path().join("plugins");
        tokio::fs::create_dir_all(&plugins_dir).await.unwrap();

        let loader = Arc::new(PluginLoader::new(plugins_dir.clone()));
        loader.ensure_plugins_dir().await.unwrap();

        let plugin_a_dir = create_test_plugin(&plugins_dir, "plugin-a", &[]).await;
        let plugin_b_dir =
            create_test_plugin(&plugins_dir, "plugin-b", &[("plugin-a", ">=1.0.0")]).await;
        let plugin_c_dir =
            create_test_plugin(&plugins_dir, "plugin-c", &[("plugin-b", ">=1.0.0")]).await;

        loader.load_plugin(&plugin_a_dir).await.unwrap();
        loader.load_plugin(&plugin_b_dir).await.unwrap();
        loader.load_plugin(&plugin_c_dir).await.unwrap();

        let registry = PluginRegistry::new(loader);
        let load_order = registry.get_load_order().await.unwrap();

        let pos_a = load_order
            .iter()
            .position(|id| id.starts_with("plugin-a"))
            .unwrap();
        let pos_b = load_order
            .iter()
            .position(|id| id.starts_with("plugin-b"))
            .unwrap();
        let pos_c = load_order
            .iter()
            .position(|id| id.starts_with("plugin-c"))
            .unwrap();

        assert!(pos_a < pos_b);
        assert!(pos_b < pos_c);
    }
}
