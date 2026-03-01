use crate::client::*;
use crate::theme::{Theme, THEMES};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Agents,
    Sessions,
    Costs,
    Models,
    Metrics,
    Plugins,
    Hooks,
    Mcp,
    Help,
}

impl Tab {
    pub fn all() -> &'static [Tab] {
        &[
            Tab::Dashboard,
            Tab::Agents,
            Tab::Sessions,
            Tab::Costs,
            Tab::Models,
            Tab::Metrics,
            Tab::Plugins,
            Tab::Hooks,
            Tab::Mcp,
            Tab::Help,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Tab::Dashboard => "Dashboard",
            Tab::Agents => "Agents",
            Tab::Sessions => "Sessions",
            Tab::Costs => "Costs",
            Tab::Models => "Models",
            Tab::Metrics => "Metrics",
            Tab::Plugins => "Plugins",
            Tab::Hooks => "Hooks",
            Tab::Mcp => "MCP",
            Tab::Help => "Help",
        }
    }

    pub fn index(&self) -> usize {
        Tab::all().iter().position(|t| t == self).unwrap_or(0)
    }

    pub fn from_index(i: usize) -> Tab {
        Tab::all().get(i).copied().unwrap_or(Tab::Dashboard)
    }

    pub fn from_key(c: char) -> Option<Tab> {
        match c {
            '1' => Some(Tab::Dashboard),
            '2' => Some(Tab::Agents),
            '3' => Some(Tab::Sessions),
            '4' => Some(Tab::Costs),
            '5' => Some(Tab::Models),
            '6' => Some(Tab::Metrics),
            '7' => Some(Tab::Plugins),
            '8' => Some(Tab::Hooks),
            '9' => Some(Tab::Mcp),
            '0' => Some(Tab::Help),
            _ => None,
        }
    }
}

pub struct App {
    pub running: bool,
    pub current_tab: Tab,
    pub theme_index: usize,
    pub scroll_offset: usize,
    pub selected_index: usize,
    pub search_query: String,
    pub searching: bool,
    pub status_message: Option<String>,

    pub agents: Vec<Agent>,
    pub sessions: Vec<Session>,
    pub cost_summary: Option<CostSummary>,
    pub daily_costs: Vec<DailyCostSummary>,
    pub models: Vec<ModelInfo>,
    pub metrics: Option<SystemMetrics>,
    pub metrics_history: Option<MetricsHistory>,
    pub plugins: Vec<PluginManifest>,
    pub hooks: Vec<HookConfig>,
    pub mcp_servers: Vec<McpServer>,
    pub stats: Option<DashboardStats>,
    pub activity: Vec<ActivityEvent>,
    pub health: Option<HealthStatus>,
    pub last_error: Option<String>,
    pub connected: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            current_tab: Tab::Dashboard,
            theme_index: 0,
            scroll_offset: 0,
            selected_index: 0,
            search_query: String::new(),
            searching: false,
            status_message: None,

            agents: Vec::new(),
            sessions: Vec::new(),
            cost_summary: None,
            daily_costs: Vec::new(),
            models: Vec::new(),
            metrics: None,
            metrics_history: None,
            plugins: Vec::new(),
            hooks: Vec::new(),
            mcp_servers: Vec::new(),
            stats: None,
            activity: Vec::new(),
            health: None,
            last_error: None,
            connected: false,
        }
    }

    pub fn theme(&self) -> &Theme {
        &THEMES[self.theme_index]
    }

    pub fn next_theme(&mut self) {
        self.theme_index = (self.theme_index + 1) % THEMES.len();
        self.status_message = Some(format!("Theme: {}", self.theme().name));
    }

    pub fn next_tab(&mut self) {
        let tabs = Tab::all();
        let idx = (self.current_tab.index() + 1) % tabs.len();
        self.switch_tab(Tab::from_index(idx));
    }

    pub fn prev_tab(&mut self) {
        let tabs = Tab::all();
        let idx = if self.current_tab.index() == 0 {
            tabs.len() - 1
        } else {
            self.current_tab.index() - 1
        };
        self.switch_tab(Tab::from_index(idx));
    }

    pub fn switch_tab(&mut self, tab: Tab) {
        self.current_tab = tab;
        self.scroll_offset = 0;
        self.selected_index = 0;
    }

    pub fn scroll_down(&mut self) {
        self.selected_index = self.selected_index.saturating_add(1);
        self.clamp_selection();
    }

    pub fn scroll_up(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
    }

    pub fn list_len(&self) -> usize {
        match self.current_tab {
            Tab::Agents => self.agents.len(),
            Tab::Sessions => self.sessions.len(),
            Tab::Models => self.models.len(),
            Tab::Plugins => self.plugins.len(),
            Tab::Hooks => self.hooks.len(),
            Tab::Mcp => self.mcp_servers.len(),
            Tab::Costs => self.daily_costs.len(),
            _ => 0,
        }
    }

    pub fn clamp_selection(&mut self) {
        let len = self.list_len();
        if len > 0 && self.selected_index >= len {
            self.selected_index = len - 1;
        }
    }

    pub fn apply_refresh(&mut self, r: RefreshResult) {
        self.connected = r.connected;
        if let Some(e) = r.error {
            self.last_error = Some(e);
        } else {
            self.last_error = None;
        }
        if let Some(h) = r.health {
            self.health = Some(h);
        }
        if let Some(s) = r.stats {
            self.stats = Some(s);
        }
        if let Some(a) = r.activity {
            self.activity = a;
        }
        if let Some(m) = r.metrics {
            self.metrics = Some(m);
        }
        if let Some(h) = r.metrics_history {
            self.metrics_history = Some(h);
        }
        if let Some(a) = r.agents {
            self.agents = a;
        }
        if let Some(s) = r.sessions {
            self.sessions = s;
        }
        if let Some(c) = r.cost_summary {
            self.cost_summary = Some(c);
        }
        if let Some(d) = r.daily_costs {
            self.daily_costs = d;
        }
        if let Some(m) = r.models {
            self.models = m;
        }
        if let Some(p) = r.plugins {
            self.plugins = p;
        }
        if let Some(h) = r.hooks {
            self.hooks = h;
        }
        if let Some(m) = r.mcp_servers {
            self.mcp_servers = m;
        }
        self.clamp_selection();
    }
}
