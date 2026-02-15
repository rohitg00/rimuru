pub mod agent_detection;
pub mod bridge;
pub mod config;
pub mod operations;
pub mod sync;
pub mod types;

pub use agent_detection::{
    AgentDetector, AgentMapping, AgentStatus, CompatibilityEstimate, ConfidenceLevel,
    DetectedAgent, DetectionResult, SkillSuggestion,
};
pub use bridge::{SkillKitBridge, SkillKitBridgeBuilder};
pub use config::{
    detect_npx_skillkit, detect_skillkit_installation, InstalledSkillsRegistry, SkillKitConfig,
    SkillKitConfigManager, SkillKitPreferences,
};
pub use operations::{
    InstallOptions, InstallProgress, InstallResult, ListFilter, ListOptions, ListStats,
    PublishOptions, PublishProgress, RecommendOptions, SearchOptions, SearchPagination,
    SkillInstaller, SkillLister, SkillPublisher, SkillRecommender, SkillSearcher, SkillTranslator,
    TranslateOptions, TranslateProgress, ValidationError, ValidationResult, ValidationWarning,
    VersionBump, WorkflowContext,
};
pub use sync::{
    CompatibilityChange, CompatibilityChangeType, SkillSyncer, SkillUpdate, SkillVersion,
    SyncError, SyncOptions, SyncPhase, SyncProgress, SyncResult, SyncStatus,
};
pub use types::{
    InstalledSkill, MarketplaceStats, PublishResult, SearchFilters, Skill, SkillKitAgent,
    SkillKitInfo, SkillKitInstallationStatus, SkillRecommendation, SkillSearchResult,
    TranslationResult,
};
