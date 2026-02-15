use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info};

use super::types::{Skill, SkillKitAgent};
use crate::error::{RimuruError, RimuruResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedAgent {
    pub agent: SkillKitAgent,
    pub status: AgentStatus,
    pub skills_dir: Option<PathBuf>,
    pub config_path: Option<PathBuf>,
    pub version: Option<String>,
    pub installed_skills_count: usize,
}

impl DetectedAgent {
    pub fn new(agent: SkillKitAgent) -> Self {
        Self {
            agent,
            status: AgentStatus::Unknown,
            skills_dir: None,
            config_path: None,
            version: None,
            installed_skills_count: 0,
        }
    }

    pub fn is_available(&self) -> bool {
        matches!(
            self.status,
            AgentStatus::Installed | AgentStatus::InstalledWithSkills
        )
    }

    pub fn has_skills(&self) -> bool {
        self.installed_skills_count > 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Installed,
    InstalledWithSkills,
    NotInstalled,
    ConfigOnly,
    Unknown,
}

impl std::fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Installed => write!(f, "Installed"),
            Self::InstalledWithSkills => write!(f, "Installed (with skills)"),
            Self::NotInstalled => write!(f, "Not installed"),
            Self::ConfigOnly => write!(f, "Config only"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMapping {
    pub rimuru_name: String,
    pub skillkit_agent: SkillKitAgent,
    pub aliases: Vec<String>,
    pub config_dir: PathBuf,
    pub skills_dir: PathBuf,
    pub detection_files: Vec<String>,
}

impl AgentMapping {
    pub fn new(
        rimuru_name: &str,
        skillkit_agent: SkillKitAgent,
        config_dir: PathBuf,
        skills_dir: PathBuf,
    ) -> Self {
        Self {
            rimuru_name: rimuru_name.to_string(),
            skillkit_agent,
            aliases: Vec::new(),
            config_dir,
            skills_dir,
            detection_files: Vec::new(),
        }
    }

    pub fn with_aliases(mut self, aliases: Vec<&str>) -> Self {
        self.aliases = aliases.into_iter().map(String::from).collect();
        self
    }

    pub fn with_detection_files(mut self, files: Vec<&str>) -> Self {
        self.detection_files = files.into_iter().map(String::from).collect();
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    pub agents: Vec<DetectedAgent>,
    pub total_detected: usize,
    pub total_with_skills: usize,
    pub scan_duration_ms: u64,
}

impl DetectionResult {
    pub fn empty() -> Self {
        Self {
            agents: Vec::new(),
            total_detected: 0,
            total_with_skills: 0,
            scan_duration_ms: 0,
        }
    }

    pub fn available_agents(&self) -> Vec<&DetectedAgent> {
        self.agents.iter().filter(|a| a.is_available()).collect()
    }

    pub fn agents_with_skills(&self) -> Vec<&DetectedAgent> {
        self.agents.iter().filter(|a| a.has_skills()).collect()
    }
}

pub struct AgentDetector {
    #[allow(dead_code)]
    home_dir: PathBuf,
    mappings: HashMap<SkillKitAgent, AgentMapping>,
}

impl AgentDetector {
    pub fn new() -> RimuruResult<Self> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| RimuruError::Config("Could not determine home directory".to_string()))?;

        let mut detector = Self {
            home_dir: home_dir.clone(),
            mappings: HashMap::new(),
        };

        detector.initialize_mappings(&home_dir);
        Ok(detector)
    }

    fn initialize_mappings(&mut self, home: &PathBuf) {
        self.mappings.insert(
            SkillKitAgent::ClaudeCode,
            AgentMapping::new(
                "Claude Code",
                SkillKitAgent::ClaudeCode,
                home.join(".claude"),
                home.join(".claude").join("skills"),
            )
            .with_aliases(vec!["claude", "claude-code", "anthropic"])
            .with_detection_files(vec!["CLAUDE.md", "settings.json", "config.json"]),
        );

        self.mappings.insert(
            SkillKitAgent::Cursor,
            AgentMapping::new(
                "Cursor",
                SkillKitAgent::Cursor,
                home.join(".cursor"),
                home.join(".cursor").join("skills"),
            )
            .with_aliases(vec!["cursor", "cursor-ai"])
            .with_detection_files(vec!["settings.json", "config.json"]),
        );

        self.mappings.insert(
            SkillKitAgent::Codex,
            AgentMapping::new(
                "Codex",
                SkillKitAgent::Codex,
                home.join(".codex"),
                home.join(".codex").join("skills"),
            )
            .with_aliases(vec!["codex", "openai-codex"])
            .with_detection_files(vec!["config.json"]),
        );

        self.mappings.insert(
            SkillKitAgent::GeminiCli,
            AgentMapping::new(
                "Gemini CLI",
                SkillKitAgent::GeminiCli,
                home.join(".gemini"),
                home.join(".gemini").join("skills"),
            )
            .with_aliases(vec!["gemini", "gemini-cli", "google-gemini"])
            .with_detection_files(vec!["config.json", "settings.json"]),
        );

        self.mappings.insert(
            SkillKitAgent::OpenCode,
            AgentMapping::new(
                "OpenCode",
                SkillKitAgent::OpenCode,
                home.join(".opencode"),
                home.join(".opencode").join("skills"),
            )
            .with_aliases(vec!["opencode", "open-code"])
            .with_detection_files(vec!["config.json"]),
        );

        self.mappings.insert(
            SkillKitAgent::Windsurf,
            AgentMapping::new(
                "Windsurf",
                SkillKitAgent::Windsurf,
                home.join(".windsurf"),
                home.join(".windsurf").join("skills"),
            )
            .with_aliases(vec!["windsurf", "codeium"])
            .with_detection_files(vec!["config.json", "settings.json"]),
        );

        self.mappings.insert(
            SkillKitAgent::GithubCopilot,
            AgentMapping::new(
                "GitHub Copilot",
                SkillKitAgent::GithubCopilot,
                home.join(".copilot"),
                home.join(".copilot").join("skills"),
            )
            .with_aliases(vec!["copilot", "github-copilot", "ghcopilot"])
            .with_detection_files(vec!["config.json", "hosts.json"]),
        );

        self.mappings.insert(
            SkillKitAgent::Cline,
            AgentMapping::new(
                "Cline",
                SkillKitAgent::Cline,
                home.join(".cline"),
                home.join(".cline").join("skills"),
            )
            .with_aliases(vec!["cline"])
            .with_detection_files(vec!["config.json"]),
        );

        self.mappings.insert(
            SkillKitAgent::Continue,
            AgentMapping::new(
                "Continue",
                SkillKitAgent::Continue,
                home.join(".continue"),
                home.join(".continue").join("skills"),
            )
            .with_aliases(vec!["continue", "continue-dev"])
            .with_detection_files(vec!["config.json", "config.yaml"]),
        );

        self.mappings.insert(
            SkillKitAgent::Roo,
            AgentMapping::new(
                "Roo",
                SkillKitAgent::Roo,
                home.join(".roo"),
                home.join(".roo").join("skills"),
            )
            .with_aliases(vec!["roo", "roo-code"])
            .with_detection_files(vec!["config.json"]),
        );

        self.mappings.insert(
            SkillKitAgent::Goose,
            AgentMapping::new(
                "Goose",
                SkillKitAgent::Goose,
                home.join(".goose"),
                home.join(".goose").join("skills"),
            )
            .with_aliases(vec!["goose", "block-goose"])
            .with_detection_files(vec!["config.yaml", "config.json"]),
        );

        self.mappings.insert(
            SkillKitAgent::Amp,
            AgentMapping::new(
                "Amp",
                SkillKitAgent::Amp,
                home.join(".amp"),
                home.join(".amp").join("skills"),
            )
            .with_aliases(vec!["amp", "sourcegraph-amp"])
            .with_detection_files(vec!["config.json"]),
        );

        self.mappings.insert(
            SkillKitAgent::Clawdbot,
            AgentMapping::new(
                "Clawdbot",
                SkillKitAgent::Clawdbot,
                home.join(".clawdbot"),
                home.join(".clawdbot").join("skills"),
            )
            .with_aliases(vec!["clawdbot", "moltbot"])
            .with_detection_files(vec!["config.json"]),
        );

        self.mappings.insert(
            SkillKitAgent::Droid,
            AgentMapping::new(
                "Droid",
                SkillKitAgent::Droid,
                home.join(".droid"),
                home.join(".droid").join("skills"),
            )
            .with_aliases(vec!["droid"])
            .with_detection_files(vec!["config.json"]),
        );

        self.mappings.insert(
            SkillKitAgent::Kilo,
            AgentMapping::new(
                "Kilo",
                SkillKitAgent::Kilo,
                home.join(".kilo"),
                home.join(".kilo").join("skills"),
            )
            .with_aliases(vec!["kilo"])
            .with_detection_files(vec!["config.json"]),
        );

        self.mappings.insert(
            SkillKitAgent::KiroCli,
            AgentMapping::new(
                "Kiro CLI",
                SkillKitAgent::KiroCli,
                home.join(".kiro"),
                home.join(".kiro").join("skills"),
            )
            .with_aliases(vec!["kiro", "kiro-cli"])
            .with_detection_files(vec!["config.json"]),
        );

        self.mappings.insert(
            SkillKitAgent::Trae,
            AgentMapping::new(
                "Trae",
                SkillKitAgent::Trae,
                home.join(".trae"),
                home.join(".trae").join("skills"),
            )
            .with_aliases(vec!["trae"])
            .with_detection_files(vec!["config.json"]),
        );

        self.mappings.insert(
            SkillKitAgent::OpenHands,
            AgentMapping::new(
                "OpenHands",
                SkillKitAgent::OpenHands,
                home.join(".openhands"),
                home.join(".openhands").join("skills"),
            )
            .with_aliases(vec!["openhands", "open-hands"])
            .with_detection_files(vec!["config.json"]),
        );

        for agent in &[
            SkillKitAgent::Antigravity,
            SkillKitAgent::CodeBuddy,
            SkillKitAgent::CommandCode,
            SkillKitAgent::Crush,
            SkillKitAgent::Factory,
            SkillKitAgent::McpJam,
            SkillKitAgent::Mux,
            SkillKitAgent::Neovate,
            SkillKitAgent::Pi,
            SkillKitAgent::Qoder,
            SkillKitAgent::Qwen,
            SkillKitAgent::Vercel,
            SkillKitAgent::ZenCoder,
            SkillKitAgent::Universal,
        ] {
            if !self.mappings.contains_key(agent) {
                let agent_name = agent.as_str();
                self.mappings.insert(
                    *agent,
                    AgentMapping::new(
                        agent.display_name(),
                        *agent,
                        home.join(".skillkit").join("agents").join(agent_name),
                        home.join(".skillkit")
                            .join("agents")
                            .join(agent_name)
                            .join("skills"),
                    )
                    .with_detection_files(vec!["config.json"]),
                );
            }
        }
    }

    pub fn detect_all(&self) -> RimuruResult<DetectionResult> {
        let start = std::time::Instant::now();
        let mut result = DetectionResult::empty();

        info!("Starting agent detection scan");

        for agent in SkillKitAgent::all() {
            let detected = self.detect_agent(agent)?;
            if detected.is_available() {
                result.total_detected += 1;
                if detected.has_skills() {
                    result.total_with_skills += 1;
                }
            }
            result.agents.push(detected);
        }

        result.scan_duration_ms = start.elapsed().as_millis() as u64;

        info!(
            "Agent detection completed in {}ms: {} detected, {} with skills",
            result.scan_duration_ms, result.total_detected, result.total_with_skills
        );

        Ok(result)
    }

    pub fn detect_agent(&self, agent: &SkillKitAgent) -> RimuruResult<DetectedAgent> {
        let mut detected = DetectedAgent::new(*agent);

        let mapping = match self.mappings.get(agent) {
            Some(m) => m,
            None => {
                detected.status = AgentStatus::Unknown;
                return Ok(detected);
            }
        };

        detected.config_path = Some(mapping.config_dir.clone());
        detected.skills_dir = Some(mapping.skills_dir.clone());

        let config_exists = mapping.config_dir.exists();
        let skills_exist = mapping.skills_dir.exists();
        let has_detection_files = mapping
            .detection_files
            .iter()
            .any(|f| mapping.config_dir.join(f).exists());

        if !config_exists && !skills_exist {
            detected.status = AgentStatus::NotInstalled;
        } else if skills_exist {
            let skills_count = self.count_skills_in_dir(&mapping.skills_dir);
            detected.installed_skills_count = skills_count;

            if skills_count > 0 {
                detected.status = AgentStatus::InstalledWithSkills;
            } else {
                detected.status = AgentStatus::Installed;
            }
        } else if config_exists || has_detection_files {
            detected.status = AgentStatus::ConfigOnly;
        } else {
            detected.status = AgentStatus::Unknown;
        }

        debug!(
            "Detected agent {}: status={}, skills={}",
            agent, detected.status, detected.installed_skills_count
        );

        Ok(detected)
    }

    fn count_skills_in_dir(&self, dir: &PathBuf) -> usize {
        if !dir.exists() {
            return 0;
        }

        match std::fs::read_dir(dir) {
            Ok(entries) => entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let path = e.path();
                    if path.is_dir() {
                        path.join("SKILL.md").exists() || path.join("skill.json").exists()
                    } else {
                        path.extension().map(|ext| ext == "md").unwrap_or(false)
                    }
                })
                .count(),
            Err(_) => 0,
        }
    }

    pub fn get_mapping(&self, agent: &SkillKitAgent) -> Option<&AgentMapping> {
        self.mappings.get(agent)
    }

    pub fn resolve_agent_name(&self, name: &str) -> Option<SkillKitAgent> {
        let name_lower = name.to_lowercase();

        if let Some(agent) = SkillKitAgent::parse(&name_lower) {
            return Some(agent);
        }

        for (agent, mapping) in &self.mappings {
            if mapping.rimuru_name.to_lowercase() == name_lower {
                return Some(*agent);
            }
            if mapping
                .aliases
                .iter()
                .any(|a| a.to_lowercase() == name_lower)
            {
                return Some(*agent);
            }
        }

        None
    }

    pub fn suggest_skills_for_agents(
        &self,
        agents: &[SkillKitAgent],
    ) -> RimuruResult<Vec<SkillSuggestion>> {
        let mut suggestions = Vec::new();

        for agent in agents {
            let agent_suggestions = self.get_agent_skill_suggestions(agent)?;
            suggestions.extend(agent_suggestions);
        }

        suggestions.sort_by(|a, b| {
            b.priority
                .partial_cmp(&a.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        suggestions.truncate(10);

        Ok(suggestions)
    }

    fn get_agent_skill_suggestions(
        &self,
        agent: &SkillKitAgent,
    ) -> RimuruResult<Vec<SkillSuggestion>> {
        let suggestions = match agent {
            SkillKitAgent::ClaudeCode => vec![
                SkillSuggestion::new(
                    "pro-workflow",
                    "Battle-tested Claude Code workflows",
                    *agent,
                    0.95,
                ),
                SkillSuggestion::new(
                    "coding-standards",
                    "Universal coding standards and best practices",
                    *agent,
                    0.90,
                ),
                SkillSuggestion::new(
                    "tdd-workflow",
                    "Test-driven development enforcement",
                    *agent,
                    0.85,
                ),
            ],
            SkillKitAgent::Cursor => vec![
                SkillSuggestion::new(
                    "cursor-rules",
                    "Optimized rules for Cursor AI",
                    *agent,
                    0.95,
                ),
                SkillSuggestion::new(
                    "frontend-patterns",
                    "React and Next.js best practices",
                    *agent,
                    0.85,
                ),
            ],
            SkillKitAgent::Codex => vec![SkillSuggestion::new(
                "codex-optimization",
                "Optimized prompts for Codex",
                *agent,
                0.90,
            )],
            SkillKitAgent::GeminiCli => vec![SkillSuggestion::new(
                "gemini-patterns",
                "Gemini CLI best practices",
                *agent,
                0.90,
            )],
            SkillKitAgent::Windsurf => vec![SkillSuggestion::new(
                "windsurf-rules",
                "Windsurf optimization rules",
                *agent,
                0.90,
            )],
            SkillKitAgent::GithubCopilot => vec![SkillSuggestion::new(
                "copilot-tips",
                "GitHub Copilot productivity tips",
                *agent,
                0.90,
            )],
            _ => vec![SkillSuggestion::new(
                "universal-coding",
                "Universal coding best practices",
                *agent,
                0.80,
            )],
        };

        Ok(suggestions)
    }

    pub fn get_compatible_agents_for_skill(&self, skill: &Skill) -> Vec<SkillKitAgent> {
        if skill.agents.is_empty() {
            SkillKitAgent::all().to_vec()
        } else {
            skill.agents.clone()
        }
    }

    pub fn estimate_skill_compatibility(
        &self,
        skill: &Skill,
        target_agent: &SkillKitAgent,
    ) -> CompatibilityEstimate {
        let source_agents = &skill.agents;

        if source_agents.is_empty() || source_agents.contains(target_agent) {
            return CompatibilityEstimate {
                score: 1.0,
                confidence: ConfidenceLevel::High,
                warnings: Vec::new(),
                notes: vec!["Native support".to_string()],
            };
        }

        let similar_agents = self.get_similar_agents(target_agent);
        let has_similar = source_agents.iter().any(|a| similar_agents.contains(a));

        if has_similar {
            CompatibilityEstimate {
                score: 0.8,
                confidence: ConfidenceLevel::Medium,
                warnings: vec!["Minor translation may be needed".to_string()],
                notes: vec!["Similar agent architecture".to_string()],
            }
        } else {
            CompatibilityEstimate {
                score: 0.5,
                confidence: ConfidenceLevel::Low,
                warnings: vec!["Significant translation needed".to_string()],
                notes: vec!["Different agent architecture".to_string()],
            }
        }
    }

    fn get_similar_agents(&self, agent: &SkillKitAgent) -> Vec<SkillKitAgent> {
        match agent {
            SkillKitAgent::ClaudeCode => {
                vec![
                    SkillKitAgent::Cursor,
                    SkillKitAgent::Cline,
                    SkillKitAgent::Roo,
                ]
            }
            SkillKitAgent::Cursor => vec![
                SkillKitAgent::ClaudeCode,
                SkillKitAgent::Windsurf,
                SkillKitAgent::Cline,
            ],
            SkillKitAgent::Windsurf => {
                vec![SkillKitAgent::Cursor, SkillKitAgent::Continue]
            }
            SkillKitAgent::GithubCopilot => vec![SkillKitAgent::Continue, SkillKitAgent::CodeBuddy],
            SkillKitAgent::Continue => vec![
                SkillKitAgent::GithubCopilot,
                SkillKitAgent::Windsurf,
                SkillKitAgent::Cline,
            ],
            _ => Vec::new(),
        }
    }
}

impl Default for AgentDetector {
    fn default() -> Self {
        Self::new().expect("Failed to create AgentDetector")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSuggestion {
    pub skill_name: String,
    pub description: String,
    pub for_agent: SkillKitAgent,
    pub priority: f32,
    pub reason: Option<String>,
}

impl SkillSuggestion {
    pub fn new(name: &str, description: &str, agent: SkillKitAgent, priority: f32) -> Self {
        Self {
            skill_name: name.to_string(),
            description: description.to_string(),
            for_agent: agent,
            priority,
            reason: None,
        }
    }

    pub fn with_reason(mut self, reason: &str) -> Self {
        self.reason = Some(reason.to_string());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityEstimate {
    pub score: f32,
    pub confidence: ConfidenceLevel,
    pub warnings: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    High,
    Medium,
    Low,
    Unknown,
}

impl std::fmt::Display for ConfidenceLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::High => write!(f, "High"),
            Self::Medium => write!(f, "Medium"),
            Self::Low => write!(f, "Low"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detected_agent_new() {
        let agent = DetectedAgent::new(SkillKitAgent::ClaudeCode);
        assert_eq!(agent.agent, SkillKitAgent::ClaudeCode);
        assert_eq!(agent.status, AgentStatus::Unknown);
        assert!(!agent.is_available());
        assert!(!agent.has_skills());
    }

    #[test]
    fn test_detected_agent_is_available() {
        let mut agent = DetectedAgent::new(SkillKitAgent::ClaudeCode);

        agent.status = AgentStatus::Installed;
        assert!(agent.is_available());

        agent.status = AgentStatus::InstalledWithSkills;
        assert!(agent.is_available());

        agent.status = AgentStatus::NotInstalled;
        assert!(!agent.is_available());

        agent.status = AgentStatus::ConfigOnly;
        assert!(!agent.is_available());
    }

    #[test]
    fn test_detected_agent_has_skills() {
        let mut agent = DetectedAgent::new(SkillKitAgent::ClaudeCode);
        assert!(!agent.has_skills());

        agent.installed_skills_count = 5;
        assert!(agent.has_skills());
    }

    #[test]
    fn test_agent_status_display() {
        assert_eq!(AgentStatus::Installed.to_string(), "Installed");
        assert_eq!(
            AgentStatus::InstalledWithSkills.to_string(),
            "Installed (with skills)"
        );
        assert_eq!(AgentStatus::NotInstalled.to_string(), "Not installed");
        assert_eq!(AgentStatus::ConfigOnly.to_string(), "Config only");
        assert_eq!(AgentStatus::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn test_agent_mapping_new() {
        let home = PathBuf::from("/home/user");
        let mapping = AgentMapping::new(
            "Claude Code",
            SkillKitAgent::ClaudeCode,
            home.join(".claude"),
            home.join(".claude").join("skills"),
        );

        assert_eq!(mapping.rimuru_name, "Claude Code");
        assert_eq!(mapping.skillkit_agent, SkillKitAgent::ClaudeCode);
        assert!(mapping.aliases.is_empty());
    }

    #[test]
    fn test_agent_mapping_with_aliases() {
        let home = PathBuf::from("/home/user");
        let mapping = AgentMapping::new(
            "Claude Code",
            SkillKitAgent::ClaudeCode,
            home.join(".claude"),
            home.join(".claude").join("skills"),
        )
        .with_aliases(vec!["claude", "claude-code"]);

        assert_eq!(mapping.aliases.len(), 2);
        assert!(mapping.aliases.contains(&"claude".to_string()));
    }

    #[test]
    fn test_detection_result_empty() {
        let result = DetectionResult::empty();
        assert!(result.agents.is_empty());
        assert_eq!(result.total_detected, 0);
        assert_eq!(result.total_with_skills, 0);
    }

    #[test]
    fn test_detection_result_available_agents() {
        let mut result = DetectionResult::empty();

        let mut agent1 = DetectedAgent::new(SkillKitAgent::ClaudeCode);
        agent1.status = AgentStatus::Installed;

        let mut agent2 = DetectedAgent::new(SkillKitAgent::Cursor);
        agent2.status = AgentStatus::NotInstalled;

        result.agents.push(agent1);
        result.agents.push(agent2);

        let available = result.available_agents();
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].agent, SkillKitAgent::ClaudeCode);
    }

    #[test]
    fn test_skill_suggestion_new() {
        let suggestion =
            SkillSuggestion::new("test-skill", "A test skill", SkillKitAgent::ClaudeCode, 0.9);

        assert_eq!(suggestion.skill_name, "test-skill");
        assert_eq!(suggestion.description, "A test skill");
        assert_eq!(suggestion.for_agent, SkillKitAgent::ClaudeCode);
        assert!((suggestion.priority - 0.9).abs() < f32::EPSILON);
        assert!(suggestion.reason.is_none());
    }

    #[test]
    fn test_skill_suggestion_with_reason() {
        let suggestion =
            SkillSuggestion::new("test-skill", "A test skill", SkillKitAgent::ClaudeCode, 0.9)
                .with_reason("Popular among users");

        assert_eq!(suggestion.reason, Some("Popular among users".to_string()));
    }

    #[test]
    fn test_confidence_level_display() {
        assert_eq!(ConfidenceLevel::High.to_string(), "High");
        assert_eq!(ConfidenceLevel::Medium.to_string(), "Medium");
        assert_eq!(ConfidenceLevel::Low.to_string(), "Low");
        assert_eq!(ConfidenceLevel::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn test_agent_detector_resolve_agent_name() {
        let detector = AgentDetector::new().unwrap();

        assert_eq!(
            detector.resolve_agent_name("claude-code"),
            Some(SkillKitAgent::ClaudeCode)
        );
        assert_eq!(
            detector.resolve_agent_name("CURSOR"),
            Some(SkillKitAgent::Cursor)
        );
        assert_eq!(
            detector.resolve_agent_name("copilot"),
            Some(SkillKitAgent::GithubCopilot)
        );
        assert_eq!(detector.resolve_agent_name("invalid-agent"), None);
    }

    #[test]
    fn test_compatibility_estimate() {
        let estimate = CompatibilityEstimate {
            score: 0.8,
            confidence: ConfidenceLevel::Medium,
            warnings: vec!["Minor changes needed".to_string()],
            notes: vec!["Similar architecture".to_string()],
        };

        assert!((estimate.score - 0.8).abs() < f32::EPSILON);
        assert_eq!(estimate.confidence, ConfidenceLevel::Medium);
        assert_eq!(estimate.warnings.len(), 1);
        assert_eq!(estimate.notes.len(), 1);
    }
}
