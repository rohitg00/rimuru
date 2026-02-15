pub mod builtin;
mod loader;
mod manifest;
mod registry;
mod sandbox;
mod traits;
mod types;

pub use builtin::{
    create_builtin_exporter, create_builtin_notifier, is_builtin_plugin, list_builtin_plugins,
    CsvExporterConfig, CsvExporterPlugin, DiscordNotifierConfig, DiscordNotifierPlugin, HttpMethod,
    JsonExporterConfig, JsonExporterPlugin, LineEnding, SlackNotifierConfig, SlackNotifierPlugin,
    WebhookNotifierConfig, WebhookNotifierPlugin,
};
pub use loader::{LoadedPlugin, PluginLoader};

pub use manifest::{
    create_example_manifest, AgentCapability, CapabilitiesSection, ConfigSection,
    ExporterCapability, HookRegistration, NotifierCapability, PluginManifest, PluginMetadata,
    ViewCapability,
};

pub use registry::{
    CapabilityProvider, DependencyResolution, PluginConflict, PluginRegistry, ResolvedDependency,
};

pub use sandbox::{
    AccessViolation, Permission, ResourceLimits, ResourceUsage, Sandbox, SandboxConfig,
    SandboxManager,
};

pub use traits::{
    AgentPlugin, DynAgentPlugin, DynExporterPlugin, DynNotifierPlugin, DynPlugin, DynViewPlugin,
    ExportData, ExportOptions, ExporterPlugin, Notification, NotificationLevel, NotifierPlugin,
    Plugin, PluginFactory, SessionCallback, ViewAction, ViewContext, ViewInput, ViewOutput,
    ViewPlugin, WidgetData,
};

pub use types::{
    PluginCapability, PluginConfig, PluginContext, PluginDependency, PluginEvent, PluginInfo,
    PluginPermission, PluginState, PluginStatus, PluginType,
};
