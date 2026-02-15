use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use crate::error::{RimuruError, RimuruResult};
use crate::skillkit::{SkillKitAgent, SkillKitBridge, TranslationResult};

#[derive(Debug, Clone)]
pub struct TranslateOptions {
    pub skill_name: String,
    pub from_agent: SkillKitAgent,
    pub to_agents: Vec<SkillKitAgent>,
    pub output_path: Option<PathBuf>,
    pub preserve_original: bool,
    pub dry_run: bool,
}

impl TranslateOptions {
    pub fn new(skill_name: &str, from: SkillKitAgent, to: SkillKitAgent) -> Self {
        Self {
            skill_name: skill_name.to_string(),
            from_agent: from,
            to_agents: vec![to],
            output_path: None,
            preserve_original: true,
            dry_run: false,
        }
    }

    pub fn to_multiple(mut self, agents: Vec<SkillKitAgent>) -> Self {
        self.to_agents = agents;
        self
    }

    pub fn with_output_path(mut self, path: PathBuf) -> Self {
        self.output_path = Some(path);
        self
    }

    pub fn replace_original(mut self) -> Self {
        self.preserve_original = false;
        self
    }

    pub fn dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }

    pub fn validate(&self) -> RimuruResult<()> {
        if self.skill_name.is_empty() {
            return Err(RimuruError::ValidationError(
                "Skill name is required".to_string(),
            ));
        }
        if self.to_agents.is_empty() {
            return Err(RimuruError::ValidationError(
                "At least one target agent is required".to_string(),
            ));
        }
        if self.to_agents.contains(&self.from_agent) {
            return Err(RimuruError::ValidationError(
                "Cannot translate to the same agent".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchTranslationResult {
    pub skill_name: String,
    pub from_agent: SkillKitAgent,
    pub results: Vec<TranslationResult>,
    pub total_duration_ms: u64,
}

impl BatchTranslationResult {
    pub fn success_count(&self) -> usize {
        self.results.iter().filter(|r| r.success).count()
    }

    pub fn failure_count(&self) -> usize {
        self.results.iter().filter(|r| !r.success).count()
    }

    pub fn is_complete_success(&self) -> bool {
        self.results.iter().all(|r| r.success)
    }

    pub fn get_warnings(&self) -> Vec<&str> {
        self.results
            .iter()
            .flat_map(|r| r.warnings.iter().map(|s| s.as_str()))
            .collect()
    }

    pub fn get_errors(&self) -> Vec<&str> {
        self.results
            .iter()
            .flat_map(|r| r.errors.iter().map(|s| s.as_str()))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub enum TranslateProgress {
    Started {
        skill_name: String,
        from_agent: SkillKitAgent,
        total_targets: usize,
    },
    AnalyzingSkill,
    TranslatingTo {
        agent: SkillKitAgent,
        current: usize,
        total: usize,
    },
    AgentCompleted {
        agent: SkillKitAgent,
        success: bool,
        warnings: Vec<String>,
    },
    Completed {
        success_count: usize,
        failure_count: usize,
        duration_ms: u64,
    },
    Error {
        message: String,
    },
}

pub struct SkillTranslator {
    bridge: Arc<SkillKitBridge>,
}

impl SkillTranslator {
    pub fn new(bridge: Arc<SkillKitBridge>) -> Self {
        Self { bridge }
    }

    pub async fn translate(
        &self,
        options: TranslateOptions,
    ) -> RimuruResult<BatchTranslationResult> {
        options.validate()?;
        let start = std::time::Instant::now();

        info!(
            "Translating skill '{}' from {} to {:?}",
            options.skill_name, options.from_agent, options.to_agents
        );

        if options.dry_run {
            return self.dry_run_translate(&options);
        }

        let mut results = Vec::new();

        for to_agent in &options.to_agents {
            let result = self
                .bridge
                .translate(&options.skill_name, options.from_agent, *to_agent)
                .await?;

            if !result.warnings.is_empty() {
                for warning in &result.warnings {
                    warn!("Translation warning for {}: {}", to_agent, warning);
                }
            }

            results.push(result);
        }

        let batch_result = BatchTranslationResult {
            skill_name: options.skill_name,
            from_agent: options.from_agent,
            results,
            total_duration_ms: start.elapsed().as_millis() as u64,
        };

        debug!(
            "Translation completed: {} success, {} failed in {}ms",
            batch_result.success_count(),
            batch_result.failure_count(),
            batch_result.total_duration_ms
        );

        Ok(batch_result)
    }

    pub async fn translate_with_progress(
        &self,
        options: TranslateOptions,
        progress_tx: mpsc::Sender<TranslateProgress>,
    ) -> RimuruResult<BatchTranslationResult> {
        options.validate()?;
        let start = std::time::Instant::now();

        let total_targets = options.to_agents.len();

        let _ = progress_tx
            .send(TranslateProgress::Started {
                skill_name: options.skill_name.clone(),
                from_agent: options.from_agent,
                total_targets,
            })
            .await;

        let _ = progress_tx.send(TranslateProgress::AnalyzingSkill).await;

        if options.dry_run {
            let result = self.dry_run_translate(&options)?;
            let _ = progress_tx
                .send(TranslateProgress::Completed {
                    success_count: 0,
                    failure_count: 0,
                    duration_ms: start.elapsed().as_millis() as u64,
                })
                .await;
            return Ok(result);
        }

        let mut results = Vec::new();

        for (idx, to_agent) in options.to_agents.iter().enumerate() {
            let _ = progress_tx
                .send(TranslateProgress::TranslatingTo {
                    agent: *to_agent,
                    current: idx + 1,
                    total: total_targets,
                })
                .await;

            let result = self
                .bridge
                .translate(&options.skill_name, options.from_agent, *to_agent)
                .await;

            match result {
                Ok(r) => {
                    let _ = progress_tx
                        .send(TranslateProgress::AgentCompleted {
                            agent: *to_agent,
                            success: r.success,
                            warnings: r.warnings.clone(),
                        })
                        .await;
                    results.push(r);
                }
                Err(e) => {
                    let _ = progress_tx
                        .send(TranslateProgress::AgentCompleted {
                            agent: *to_agent,
                            success: false,
                            warnings: vec![],
                        })
                        .await;
                    results.push(TranslationResult::failure(
                        &options.skill_name,
                        options.from_agent,
                        *to_agent,
                        &e.to_string(),
                    ));
                }
            }
        }

        let batch_result = BatchTranslationResult {
            skill_name: options.skill_name,
            from_agent: options.from_agent,
            results,
            total_duration_ms: start.elapsed().as_millis() as u64,
        };

        let _ = progress_tx
            .send(TranslateProgress::Completed {
                success_count: batch_result.success_count(),
                failure_count: batch_result.failure_count(),
                duration_ms: batch_result.total_duration_ms,
            })
            .await;

        Ok(batch_result)
    }

    pub async fn translate_single(
        &self,
        skill_name: &str,
        from: SkillKitAgent,
        to: SkillKitAgent,
    ) -> RimuruResult<TranslationResult> {
        info!("Translating skill '{}' from {} to {}", skill_name, from, to);
        self.bridge.translate(skill_name, from, to).await
    }

    pub async fn translate_to_all(
        &self,
        skill_name: &str,
        from: SkillKitAgent,
    ) -> RimuruResult<BatchTranslationResult> {
        let to_agents: Vec<SkillKitAgent> = SkillKitAgent::all()
            .iter()
            .filter(|a| **a != from)
            .copied()
            .collect();

        let options = TranslateOptions {
            skill_name: skill_name.to_string(),
            from_agent: from,
            to_agents,
            output_path: None,
            preserve_original: true,
            dry_run: false,
        };

        self.translate(options).await
    }

    pub fn get_compatible_agents(&self, from: SkillKitAgent) -> Vec<SkillKitAgent> {
        SkillKitAgent::all()
            .iter()
            .filter(|a| **a != from)
            .copied()
            .collect()
    }

    pub fn estimate_compatibility(
        &self,
        from: SkillKitAgent,
        to: SkillKitAgent,
    ) -> TranslationCompatibility {
        let high_compat_pairs = [
            (SkillKitAgent::ClaudeCode, SkillKitAgent::Cursor),
            (SkillKitAgent::ClaudeCode, SkillKitAgent::Windsurf),
            (SkillKitAgent::Cursor, SkillKitAgent::Windsurf),
            (SkillKitAgent::ClaudeCode, SkillKitAgent::Cline),
        ];

        let is_high = high_compat_pairs
            .iter()
            .any(|(a, b)| (*a == from && *b == to) || (*b == from && *a == to));

        if is_high {
            TranslationCompatibility::High
        } else if from == SkillKitAgent::Universal || to == SkillKitAgent::Universal {
            TranslationCompatibility::High
        } else {
            TranslationCompatibility::Medium
        }
    }

    fn dry_run_translate(
        &self,
        options: &TranslateOptions,
    ) -> RimuruResult<BatchTranslationResult> {
        info!(
            "Dry run: would translate {} from {} to {:?}",
            options.skill_name, options.from_agent, options.to_agents
        );

        let results = options
            .to_agents
            .iter()
            .map(|to_agent| {
                TranslationResult::success(
                    &options.skill_name,
                    options.from_agent,
                    *to_agent,
                    "[dry-run]",
                )
            })
            .collect();

        Ok(BatchTranslationResult {
            skill_name: options.skill_name.clone(),
            from_agent: options.from_agent,
            results,
            total_duration_ms: 0,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TranslationCompatibility {
    High,
    Medium,
    Low,
}

impl TranslationCompatibility {
    pub fn description(&self) -> &'static str {
        match self {
            Self::High => "Full compatibility expected with minimal changes",
            Self::Medium => "Good compatibility with some feature adjustments",
            Self::Low => "Limited compatibility, significant changes may be needed",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_options_builder() {
        let options = TranslateOptions::new(
            "test-skill",
            SkillKitAgent::ClaudeCode,
            SkillKitAgent::Cursor,
        );

        assert_eq!(options.skill_name, "test-skill");
        assert_eq!(options.from_agent, SkillKitAgent::ClaudeCode);
        assert_eq!(options.to_agents, vec![SkillKitAgent::Cursor]);
        assert!(options.preserve_original);
    }

    #[test]
    fn test_translate_options_multiple() {
        let options = TranslateOptions::new(
            "test-skill",
            SkillKitAgent::ClaudeCode,
            SkillKitAgent::Cursor,
        )
        .to_multiple(vec![
            SkillKitAgent::Cursor,
            SkillKitAgent::Windsurf,
            SkillKitAgent::Cline,
        ]);

        assert_eq!(options.to_agents.len(), 3);
    }

    #[test]
    fn test_translate_options_validation() {
        let empty_name =
            TranslateOptions::new("", SkillKitAgent::ClaudeCode, SkillKitAgent::Cursor);
        assert!(empty_name.validate().is_err());

        let same_agent =
            TranslateOptions::new("test", SkillKitAgent::ClaudeCode, SkillKitAgent::ClaudeCode);
        assert!(same_agent.validate().is_err());

        let valid = TranslateOptions::new("test", SkillKitAgent::ClaudeCode, SkillKitAgent::Cursor);
        assert!(valid.validate().is_ok());
    }

    #[test]
    fn test_batch_translation_result() {
        let results = vec![
            TranslationResult::success(
                "test",
                SkillKitAgent::ClaudeCode,
                SkillKitAgent::Cursor,
                "/path",
            ),
            TranslationResult::failure(
                "test",
                SkillKitAgent::ClaudeCode,
                SkillKitAgent::Windsurf,
                "error",
            ),
        ];

        let batch = BatchTranslationResult {
            skill_name: "test".to_string(),
            from_agent: SkillKitAgent::ClaudeCode,
            results,
            total_duration_ms: 100,
        };

        assert_eq!(batch.success_count(), 1);
        assert_eq!(batch.failure_count(), 1);
        assert!(!batch.is_complete_success());
    }

    #[test]
    fn test_translation_compatibility() {
        let compat = TranslationCompatibility::High;
        assert!(!compat.description().is_empty());
    }
}
