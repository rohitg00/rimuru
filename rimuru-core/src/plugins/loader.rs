use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::error::{RimuruError, RimuruResult};

use super::manifest::PluginManifest;
use super::traits::DynPlugin;
use super::types::{
    PluginConfig, PluginContext, PluginEvent, PluginInfo, PluginState, PluginStatus,
};

pub struct LoadedPlugin {
    pub manifest: PluginManifest,
    pub state: PluginState,
    pub plugin_dir: PathBuf,
    pub instance: Option<DynPlugin>,
}

impl LoadedPlugin {
    pub fn new(manifest: PluginManifest, plugin_dir: PathBuf) -> Self {
        let info = PluginInfo {
            name: manifest.plugin.name.clone(),
            version: manifest.plugin.version.clone(),
            author: manifest.plugin.author.clone(),
            description: manifest.plugin.description.clone(),
            capabilities: manifest.capabilities(),
            homepage: manifest.plugin.homepage.clone(),
            repository: manifest.plugin.repository.clone(),
            license: manifest.plugin.license.clone(),
        };

        Self {
            manifest,
            state: PluginState::new(info),
            plugin_dir,
            instance: None,
        }
    }

    pub fn plugin_id(&self) -> String {
        self.manifest.plugin_id()
    }

    pub fn is_loaded(&self) -> bool {
        self.state.status == PluginStatus::Loaded || self.state.status == PluginStatus::Enabled
    }

    pub fn is_enabled(&self) -> bool {
        self.state.status == PluginStatus::Enabled
    }
}

pub struct PluginLoader {
    plugins_dir: PathBuf,
    loaded_plugins: Arc<RwLock<HashMap<String, LoadedPlugin>>>,
    event_handlers: Arc<RwLock<Vec<Box<dyn Fn(PluginEvent) + Send + Sync>>>>,
}

impl PluginLoader {
    pub fn new(plugins_dir: PathBuf) -> Self {
        Self {
            plugins_dir,
            loaded_plugins: Arc::new(RwLock::new(HashMap::new())),
            event_handlers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn with_default_dir() -> RimuruResult<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| RimuruError::plugin("Could not determine home directory"))?;
        let plugins_dir = home.join(".rimuru").join("plugins");
        Ok(Self::new(plugins_dir))
    }

    pub fn plugins_dir(&self) -> &Path {
        &self.plugins_dir
    }

    pub async fn ensure_plugins_dir(&self) -> RimuruResult<()> {
        if !self.plugins_dir.exists() {
            tokio::fs::create_dir_all(&self.plugins_dir)
                .await
                .map_err(|e| {
                    RimuruError::plugin(format!(
                        "Failed to create plugins directory {:?}: {}",
                        self.plugins_dir, e
                    ))
                })?;
            info!("Created plugins directory: {:?}", self.plugins_dir);
        }
        Ok(())
    }

    pub async fn discover_plugins(&self) -> RimuruResult<Vec<PathBuf>> {
        self.ensure_plugins_dir().await?;

        let mut discovered = Vec::new();
        let mut entries = tokio::fs::read_dir(&self.plugins_dir)
            .await
            .map_err(|e| RimuruError::plugin(format!("Failed to read plugins directory: {}", e)))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| RimuruError::plugin(format!("Failed to read directory entry: {}", e)))?
        {
            let path = entry.path();
            if path.is_dir() {
                let manifest_path = path.join(PluginManifest::FILENAME);
                if manifest_path.exists() {
                    discovered.push(path);
                } else {
                    debug!("Skipping directory without manifest: {:?}", path);
                }
            }
        }

