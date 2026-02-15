use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::adapters::{ActiveSession, AdapterRegistry, SessionHistory};
use crate::error::RimuruResult;
use crate::models::AgentType;

use super::cost_aggregator::TimeRange;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSession {
    pub session_id: Uuid,
    pub adapter_name: String,
    pub agent_type: AgentType,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_tokens: i64,
    pub model_name: Option<String>,
    pub cost_usd: Option<f64>,
    pub project_path: Option<String>,
    pub duration_seconds: Option<i64>,
}

impl UnifiedSession {
    pub fn from_active(adapter_name: String, active: &ActiveSession) -> Self {
        Self {
            session_id: active.session_id,
            adapter_name,
            agent_type: active.agent_type,
            started_at: active.started_at,
            ended_at: None,
            is_active: true,
            total_input_tokens: active.current_tokens,
            total_output_tokens: 0,
            total_tokens: active.current_tokens,
            model_name: active.model_name.clone(),
            cost_usd: None,
            project_path: active.project_path.clone(),
            duration_seconds: Some(active.duration_seconds()),
        }
    }

    pub fn from_history(adapter_name: String, history: &SessionHistory) -> Self {
        Self {
            session_id: history.session_id,
            adapter_name,
            agent_type: history.agent_type,
            started_at: history.started_at,
            ended_at: history.ended_at,
            is_active: false,
            total_input_tokens: history.total_input_tokens,
            total_output_tokens: history.total_output_tokens,
            total_tokens: history.total_tokens(),
            model_name: history.model_name.clone(),
            cost_usd: history.cost_usd,
            project_path: history.project_path.clone(),
            duration_seconds: history.duration_seconds(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub completed_sessions: usize,
    pub total_duration_seconds: i64,
    pub average_duration_seconds: f64,
    pub total_tokens: i64,
    pub average_tokens_per_session: f64,
    pub total_cost: f64,
    pub average_cost_per_session: f64,
}

impl Default for SessionStats {
    fn default() -> Self {
        Self {
            total_sessions: 0,
            active_sessions: 0,
            completed_sessions: 0,
            total_duration_seconds: 0,
            average_duration_seconds: 0.0,
            total_tokens: 0,
            average_tokens_per_session: 0.0,
            total_cost: 0.0,
            average_cost_per_session: 0.0,
        }
    }
}

impl SessionStats {
    pub fn add_session(&mut self, session: &UnifiedSession) {
        self.total_sessions += 1;

        if session.is_active {
            self.active_sessions += 1;
        } else {
            self.completed_sessions += 1;
        }

        if let Some(duration) = session.duration_seconds {
            self.total_duration_seconds += duration;
        }

        self.total_tokens += session.total_tokens;

        if let Some(cost) = session.cost_usd {
            self.total_cost += cost;
        }

        self.recalculate_averages();
    }

    fn recalculate_averages(&mut self) {
        if self.total_sessions > 0 {
            self.average_duration_seconds =
                self.total_duration_seconds as f64 / self.total_sessions as f64;
            self.average_tokens_per_session = self.total_tokens as f64 / self.total_sessions as f64;
            self.average_cost_per_session = self.total_cost / self.total_sessions as f64;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionReport {
    pub time_range: TimeRange,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: DateTime<Utc>,
    pub stats: SessionStats,
    pub by_agent: HashMap<String, SessionStats>,
    pub by_agent_type: HashMap<AgentType, SessionStats>,
    pub by_model: HashMap<String, SessionStats>,
    pub sessions: Vec<UnifiedSession>,
}

impl SessionReport {
    pub fn new(time_range: TimeRange, start_time: Option<DateTime<Utc>>) -> Self {
        Self {
            time_range,
            start_time,
            end_time: Utc::now(),
            stats: SessionStats::default(),
            by_agent: HashMap::new(),
            by_agent_type: HashMap::new(),
            by_model: HashMap::new(),
            sessions: Vec::new(),
        }
    }

    pub fn add_session(&mut self, session: UnifiedSession) {
        self.stats.add_session(&session);

        self.by_agent
            .entry(session.adapter_name.clone())
            .or_default()
            .add_session(&session);

        self.by_agent_type
            .entry(session.agent_type)
            .or_default()
            .add_session(&session);

        if let Some(ref model) = session.model_name {
            self.by_model
                .entry(model.clone())
                .or_default()
                .add_session(&session);
        }

        self.sessions.push(session);
    }

    pub fn sort_by_start_time_desc(&mut self) {
        self.sessions
            .sort_by(|a, b| b.started_at.cmp(&a.started_at));
    }

    pub fn sort_by_cost_desc(&mut self) {
        self.sessions.sort_by(|a, b| {
            let cost_a = a.cost_usd.unwrap_or(0.0);
            let cost_b = b.cost_usd.unwrap_or(0.0);
            cost_b
                .partial_cmp(&cost_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    pub fn sort_by_tokens_desc(&mut self) {
        self.sessions
            .sort_by(|a, b| b.total_tokens.cmp(&a.total_tokens));
    }
}

#[derive(Debug, Clone, Default)]
pub struct SessionFilter {
    pub adapter_names: Option<Vec<String>>,
    pub agent_types: Option<Vec<AgentType>>,
    pub models: Option<Vec<String>>,
    pub active_only: bool,
    pub completed_only: bool,
    pub min_duration_seconds: Option<i64>,
    pub max_duration_seconds: Option<i64>,
    pub min_tokens: Option<i64>,
    pub project_path_contains: Option<String>,
}

impl SessionFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_adapters(mut self, adapters: Vec<String>) -> Self {
        self.adapter_names = Some(adapters);
        self
    }

    pub fn with_agent_types(mut self, types: Vec<AgentType>) -> Self {
        self.agent_types = Some(types);
        self
    }

    pub fn with_models(mut self, models: Vec<String>) -> Self {
        self.models = Some(models);
        self
    }

    pub fn active_only(mut self) -> Self {
        self.active_only = true;
        self.completed_only = false;
        self
    }

    pub fn completed_only(mut self) -> Self {
        self.completed_only = true;
        self.active_only = false;
        self
    }

    pub fn with_min_duration(mut self, seconds: i64) -> Self {
        self.min_duration_seconds = Some(seconds);
        self
    }

    pub fn with_max_duration(mut self, seconds: i64) -> Self {
        self.max_duration_seconds = Some(seconds);
        self
    }

    pub fn with_min_tokens(mut self, tokens: i64) -> Self {
        self.min_tokens = Some(tokens);
        self
    }

    pub fn with_project_path(mut self, path_contains: String) -> Self {
        self.project_path_contains = Some(path_contains);
        self
    }

    pub fn matches(&self, session: &UnifiedSession) -> bool {
        if let Some(ref adapters) = self.adapter_names {
            if !adapters.contains(&session.adapter_name) {
                return false;
            }
        }

        if let Some(ref types) = self.agent_types {
            if !types.contains(&session.agent_type) {
                return false;
            }
        }

        if let Some(ref models) = self.models {
            if let Some(ref model) = session.model_name {
                if !models.contains(model) {
                    return false;
                }
            } else {
                return false;
            }
        }

        if self.active_only && !session.is_active {
            return false;
        }

        if self.completed_only && session.is_active {
            return false;
        }

        if let Some(min_duration) = self.min_duration_seconds {
            if let Some(duration) = session.duration_seconds {
                if duration < min_duration {
                    return false;
                }
            } else {
                return false;
            }
        }

        if let Some(max_duration) = self.max_duration_seconds {
            if let Some(duration) = session.duration_seconds {
                if duration > max_duration {
                    return false;
                }
            }
        }

        if let Some(min_tokens) = self.min_tokens {
            if session.total_tokens < min_tokens {
                return false;
            }
        }

        if let Some(ref path_contains) = self.project_path_contains {
            if let Some(ref project_path) = session.project_path {
                if !project_path.contains(path_contains) {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

pub struct SessionAggregator {
    registry: Arc<AdapterRegistry>,
}

impl SessionAggregator {
    pub fn new(registry: Arc<AdapterRegistry>) -> Self {
        Self { registry }
    }

    pub async fn get_all_active_sessions(&self) -> RimuruResult<Vec<UnifiedSession>> {
        let adapter_names = self.registry.list_names().await;
        let mut all_sessions = Vec::new();

        for name in adapter_names {
            if let Some(adapter) = self.registry.get(&name).await {
                let adapter_guard = adapter.read().await;

                if let Ok(sessions) = adapter_guard.get_active_sessions().await {
                    for session in sessions {
                        all_sessions.push(UnifiedSession::from_active(name.clone(), &session));
                    }
                }
            }
        }

        all_sessions.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        Ok(all_sessions)
    }

    pub async fn get_active_session_count(&self) -> usize {
        self.registry.get_all_active_sessions().await.len()
    }

    pub async fn get_session_history(
        &self,
        limit: Option<usize>,
        time_range: TimeRange,
    ) -> RimuruResult<Vec<UnifiedSession>> {
        let since = time_range.to_datetime();
        self.get_session_history_since(limit, since).await
    }

    pub async fn get_session_history_since(
        &self,
        limit: Option<usize>,
        since: Option<DateTime<Utc>>,
    ) -> RimuruResult<Vec<UnifiedSession>> {
        let adapter_names = self.registry.list_names().await;
        let mut all_sessions = Vec::new();

        for name in adapter_names {
            if let Some(adapter) = self.registry.get(&name).await {
                let adapter_guard = adapter.read().await;

                if let Ok(history) = adapter_guard.get_session_history(None, since).await {
                    for session in history {
                        all_sessions.push(UnifiedSession::from_history(name.clone(), &session));
                    }
                }
            }
        }

        all_sessions.sort_by(|a, b| b.started_at.cmp(&a.started_at));

        if let Some(limit) = limit {
            all_sessions.truncate(limit);
        }

        Ok(all_sessions)
    }

    pub async fn get_session_by_id(
        &self,
        session_id: Uuid,
    ) -> RimuruResult<Option<UnifiedSession>> {
        if let Some(history) = self.registry.find_session(session_id).await {
            let adapter_names = self.registry.list_names().await;
            let adapter_name = adapter_names.first().cloned().unwrap_or_default();
            return Ok(Some(UnifiedSession::from_history(adapter_name, &history)));
        }

        Ok(None)
    }

    pub async fn get_session_report(&self, time_range: TimeRange) -> RimuruResult<SessionReport> {
        let since = time_range.to_datetime();
        self.get_session_report_since(time_range, since).await
    }

    pub async fn get_session_report_since(
        &self,
        time_range: TimeRange,
        since: Option<DateTime<Utc>>,
    ) -> RimuruResult<SessionReport> {
        let mut report = SessionReport::new(time_range, since);

        let active_sessions = self.get_all_active_sessions().await?;
        for session in active_sessions {
            if let Some(start) = since {
                if session.started_at >= start {
                    report.add_session(session);
                }
            } else {
                report.add_session(session);
            }
        }

        let history = self.get_session_history_since(None, since).await?;
        for session in history {
            report.add_session(session);
        }

        report.sort_by_start_time_desc();

        Ok(report)
    }

    pub async fn get_filtered_sessions(
        &self,
        time_range: TimeRange,
        filter: SessionFilter,
    ) -> RimuruResult<Vec<UnifiedSession>> {
        let report = self.get_session_report(time_range).await?;

        let filtered: Vec<UnifiedSession> = report
            .sessions
            .into_iter()
            .filter(|s| filter.matches(s))
            .collect();

        Ok(filtered)
    }

    pub async fn get_filtered_report(
        &self,
        time_range: TimeRange,
        filter: SessionFilter,
    ) -> RimuruResult<SessionReport> {
        let full_report = self.get_session_report(time_range).await?;

        let mut filtered_report = SessionReport::new(time_range, full_report.start_time);

        for session in full_report.sessions {
            if filter.matches(&session) {
                filtered_report.add_session(session);
            }
        }

        Ok(filtered_report)
    }

    pub async fn get_sessions_by_agent(
        &self,
        adapter_name: &str,
    ) -> RimuruResult<Vec<UnifiedSession>> {
        let filter = SessionFilter::new().with_adapters(vec![adapter_name.to_string()]);
        self.get_filtered_sessions(TimeRange::AllTime, filter).await
    }

    pub async fn get_sessions_by_agent_type(
        &self,
        agent_type: AgentType,
    ) -> RimuruResult<Vec<UnifiedSession>> {
        let filter = SessionFilter::new().with_agent_types(vec![agent_type]);
        self.get_filtered_sessions(TimeRange::AllTime, filter).await
    }

    pub async fn get_sessions_by_project(
        &self,
        project_path: &str,
    ) -> RimuruResult<Vec<UnifiedSession>> {
        let filter = SessionFilter::new().with_project_path(project_path.to_string());
        self.get_filtered_sessions(TimeRange::AllTime, filter).await
    }

    pub async fn get_recent_sessions(&self, count: usize) -> RimuruResult<Vec<UnifiedSession>> {
        self.get_session_history_since(Some(count), None).await
    }

    pub async fn get_sessions_last_n_days(&self, days: i64) -> RimuruResult<Vec<UnifiedSession>> {
        let since = Some(Utc::now() - Duration::days(days));
        self.get_session_history_since(None, since).await
    }

    pub async fn get_stats(&self, time_range: TimeRange) -> RimuruResult<SessionStats> {
        let report = self.get_session_report(time_range).await?;
        Ok(report.stats)
    }

    pub async fn get_stats_by_agent(
        &self,
        time_range: TimeRange,
    ) -> RimuruResult<HashMap<String, SessionStats>> {
        let report = self.get_session_report(time_range).await?;
        Ok(report.by_agent)
    }

    pub async fn get_stats_by_agent_type(
        &self,
        time_range: TimeRange,
    ) -> RimuruResult<HashMap<AgentType, SessionStats>> {
        let report = self.get_session_report(time_range).await?;
        Ok(report.by_agent_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_session_from_active() {
        let active = ActiveSession {
            session_id: Uuid::new_v4(),
            agent_type: AgentType::ClaudeCode,
            started_at: Utc::now() - Duration::minutes(30),
            current_tokens: 5000,
            model_name: Some("claude-3-opus".to_string()),
            project_path: Some("/home/user/project".to_string()),
        };

        let unified = UnifiedSession::from_active("claude-1".to_string(), &active);

        assert!(unified.is_active);
        assert_eq!(unified.adapter_name, "claude-1");
        assert_eq!(unified.agent_type, AgentType::ClaudeCode);
        assert_eq!(unified.total_tokens, 5000);
        assert!(unified.duration_seconds.is_some());
    }

    #[test]
    fn test_unified_session_from_history() {
        let history = SessionHistory {
            session_id: Uuid::new_v4(),
            agent_type: AgentType::OpenCode,
            started_at: Utc::now() - Duration::hours(2),
            ended_at: Some(Utc::now() - Duration::hours(1)),
            total_input_tokens: 10000,
            total_output_tokens: 5000,
            model_name: Some("gpt-4".to_string()),
            cost_usd: Some(0.50),
            project_path: Some("/home/user/project".to_string()),
        };

        let unified = UnifiedSession::from_history("opencode-1".to_string(), &history);

        assert!(!unified.is_active);
        assert_eq!(unified.adapter_name, "opencode-1");
        assert_eq!(unified.total_tokens, 15000);
        assert_eq!(unified.cost_usd, Some(0.50));
    }

    #[test]
    fn test_session_stats() {
        let mut stats = SessionStats::default();

        let session1 = UnifiedSession {
            session_id: Uuid::new_v4(),
            adapter_name: "test".to_string(),
            agent_type: AgentType::ClaudeCode,
            started_at: Utc::now(),
            ended_at: Some(Utc::now()),
            is_active: false,
            total_input_tokens: 1000,
            total_output_tokens: 500,
            total_tokens: 1500,
            model_name: None,
            cost_usd: Some(0.10),
            project_path: None,
            duration_seconds: Some(3600),
        };

        stats.add_session(&session1);

        assert_eq!(stats.total_sessions, 1);
        assert_eq!(stats.completed_sessions, 1);
        assert_eq!(stats.active_sessions, 0);
        assert_eq!(stats.total_tokens, 1500);
        assert!((stats.total_cost - 0.10).abs() < 0.001);
    }

    #[test]
    fn test_session_filter() {
        let filter = SessionFilter::new()
            .with_agent_types(vec![AgentType::ClaudeCode])
            .completed_only()
            .with_min_tokens(1000);

        let session1 = UnifiedSession {
            session_id: Uuid::new_v4(),
            adapter_name: "claude-1".to_string(),
            agent_type: AgentType::ClaudeCode,
            started_at: Utc::now(),
            ended_at: Some(Utc::now()),
            is_active: false,
            total_input_tokens: 1000,
            total_output_tokens: 500,
            total_tokens: 1500,
            model_name: None,
            cost_usd: None,
            project_path: None,
            duration_seconds: Some(3600),
        };

        let session2 = UnifiedSession {
            session_id: Uuid::new_v4(),
            adapter_name: "opencode-1".to_string(),
            agent_type: AgentType::OpenCode,
            started_at: Utc::now(),
            ended_at: Some(Utc::now()),
            is_active: false,
            total_input_tokens: 2000,
            total_output_tokens: 1000,
            total_tokens: 3000,
            model_name: None,
            cost_usd: None,
            project_path: None,
            duration_seconds: Some(3600),
        };

        let session3 = UnifiedSession {
            session_id: Uuid::new_v4(),
            adapter_name: "claude-2".to_string(),
            agent_type: AgentType::ClaudeCode,
            started_at: Utc::now(),
            ended_at: None,
            is_active: true,
            total_input_tokens: 500,
            total_output_tokens: 0,
            total_tokens: 500,
            model_name: None,
            cost_usd: None,
            project_path: None,
            duration_seconds: Some(600),
        };

        assert!(filter.matches(&session1));
        assert!(!filter.matches(&session2));
        assert!(!filter.matches(&session3));
    }

    #[test]
    fn test_session_filter_project_path() {
        let filter = SessionFilter::new().with_project_path("myproject".to_string());

        let session1 = UnifiedSession {
            session_id: Uuid::new_v4(),
            adapter_name: "test".to_string(),
            agent_type: AgentType::ClaudeCode,
            started_at: Utc::now(),
            ended_at: None,
            is_active: true,
            total_input_tokens: 0,
            total_output_tokens: 0,
            total_tokens: 0,
            model_name: None,
            cost_usd: None,
            project_path: Some("/home/user/myproject".to_string()),
            duration_seconds: None,
        };

        let session2 = UnifiedSession {
            session_id: Uuid::new_v4(),
            adapter_name: "test".to_string(),
            agent_type: AgentType::ClaudeCode,
            started_at: Utc::now(),
            ended_at: None,
            is_active: true,
            total_input_tokens: 0,
            total_output_tokens: 0,
            total_tokens: 0,
            model_name: None,
            cost_usd: None,
            project_path: Some("/home/user/other".to_string()),
            duration_seconds: None,
        };

        assert!(filter.matches(&session1));
        assert!(!filter.matches(&session2));
    }

    #[test]
    fn test_session_report() {
        let mut report = SessionReport::new(TimeRange::Today, Some(Utc::now()));

        let session = UnifiedSession {
            session_id: Uuid::new_v4(),
            adapter_name: "claude-1".to_string(),
            agent_type: AgentType::ClaudeCode,
            started_at: Utc::now(),
            ended_at: Some(Utc::now()),
            is_active: false,
            total_input_tokens: 1000,
            total_output_tokens: 500,
            total_tokens: 1500,
            model_name: Some("claude-3-opus".to_string()),
            cost_usd: Some(0.10),
            project_path: None,
            duration_seconds: Some(3600),
        };

        report.add_session(session);

        assert_eq!(report.sessions.len(), 1);
        assert_eq!(report.stats.total_sessions, 1);
        assert!(report.by_agent.contains_key("claude-1"));
        assert!(report.by_agent_type.contains_key(&AgentType::ClaudeCode));
        assert!(report.by_model.contains_key("claude-3-opus"));
    }

    #[tokio::test]
    async fn test_session_aggregator_creation() {
        let registry = Arc::new(AdapterRegistry::new());
        let aggregator = SessionAggregator::new(registry);

        let count = aggregator.get_active_session_count().await;
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_session_aggregator_empty_report() {
        let registry = Arc::new(AdapterRegistry::new());
        let aggregator = SessionAggregator::new(registry);

        let report = aggregator
            .get_session_report(TimeRange::Today)
            .await
            .unwrap();
        assert_eq!(report.stats.total_sessions, 0);
        assert!(report.sessions.is_empty());
    }

    #[tokio::test]
    async fn test_session_aggregator_recent_sessions() {
        let registry = Arc::new(AdapterRegistry::new());
        let aggregator = SessionAggregator::new(registry);

        let sessions = aggregator.get_recent_sessions(10).await.unwrap();
        assert!(sessions.is_empty());
    }
}
