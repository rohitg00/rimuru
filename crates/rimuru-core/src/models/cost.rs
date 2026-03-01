use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::AgentType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostRecord {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub agent_type: AgentType,
    pub session_id: Option<Uuid>,
    pub model: String,
    pub provider: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
    pub input_cost: f64,
    pub output_cost: f64,
    pub total_cost: f64,
    pub recorded_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

impl CostRecord {
    pub fn new(
        agent_id: Uuid,
        agent_type: AgentType,
        model: String,
        provider: String,
        input_tokens: u64,
        output_tokens: u64,
        input_cost: f64,
        output_cost: f64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            agent_id,
            agent_type,
            session_id: None,
            model,
            provider,
            input_tokens,
            output_tokens,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
            input_cost,
            output_cost,
            total_cost: input_cost + output_cost,
            recorded_at: Utc::now(),
            metadata: serde_json::json!({}),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSummary {
    pub total_cost: f64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_records: u64,
    pub by_agent: Vec<AgentCostSummary>,
    pub by_model: Vec<ModelCostSummary>,
    pub period_start: Option<DateTime<Utc>>,
    pub period_end: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCostSummary {
    pub agent_id: Uuid,
    pub agent_type: AgentType,
    pub agent_name: String,
    pub total_cost: f64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub record_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCostSummary {
    pub model: String,
    pub provider: String,
    pub total_cost: f64,
    pub total_tokens: u64,
    pub record_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyCostSummary {
    pub date: NaiveDate,
    pub total_cost: f64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub record_count: u64,
    pub by_agent: Vec<AgentCostSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostFilter {
    pub agent_id: Option<Uuid>,
    pub agent_type: Option<AgentType>,
    pub model: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub group_by: Option<CostGroupBy>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CostGroupBy {
    Agent,
    Model,
    Day,
    Provider,
}
