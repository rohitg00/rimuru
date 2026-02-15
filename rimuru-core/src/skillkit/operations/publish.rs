use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info};

use crate::error::{RimuruError, RimuruResult};
use crate::skillkit::{PublishResult, SkillKitBridge};

#[derive(Debug, Clone)]
pub struct PublishOptions {
    pub skill_path: PathBuf,
    pub dry_run: bool,
    pub validate_only: bool,
    pub bump_version: Option<VersionBump>,
    pub force: bool,
    pub tags: Vec<String>,
}

impl PublishOptions {
    pub fn new<P: AsRef<Path>>(skill_path: P) -> Self {
        Self {
            skill_path: skill_path.as_ref().to_path_buf(),
            dry_run: false,
            validate_only: false,
            bump_version: None,
            force: false,
            tags: Vec::new(),
        }
    }

    pub fn dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }

    pub fn validate_only(mut self) -> Self {
        self.validate_only = true;
        self
    }

    pub fn with_version_bump(mut self, bump: VersionBump) -> Self {
        self.bump_version = Some(bump);
        self
    }

    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn validate(&self) -> RimuruResult<()> {
        if !self.skill_path.exists() {
            return Err(RimuruError::ValidationError(format!(
                "Skill path does not exist: {}",
                self.skill_path.display()
            )));
        }

        let skill_md = self.skill_path.join("SKILL.md");
        if !skill_md.exists() && !self.skill_path.ends_with("SKILL.md") {
            return Err(RimuruError::ValidationError(
                "No SKILL.md found in the specified path".to_string(),
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VersionBump {
    Major,
    Minor,
    Patch,
}

impl VersionBump {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Major => "major",
            Self::Minor => "minor",
            Self::Patch => "patch",
        }
    }

    pub fn apply(&self, version: &str) -> String {
        let parts: Vec<u32> = version.split('.').filter_map(|p| p.parse().ok()).collect();

        if parts.len() != 3 {
            return "1.0.0".to_string();
        }

        match self {
            Self::Major => format!("{}.0.0", parts[0] + 1),
            Self::Minor => format!("{}.{}.0", parts[0], parts[1] + 1),
            Self::Patch => format!("{}.{}.{}", parts[0], parts[1], parts[2] + 1),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub skill_name: Option<String>,
    pub skill_version: Option<String>,
}

impl ValidationResult {
    pub fn success(name: &str, version: &str) -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            skill_name: Some(name.to_string()),
            skill_version: Some(version.to_string()),
        }
    }

    pub fn failure(errors: Vec<ValidationError>) -> Self {
        Self {
            is_valid: false,
            errors,
            warnings: Vec::new(),
            skill_name: None,
            skill_version: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub code: String,
    pub message: String,
    pub location: Option<String>,
}

impl ValidationError {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            location: None,
        }
    }

    pub fn with_location(mut self, location: &str) -> Self {
        self.location = Some(location.to_string());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub code: String,
    pub message: String,
    pub suggestion: Option<String>,
}

impl ValidationWarning {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            suggestion: None,
        }
    }

    pub fn with_suggestion(mut self, suggestion: &str) -> Self {
        self.suggestion = Some(suggestion.to_string());
        self
    }
}

#[derive(Debug, Clone)]
pub enum PublishProgress {
    Started { skill_path: String },
    Validating,
    ValidationComplete { is_valid: bool, error_count: usize },
    Packaging,
    Uploading { bytes_sent: u64, total_bytes: u64 },
    Processing,
    Completed { result: PublishResult },
    Error { message: String },
}

pub struct SkillPublisher {
    bridge: Arc<SkillKitBridge>,
}

impl SkillPublisher {
    pub fn new(bridge: Arc<SkillKitBridge>) -> Self {
        Self { bridge }
    }

    pub async fn publish(&self, options: PublishOptions) -> RimuruResult<PublishResult> {
        options.validate()?;

        info!("Publishing skill from: {}", options.skill_path.display());

        if options.validate_only {
            let validation = self.validate(&options.skill_path).await?;
            if validation.is_valid {
                return Ok(PublishResult {
                    skill_name: validation.skill_name.unwrap_or_default(),
                    version: validation.skill_version.unwrap_or_default(),
                    success: true,
                    marketplace_url: None,
                    errors: Vec::new(),
                });
            } else {
                return Ok(PublishResult {
                    skill_name: String::new(),
                    version: String::new(),
                    success: false,
                    marketplace_url: None,
                    errors: validation
                        .errors
                        .iter()
                        .map(|e| e.message.clone())
                        .collect(),
                });
            }
        }

        if options.dry_run {
            return self.dry_run_publish(&options).await;
        }

        let skill_path_str = options.skill_path.to_string_lossy().to_string();
        let result = self.bridge.publish(&skill_path_str).await?;

        debug!("Publish result: {:?}", result);
        Ok(result)
    }

    pub async fn publish_with_progress(
        &self,
        options: PublishOptions,
        progress_tx: mpsc::Sender<PublishProgress>,
    ) -> RimuruResult<PublishResult> {
        let _ = progress_tx
            .send(PublishProgress::Started {
                skill_path: options.skill_path.to_string_lossy().to_string(),
            })
            .await;

        let _ = progress_tx.send(PublishProgress::Validating).await;

        let validation = self.validate(&options.skill_path).await?;
        let _ = progress_tx
            .send(PublishProgress::ValidationComplete {
                is_valid: validation.is_valid,
                error_count: validation.errors.len(),
            })
            .await;

        if !validation.is_valid {
            let error_messages: Vec<String> = validation
                .errors
                .iter()
                .map(|e| e.message.clone())
                .collect();
            let _ = progress_tx
                .send(PublishProgress::Error {
                    message: error_messages.join("; "),
                })
                .await;
            return Err(RimuruError::ValidationError(
                "Skill validation failed".to_string(),
            ));
        }

        let _ = progress_tx.send(PublishProgress::Packaging).await;
        let _ = progress_tx.send(PublishProgress::Processing).await;

        let result = self.publish(options).await;

        match &result {
            Ok(r) => {
                let _ = progress_tx
                    .send(PublishProgress::Completed { result: r.clone() })
                    .await;
            }
            Err(e) => {
                let _ = progress_tx
                    .send(PublishProgress::Error {
                        message: e.to_string(),
                    })
                    .await;
            }
        }

        result
    }

    pub async fn validate(&self, skill_path: &Path) -> RimuruResult<ValidationResult> {
        info!("Validating skill at: {}", skill_path.display());

        let skill_path_str = skill_path.to_string_lossy().to_string();
        let is_valid = self.bridge.validate_skill(&skill_path_str).await?;

        if is_valid {
            let skill_name = skill_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            Ok(ValidationResult::success(&skill_name, "1.0.0"))
        } else {
            Ok(ValidationResult::failure(vec![ValidationError::new(
                "INVALID_SKILL",
                "Skill validation failed",
            )]))
        }
    }

    pub async fn quick_publish<P: AsRef<Path>>(
        &self,
        skill_path: P,
    ) -> RimuruResult<PublishResult> {
        let options = PublishOptions::new(skill_path);
        self.publish(options).await
    }

    pub async fn check_can_publish(&self, skill_path: &Path) -> RimuruResult<bool> {
        let validation = self.validate(skill_path).await?;
        Ok(validation.is_valid)
    }

    async fn dry_run_publish(&self, options: &PublishOptions) -> RimuruResult<PublishResult> {
        info!(
            "Dry run: would publish skill from {}",
            options.skill_path.display()
        );

        let validation = self.validate(&options.skill_path).await?;

        if !validation.is_valid {
            return Ok(PublishResult {
                skill_name: String::new(),
                version: String::new(),
                success: false,
                marketplace_url: None,
                errors: validation
                    .errors
                    .iter()
                    .map(|e| e.message.clone())
                    .collect(),
            });
        }

        Ok(PublishResult {
            skill_name: validation
                .skill_name
                .unwrap_or_else(|| "test-skill".to_string()),
            version: validation
                .skill_version
                .unwrap_or_else(|| "1.0.0".to_string()),
            success: true,
            marketplace_url: Some(
                "https://agenstskills.com/skills/test-skill (dry-run)".to_string(),
            ),
            errors: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_publish_options_builder() {
        let options = PublishOptions::new("/path/to/skill")
            .dry_run()
            .with_version_bump(VersionBump::Minor)
            .with_tags(vec!["rust".to_string()]);

        assert!(options.dry_run);
        assert_eq!(options.bump_version, Some(VersionBump::Minor));
        assert_eq!(options.tags.len(), 1);
    }

    #[test]
    fn test_version_bump() {
        assert_eq!(VersionBump::Major.apply("1.2.3"), "2.0.0");
        assert_eq!(VersionBump::Minor.apply("1.2.3"), "1.3.0");
        assert_eq!(VersionBump::Patch.apply("1.2.3"), "1.2.4");
        assert_eq!(VersionBump::Patch.apply("invalid"), "1.0.0");
    }

    #[test]
    fn test_validation_result() {
        let success = ValidationResult::success("test-skill", "1.0.0");
        assert!(success.is_valid);
        assert_eq!(success.skill_name, Some("test-skill".to_string()));

        let failure = ValidationResult::failure(vec![ValidationError::new("ERR", "Error message")]);
        assert!(!failure.is_valid);
        assert_eq!(failure.errors.len(), 1);
    }

    #[test]
    fn test_validation_error() {
        let error =
            ValidationError::new("MISSING_FIELD", "Name is required").with_location("SKILL.md:1");

        assert_eq!(error.code, "MISSING_FIELD");
        assert_eq!(error.location, Some("SKILL.md:1".to_string()));
    }

    #[test]
    fn test_validation_warning() {
        let warning = ValidationWarning::new("DEPRECATED", "Feature X is deprecated")
            .with_suggestion("Use feature Y instead");

        assert_eq!(warning.code, "DEPRECATED");
        assert!(warning.suggestion.is_some());
    }

    #[test]
    fn test_publish_options_validation_missing_path() {
        let options = PublishOptions::new("/nonexistent/path");
        let result = options.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_publish_options_validation_with_skill_md() {
        let dir = tempdir().unwrap();
        let skill_md_path = dir.path().join("SKILL.md");
        fs::write(&skill_md_path, "# Test Skill").unwrap();

        let options = PublishOptions::new(dir.path());
        let result = options.validate();
        assert!(result.is_ok());
    }
}