        info!("Discovered {} plugins", discovered.len());
        Ok(discovered)
    }

    pub async fn load_manifest(&self, plugin_dir: &Path) -> RimuruResult<PluginManifest> {
        let manifest_path = plugin_dir.join(PluginManifest::FILENAME);
        let manifest = PluginManifest::load_from_file(&manifest_path).await?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub async fn load_plugin(&self, plugin_dir: &Path) -> RimuruResult<String> {
        let manifest = self.load_manifest(plugin_dir).await?;
        let plugin_id = manifest.plugin_id();

        {
            let plugins = self.loaded_plugins.read().await;
            if plugins.contains_key(&plugin_id) {
                return Err(RimuruError::PluginAlreadyLoaded(plugin_id));
            }
        }

        let mut loaded = LoadedPlugin::new(manifest, plugin_dir.to_path_buf());
        loaded.state.status = PluginStatus::Loaded;
        loaded.state.loaded_at = Some(chrono::Utc::now());

        let event = PluginEvent::loaded(&plugin_id);

        {
            let mut plugins = self.loaded_plugins.write().await;
            plugins.insert(plugin_id.clone(), loaded);
        }

        self.emit_event(event).await;
        info!("Loaded plugin: {}", plugin_id);

        Ok(plugin_id)
    }

    pub async fn load_all_plugins(&self) -> RimuruResult<Vec<String>> {
        let discovered = self.discover_plugins().await?;
        let mut loaded_ids = Vec::new();

        for plugin_dir in discovered {
            match self.load_plugin(&plugin_dir).await {
                Ok(id) => {
                    loaded_ids.push(id);
                }
                Err(e) => {
                    warn!("Failed to load plugin from {:?}: {}", plugin_dir, e);
                }
            }
        }

        Ok(loaded_ids)
    }

    pub async fn unload_plugin(&self, plugin_id: &str) -> RimuruResult<()> {
        let mut plugins = self.loaded_plugins.write().await;

        let plugin = plugins
            .get_mut(plugin_id)
            .ok_or_else(|| RimuruError::PluginNotFound(plugin_id.to_string()))?;

        if let Some(ref mut instance) = plugin.instance {
            if let Err(e) = instance.shutdown().await {
                error!("Error during plugin shutdown: {}", e);
            }
        }

        plugins.remove(plugin_id);
        drop(plugins);

        let event = PluginEvent::Unloaded {
            plugin_id: plugin_id.to_string(),
            timestamp: chrono::Utc::now(),
        };
        self.emit_event(event).await;

        info!("Unloaded plugin: {}", plugin_id);
        Ok(())
    }

    pub async fn get_plugin_state(&self, plugin_id: &str) -> RimuruResult<PluginState> {
        let plugins = self.loaded_plugins.read().await;
        let plugin = plugins
            .get(plugin_id)
            .ok_or_else(|| RimuruError::PluginNotFound(plugin_id.to_string()))?;
        Ok(plugin.state.clone())
    }

    pub async fn get_all_plugins(&self) -> Vec<PluginState> {
        let plugins = self.loaded_plugins.read().await;
        plugins.values().map(|p| p.state.clone()).collect()
    }

    pub async fn get_loaded_plugin_ids(&self) -> Vec<String> {
        let plugins = self.loaded_plugins.read().await;
        plugins.keys().cloned().collect()
    }

    pub async fn get_plugin_manifest(&self, plugin_id: &str) -> RimuruResult<PluginManifest> {
        let plugins = self.loaded_plugins.read().await;
        let plugin = plugins
            .get(plugin_id)
            .ok_or_else(|| RimuruError::PluginNotFound(plugin_id.to_string()))?;
        Ok(plugin.manifest.clone())
    }

    pub async fn get_plugin_dir(&self, plugin_id: &str) -> RimuruResult<PathBuf> {
        let plugins = self.loaded_plugins.read().await;
        let plugin = plugins
            .get(plugin_id)
            .ok_or_else(|| RimuruError::PluginNotFound(plugin_id.to_string()))?;
        Ok(plugin.plugin_dir.clone())
    }

    pub async fn enable_plugin(&self, plugin_id: &str) -> RimuruResult<()> {
        let mut plugins = self.loaded_plugins.write().await;
        let plugin = plugins
            .get_mut(plugin_id)
            .ok_or_else(|| RimuruError::PluginNotFound(plugin_id.to_string()))?;

        if plugin.state.status == PluginStatus::Enabled {
            return Ok(());
        }

        if plugin.state.status != PluginStatus::Loaded
            && plugin.state.status != PluginStatus::Disabled
        {
            return Err(RimuruError::plugin(format!(
                "Cannot enable plugin in state {:?}",
                plugin.state.status
            )));
        }

        plugin.state.status = PluginStatus::Enabled;
        plugin.state.config.enabled = true;

        drop(plugins);

        let event = PluginEvent::enabled(plugin_id);
        self.emit_event(event).await;

        info!("Enabled plugin: {}", plugin_id);
        Ok(())
    }

    pub async fn disable_plugin(&self, plugin_id: &str) -> RimuruResult<()> {
        let mut plugins = self.loaded_plugins.write().await;
        let plugin = plugins
            .get_mut(plugin_id)
            .ok_or_else(|| RimuruError::PluginNotFound(plugin_id.to_string()))?;

        if plugin.state.status == PluginStatus::Disabled {
            return Ok(());
        }

        if plugin.state.status != PluginStatus::Enabled {
            return Err(RimuruError::plugin(format!(
                "Cannot disable plugin in state {:?}",
                plugin.state.status
            )));
        }

        plugin.state.status = PluginStatus::Disabled;
        plugin.state.config.enabled = false;

        drop(plugins);

        let event = PluginEvent::disabled(plugin_id);
        self.emit_event(event).await;

        info!("Disabled plugin: {}", plugin_id);
        Ok(())
    }

    pub async fn configure_plugin(
        &self,
        plugin_id: &str,
        config: PluginConfig,
    ) -> RimuruResult<()> {
        let mut plugins = self.loaded_plugins.write().await;
        let plugin = plugins
            .get_mut(plugin_id)
            .ok_or_else(|| RimuruError::PluginNotFound(plugin_id.to_string()))?;

        plugin.state.config = config.clone();

        if let Some(ref mut instance) = plugin.instance {
            instance.configure(config)?;
        }

        drop(plugins);

        let event = PluginEvent::ConfigChanged {
            plugin_id: plugin_id.to_string(),
            timestamp: chrono::Utc::now(),
        };
        self.emit_event(event).await;

        info!("Configured plugin: {}", plugin_id);
        Ok(())
    }

    pub async fn set_plugin_error(&self, plugin_id: &str, error: String) -> RimuruResult<()> {
        let mut plugins = self.loaded_plugins.write().await;
        let plugin = plugins
            .get_mut(plugin_id)
            .ok_or_else(|| RimuruError::PluginNotFound(plugin_id.to_string()))?;

        plugin.state.status = PluginStatus::Error;
        plugin.state.error = Some(error.clone());

        drop(plugins);

        let event = PluginEvent::error(plugin_id, error);
        self.emit_event(event).await;

        Ok(())
    }

    pub async fn add_event_handler<F>(&self, handler: F)
    where
        F: Fn(PluginEvent) + Send + Sync + 'static,
    {
        let mut handlers = self.event_handlers.write().await;
        handlers.push(Box::new(handler));
    }

    async fn emit_event(&self, event: PluginEvent) {
        let handlers = self.event_handlers.read().await;
        for handler in handlers.iter() {
            handler(event.clone());
        }
    }

    pub async fn install_plugin(&self, source: &str) -> RimuruResult<String> {
        let source_path = PathBuf::from(source);

        if source_path.exists() && source_path.is_dir() {
            return self.install_from_local(&source_path).await;
        }

        if source.starts_with("http://") || source.starts_with("https://") {
            return Err(RimuruError::NotSupported(
                "URL-based plugin installation not yet implemented".to_string(),
            ));
        }

        Err(RimuruError::plugin(format!(
            "Invalid plugin source: {}. Expected local path or URL",
            source
        )))
    }

    async fn install_from_local(&self, source_path: &Path) -> RimuruResult<String> {
        let manifest = self.load_manifest(source_path).await?;
        let plugin_name = manifest.plugin.name.clone();

        let target_dir = self.plugins_dir.join(&plugin_name);

        if target_dir.exists() {
            return Err(RimuruError::AlreadyExists(format!(
                "Plugin directory already exists: {:?}",
                target_dir
            )));
        }

        copy_dir_recursive(source_path, &target_dir).await?;

        let plugin_id = self.load_plugin(&target_dir).await?;
        info!("Installed plugin from {:?}: {}", source_path, plugin_id);

        Ok(plugin_id)
    }

    pub async fn uninstall_plugin(&self, plugin_id: &str) -> RimuruResult<()> {
        let plugin_dir = self.get_plugin_dir(plugin_id).await?;

        self.unload_plugin(plugin_id).await?;

        tokio::fs::remove_dir_all(&plugin_dir).await.map_err(|e| {
            RimuruError::plugin(format!("Failed to remove plugin directory: {}", e))
        })?;

        info!("Uninstalled plugin: {}", plugin_id);
        Ok(())
    }

    pub async fn create_plugin_context(&self, plugin_id: &str) -> RimuruResult<PluginContext> {
        let plugins = self.loaded_plugins.read().await;
        let plugin = plugins
            .get(plugin_id)
            .ok_or_else(|| RimuruError::PluginNotFound(plugin_id.to_string()))?;

        let data_dir = plugin.plugin_dir.join("data");
        if !data_dir.exists() {
            tokio::fs::create_dir_all(&data_dir).await.map_err(|e| {
                RimuruError::plugin(format!("Failed to create plugin data directory: {}", e))
            })?;
        }

        Ok(PluginContext::new(plugin_id, data_dir).with_config(plugin.state.config.clone()))
    }
}

