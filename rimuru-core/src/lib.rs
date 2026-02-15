#![allow(
    clippy::needless_borrows_for_generic_args,
    clippy::manual_range_contains,
    clippy::assertions_on_constants,
    clippy::derivable_impls,
    clippy::type_complexity,
    clippy::ptr_arg,
    clippy::if_same_then_else,
    clippy::wrong_self_convention,
    clippy::manual_clamp,
    clippy::map_entry,
    clippy::len_zero,
    dead_code,
    unused_imports,
    unused_variables,
    unused_mut
)]

pub mod adapters;
pub mod config;
pub mod db;
pub mod discovery;
pub mod error;
pub mod hooks;
pub mod metrics;
pub mod models;
pub mod plugins;
pub mod repo;
pub mod services;
pub mod skillkit;
pub mod sync;

pub use adapters::{
    ActiveSession, AdapterInfo, AdapterRegistry, AdapterStatus, AgentAdapter, ClaudeCodeAdapter,
    ClaudeCodeConfig, ClaudeCodeCostCalculator, CostTracker, FullAdapter, OpenCodeAdapter,
    OpenCodeConfig, OpenCodeCostCalculator, SessionEvent, SessionEventCallback, SessionHistory,
    SessionMonitor, UsageStats,
};
pub use config::{
    ensure_cache_dir, ensure_config_dir, ensure_data_dir, get_cache_dir, get_config_dir,
    get_data_dir, AgentsConfig, ConfigLoadError, DatabaseConfig as RimuruDatabaseConfig,
    DesktopConfig, DisplayConfig, LoggingConfig, MetricsConfig, RimuruConfig, SupportedAgents,
    SyncConfig, SyncProviders, TuiConfig,
};
pub use db::{init_database, init_database_with_url, Database, DatabaseConfig, DatabaseError};
pub use discovery::{
    AgentDiscovery, AgentInstallation, AgentScanner, ClaudeCodeDiscovery, DetectionMethod,
    DiscoveredAgent, OpenCodeDiscovery, ScanOptions, ScanResult,
};
pub use error::{
    retry_async, retry_async_with_config, CliErrorDisplay, ErrorContext, RetryConfig, RimuruError,
    RimuruResult,
};
pub use hooks::{
    trigger_hook, trigger_hook_with_data, CostAlertConfig, CostAlertHandler, DynHookHandler, Hook,
    HookConfig, HookContext, HookData, HookExecution, HookHandler, HookHandlerInfo, HookManager,
    HookResult, MetricsExportConfig, MetricsExportHandler, SessionLogConfig, SessionLogFormat,
    SessionLogHandler, SessionStartLogHandler, WebhookConfig, WebhookHandler,
};
pub use metrics::{MetricsCollector, MetricsCollectorConfig, SessionAggregator, SystemCollector};
pub use models::{
    Agent, AgentType, CostRecord, CostSummary, MetricsSnapshot, ModelInfo, Session, SessionStatus,
    SystemMetrics,
};
pub use plugins::{
    create_builtin_exporter, create_builtin_notifier, create_example_manifest, is_builtin_plugin,
    list_builtin_plugins, AccessViolation, AgentCapability, AgentPlugin, CapabilitiesSection,
    CapabilityProvider, ConfigSection, CsvExporterConfig, CsvExporterPlugin, DependencyResolution,
    DiscordNotifierConfig, DiscordNotifierPlugin, DynAgentPlugin, DynExporterPlugin,
    DynNotifierPlugin, DynPlugin, DynViewPlugin, ExportData, ExportOptions, ExporterCapability,
    ExporterPlugin, HookRegistration, HttpMethod, JsonExporterConfig, JsonExporterPlugin,
    LineEnding, LoadedPlugin, Notification, NotificationLevel, NotifierCapability, NotifierPlugin,
    Permission, Plugin, PluginCapability, PluginConfig, PluginConflict, PluginContext,
    PluginDependency, PluginEvent, PluginFactory, PluginInfo, PluginLoader, PluginManifest,
    PluginMetadata, PluginPermission, PluginRegistry, PluginState, PluginStatus, PluginType,
    ResolvedDependency, ResourceLimits, ResourceUsage, Sandbox, SandboxConfig, SandboxManager,
    SessionCallback, SlackNotifierConfig, SlackNotifierPlugin, ViewAction, ViewCapability,
    ViewContext, ViewInput, ViewOutput, ViewPlugin, WebhookNotifierConfig, WebhookNotifierPlugin,
    WidgetData,
};
pub use repo::{
    AgentRepository, CostRepository, MetricsRepository, ModelRepository, Repository,
    SessionRepository,
};
pub use services::{
    AdapterHealth, AdapterManager, AdapterManagerConfig, AggregatedCostReport, CostAggregator,
    CostBreakdown, CostFilter, SessionAggregator as ServiceSessionAggregator, SessionFilter,
    SessionReport, SessionStats, TimeRange, UnifiedSession,
};
pub use skillkit::{
    detect_npx_skillkit, detect_skillkit_installation, AgentDetector, AgentMapping, AgentStatus,
    CompatibilityChange, CompatibilityChangeType, CompatibilityEstimate, ConfidenceLevel,
    DetectedAgent, DetectionResult, InstallOptions, InstallProgress, InstallResult, InstalledSkill,
    InstalledSkillsRegistry, ListFilter, ListOptions, ListStats, MarketplaceStats, PublishOptions,
    PublishProgress, PublishResult, RecommendOptions, SearchFilters, SearchOptions,
    SearchPagination, Skill, SkillInstaller, SkillKitAgent, SkillKitBridge, SkillKitBridgeBuilder,
    SkillKitConfig, SkillKitConfigManager, SkillKitInfo, SkillKitInstallationStatus,
    SkillKitPreferences, SkillLister, SkillPublisher, SkillRecommendation, SkillRecommender,
    SkillSearchResult, SkillSearcher, SkillSuggestion, SkillSyncer, SkillTranslator, SkillUpdate,
    SkillVersion, SyncError as SkillKitSyncError, SyncOptions as SkillKitSyncOptions, SyncPhase,
    SyncProgress as SkillKitSyncProgress, SyncResult as SkillKitSyncResult,
    SyncStatus as SkillKitSyncStatus, TranslateOptions, TranslateProgress, TranslationResult,
    ValidationError, ValidationResult, ValidationWarning, VersionBump, WorkflowContext,
};
pub use sync::{
    AnthropicSyncProvider, BackgroundSyncScheduler, ConflictResolution, ExtendedModelInfo,
    GoogleSyncProvider, LiteLLMSyncProvider, ModelAggregator, ModelCapability, ModelSource,
    ModelSyncProvider, OpenAISyncProvider, OpenRouterSyncProvider, ProviderSyncStatus,
    RateLimitConfig, SyncError, SyncHistory, SyncHistoryEntry, SyncModuleConfig,
    SyncProviderConfig, SyncResult, SyncScheduler, SyncStatus,
};
