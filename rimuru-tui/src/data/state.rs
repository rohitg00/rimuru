use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct AppData {
    pub agents: Vec<AgentData>,
    pub sessions: Vec<SessionData>,
    pub costs: CostData,
    pub metrics: MetricsData,
    pub last_updated: Option<DateTime<Utc>>,
}

impl AppData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.agents.is_empty() && self.sessions.is_empty()
    }

    pub fn active_sessions_count(&self) -> usize {
        self.sessions.iter().filter(|s| s.is_active).count()
    }

    pub fn total_agents(&self) -> usize {
        self.agents.len()
    }

    pub fn connected_agents_count(&self) -> usize {
        self.agents.iter().filter(|a| a.is_connected).count()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentData {
    pub id: Uuid,
    pub name: String,
    pub agent_type: String,
    pub is_connected: bool,
    pub active_sessions: i32,
    pub tokens_today: i64,
    pub cost_today: f64,
    pub last_activity: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl AgentData {
    pub fn status_label(&self) -> &'static str {
        if self.active_sessions > 0 {
            "Connected"
        } else if self.is_connected {
            "Idle"
        } else {
            "Disconnected"
        }
    }

    pub fn status_icon(&self) -> &'static str {
        if self.active_sessions > 0 {
            "●"
        } else if self.is_connected {
            "◐"
        } else {
            "○"
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub agent_name: String,
    pub is_active: bool,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_seconds: Option<i64>,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cost: f64,
    pub model_name: Option<String>,
    pub metadata: serde_json::Value,
}

impl SessionData {
    pub fn total_tokens(&self) -> i64 {
        self.input_tokens + self.output_tokens
    }

    pub fn formatted_duration(&self) -> String {
        if let Some(secs) = self.duration_seconds {
            let hours = secs / 3600;
            let mins = (secs % 3600) / 60;
            let secs = secs % 60;
            if hours > 0 {
                format!("{}h {}m {}s", hours, mins, secs)
            } else if mins > 0 {
                format!("{}m {}s", mins, secs)
            } else {
                format!("{}s", secs)
            }
        } else if self.is_active {
            let elapsed = (Utc::now() - self.started_at).num_seconds();
            let mins = elapsed / 60;
            let secs = elapsed % 60;
            format!("{}m {}s (active)", mins, secs)
        } else {
            "-".to_string()
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CostData {
    pub today_total: f64,
    pub week_total: f64,
    pub month_total: f64,
    pub all_time_total: f64,
    pub today_tokens: i64,
    pub today_sessions: i64,
    pub by_agent: Vec<AgentCostSummary>,
    pub by_model: Vec<ModelCostSummary>,
    pub daily_costs: Vec<DailyCost>,
}

impl CostData {
    pub fn today_trend(&self) -> CostTrend {
        if self.daily_costs.len() < 2 {
            return CostTrend::Stable;
        }

        let yesterday = self.daily_costs.get(1).map(|d| d.cost).unwrap_or(0.0);
        if yesterday == 0.0 {
            return CostTrend::Stable;
        }

        let change = (self.today_total - yesterday) / yesterday;
        if change > 0.1 {
            CostTrend::Up
        } else if change < -0.1 {
            CostTrend::Down
        } else {
            CostTrend::Stable
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CostTrend {
    Up,
    Down,
    Stable,
}

impl CostTrend {
    pub fn icon(&self) -> &'static str {
        match self {
            CostTrend::Up => "↑",
            CostTrend::Down => "↓",
            CostTrend::Stable => "→",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCostSummary {
    pub agent_id: Uuid,
    pub agent_name: String,
    pub cost: f64,
    pub tokens: i64,
    pub sessions: i64,
    pub trend: CostTrend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCostSummary {
    pub model_name: String,
    pub cost: f64,
    pub tokens: i64,
    pub sessions: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyCost {
    pub date: DateTime<Utc>,
    pub cost: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetricsData {
    pub cpu_percent: f32,
    pub memory_used_mb: i64,
    pub memory_total_mb: i64,
    pub disk_used_mb: i64,
    pub disk_total_mb: i64,
    pub active_sessions: i32,
    pub cpu_history: Vec<f32>,
    pub memory_history: Vec<f32>,
    pub peak_cpu: f32,
    pub average_cpu: f32,
}

impl MetricsData {
    pub fn memory_percent(&self) -> f32 {
        if self.memory_total_mb > 0 {
            (self.memory_used_mb as f32 / self.memory_total_mb as f32) * 100.0
        } else {
            0.0
        }
    }

    pub fn disk_percent(&self) -> f32 {
        if self.disk_total_mb > 0 {
            (self.disk_used_mb as f32 / self.disk_total_mb as f32) * 100.0
        } else {
            0.0
        }
    }

    pub fn memory_available_mb(&self) -> i64 {
        self.memory_total_mb.saturating_sub(self.memory_used_mb)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_data_default() {
        let data = AppData::new();
        assert!(data.is_empty());
        assert_eq!(data.active_sessions_count(), 0);
        assert_eq!(data.total_agents(), 0);
        assert_eq!(data.connected_agents_count(), 0);
        assert!(data.last_updated.is_none());
    }

    #[test]
    fn test_agent_data_status() {
        let mut agent = AgentData {
            id: Uuid::new_v4(),
            name: "test-agent".to_string(),
            agent_type: "claude_code".to_string(),
            is_connected: true,
            active_sessions: 2,
            tokens_today: 1000,
            cost_today: 0.05,
            last_activity: Some(Utc::now()),
            created_at: Utc::now(),
        };

        assert_eq!(agent.status_label(), "Connected");
        assert_eq!(agent.status_icon(), "●");

        agent.active_sessions = 0;
        assert_eq!(agent.status_label(), "Idle");
        assert_eq!(agent.status_icon(), "◐");

        agent.is_connected = false;
        assert_eq!(agent.status_label(), "Disconnected");
        assert_eq!(agent.status_icon(), "○");
    }

    #[test]
    fn test_session_data_tokens() {
        let session = SessionData {
            id: Uuid::new_v4(),
            agent_id: Uuid::new_v4(),
            agent_name: "test-agent".to_string(),
            is_active: false,
            status: "completed".to_string(),
            started_at: Utc::now(),
            ended_at: Some(Utc::now()),
            duration_seconds: Some(120),
            input_tokens: 1000,
            output_tokens: 500,
            cost: 0.03,
            model_name: Some("claude-3-opus".to_string()),
            metadata: serde_json::json!({}),
        };

        assert_eq!(session.total_tokens(), 1500);
        assert_eq!(session.formatted_duration(), "2m 0s");
    }

    #[test]
    fn test_session_duration_formatting() {
        let mut session = SessionData {
            id: Uuid::new_v4(),
            agent_id: Uuid::new_v4(),
            agent_name: "test".to_string(),
            is_active: false,
            status: "completed".to_string(),
            started_at: Utc::now(),
            ended_at: Some(Utc::now()),
            duration_seconds: Some(3661),
            input_tokens: 0,
            output_tokens: 0,
            cost: 0.0,
            model_name: None,
            metadata: serde_json::json!({}),
        };

        assert_eq!(session.formatted_duration(), "1h 1m 1s");

        session.duration_seconds = Some(65);
        assert_eq!(session.formatted_duration(), "1m 5s");

        session.duration_seconds = Some(30);
        assert_eq!(session.formatted_duration(), "30s");

        session.duration_seconds = None;
        session.is_active = false;
        assert_eq!(session.formatted_duration(), "-");
    }

    #[test]
    fn test_cost_data_trend() {
        let mut cost_data = CostData {
            today_total: 1.0,
            daily_costs: vec![
                DailyCost {
                    date: Utc::now(),
                    cost: 1.0,
                },
                DailyCost {
                    date: Utc::now(),
                    cost: 0.5,
                },
            ],
            ..Default::default()
        };

        assert_eq!(cost_data.today_trend(), CostTrend::Up);

        cost_data.today_total = 0.3;
        assert_eq!(cost_data.today_trend(), CostTrend::Down);

        cost_data.today_total = 0.52;
        assert_eq!(cost_data.today_trend(), CostTrend::Stable);
    }

    #[test]
    fn test_cost_trend_icons() {
        assert_eq!(CostTrend::Up.icon(), "↑");
        assert_eq!(CostTrend::Down.icon(), "↓");
        assert_eq!(CostTrend::Stable.icon(), "→");
    }

    #[test]
    fn test_metrics_data_calculations() {
        let metrics = MetricsData {
            cpu_percent: 50.0,
            memory_used_mb: 8192,
            memory_total_mb: 16384,
            disk_used_mb: 100000,
            disk_total_mb: 500000,
            active_sessions: 3,
            cpu_history: vec![40.0, 45.0, 50.0],
            memory_history: vec![],
            peak_cpu: 75.0,
            average_cpu: 45.0,
        };

        assert_eq!(metrics.memory_percent(), 50.0);
        assert_eq!(metrics.disk_percent(), 20.0);
        assert_eq!(metrics.memory_available_mb(), 8192);
    }

    #[test]
    fn test_metrics_data_zero_totals() {
        let metrics = MetricsData::default();

        assert_eq!(metrics.memory_percent(), 0.0);
        assert_eq!(metrics.disk_percent(), 0.0);
        assert_eq!(metrics.memory_available_mb(), 0);
    }

    #[test]
    fn test_app_data_counts() {
        let mut data = AppData::new();

        data.agents.push(AgentData {
            id: Uuid::new_v4(),
            name: "agent1".to_string(),
            agent_type: "claude_code".to_string(),
            is_connected: true,
            active_sessions: 1,
            tokens_today: 0,
            cost_today: 0.0,
            last_activity: None,
            created_at: Utc::now(),
        });

        data.agents.push(AgentData {
            id: Uuid::new_v4(),
            name: "agent2".to_string(),
            agent_type: "cursor".to_string(),
            is_connected: false,
            active_sessions: 0,
            tokens_today: 0,
            cost_today: 0.0,
            last_activity: None,
            created_at: Utc::now(),
        });

        data.sessions.push(SessionData {
            id: Uuid::new_v4(),
            agent_id: Uuid::new_v4(),
            agent_name: "agent1".to_string(),
            is_active: true,
            status: "active".to_string(),
            started_at: Utc::now(),
            ended_at: None,
            duration_seconds: None,
            input_tokens: 0,
            output_tokens: 0,
            cost: 0.0,
            model_name: None,
            metadata: serde_json::json!({}),
        });

        data.sessions.push(SessionData {
            id: Uuid::new_v4(),
            agent_id: Uuid::new_v4(),
            agent_name: "agent1".to_string(),
            is_active: false,
            status: "completed".to_string(),
            started_at: Utc::now(),
            ended_at: Some(Utc::now()),
            duration_seconds: Some(60),
            input_tokens: 100,
            output_tokens: 50,
            cost: 0.01,
            model_name: None,
            metadata: serde_json::json!({}),
        });

        assert!(!data.is_empty());
        assert_eq!(data.total_agents(), 2);
        assert_eq!(data.connected_agents_count(), 1);
        assert_eq!(data.active_sessions_count(), 1);
    }
}
