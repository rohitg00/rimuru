use serde_json::Value;

use crate::client::ApiClient;
use crate::theme::{self, Theme};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Agents,
    Sessions,
    Costs,
    Models,
    Advisor,
    Hooks,
    Plugins,
    Mcp,
    Metrics,
}

impl Tab {
    pub fn all() -> &'static [Tab] {
        &[
            Tab::Dashboard,
            Tab::Agents,
            Tab::Sessions,
            Tab::Costs,
            Tab::Models,
            Tab::Advisor,
            Tab::Hooks,
            Tab::Plugins,
            Tab::Mcp,
            Tab::Metrics,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Tab::Dashboard => "Dashboard",
            Tab::Agents => "Agents",
            Tab::Sessions => "Sessions",
            Tab::Costs => "Costs",
            Tab::Models => "Models",
            Tab::Advisor => "Advisor",
            Tab::Hooks => "Hooks",
            Tab::Plugins => "Plugins",
            Tab::Mcp => "MCP",
            Tab::Metrics => "Metrics",
        }
    }

    pub fn index(&self) -> usize {
        Tab::all().iter().position(|t| t == self).unwrap_or(0)
    }
}

pub struct App {
    pub tab: Tab,
    pub scroll: usize,
    pub connected: bool,
    pub theme_idx: usize,

    pub stats: Value,
    pub hardware: Value,
    pub agents: Vec<Value>,
    pub sessions: Vec<Value>,
    pub cost_summary: Value,
    pub daily_costs: Vec<Value>,
    pub models: Vec<Value>,
    pub advisories: Vec<Value>,
    pub catalog: Vec<Value>,
    pub metrics: Value,
    pub activity: Vec<Value>,
    pub total_savings: f64,
    pub hooks: Vec<Value>,
    pub plugins: Vec<Value>,
    pub mcp_servers: Vec<Value>,

    catalog_summary_cache: (usize, usize, usize, usize),
}

impl App {
    pub fn theme(&self) -> &'static Theme {
        theme::theme_by_index(self.theme_idx)
    }

    pub fn next_theme(&mut self) {
        self.theme_idx = (self.theme_idx + 1) % theme::ALL_THEMES.len();
    }

    pub fn new() -> Self {
        Self {
            tab: Tab::Dashboard,
            scroll: 0,
            connected: false,
            theme_idx: 0,
            stats: Value::Null,
            hardware: Value::Null,
            agents: Vec::new(),
            sessions: Vec::new(),
            cost_summary: Value::Null,
            daily_costs: Vec::new(),
            models: Vec::new(),
            advisories: Vec::new(),
            catalog: Vec::new(),
            metrics: Value::Null,
            activity: Vec::new(),
            total_savings: 0.0,
            hooks: Vec::new(),
            plugins: Vec::new(),
            mcp_servers: Vec::new(),
            catalog_summary_cache: (0, 0, 0, 0),
        }
    }

    pub fn next_tab(&mut self) {
        let tabs = Tab::all();
        let idx = self.tab.index();
        self.tab = tabs[(idx + 1) % tabs.len()];
        self.scroll = 0;
    }

    pub fn prev_tab(&mut self) {
        let tabs = Tab::all();
        let idx = self.tab.index();
        self.tab = if idx == 0 {
            tabs[tabs.len() - 1]
        } else {
            tabs[idx - 1]
        };
        self.scroll = 0;
    }

    pub fn scroll_down(&mut self) {
        let max = self.list_len().saturating_sub(1);
        if self.scroll < max {
            self.scroll += 1;
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
        }
    }

    fn list_len(&self) -> usize {
        match self.tab {
            Tab::Agents => self.agents.len(),
            Tab::Sessions => self.sessions.len(),
            Tab::Costs => self.daily_costs.len(),
            Tab::Models => self.models.len(),
            Tab::Advisor => self.catalog.len(),
            Tab::Hooks => self.hooks.len(),
            Tab::Plugins => self.plugins.len(),
            Tab::Mcp => self.mcp_servers.len(),
            _ => 0,
        }
    }

    pub fn catalog_summary(&self) -> (usize, usize, usize, usize) {
        self.catalog_summary_cache
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

        match self.tab {
            Tab::Dashboard => {
                if let Some(v) = client.get("/stats").await {
                    self.stats = v;
                }
                if let Some(v) = client.get("/system").await {
                    self.hardware = v;
                }
                if let Some(v) = client.get("/activity").await {
                    if let Some(arr) = v.as_array() {
                        self.activity = arr.clone();
                    }
                }
                if let Some(v) = client.get("/models/advisor").await {
                    if let Some(arr) = v.as_array() {
                        self.total_savings = arr
                            .iter()
                            .filter(|a| {
                                a.get("can_run_locally")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false)
                            })
                            .map(|a| {
                                a.get("potential_savings")
                                    .and_then(|v| v.as_f64())
                                    .unwrap_or(0.0)
                            })
                            .sum();
                    }
                }
            }
            Tab::Agents => {
                if let Some(v) = client.get("/agents").await {
                    if let Some(arr) = v.as_array() {
                        self.agents = arr.clone();
                    }
                }
            }
            Tab::Sessions => {
                if let Some(v) = client.get("/sessions").await {
                    if let Some(arr) = v.as_array() {
                        self.sessions = arr.clone();
                    }
                }
            }
            Tab::Costs => {
                if let Some(v) = client.get("/costs/summary").await {
                    self.cost_summary = v.get("summary").cloned().unwrap_or(v);
                }
                if let Some(v) = client.get("/costs/daily").await {
                    if let Some(arr) = v.as_array() {
                        self.daily_costs = arr.clone();
                    }
                }
            }
            Tab::Models => {
                if let Some(v) = client.get("/models").await {
                    if let Some(arr) = v.as_array() {
                        self.models = arr.clone();
                    }
                }
                if let Some(v) = client.get("/models/advisor").await {
                    if let Some(arr) = v.as_array() {
                        self.advisories = arr.clone();
                    }
                }
            }
            Tab::Advisor => {
                if let Some(v) = client.get("/models/catalog/runnable").await {
                    if let Some(entries) = v.get("entries").and_then(|e| e.as_array()) {
                        self.catalog = entries.clone();
                    }
                    if let Some(summary) = v.get("summary") {
                        let perfect =
                            summary.get("perfect").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                        let good =
                            summary.get("good").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                        let marginal = summary
                            .get("marginal")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as usize;
                        let total = summary
                            .get("catalog_size")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as usize;
                        self.catalog_summary_cache = (perfect, good, marginal, total);
                    }
                }
            }
            Tab::Hooks => {
                if let Some(v) = client.get("/hooks").await {
                    if let Some(arr) = v.as_array() {
                        self.hooks = arr.clone();
                    }
                }
            }
            Tab::Plugins => {
                if let Some(v) = client.get("/plugins").await {
                    if let Some(arr) = v.as_array() {
                        self.plugins = arr.clone();
                    }
                }
            }
            Tab::Mcp => {
                if let Some(v) = client.get("/mcp").await {
                    if let Some(arr) = v.as_array() {
                        self.mcp_servers = arr.clone();
                    }
                }
            }
            Tab::Metrics => {
                if let Some(v) = client.get("/metrics").await {
                    let inner = v.get("metrics").cloned().unwrap_or(v);
                    self.metrics = inner;
                }
                if let Some(v) = client.get("/system").await {
                    self.hardware = v;
                }
            }
        }
    }
}
