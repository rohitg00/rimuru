use crate::state::AppState;
use rimuru_core::skillkit::{
    InstalledSkill, PublishResult, SearchFilters, Skill, SkillKitAgent, SkillRecommendation,
    SkillSearchResult, TranslationResult,
};
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{debug, error, info};

#[derive(Debug, Serialize)]
pub struct SkillResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub tags: Vec<String>,
    pub source: String,
    pub downloads: u64,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Skill> for SkillResponse {
    fn from(s: Skill) -> Self {
        SkillResponse {
            id: s.slug.clone(),
            name: s.name,
            slug: s.slug,
            description: s.description,
            author: s.author.unwrap_or_default(),
            version: s.version.unwrap_or_else(|| "1.0.0".to_string()),
            tags: s.tags,
            source: s.source.unwrap_or_default(),
            downloads: s.downloads.unwrap_or(0),
            created_at: s.created_at.map(|dt| dt.to_rfc3339()).unwrap_or_default(),
            updated_at: s.updated_at.map(|dt| dt.to_rfc3339()).unwrap_or_default(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct InstalledSkillResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub tags: Vec<String>,
    pub source: String,
    pub downloads: u64,
    pub created_at: String,
    pub updated_at: String,
    pub installed_at: String,
    pub enabled: bool,
    pub agents: Vec<String>,
    pub install_path: String,
}

impl From<InstalledSkill> for InstalledSkillResponse {
    fn from(is: InstalledSkill) -> Self {
        InstalledSkillResponse {
            id: is.skill.slug.clone(),
            name: is.skill.name,
            slug: is.skill.slug,
            description: is.skill.description,
            author: is.skill.author.unwrap_or_default(),
            version: is.skill.version.unwrap_or_else(|| "1.0.0".to_string()),
            tags: is.skill.tags,
            source: is.skill.source.unwrap_or_default(),
            downloads: is.skill.downloads.unwrap_or(0),
            created_at: is
                .skill
                .created_at
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default(),
            updated_at: is
                .skill
                .updated_at
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default(),
            installed_at: is.installed_at.to_rfc3339(),
            enabled: is.enabled,
            agents: is.installed_for.iter().map(|a| a.to_string()).collect(),
            install_path: is.path,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SkillSearchResponse {
    pub skills: Vec<SkillResponse>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
}

impl From<SkillSearchResult> for SkillSearchResponse {
    fn from(sr: SkillSearchResult) -> Self {
        SkillSearchResponse {
            skills: sr.skills.into_iter().map(SkillResponse::from).collect(),
            total: sr.total,
            page: sr.page,
            per_page: sr.per_page,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SkillRecommendationResponse {
    pub skill: SkillResponse,
    pub confidence: f32,
    pub reason: String,
}

impl From<SkillRecommendation> for SkillRecommendationResponse {
    fn from(rec: SkillRecommendation) -> Self {
        SkillRecommendationResponse {
            skill: SkillResponse::from(rec.skill),
            confidence: rec.confidence,
            reason: rec.reason,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TranslationResponse {
    pub success: bool,
    pub original_agent: String,
    pub target_agent: String,
    pub output_path: String,
    pub warnings: Vec<String>,
}

impl From<TranslationResult> for TranslationResponse {
    fn from(tr: TranslationResult) -> Self {
        TranslationResponse {
            success: tr.success,
            original_agent: tr.from_agent.to_string(),
            target_agent: tr.to_agent.to_string(),
            output_path: tr.output_path.unwrap_or_default(),
            warnings: tr.warnings,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SkillSearchFilters {
    pub query: Option<String>,
    pub agent: Option<String>,
    pub tags: Option<Vec<String>>,
    pub limit: Option<usize>,
    pub page: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct SkillInstallRequest {
    pub skill_id: String,
    pub agents: Vec<String>,
    pub install_all: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SkillTranslateRequest {
    pub skill_id: String,
    pub from_agent: String,
    pub to_agent: String,
}

fn parse_agent(agent_str: &str) -> Result<SkillKitAgent, String> {
    SkillKitAgent::parse(agent_str).ok_or_else(|| format!("Unknown agent: {}", agent_str))
}

fn parse_agents(agent_strs: &[String]) -> Result<Vec<SkillKitAgent>, String> {
    agent_strs.iter().map(|s| parse_agent(s)).collect()
}

#[tauri::command]
pub async fn search_skills(
    state: State<'_, AppState>,
    filters: Option<SkillSearchFilters>,
) -> Result<SkillSearchResponse, String> {
    info!("Searching skills with filters: {:?}", filters);

    let bridge = state.skillkit_bridge.read().await;
    let bridge = bridge.as_ref().ok_or("SkillKit not initialized")?;

    let query = filters
        .as_ref()
        .and_then(|f| f.query.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("");

    let search_filters = filters.as_ref().map(|f| {
        let mut sf = SearchFilters::default();
        if let Some(agent_str) = &f.agent {
            sf.agent = SkillKitAgent::parse(agent_str);
        }
        if let Some(tags) = &f.tags {
            sf.tags = tags.clone();
        }
        sf
    });

    let result = bridge
        .search(query, search_filters)
        .await
        .map_err(|e| e.to_string())?;

    debug!("Search returned {} skills", result.skills.len());
    Ok(SkillSearchResponse::from(result))
}

#[tauri::command]
pub async fn get_installed_skills(
    state: State<'_, AppState>,
    agent: Option<String>,
) -> Result<Vec<InstalledSkillResponse>, String> {
    info!("Getting installed skills for agent: {:?}", agent);

    let bridge = state.skillkit_bridge.read().await;
    let bridge = bridge.as_ref().ok_or("SkillKit not initialized")?;

    let skills = if let Some(agent_str) = agent {
        let agent = parse_agent(&agent_str)?;
        bridge
            .list_for_agent(agent)
            .await
            .map_err(|e| e.to_string())?
    } else {
        bridge.list_installed().await.map_err(|e| e.to_string())?
    };

    debug!("Found {} installed skills", skills.len());
    Ok(skills
        .into_iter()
        .map(InstalledSkillResponse::from)
        .collect())
}

#[tauri::command]
pub async fn get_skill_details(
    state: State<'_, AppState>,
    skill_id: String,
) -> Result<Option<SkillResponse>, String> {
    info!("Getting skill details for: {}", skill_id);

    let bridge = state.skillkit_bridge.read().await;
    let bridge = bridge.as_ref().ok_or("SkillKit not initialized")?;

    match bridge.get_skill_details(&skill_id).await {
        Ok(skill) => {
            debug!("Found skill: {}", skill.name);
            Ok(Some(SkillResponse::from(skill)))
        }
        Err(e) => {
            debug!("Skill not found: {}", e);
            Ok(None)
        }
    }
}

#[tauri::command]
pub async fn install_skill(
    state: State<'_, AppState>,
    request: SkillInstallRequest,
) -> Result<InstalledSkillResponse, String> {
    info!(
        "Installing skill {} for agents: {:?}",
        request.skill_id, request.agents
    );

    let bridge = state.skillkit_bridge.read().await;
    let bridge = bridge.as_ref().ok_or("SkillKit not initialized")?;

    let installed = if request.install_all.unwrap_or(false) {
        bridge
            .install_for_all(&request.skill_id)
            .await
            .map_err(|e| e.to_string())?
    } else {
        let agents = parse_agents(&request.agents)?;
        bridge
            .install(&request.skill_id, &agents)
            .await
            .map_err(|e| e.to_string())?
    };

    info!("Successfully installed skill: {}", request.skill_id);
    Ok(InstalledSkillResponse::from(installed))
}

#[tauri::command]
pub async fn uninstall_skill(
    state: State<'_, AppState>,
    skill_id: String,
    _agent: Option<String>,
) -> Result<bool, String> {
    info!("Uninstalling skill: {}", skill_id);

    let bridge = state.skillkit_bridge.read().await;
    let bridge = bridge.as_ref().ok_or("SkillKit not initialized")?;

    bridge
        .uninstall(&skill_id)
        .await
        .map_err(|e| e.to_string())?;

    info!("Successfully uninstalled skill: {}", skill_id);
    Ok(true)
}

#[tauri::command]
pub async fn translate_skill(
    state: State<'_, AppState>,
    request: SkillTranslateRequest,
) -> Result<TranslationResponse, String> {
    info!(
        "Translating skill {} from {} to {}",
        request.skill_id, request.from_agent, request.to_agent
    );

    let bridge = state.skillkit_bridge.read().await;
    let bridge = bridge.as_ref().ok_or("SkillKit not initialized")?;

    let from_agent = parse_agent(&request.from_agent)?;
    let to_agent = parse_agent(&request.to_agent)?;

    let result = bridge
        .translate(&request.skill_id, from_agent, to_agent)
        .await
        .map_err(|e| e.to_string())?;

    if result.success {
        info!("Successfully translated skill: {}", request.skill_id);
    } else {
        error!(
            "Failed to translate skill: {} - {:?}",
            request.skill_id, result.errors
        );
    }

    Ok(TranslationResponse::from(result))
}

#[tauri::command]
pub async fn get_skill_recommendations(
    state: State<'_, AppState>,
    workflow: Option<String>,
) -> Result<Vec<SkillRecommendationResponse>, String> {
    info!("Getting skill recommendations for workflow: {:?}", workflow);

    let bridge = state.skillkit_bridge.read().await;
    let bridge = bridge.as_ref().ok_or("SkillKit not initialized")?;

    let recommendations = if let Some(workflow_desc) = workflow {
        bridge
            .recommend_for_workflow(&workflow_desc)
            .await
            .map_err(|e| e.to_string())?
    } else {
        bridge.recommend().await.map_err(|e| e.to_string())?
    };

    debug!("Got {} recommendations", recommendations.len());
    Ok(recommendations
        .into_iter()
        .map(SkillRecommendationResponse::from)
        .collect())
}

#[tauri::command]
pub async fn publish_skill(
    state: State<'_, AppState>,
    skill_path: String,
) -> Result<PublishResultResponse, String> {
    info!("Publishing skill from path: {}", skill_path);

    let bridge = state.skillkit_bridge.read().await;
    let bridge = bridge.as_ref().ok_or("SkillKit not initialized")?;

    let result = bridge
        .publish(&skill_path)
        .await
        .map_err(|e| e.to_string())?;

    if result.success {
        info!("Successfully published skill: {}", result.skill_name);
    } else {
        error!(
            "Failed to publish skill: {} - {:?}",
            result.skill_name, result.errors
        );
    }

    Ok(PublishResultResponse::from(result))
}

#[derive(Debug, Serialize)]
pub struct PublishResultResponse {
    pub skill_name: String,
    pub version: String,
    pub success: bool,
    pub marketplace_url: Option<String>,
    pub errors: Vec<String>,
}

impl From<PublishResult> for PublishResultResponse {
    fn from(pr: PublishResult) -> Self {
        PublishResultResponse {
            skill_name: pr.skill_name,
            version: pr.version,
            success: pr.success,
            marketplace_url: pr.marketplace_url,
            errors: pr.errors,
        }
    }
}

#[tauri::command]
pub async fn enable_skill(
    state: State<'_, AppState>,
    skill_id: String,
    _agent: Option<String>,
) -> Result<bool, String> {
    info!("Enabling skill: {}", skill_id);

    let mut bridge = state.skillkit_bridge.write().await;
    let bridge = bridge.as_mut().ok_or("SkillKit not initialized")?;

    let config_manager = bridge.config_manager_mut();
    config_manager
        .enable_skill(&skill_id)
        .map_err(|e| e.to_string())?;

    info!("Successfully enabled skill: {}", skill_id);
    Ok(true)
}

#[tauri::command]
pub async fn disable_skill(
    state: State<'_, AppState>,
    skill_id: String,
    _agent: Option<String>,
) -> Result<bool, String> {
    info!("Disabling skill: {}", skill_id);

    let mut bridge = state.skillkit_bridge.write().await;
    let bridge = bridge.as_mut().ok_or("SkillKit not initialized")?;

    let config_manager = bridge.config_manager_mut();
    config_manager
        .disable_skill(&skill_id)
        .map_err(|e| e.to_string())?;

    info!("Successfully disabled skill: {}", skill_id);
    Ok(true)
}

#[tauri::command]
pub async fn sync_skills(state: State<'_, AppState>) -> Result<bool, String> {
    info!("Syncing skills with marketplace");

    let bridge = state.skillkit_bridge.read().await;
    let bridge = bridge.as_ref().ok_or("SkillKit not initialized")?;

    bridge.sync().await.map_err(|e| e.to_string())?;

    info!("Successfully synced skills");
    Ok(true)
}

#[tauri::command]
pub async fn get_skillkit_status(
    state: State<'_, AppState>,
) -> Result<SkillKitStatusResponse, String> {
    let bridge = state.skillkit_bridge.read().await;

    match bridge.as_ref() {
        Some(b) => {
            let info = b.skillkit_info();
            Ok(SkillKitStatusResponse {
                installed: info.installed,
                available: b.is_available(),
                version: info.version.clone(),
                path: info.path.clone(),
            })
        }
        None => Ok(SkillKitStatusResponse {
            installed: false,
            available: false,
            version: None,
            path: None,
        }),
    }
}

#[derive(Debug, Serialize)]
pub struct SkillKitStatusResponse {
    pub installed: bool,
    pub available: bool,
    pub version: Option<String>,
    pub path: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_agent() {
        assert!(parse_agent("claude-code").is_ok());
        assert!(parse_agent("cursor").is_ok());
        assert!(parse_agent("invalid-agent").is_err());
    }

    #[test]
    fn test_parse_agents() {
        let agents = vec!["claude-code".to_string(), "cursor".to_string()];
        let result = parse_agents(&agents);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_skill_response_from_skill() {
        use rimuru_core::skillkit::Skill;

        let skill = Skill::new("Test Skill", "A test skill");
        let response = SkillResponse::from(skill);

        assert_eq!(response.name, "Test Skill");
        assert_eq!(response.slug, "test-skill");
        assert_eq!(response.description, "A test skill");
    }
}
