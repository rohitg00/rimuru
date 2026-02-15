use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum SkillKitAgent {
    ClaudeCode,
    Cursor,
    Codex,
    GeminiCli,
    OpenCode,
    Antigravity,
    Amp,
    Clawdbot,
    Droid,
    GithubCopilot,
    Goose,
    Kilo,
    KiroCli,
    Roo,
    Trae,
    Windsurf,
    Universal,
    Cline,
    CodeBuddy,
    CommandCode,
    Continue,
    Crush,
    Factory,
    McpJam,
    Mux,
    Neovate,
    OpenHands,
    Pi,
    Qoder,
    Qwen,
    Vercel,
    ZenCoder,
}

impl SkillKitAgent {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude-code",
            Self::Cursor => "cursor",
            Self::Codex => "codex",
            Self::GeminiCli => "gemini-cli",
            Self::OpenCode => "opencode",
            Self::Antigravity => "antigravity",
            Self::Amp => "amp",
            Self::Clawdbot => "clawdbot",
            Self::Droid => "droid",
            Self::GithubCopilot => "github-copilot",
            Self::Goose => "goose",
            Self::Kilo => "kilo",
            Self::KiroCli => "kiro-cli",
            Self::Roo => "roo",
            Self::Trae => "trae",
            Self::Windsurf => "windsurf",
            Self::Universal => "universal",
            Self::Cline => "cline",
            Self::CodeBuddy => "codebuddy",
            Self::CommandCode => "commandcode",
            Self::Continue => "continue",
            Self::Crush => "crush",
            Self::Factory => "factory",
            Self::McpJam => "mcpjam",
            Self::Mux => "mux",
            Self::Neovate => "neovate",
            Self::OpenHands => "openhands",
            Self::Pi => "pi",
            Self::Qoder => "qoder",
            Self::Qwen => "qwen",
            Self::Vercel => "vercel",
            Self::ZenCoder => "zencoder",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::ClaudeCode => "Claude Code",
            Self::Cursor => "Cursor",
            Self::Codex => "Codex",
            Self::GeminiCli => "Gemini CLI",
            Self::OpenCode => "OpenCode",
            Self::Antigravity => "Antigravity",
            Self::Amp => "Amp",
            Self::Clawdbot => "Clawdbot",
            Self::Droid => "Droid",
            Self::GithubCopilot => "GitHub Copilot",
            Self::Goose => "Goose",
            Self::Kilo => "Kilo",
            Self::KiroCli => "Kiro CLI",
            Self::Roo => "Roo",
            Self::Trae => "Trae",
            Self::Windsurf => "Windsurf",
            Self::Universal => "Universal",
            Self::Cline => "Cline",
            Self::CodeBuddy => "CodeBuddy",
            Self::CommandCode => "CommandCode",
            Self::Continue => "Continue",
            Self::Crush => "Crush",
            Self::Factory => "Factory",
            Self::McpJam => "MCP Jam",
            Self::Mux => "Mux",
            Self::Neovate => "Neovate",
            Self::OpenHands => "OpenHands",
            Self::Pi => "Pi",
            Self::Qoder => "Qoder",
            Self::Qwen => "Qwen",
            Self::Vercel => "Vercel",
            Self::ZenCoder => "ZenCoder",
        }
    }

    pub fn all() -> &'static [SkillKitAgent] {
        &[
            Self::ClaudeCode,
            Self::Cursor,
            Self::Codex,
            Self::GeminiCli,
            Self::OpenCode,
            Self::Antigravity,
            Self::Amp,
            Self::Clawdbot,
            Self::Droid,
            Self::GithubCopilot,
            Self::Goose,
            Self::Kilo,
            Self::KiroCli,
            Self::Roo,
            Self::Trae,
            Self::Windsurf,
            Self::Universal,
            Self::Cline,
            Self::CodeBuddy,
            Self::CommandCode,
            Self::Continue,
            Self::Crush,
            Self::Factory,
            Self::McpJam,
            Self::Mux,
            Self::Neovate,
            Self::OpenHands,
            Self::Pi,
            Self::Qoder,
            Self::Qwen,
            Self::Vercel,
            Self::ZenCoder,
        ]
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "claude-code" | "claudecode" => Some(Self::ClaudeCode),
            "cursor" => Some(Self::Cursor),
            "codex" => Some(Self::Codex),
            "gemini-cli" | "geminicli" => Some(Self::GeminiCli),
            "opencode" => Some(Self::OpenCode),
            "antigravity" => Some(Self::Antigravity),
            "amp" => Some(Self::Amp),
            "clawdbot" => Some(Self::Clawdbot),
            "droid" => Some(Self::Droid),
            "github-copilot" | "githubcopilot" | "copilot" => Some(Self::GithubCopilot),
            "goose" => Some(Self::Goose),
            "kilo" => Some(Self::Kilo),
            "kiro-cli" | "kirocli" => Some(Self::KiroCli),
            "roo" => Some(Self::Roo),
            "trae" => Some(Self::Trae),
            "windsurf" => Some(Self::Windsurf),
            "universal" => Some(Self::Universal),
            "cline" => Some(Self::Cline),
            "codebuddy" => Some(Self::CodeBuddy),
            "commandcode" => Some(Self::CommandCode),
            "continue" => Some(Self::Continue),
            "crush" => Some(Self::Crush),
            "factory" => Some(Self::Factory),
            "mcpjam" | "mcp-jam" => Some(Self::McpJam),
            "mux" => Some(Self::Mux),
            "neovate" => Some(Self::Neovate),
            "openhands" => Some(Self::OpenHands),
            "pi" => Some(Self::Pi),
            "qoder" => Some(Self::Qoder),
            "qwen" => Some(Self::Qwen),
            "vercel" => Some(Self::Vercel),
            "zencoder" => Some(Self::ZenCoder),
            _ => None,
        }
    }
}

