use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub enum PtySessionStatus {
    Starting,
    Running,
    Completed,
    Failed,
    Terminated,
}

#[derive(Serialize, Clone, Debug)]
pub struct PtySessionInfo {
    pub id: String,
    pub agent_type: String,
    pub agent_name: String,
    pub working_dir: String,
    pub started_at: String,
    pub status: PtySessionStatus,
    pub pid: Option<u32>,
    pub cumulative_cost_usd: f64,
    pub token_count: i64,
}