async fn copy_dir_recursive(src: &Path, dst: &Path) -> RimuruResult<()> {
    tokio::fs::create_dir_all(dst)
        .await
        .map_err(|e| RimuruError::plugin(format!("Failed to create directory {:?}: {}", dst, e)))?;

    let mut entries = tokio::fs::read_dir(src)
        .await
        .map_err(|e| RimuruError::plugin(format!("Failed to read directory {:?}: {}", src, e)))?;

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| RimuruError::plugin(format!("Failed to read directory entry: {}", e)))?
    {
        let entry_path = entry.path();
        let file_name = entry.file_name();
        let dest_path = dst.join(&file_name);

        if entry_path.is_dir() {
            Box::pin(copy_dir_recursive(&entry_path, &dest_path)).await?;
        } else {
            tokio::fs::copy(&entry_path, &dest_path)
                .await
                .map_err(|e| {
                    RimuruError::plugin(format!(
                        "Failed to copy file {:?} to {:?}: {}",
                        entry_path, dest_path, e
                    ))
                })?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_plugin(dir: &Path, name: &str) -> PathBuf {
        let plugin_dir = dir.join(name);
        tokio::fs::create_dir_all(&plugin_dir).await.unwrap();

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
"#,
            name
        );

        tokio::fs::write(plugin_dir.join(PluginManifest::FILENAME), manifest_content)
            .await
            .unwrap();

        plugin_dir
    }

    #[tokio::test]
    async fn test_discover_plugins() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path().to_path_buf();

        create_test_plugin(&plugins_dir, "test-plugin-1").await;
        create_test_plugin(&plugins_dir, "test-plugin-2").await;

        let loader = PluginLoader::new(plugins_dir);
        let discovered = loader.discover_plugins().await.unwrap();

        assert_eq!(discovered.len(), 2);
    }

    #[tokio::test]
    async fn test_load_plugin() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path().to_path_buf();

        let plugin_dir = create_test_plugin(&plugins_dir, "test-plugin").await;

        let loader = PluginLoader::new(plugins_dir);
        let plugin_id = loader.load_plugin(&plugin_dir).await.unwrap();

        assert_eq!(plugin_id, "test-plugin@1.0.0");

        let state = loader.get_plugin_state(&plugin_id).await.unwrap();
        assert_eq!(state.status, PluginStatus::Loaded);
    }

    #[tokio::test]
    async fn test_load_all_plugins() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path().to_path_buf();

        create_test_plugin(&plugins_dir, "plugin-a").await;
        create_test_plugin(&plugins_dir, "plugin-b").await;

        let loader = PluginLoader::new(plugins_dir);
        let loaded = loader.load_all_plugins().await.unwrap();

        assert_eq!(loaded.len(), 2);
    }

    #[tokio::test]
    async fn test_enable_disable_plugin() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path().to_path_buf();

        let plugin_dir = create_test_plugin(&plugins_dir, "test-plugin").await;

        let loader = PluginLoader::new(plugins_dir);
        let plugin_id = loader.load_plugin(&plugin_dir).await.unwrap();

        loader.enable_plugin(&plugin_id).await.unwrap();
        let state = loader.get_plugin_state(&plugin_id).await.unwrap();
        assert_eq!(state.status, PluginStatus::Enabled);

        loader.disable_plugin(&plugin_id).await.unwrap();
        let state = loader.get_plugin_state(&plugin_id).await.unwrap();
        assert_eq!(state.status, PluginStatus::Disabled);
    }

    #[tokio::test]
    async fn test_unload_plugin() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path().to_path_buf();

        let plugin_dir = create_test_plugin(&plugins_dir, "test-plugin").await;

        let loader = PluginLoader::new(plugins_dir);
        let plugin_id = loader.load_plugin(&plugin_dir).await.unwrap();

        loader.unload_plugin(&plugin_id).await.unwrap();

        let result = loader.get_plugin_state(&plugin_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_configure_plugin() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path().to_path_buf();

        let plugin_dir = create_test_plugin(&plugins_dir, "test-plugin").await;

        let loader = PluginLoader::new(plugins_dir);
        let plugin_id = loader.load_plugin(&plugin_dir).await.unwrap();

        let config = PluginConfig::new().with_setting("key", "value");
        loader.configure_plugin(&plugin_id, config).await.unwrap();

        let state = loader.get_plugin_state(&plugin_id).await.unwrap();
        let value: Option<String> = state.config.get_setting("key");
        assert_eq!(value, Some("value".to_string()));
    }

    #[tokio::test]
    async fn test_install_from_local() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        let plugins_dir = temp_dir.path().join("plugins");

        tokio::fs::create_dir_all(&source_dir).await.unwrap();
        tokio::fs::create_dir_all(&plugins_dir).await.unwrap();

        let manifest_content = r#"
[plugin]
name = "installable-plugin"
version = "1.0.0"
author = "Test"
description = "Test plugin"

[capabilities.exporter]
format = "json"
file_extension = "json"
supports_sessions = true
supports_costs = true
"#;

        tokio::fs::write(source_dir.join(PluginManifest::FILENAME), manifest_content)
            .await
            .unwrap();

        let loader = PluginLoader::new(plugins_dir.clone());
        let plugin_id = loader
            .install_plugin(source_dir.to_str().unwrap())
            .await
            .unwrap();

        assert_eq!(plugin_id, "installable-plugin@1.0.0");
        assert!(plugins_dir.join("installable-plugin").exists());
    }

    #[tokio::test]
    async fn test_duplicate_load_fails() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path().to_path_buf();

        let plugin_dir = create_test_plugin(&plugins_dir, "test-plugin").await;

        let loader = PluginLoader::new(plugins_dir);
        loader.load_plugin(&plugin_dir).await.unwrap();

        let result = loader.load_plugin(&plugin_dir).await;
        assert!(matches!(result, Err(RimuruError::PluginAlreadyLoaded(_))));
    }
}
