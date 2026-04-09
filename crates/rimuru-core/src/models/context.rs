use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBreakdown {
    pub session_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub total_tokens: u64,
    pub system_prompt_tokens: u64,
    pub conversation_tokens: u64,
    pub tool_schema_tokens: u64,
    pub tool_result_tokens: u64,
    pub file_read_tokens: u64,
    pub bash_output_tokens: u64,
    pub mcp_tokens: u64,
    pub user_tokens: u64,
    pub assistant_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
    pub turns: Vec<TurnRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnRecord {
    pub turn_index: u32,
    pub role: String,
    pub model: Option<String>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read: u64,
    pub cache_write: u64,
    pub tool_calls: Vec<ToolCallRecord>,
    pub timestamp: Option<String>,
    pub content_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    pub tool_name: String,
    pub tool_id: Option<String>,
    pub input_tokens_estimate: u64,
    pub output_tokens_estimate: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextUtilization {
    pub session_id: Uuid,
    pub model: String,
    pub context_window_size: u64,
    pub tokens_used: u64,
    pub utilization_percent: f64,
    pub is_near_limit: bool,
}

impl ContextBreakdown {
    pub fn new(session_id: Uuid) -> Self {
        Self {
            session_id,
            timestamp: Utc::now(),
            total_tokens: 0,
            system_prompt_tokens: 0,
            conversation_tokens: 0,
            tool_schema_tokens: 0,
            tool_result_tokens: 0,
            file_read_tokens: 0,
            bash_output_tokens: 0,
            mcp_tokens: 0,
            user_tokens: 0,
            assistant_tokens: 0,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
            turns: Vec::new(),
        }
    }

    pub fn waste_percent(&self) -> f64 {
        if self.total_tokens == 0 {
            return 0.0;
        }
        let wasted = self.tool_schema_tokens + self.bash_output_tokens;
        (wasted as f64 / self.total_tokens as f64) * 100.0
    }
}
