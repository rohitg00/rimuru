use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CostRecord {
    pub id: Uuid,
    pub session_id: Uuid,
    pub agent_id: Uuid,
    pub model_name: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cost_usd: f64,
    pub recorded_at: DateTime<Utc>,
}

impl CostRecord {
    pub fn new(
        session_id: Uuid,
        agent_id: Uuid,
        model_name: String,
        input_tokens: i64,
        output_tokens: i64,
        cost_usd: f64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            session_id,
            agent_id,
            model_name,
            input_tokens,
            output_tokens,
            cost_usd,
            recorded_at: Utc::now(),
        }
    }

    pub fn total_tokens(&self) -> i64 {
        self.input_tokens + self.output_tokens
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CostSummary {
    pub total_cost_usd: f64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub record_count: i64,
}

impl CostSummary {
    pub fn total_tokens(&self) -> i64 {
        self.total_input_tokens + self.total_output_tokens
    }

    pub fn average_cost_per_request(&self) -> f64 {
        if self.record_count > 0 {
            self.total_cost_usd / self.record_count as f64
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_record_new() {
        let session_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();
        let record = CostRecord::new(
            session_id,
            agent_id,
            "claude-3-opus".to_string(),
            1000,
            500,
            0.05,
        );

        assert_eq!(record.session_id, session_id);
        assert_eq!(record.agent_id, agent_id);
        assert_eq!(record.model_name, "claude-3-opus");
        assert_eq!(record.input_tokens, 1000);
        assert_eq!(record.output_tokens, 500);
        assert_eq!(record.cost_usd, 0.05);
    }

    #[test]
    fn test_cost_record_total_tokens() {
        let record = CostRecord::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "gpt-4".to_string(),
            1000,
            500,
            0.03,
        );

        assert_eq!(record.total_tokens(), 1500);
    }

    #[test]
    fn test_cost_summary() {
        let summary = CostSummary {
            total_cost_usd: 1.0,
            total_input_tokens: 10000,
            total_output_tokens: 5000,
            record_count: 10,
        };

        assert_eq!(summary.total_tokens(), 15000);
        assert_eq!(summary.average_cost_per_request(), 0.1);
    }

    #[test]
    fn test_cost_summary_empty() {
        let summary = CostSummary::default();

        assert_eq!(summary.total_tokens(), 0);
        assert_eq!(summary.average_cost_per_request(), 0.0);
    }
}