impl std::fmt::Display for SkillKitAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub slug: String,
    pub description: String,
    pub author: Option<String>,
    pub source: Option<String>,
    pub repository: Option<String>,
    pub tags: Vec<String>,
    pub agents: Vec<SkillKitAgent>,
    pub version: Option<String>,
    pub downloads: Option<u64>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Skill {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            slug: name.to_lowercase().replace(' ', "-"),
            description: description.to_string(),
            author: None,
            source: None,
            repository: None,
            tags: Vec::new(),
            agents: Vec::new(),
            version: None,
            downloads: None,
            created_at: None,
            updated_at: None,
        }
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_agents(mut self, agents: Vec<SkillKitAgent>) -> Self {
        self.agents = agents;
        self
    }

    pub fn is_compatible_with(&self, agent: &SkillKitAgent) -> bool {
        self.agents.is_empty() || self.agents.contains(agent)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSearchResult {
    pub skills: Vec<Skill>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
    pub query: Option<String>,
    pub filters: SearchFilters,
}

impl SkillSearchResult {
    pub fn empty() -> Self {
        Self {
            skills: Vec::new(),
            total: 0,
            page: 1,
            per_page: 20,
            query: None,
            filters: SearchFilters::default(),
        }
    }

    pub fn has_more(&self) -> bool {
        self.page * self.per_page < self.total
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFilters {
    pub agent: Option<SkillKitAgent>,
    pub tags: Vec<String>,
    pub author: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledSkill {
    pub skill: Skill,
    pub installed_at: DateTime<Utc>,
    pub installed_for: Vec<SkillKitAgent>,
    pub path: String,
    pub enabled: bool,
}

impl InstalledSkill {
    pub fn new(skill: Skill, path: &str, agents: Vec<SkillKitAgent>) -> Self {
        Self {
            skill,
            installed_at: Utc::now(),
            installed_for: agents,
            path: path.to_string(),
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResult {
    pub skill_name: String,
    pub from_agent: SkillKitAgent,
    pub to_agent: SkillKitAgent,
    pub success: bool,
    pub output_path: Option<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub duration_ms: u64,
}

impl TranslationResult {
    pub fn success(
        skill_name: &str,
        from: SkillKitAgent,
        to: SkillKitAgent,
        output_path: &str,
    ) -> Self {
        Self {
            skill_name: skill_name.to_string(),
            from_agent: from,
            to_agent: to,
            success: true,
            output_path: Some(output_path.to_string()),
            warnings: Vec::new(),
            errors: Vec::new(),
            duration_ms: 0,
        }
    }

    pub fn failure(skill_name: &str, from: SkillKitAgent, to: SkillKitAgent, error: &str) -> Self {
        Self {
            skill_name: skill_name.to_string(),
            from_agent: from,
            to_agent: to,
            success: false,
            output_path: None,
            warnings: Vec::new(),
            errors: vec![error.to_string()],
            duration_ms: 0,
        }
    }

    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }

    pub fn with_warning(mut self, warning: &str) -> Self {
        self.warnings.push(warning.to_string());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceStats {
    pub total_skills: usize,
    pub total_downloads: u64,
    pub total_authors: usize,
    pub skills_by_agent: HashMap<SkillKitAgent, usize>,
    pub trending_skills: Vec<Skill>,
    pub recent_skills: Vec<Skill>,
    pub last_updated: DateTime<Utc>,
}

impl Default for MarketplaceStats {
    fn default() -> Self {
        Self {
            total_skills: 0,
            total_downloads: 0,
            total_authors: 0,
            skills_by_agent: HashMap::new(),
            trending_skills: Vec::new(),
            recent_skills: Vec::new(),
            last_updated: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRecommendation {
    pub skill: Skill,
    pub reason: String,
    pub confidence: f32,
    pub based_on: Vec<String>,
}

impl SkillRecommendation {
    pub fn new(skill: Skill, reason: &str, confidence: f32) -> Self {
        Self {
            skill,
            reason: reason.to_string(),
            confidence,
            based_on: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishResult {
    pub skill_name: String,
    pub version: String,
    pub success: bool,
    pub marketplace_url: Option<String>,
    pub errors: Vec<String>,
}

impl PublishResult {
    pub fn success(skill_name: &str, version: &str, url: &str) -> Self {
        Self {
            skill_name: skill_name.to_string(),
            version: version.to_string(),
            success: true,
            marketplace_url: Some(url.to_string()),
            errors: Vec::new(),
        }
    }

    pub fn failure(skill_name: &str, error: &str) -> Self {
        Self {
            skill_name: skill_name.to_string(),
            version: String::new(),
            success: false,
            marketplace_url: None,
            errors: vec![error.to_string()],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SkillKitInstallationStatus {
    Installed,
    NotInstalled,
    OutOfDate,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillKitInfo {
    pub installed: bool,
    pub version: Option<String>,
    pub status: SkillKitInstallationStatus,
    pub path: Option<String>,
    pub config_path: Option<String>,
}

impl Default for SkillKitInfo {
    fn default() -> Self {
        Self {
            installed: false,
            version: None,
            status: SkillKitInstallationStatus::Unknown,
            path: None,
            config_path: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skillkit_agent_as_str() {
        assert_eq!(SkillKitAgent::ClaudeCode.as_str(), "claude-code");
        assert_eq!(SkillKitAgent::Cursor.as_str(), "cursor");
        assert_eq!(SkillKitAgent::GithubCopilot.as_str(), "github-copilot");
    }

    #[test]
    fn test_skillkit_agent_from_str() {
        assert_eq!(
            SkillKitAgent::parse("claude-code"),
            Some(SkillKitAgent::ClaudeCode)
        );
        assert_eq!(SkillKitAgent::parse("CURSOR"), Some(SkillKitAgent::Cursor));
        assert_eq!(
            SkillKitAgent::parse("copilot"),
            Some(SkillKitAgent::GithubCopilot)
        );
        assert_eq!(SkillKitAgent::parse("invalid"), None);
    }

    #[test]
    fn test_skill_new() {
        let skill = Skill::new("Test Skill", "A test skill description");
        assert_eq!(skill.name, "Test Skill");
        assert_eq!(skill.slug, "test-skill");
        assert_eq!(skill.description, "A test skill description");
        assert!(skill.tags.is_empty());
        assert!(skill.agents.is_empty());
    }

    #[test]
    fn test_skill_with_tags() {
        let skill =
            Skill::new("Test", "Desc").with_tags(vec!["rust".to_string(), "cli".to_string()]);
        assert_eq!(skill.tags.len(), 2);
        assert!(skill.tags.contains(&"rust".to_string()));
    }

    #[test]
    fn test_skill_compatibility() {
        let universal_skill = Skill::new("Universal", "Works everywhere");
        assert!(universal_skill.is_compatible_with(&SkillKitAgent::ClaudeCode));
        assert!(universal_skill.is_compatible_with(&SkillKitAgent::Cursor));

        let specific_skill =
            Skill::new("Specific", "Only for Claude").with_agents(vec![SkillKitAgent::ClaudeCode]);
        assert!(specific_skill.is_compatible_with(&SkillKitAgent::ClaudeCode));
        assert!(!specific_skill.is_compatible_with(&SkillKitAgent::Cursor));
    }

    #[test]
    fn test_search_result_has_more() {
        let mut result = SkillSearchResult::empty();
        result.total = 50;
        result.page = 1;
        result.per_page = 20;
        assert!(result.has_more());

        result.page = 3;
        assert!(!result.has_more());
    }

    #[test]
    fn test_translation_result() {
        let success = TranslationResult::success(
            "test-skill",
            SkillKitAgent::ClaudeCode,
            SkillKitAgent::Cursor,
            "/path/to/output",
        );
        assert!(success.success);
        assert!(success.output_path.is_some());
        assert!(success.errors.is_empty());

        let failure = TranslationResult::failure(
            "test-skill",
            SkillKitAgent::ClaudeCode,
            SkillKitAgent::Cursor,
            "Translation failed",
        );
        assert!(!failure.success);
        assert!(failure.output_path.is_none());
        assert_eq!(failure.errors.len(), 1);
    }

    #[test]
    fn test_skillkit_agent_all() {
        let all_agents = SkillKitAgent::all();
        assert_eq!(all_agents.len(), 32);
        assert!(all_agents.contains(&SkillKitAgent::ClaudeCode));
        assert!(all_agents.contains(&SkillKitAgent::ZenCoder));
    }
}

/// Types that match the actual SkillKit CLI JSON output
pub mod cli_response {
    use serde::{Deserialize, Serialize};

    /// Response from `skillkit marketplace search --json`
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MarketplaceSearchResponse {
        pub skills: Vec<MarketplaceSkill>,
        pub total: usize,
        pub query: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MarketplaceSkill {
        pub id: String,
        pub name: String,
        #[serde(default)]
        pub description: String,
        #[serde(default)]
        pub source: Option<SkillSource>,
        #[serde(default)]
        pub path: String,
        #[serde(default)]
        pub tags: Vec<String>,
        #[serde(rename = "rawUrl", default)]
        pub raw_url: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SkillSource {
        pub owner: String,
        pub repo: String,
        pub name: String,
        #[serde(default)]
        pub description: String,
        #[serde(default)]
        pub official: bool,
        #[serde(default)]
        pub branch: String,
    }

    /// Response from `skillkit list --json`
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct InstalledSkillResponse {
        pub name: String,
        #[serde(default)]
        pub description: String,
        pub path: String,
        #[serde(default)]
        pub location: String,
        #[serde(default)]
        pub enabled: bool,
        #[serde(default)]
        pub quality: u32,
    }

    /// Response from `skillkit recommend --json`
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RecommendResponse {
        pub recommendations: Vec<SkillRecommendation>,
        #[serde(default)]
        pub profile: Option<ProjectProfile>,
        #[serde(rename = "totalSkillsScanned", default)]
        pub total_skills_scanned: usize,
        #[serde(default)]
        pub timestamp: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SkillRecommendation {
        pub name: String,
        #[serde(default)]
        pub description: String,
        #[serde(default)]
        pub score: f32,
        #[serde(default)]
        pub reason: Option<String>,
        #[serde(default)]
        pub source: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ProjectProfile {
        #[serde(default)]
        pub name: String,
        #[serde(rename = "type", default)]
        pub project_type: String,
        #[serde(default)]
        pub stack: Option<TechStack>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TechStack {
        #[serde(default)]
        pub languages: Vec<TechItem>,
        #[serde(default)]
        pub frameworks: Vec<TechItem>,
        #[serde(default)]
        pub libraries: Vec<TechItem>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TechItem {
        pub name: String,
        #[serde(default)]
        pub confidence: u32,
        #[serde(default)]
        pub source: String,
    }

    /// Response from `skillkit marketplace --json`
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MarketplaceStatsResponse {
        #[serde(rename = "totalSkills")]
        pub total_skills: usize,
        pub sources: usize,
        #[serde(rename = "updatedAt", default)]
        pub updated_at: Option<String>,
    }

    impl MarketplaceSkill {
        pub fn to_skill(&self) -> super::Skill {
            let author = self
                .source
                .as_ref()
                .map(|s| format!("{}/{}", s.owner, s.repo));
            let source_url = self
                .source
                .as_ref()
                .map(|s| format!("https://github.com/{}/{}", s.owner, s.repo));

            super::Skill {
                name: self.name.clone(),
                slug: self.id.clone(),
                description: self.description.clone(),
                author,
                source: source_url,
                repository: self.raw_url.clone(),
                tags: self.tags.clone(),
                agents: Vec::new(),
                version: None,
                downloads: None,
                created_at: None,
                updated_at: None,
            }
        }
    }

    impl InstalledSkillResponse {
        pub fn to_installed_skill(&self) -> super::InstalledSkill {
            let skill = super::Skill {
                name: self.name.clone(),
                slug: self.name.to_lowercase().replace(' ', "-"),
                description: self.description.clone(),
                author: None,
                source: None,
                repository: None,
                tags: Vec::new(),
                agents: Vec::new(),
                version: None,
                downloads: None,
                created_at: None,
                updated_at: None,
            };

            super::InstalledSkill {
                skill,
                installed_at: chrono::Utc::now(),
                installed_for: vec![super::SkillKitAgent::ClaudeCode],
                path: self.path.clone(),
                enabled: self.enabled,
            }
        }
    }

    impl MarketplaceSearchResponse {
        pub fn to_search_result(&self) -> super::SkillSearchResult {
            super::SkillSearchResult {
                skills: self.skills.iter().map(|s| s.to_skill()).collect(),
                total: self.total,
                page: 1,
                per_page: 20,
                query: Some(self.query.clone()),
                filters: super::SearchFilters::default(),
            }
        }
    }
}
