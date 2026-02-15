use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info};

use crate::error::RimuruResult;
use crate::skillkit::{SkillKitAgent, SkillKitBridge, SkillRecommendation};

#[derive(Debug, Clone, Default)]
pub struct RecommendOptions {
    pub workflow_description: Option<String>,
    pub context: Option<WorkflowContext>,
    pub agents: Vec<SkillKitAgent>,
    pub exclude_installed: bool,
    pub limit: Option<usize>,
    pub min_confidence: Option<f32>,
}

impl RecommendOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn for_workflow(mut self, description: &str) -> Self {
        self.workflow_description = Some(description.to_string());
        self
    }

    pub fn with_context(mut self, context: WorkflowContext) -> Self {
        self.context = Some(context);
        self
    }

    pub fn for_agents(mut self, agents: Vec<SkillKitAgent>) -> Self {
        self.agents = agents;
        self
    }

    pub fn exclude_installed(mut self) -> Self {
        self.exclude_installed = true;
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_min_confidence(mut self, confidence: f32) -> Self {
        self.min_confidence = Some(confidence.clamp(0.0, 1.0));
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkflowContext {
    pub project_type: Option<ProjectType>,
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub tools: Vec<String>,
    pub recent_activities: Vec<String>,
}

impl WorkflowContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_project_type(mut self, project_type: ProjectType) -> Self {
        self.project_type = Some(project_type);
        self
    }

    pub fn with_languages(mut self, languages: Vec<String>) -> Self {
        self.languages = languages;
        self
    }

    pub fn with_frameworks(mut self, frameworks: Vec<String>) -> Self {
        self.frameworks = frameworks;
        self
    }

    pub fn with_tools(mut self, tools: Vec<String>) -> Self {
        self.tools = tools;
        self
    }

    pub fn with_recent_activities(mut self, activities: Vec<String>) -> Self {
        self.recent_activities = activities;
        self
    }

    pub fn to_description(&self) -> String {
        let mut parts = Vec::new();

        if let Some(ref project_type) = self.project_type {
            parts.push(format!("{} project", project_type.as_str()));
        }

        if !self.languages.is_empty() {
            parts.push(format!("using {}", self.languages.join(", ")));
        }

        if !self.frameworks.is_empty() {
            parts.push(format!("with {}", self.frameworks.join(", ")));
        }

        if !self.tools.is_empty() {
            parts.push(format!("tools: {}", self.tools.join(", ")));
        }

        if parts.is_empty() {
            "general development workflow".to_string()
        } else {
            parts.join("; ")
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectType {
    Web,
    Mobile,
    Backend,
    Cli,
    Library,
    DevOps,
    DataScience,
    MachineLearning,
    GameDev,
    Embedded,
    Other,
}

impl ProjectType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Web => "web",
            Self::Mobile => "mobile",
            Self::Backend => "backend",
            Self::Cli => "CLI",
            Self::Library => "library",
            Self::DevOps => "DevOps",
            Self::DataScience => "data science",
            Self::MachineLearning => "machine learning",
            Self::GameDev => "game development",
            Self::Embedded => "embedded",
            Self::Other => "general",
        }
    }
}

#[derive(Debug, Clone)]
pub enum RecommendProgress {
    Started,
    AnalyzingWorkflow,
    FetchingRecommendations,
    FilteringResults { count: usize },
    Completed { total: usize, duration_ms: u64 },
    Error { message: String },
}

pub struct SkillRecommender {
    bridge: Arc<SkillKitBridge>,
}

impl SkillRecommender {
    pub fn new(bridge: Arc<SkillKitBridge>) -> Self {
        Self { bridge }
    }

    pub async fn recommend(
        &self,
        options: RecommendOptions,
    ) -> RimuruResult<Vec<SkillRecommendation>> {
        info!("Getting skill recommendations");

        let mut recommendations = if let Some(ref workflow) = options.workflow_description {
            self.bridge.recommend_for_workflow(workflow).await?
        } else if let Some(ref context) = options.context {
            let description = context.to_description();
            self.bridge.recommend_for_workflow(&description).await?
        } else {
            self.bridge.recommend().await?
        };

        if let Some(min_conf) = options.min_confidence {
            recommendations.retain(|r| r.confidence >= min_conf);
        }

        if !options.agents.is_empty() {
            recommendations.retain(|r| {
                r.skill.agents.is_empty()
                    || r.skill.agents.iter().any(|a| options.agents.contains(a))
            });
        }

        if options.exclude_installed {
            let installed = self.bridge.list_installed().await?;
            let installed_slugs: Vec<String> =
                installed.iter().map(|s| s.skill.slug.clone()).collect();
            recommendations.retain(|r| !installed_slugs.contains(&r.skill.slug));
        }

        recommendations.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if let Some(limit) = options.limit {
            recommendations.truncate(limit);
        }

        debug!("Got {} recommendations", recommendations.len());
        Ok(recommendations)
    }

    pub async fn recommend_with_progress(
        &self,
        options: RecommendOptions,
        progress_tx: mpsc::Sender<RecommendProgress>,
    ) -> RimuruResult<Vec<SkillRecommendation>> {
        let start = std::time::Instant::now();

        let _ = progress_tx.send(RecommendProgress::Started).await;
        let _ = progress_tx.send(RecommendProgress::AnalyzingWorkflow).await;
        let _ = progress_tx
            .send(RecommendProgress::FetchingRecommendations)
            .await;

        let result = self.recommend(options).await;

        match &result {
            Ok(recs) => {
                let _ = progress_tx
                    .send(RecommendProgress::FilteringResults { count: recs.len() })
                    .await;
                let _ = progress_tx
                    .send(RecommendProgress::Completed {
                        total: recs.len(),
                        duration_ms: start.elapsed().as_millis() as u64,
                    })
                    .await;
            }
            Err(e) => {
                let _ = progress_tx
                    .send(RecommendProgress::Error {
                        message: e.to_string(),
                    })
                    .await;
            }
        }

        result
    }

    pub async fn quick_recommend(&self, limit: usize) -> RimuruResult<Vec<SkillRecommendation>> {
        let options = RecommendOptions::new()
            .exclude_installed()
            .with_limit(limit);
        self.recommend(options).await
    }

    pub async fn recommend_for_project(
        &self,
        project_type: ProjectType,
        languages: Vec<String>,
    ) -> RimuruResult<Vec<SkillRecommendation>> {
        let context = WorkflowContext::new()
            .with_project_type(project_type)
            .with_languages(languages);

        let options = RecommendOptions::new()
            .with_context(context)
            .exclude_installed()
            .with_limit(10);

        self.recommend(options).await
    }

    pub async fn recommend_for_agent(
        &self,
        agent: SkillKitAgent,
    ) -> RimuruResult<Vec<SkillRecommendation>> {
        let options = RecommendOptions::new()
            .for_agents(vec![agent])
            .exclude_installed()
            .with_limit(10);
        self.recommend(options).await
    }

    pub async fn get_top_recommendations(
        &self,
        count: usize,
    ) -> RimuruResult<Vec<SkillRecommendation>> {
        let options = RecommendOptions::new()
            .with_min_confidence(0.7)
            .with_limit(count);
        self.recommend(options).await
    }

    pub async fn explain_recommendation(
        &self,
        recommendation: &SkillRecommendation,
    ) -> RecommendationExplanation {
        RecommendationExplanation {
            skill_name: recommendation.skill.name.clone(),
            reason: recommendation.reason.clone(),
            confidence: recommendation.confidence,
            confidence_level: ConfidenceLevel::from_score(recommendation.confidence),
            based_on: recommendation.based_on.clone(),
            compatible_agents: recommendation.skill.agents.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationExplanation {
    pub skill_name: String,
    pub reason: String,
    pub confidence: f32,
    pub confidence_level: ConfidenceLevel,
    pub based_on: Vec<String>,
    pub compatible_agents: Vec<SkillKitAgent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    High,
    Medium,
    Low,
}

impl ConfidenceLevel {
    pub fn from_score(score: f32) -> Self {
        if score >= 0.8 {
            Self::High
        } else if score >= 0.5 {
            Self::Medium
        } else {
            Self::Low
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::High => "Highly recommended based on your workflow",
            Self::Medium => "Good match for your needs",
            Self::Low => "May be useful depending on your specific use case",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recommend_options_builder() {
        let options = RecommendOptions::new()
            .for_workflow("web development with React")
            .exclude_installed()
            .with_limit(5)
            .with_min_confidence(0.7);

        assert!(options.workflow_description.is_some());
        assert!(options.exclude_installed);
        assert_eq!(options.limit, Some(5));
        assert_eq!(options.min_confidence, Some(0.7));
    }

    #[test]
    fn test_workflow_context_builder() {
        let context = WorkflowContext::new()
            .with_project_type(ProjectType::Web)
            .with_languages(vec!["TypeScript".to_string()])
            .with_frameworks(vec!["React".to_string(), "Next.js".to_string()]);

        assert_eq!(context.project_type, Some(ProjectType::Web));
        assert_eq!(context.languages.len(), 1);
        assert_eq!(context.frameworks.len(), 2);
    }

    #[test]
    fn test_workflow_context_to_description() {
        let context = WorkflowContext::new()
            .with_project_type(ProjectType::Web)
            .with_languages(vec!["TypeScript".to_string()])
            .with_frameworks(vec!["React".to_string()]);

        let description = context.to_description();
        assert!(description.contains("web"));
        assert!(description.contains("TypeScript"));
        assert!(description.contains("React"));
    }

    #[test]
    fn test_confidence_level() {
        assert_eq!(ConfidenceLevel::from_score(0.9), ConfidenceLevel::High);
        assert_eq!(ConfidenceLevel::from_score(0.6), ConfidenceLevel::Medium);
        assert_eq!(ConfidenceLevel::from_score(0.3), ConfidenceLevel::Low);
    }

    #[test]
    fn test_project_type() {
        assert_eq!(ProjectType::Web.as_str(), "web");
        assert_eq!(ProjectType::Backend.as_str(), "backend");
        assert_eq!(ProjectType::MachineLearning.as_str(), "machine learning");
    }

    #[test]
    fn test_min_confidence_clamping() {
        let options = RecommendOptions::new().with_min_confidence(1.5);
        assert_eq!(options.min_confidence, Some(1.0));

        let options = RecommendOptions::new().with_min_confidence(-0.5);
        assert_eq!(options.min_confidence, Some(0.0));
    }
}
