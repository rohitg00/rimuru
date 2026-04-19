use std::collections::VecDeque;
use std::time::Instant;

use ratatui::widgets::TableState;
use serde_json::Value;

use crate::client::ApiClient;
use crate::theme::{self, Theme};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Home,
    Costs,
    Budget,
    Models,
    Advisor,
    Context,
    McpProxy,
    Hooks,
    Plugins,
    Mcp,
    Metrics,
}

impl View {
    pub fn title(&self) -> &'static str {
        match self {
            View::Home => "home",
            View::Costs => "costs",
            View::Budget => "budget",
            View::Models => "models",
            View::Advisor => "advisor",
            View::Context => "context",
            View::McpProxy => "mcp-proxy",
            View::Hooks => "hooks",
            View::Plugins => "plugins",
            View::Mcp => "mcp",
            View::Metrics => "metrics",
        }
    }

    pub fn from_digit(d: char) -> Option<View> {
        match d {
            '1' => Some(View::Costs),
            '2' => Some(View::Budget),
            '3' => Some(View::Models),
            '4' => Some(View::Advisor),
            '5' => Some(View::Context),
            '6' => Some(View::McpProxy),
            '7' => Some(View::Hooks),
            '8' => Some(View::Plugins),
            '9' => Some(View::Mcp),
            '0' => Some(View::Metrics),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionStatusKind {
    Active,
    Completed,
    Abandoned,
    Error,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct SessionView {
    pub id: String,
    pub short_id: String,
    #[allow(dead_code)]
    pub agent_id: String,
    pub agent_type: String,
    pub agent_label: String,
    pub project_path: String,
    pub project_name: String,
    pub status: SessionStatusKind,
    pub model: String,
    pub model_short: String,
    pub total_tokens: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read: u64,
    pub cache_write: u64,
    pub total_cost: f64,
    pub messages: u64,
    pub context_pct: f64,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub token_history: Vec<u64>,
    #[allow(dead_code)]
    pub raw: Value,
}

pub struct App {
    pub view: View,
    pub table_state: TableState,
    pub selected: usize,
    pub connected: bool,
    pub theme_idx: usize,
    pub scroll: usize,

    pub sessions: Vec<SessionView>,
    pub agents: Vec<Value>,
    pub hardware: Value,
    pub cost_summary: Value,
    pub daily_costs: Vec<Value>,
    pub models: Vec<Value>,
    pub advisories: Vec<Value>,
    pub catalog: Vec<Value>,
    pub metrics: Value,
    pub hooks: Vec<Value>,
    pub plugins: Vec<Value>,
    pub mcp_servers: Vec<Value>,
    pub context_utilization: Value,
    pub context_waste: Value,
    pub mcp_proxy_stats: Value,
    pub budget_status: Value,
    pub budget_alerts: Value,
    pub total_savings: f64,
    pub catalog_summary_cache: (usize, usize, usize, usize),

    pub token_rates: VecDeque<f64>,
    pub prev_total_tokens: Option<u64>,
    pub prev_fetch_time: Option<Instant>,
    pub status_msg: Option<(String, Instant)>,
}

impl App {
    pub fn new() -> Self {
        Self {
            view: View::Home,
            table_state: {
                let mut ts = TableState::default();
                ts.select(Some(0));
                ts
            },
            selected: 0,
            connected: false,
            theme_idx: 0,
            scroll: 0,
            sessions: Vec::new(),
            agents: Vec::new(),
            hardware: Value::Null,
            cost_summary: Value::Null,
            daily_costs: Vec::new(),
            models: Vec::new(),
            advisories: Vec::new(),
            catalog: Vec::new(),
            metrics: Value::Null,
            hooks: Vec::new(),
            plugins: Vec::new(),
            mcp_servers: Vec::new(),
            context_utilization: Value::Null,
            context_waste: Value::Null,
            mcp_proxy_stats: Value::Null,
            budget_status: Value::Null,
            budget_alerts: Value::Null,
            total_savings: 0.0,
            catalog_summary_cache: (0, 0, 0, 0),
            token_rates: VecDeque::with_capacity(120),
            prev_total_tokens: None,
            prev_fetch_time: None,
            status_msg: None,
        }
    }

    pub fn theme(&self) -> &'static Theme {
        theme::theme_by_index(self.theme_idx)
    }

    pub fn next_theme(&mut self) {
        self.theme_idx = (self.theme_idx + 1) % theme::ALL_THEMES.len();
        self.set_status(format!("theme: {}", self.theme().name));
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_msg = Some((msg.into(), Instant::now()));
    }

    pub fn select_next(&mut self) {
        if self.sessions.is_empty() {
            return;
        }
        let new = (self.selected + 1).min(self.sessions.len() - 1);
        self.selected = new;
        self.table_state.select(Some(new));
    }

    pub fn select_prev(&mut self) {
        if self.sessions.is_empty() {
            return;
        }
        self.selected = self.selected.saturating_sub(1);
        self.table_state.select(Some(self.selected));
    }

    pub fn scroll_down(&mut self) {
        let max = self.scrollable_len().saturating_sub(1);
        if self.scroll < max {
            self.scroll += 1;
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
        }
    }

    fn scrollable_len(&self) -> usize {
        match self.view {
            View::Costs => self.daily_costs.len(),
            View::Budget => self
                .budget_alerts
                .get("alerts")
                .and_then(|v| v.as_array())
                .map(|a| a.len())
                .unwrap_or(0),
            View::Models => self.models.len(),
            View::Advisor => self.catalog.len(),
            View::Context => self
                .context_utilization
                .get("utilizations")
                .and_then(|v| v.as_array())
                .map(|a| a.len())
                .unwrap_or(0),
            View::McpProxy => self
                .mcp_proxy_stats
                .get("tools")
                .and_then(|v| v.as_array())
                .map(|a| a.len())
                .unwrap_or(0),
            View::Hooks => self.hooks.len(),
            View::Plugins => self.plugins.len(),
            View::Mcp => self.mcp_servers.len(),
            _ => 0,
        }
    }

    pub fn set_view(&mut self, v: View) {
        self.view = v;
        self.scroll = 0;
    }

    pub fn selected_session(&self) -> Option<&SessionView> {
        self.sessions.get(self.selected)
    }

    pub async fn fetch(&mut self, client: &ApiClient) {
        let health = client.get("/health").await;
        self.connected = health
            .as_ref()
            .and_then(|v| v.get("status"))
            .and_then(|s| s.as_str())
            .map(|s| s == "healthy")
            .unwrap_or(false);

        if !self.connected {
            return;
        }

        let (sessions_v, agents_v, system_v, ctx_util_v, budget_v, mcp_v, advisor_v) = tokio::join!(
            client.get("/sessions"),
            client.get("/agents"),
            client.get("/system"),
            client.get("/context/utilization"),
            client.get("/budget/status"),
            client.get("/mcp"),
            client.get("/models/advisor"),
        );

        if let Some(v) = ctx_util_v {
            self.context_utilization = v;
        }
        if let Some(v) = sessions_v {
            let arr = unwrap_array(&v, "sessions");
            self.ingest_sessions(arr);
        }
        if let Some(v) = agents_v {
            self.agents = unwrap_array(&v, "agents");
        }
        if let Some(v) = system_v {
            self.hardware = v.get("hardware").cloned().unwrap_or(v);
        }
        if let Some(v) = budget_v {
            self.budget_status = v;
        }
        if let Some(v) = mcp_v {
            self.mcp_servers = unwrap_array(&v, "servers");
        }
        if let Some(v) = advisor_v {
            let arr = unwrap_array(&v, "advisories");
            self.total_savings = arr
                .iter()
                .filter(|a| a.get("can_run_locally").and_then(|v| v.as_bool()).unwrap_or(false))
                .map(|a| a.get("potential_savings").and_then(|v| v.as_f64()).unwrap_or(0.0))
                .sum();
            self.advisories = arr;
        }

        self.compute_token_rate();

        match self.view {
            View::Home => {
                let mcp_stats_v = client.get("/mcp/proxy/stats").await;
                if let Some(v) = mcp_stats_v {
                    self.mcp_proxy_stats = v;
                }
            }
            View::Costs => {
                let (summary, daily) = tokio::join!(
                    client.get("/costs/summary"),
                    client.get("/costs/daily"),
                );
                if let Some(v) = summary {
                    self.cost_summary = v.get("summary").cloned().unwrap_or(v);
                }
                if let Some(v) = daily {
                    self.daily_costs = unwrap_array(&v, "daily_costs");
                }
            }
            View::Budget => {
                if let Some(v) = client.get("/budget/alerts?limit=20").await {
                    self.budget_alerts = v;
                }
            }
            View::Models => {
                if let Some(v) = client.get("/models").await {
                    self.models = unwrap_array(&v, "models");
                }
            }
            View::Advisor => {
                if let Some(v) = client.get("/models/catalog/runnable").await {
                    if let Some(entries) = v.get("entries").and_then(|e| e.as_array()) {
                        self.catalog = entries.clone();
                    }
                    if let Some(summary) = v.get("summary") {
                        let perfect = summary.get("perfect").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                        let good = summary.get("good").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                        let marginal = summary.get("marginal").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                        let total = summary.get("catalog_size").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                        self.catalog_summary_cache = (perfect, good, marginal, total);
                    }
                }
            }
            View::Context => {
                if let Some(v) = client.get("/context/waste").await {
                    self.context_waste = v;
                }
            }
            View::McpProxy => {
                if let Some(v) = client.get("/mcp/proxy/stats").await {
                    self.mcp_proxy_stats = v;
                }
            }
            View::Hooks => {
                if let Some(v) = client.get("/hooks").await {
                    self.hooks = unwrap_array(&v, "hooks");
                }
            }
            View::Plugins => {
                if let Some(v) = client.get("/plugins").await {
                    self.plugins = unwrap_array(&v, "plugins");
                }
            }
            View::Mcp => {}
            View::Metrics => {
                if let Some(v) = client.get("/metrics").await {
                    let inner = v.get("metrics").cloned().unwrap_or(v);
                    self.metrics = inner;
                }
            }
        }
    }

    fn ingest_sessions(&mut self, arr: Vec<Value>) {
        let ctx_map = self.context_percent_map();

        let mut out: Vec<SessionView> = arr
            .iter()
            .map(|raw| {
                let id = raw.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let short_id = if id.len() >= 8 { id[..8].to_string() } else { id.clone() };
                let agent_id = raw.get("agent_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let agent_type = agent_type_str(raw);
                let agent_label = agent_label_for(&agent_type);
                let project_path = raw.get("project_path").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let project_name = project_name_from_path(&project_path);
                let status = session_status_kind(raw);
                let model = raw.get("model").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let model_short = shorten_model(&model);
                let total_tokens = raw.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                let input_tokens = raw.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                let output_tokens = raw.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                let (cache_read, cache_write, turn_tokens) = summarize_turns(raw);
                let total_cost = raw.get("total_cost").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let messages = raw.get("messages").and_then(|v| v.as_u64()).unwrap_or(0);
                let started_at = raw.get("started_at").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let ended_at = raw
                    .get("ended_at")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let context_pct = ctx_map
                    .iter()
                    .find(|(sid, _)| sid == &id)
                    .map(|(_, p)| *p)
                    .unwrap_or(0.0);
                let token_history = turn_tokens;

                SessionView {
                    id,
                    short_id,
                    agent_id,
                    agent_type,
                    agent_label,
                    project_path,
                    project_name,
                    status,
                    model,
                    model_short,
                    total_tokens,
                    input_tokens,
                    output_tokens,
                    cache_read,
                    cache_write,
                    total_cost,
                    messages,
                    context_pct,
                    started_at,
                    ended_at,
                    token_history,
                    raw: raw.clone(),
                }
            })
            .collect();

        out.sort_by(|a, b| {
            let rank = |s: &SessionView| match s.status {
                SessionStatusKind::Active => 0,
                SessionStatusKind::Error => 1,
                SessionStatusKind::Completed => 2,
                SessionStatusKind::Abandoned => 3,
                SessionStatusKind::Unknown => 4,
            };
            rank(a)
                .cmp(&rank(b))
                .then_with(|| b.total_tokens.cmp(&a.total_tokens))
        });

        self.sessions = out;
        if self.sessions.is_empty() {
            self.selected = 0;
            self.table_state.select(None);
        } else {
            if self.selected >= self.sessions.len() {
                self.selected = self.sessions.len() - 1;
            }
            self.table_state.select(Some(self.selected));
        }
    }

    fn context_percent_map(&self) -> Vec<(String, f64)> {
        let Some(arr) = self
            .context_utilization
            .get("utilizations")
            .and_then(|v| v.as_array())
        else {
            return Vec::new();
        };
        arr.iter()
            .filter_map(|v| {
                let id = v.get("session_id").and_then(|x| x.as_str())?.to_string();
                let pct = v
                    .get("utilization_percent")
                    .or_else(|| v.get("percent"))
                    .and_then(|x| x.as_f64())
                    .unwrap_or(0.0);
                Some((id, pct))
            })
            .collect()
    }

    #[cfg(test)]
    pub fn ingest_sessions_public(&mut self, arr: Vec<Value>) {
        self.ingest_sessions(arr);
    }

    #[cfg(test)]
    pub fn push_token_rate_for_test(&mut self, rate: f64) {
        if self.token_rates.len() >= 120 {
            self.token_rates.pop_front();
        }
        self.token_rates.push_back(rate);
    }

    fn compute_token_rate(&mut self) {
        let now = Instant::now();
        let total: u64 = self.sessions.iter().map(|s| s.total_tokens).sum();

        let rate = match (self.prev_total_tokens, self.prev_fetch_time) {
            (Some(prev), Some(prev_t)) if total >= prev => {
                let dt = now.duration_since(prev_t).as_secs_f64().max(0.001);
                (total - prev) as f64 / dt
            }
            _ => 0.0,
        };

        if self.token_rates.len() >= 120 {
            self.token_rates.pop_front();
        }
        self.token_rates.push_back(rate);
        self.prev_total_tokens = Some(total);
        self.prev_fetch_time = Some(now);
    }
}

fn unwrap_array(v: &Value, key: &str) -> Vec<Value> {
    if let Some(a) = v.as_array() {
        return a.clone();
    }
    if let Some(a) = v.get(key).and_then(|x| x.as_array()) {
        return a.clone();
    }
    for k in ["items", "data", "results"] {
        if let Some(a) = v.get(k).and_then(|x| x.as_array()) {
            return a.clone();
        }
    }
    Vec::new()
}

fn summarize_turns(raw: &Value) -> (u64, u64, Vec<u64>) {
    let Some(turns) = raw.pointer("/metadata/turns").and_then(|v| v.as_array()) else {
        return (0, 0, Vec::new());
    };
    let mut cache_read = 0u64;
    let mut cache_write = 0u64;
    let mut history: Vec<u64> = Vec::new();
    for t in turns {
        cache_read += t.get("cache_read").and_then(|v| v.as_u64()).unwrap_or(0);
        cache_write += t.get("cache_write").and_then(|v| v.as_u64()).unwrap_or(0);
        let tokens = t.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0)
            + t.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
        if tokens > 0 {
            history.push(tokens);
        }
    }
    (cache_read, cache_write, history)
}

fn agent_type_str(raw: &Value) -> String {
    if let Some(s) = raw.get("agent_type").and_then(|v| v.as_str()) {
        return s.to_string();
    }
    raw.get("agent_type")
        .and_then(|v| serde_json::to_string(v).ok())
        .map(|s| s.trim_matches('"').to_string())
        .unwrap_or_default()
}

fn agent_label_for(agent_type: &str) -> String {
    let t = agent_type.to_lowercase();
    if t.contains("claude") {
        "*CC".to_string()
    } else if t.contains("codex") {
        ">CD".to_string()
    } else if t.contains("cursor") {
        "*CR".to_string()
    } else if t.contains("copilot") {
        "*GH".to_string()
    } else if t.contains("gemini") {
        "*GM".to_string()
    } else if t.contains("opencode") {
        "*OC".to_string()
    } else if t.contains("aider") {
        "*AD".to_string()
    } else if t.contains("windsurf") {
        "*WS".to_string()
    } else if t.is_empty() {
        "?".to_string()
    } else {
        let upper = t.chars().take(3).collect::<String>().to_uppercase();
        format!("*{}", upper)
    }
}

fn session_status_kind(raw: &Value) -> SessionStatusKind {
    let s = raw
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_lowercase();
    match s.as_str() {
        "active" | "working" | "running" => SessionStatusKind::Active,
        "completed" | "done" | "finished" => SessionStatusKind::Completed,
        "abandoned" | "idle" | "waiting" => SessionStatusKind::Abandoned,
        "error" | "errored" | "failed" => SessionStatusKind::Error,
        _ => SessionStatusKind::Unknown,
    }
}

fn project_name_from_path(path: &str) -> String {
    if path.is_empty() {
        return "—".to_string();
    }
    std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path)
        .to_string()
}

pub fn shorten_model(model: &str) -> String {
    if model.is_empty() {
        return "-".to_string();
    }
    let s = model.strip_prefix("claude-").unwrap_or(model);
    let is_1m = s.contains("[1m]");
    let s = s.trim_end_matches("[1m]");
    let base = if let Some(pos) = s.find(|c: char| c.is_ascii_digit()) {
        let name = s[..pos].trim_end_matches('-');
        let ver = s[pos..].replace('-', ".");
        if name.is_empty() { ver } else { format!("{}{}", name, ver) }
    } else {
        s.to_string()
    };
    if is_1m { format!("{}[1m]", base) } else { base }
}
