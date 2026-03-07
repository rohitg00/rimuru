use std::hash::{DefaultHasher, Hash, Hasher};

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
    #[allow(clippy::too_many_arguments)]
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

    #[allow(clippy::too_many_arguments)]
    pub fn new_for_session(
        session_id: Uuid,
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
            id: deterministic_uuid(session_id),
            agent_id,
            agent_type,
            session_id: Some(session_id),
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

fn deterministic_uuid(session_id: Uuid) -> Uuid {
    let mut hasher = DefaultHasher::new();
    "cost".hash(&mut hasher);
    session_id.hash(&mut hasher);
    let hash = hasher.finish();
    let mut bytes = [0u8; 16];
    bytes[..8].copy_from_slice(&hash.to_le_bytes());
    let mut hasher2 = DefaultHasher::new();
    hash.hash(&mut hasher2);
    bytes[8..].copy_from_slice(&hasher2.finish().to_le_bytes());
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
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
