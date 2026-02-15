pub mod claude_code;
pub mod codex;
pub mod copilot;
pub mod cursor;
pub mod factory;
pub mod goose;
pub mod opencode;
mod registry;
mod traits;
pub mod types;

pub use claude_code::{
    ClaudeCodeAdapter, ClaudeCodeConfig, ClaudeCodeCostCalculator, ClaudeCodePermissions,
    ClaudeCodeProjectState, ClaudeCodeSessionData, ClaudeCodeSettings,
    SessionParser as ClaudeCodeSessionParser,
};
pub use codex::{
    CodexAdapter, CodexConfig, CodexCostCalculator, CodexHistoryEntry, CodexSandboxPermissions,
    CodexSessionData, CodexSettings, SessionParser as CodexSessionParser,
};
pub use copilot::{
    CopilotAdapter, CopilotAppEntry, CopilotAppsConfig, CopilotConfig, CopilotCostCalculator,
    CopilotHost, CopilotHostsConfig, CopilotProduct, CopilotSessionData, CopilotTelemetryEntry,
    CopilotUsageEntry, SessionParser as CopilotSessionParser,
};
pub use cursor::{
    CursorAdapter, CursorChatEntry, CursorChatSettings, CursorComposerEntry,
    CursorComposerSettings, CursorConfig, CursorCostCalculator, CursorMessage, CursorSessionData,
    CursorSessionType, CursorSettings, CursorTabSettings, CursorTier,
    SessionParser as CursorSessionParser,
};
pub use factory::{
    create_adapter, create_adapter_with_config, AdapterConfig, AdapterFactory, AdapterFactoryConfig,
};
pub use goose::{
    GooseAdapter, GooseConfig, GooseCostCalculator, GooseExtensionConfig, GooseHistoryEntry,
    GooseMessage, GooseProfile, GooseProviderConfig, GooseSessionData, GooseSettings,
    SessionParser as GooseSessionParser,
};
pub use opencode::{
    OpenCodeAdapter, OpenCodeConfig, OpenCodeCostCalculator, OpenCodeSessionData, OpenCodeSettings,
    OpenCodeState, ProviderConfig, SessionParser as OpenCodeSessionParser,
};
pub use registry::AdapterRegistry;
pub use traits::{AgentAdapter, CostTracker, FullAdapter, SessionEventCallback, SessionMonitor};
pub use types::{
    ActiveSession, AdapterInfo, AdapterStatus, SessionEvent, SessionHistory, UsageStats,
};
