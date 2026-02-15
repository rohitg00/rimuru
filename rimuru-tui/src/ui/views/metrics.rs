use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Sparkline, Table},
    Frame,
};

use crate::app::App;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HistoricalRange {
    #[default]
    LastHour,
    LastDay,
}

impl HistoricalRange {
    pub fn label(&self) -> &'static str {
        match self {
            HistoricalRange::LastHour => "Last Hour",
            HistoricalRange::LastDay => "Last Day",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            HistoricalRange::LastHour => HistoricalRange::LastDay,
            HistoricalRange::LastDay => HistoricalRange::LastHour,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertLevel {
    Normal,
    Warning,
    Critical,
}

impl AlertLevel {
    pub fn from_percentage(value: f64, warning_threshold: f64, critical_threshold: f64) -> Self {
        if value >= critical_threshold {
            AlertLevel::Critical
        } else if value >= warning_threshold {
            AlertLevel::Warning
        } else {
            AlertLevel::Normal
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            AlertLevel::Normal => "●",
            AlertLevel::Warning => "◐",
            AlertLevel::Critical => "▲",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub cpu_history: Vec<u64>,
    pub memory_used: u64,
    pub memory_total: u64,
    pub memory_history: Vec<u64>,
    pub disk_used: u64,
    pub disk_total: u64,
    pub disk_path: String,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: 45.2,
            cpu_history: vec![35, 42, 38, 50, 45, 52, 48, 55, 42, 45, 48, 52],
            memory_used: 8_589_934_592,
            memory_total: 16_106_127_360,
            memory_history: vec![50, 52, 55, 58, 60, 62, 58, 55, 60, 62, 65, 63],
            disk_used: 1_610_612_736,
            disk_total: 10_737_418_240,
            disk_path: "~/.rimuru".to_string(),
        }
    }
}

impl SystemMetrics {
    pub fn memory_percentage(&self) -> f64 {
        if self.memory_total == 0 {
            0.0
        } else {
            (self.memory_used as f64 / self.memory_total as f64) * 100.0
        }
    }

    pub fn disk_percentage(&self) -> f64 {
        if self.disk_total == 0 {
            0.0
        } else {
            (self.disk_used as f64 / self.disk_total as f64) * 100.0
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgentSessionInfo {
    pub name: &'static str,
    pub active_sessions: u32,
    pub tokens_per_minute: u64,
    pub status: AgentSessionStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentSessionStatus {
    Active,
    Idle,
    Offline,
}

impl AgentSessionStatus {
    pub fn icon(&self) -> &'static str {
        match self {
            AgentSessionStatus::Active => "●",
            AgentSessionStatus::Idle => "◐",
            AgentSessionStatus::Offline => "○",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            AgentSessionStatus::Active => "Active",
            AgentSessionStatus::Idle => "Idle",
            AgentSessionStatus::Offline => "Offline",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MetricsViewState {
    pub refresh_rate_secs: u8,
    pub historical_range: HistoricalRange,
    pub last_update_tick: u64,
    pub alerts_enabled: bool,
    pub cpu_warning_threshold: f64,
    pub cpu_critical_threshold: f64,
    pub memory_warning_threshold: f64,
    pub memory_critical_threshold: f64,
}

impl MetricsViewState {
    pub fn new() -> Self {
        Self {
            refresh_rate_secs: 5,
            historical_range: HistoricalRange::LastHour,
            last_update_tick: 0,
            alerts_enabled: true,
            cpu_warning_threshold: 70.0,
            cpu_critical_threshold: 90.0,
            memory_warning_threshold: 75.0,
            memory_critical_threshold: 90.0,
        }
    }

    pub fn increase_refresh_rate(&mut self) {
        if self.refresh_rate_secs < 10 {
            self.refresh_rate_secs += 1;
        }
    }

    pub fn decrease_refresh_rate(&mut self) {
        if self.refresh_rate_secs > 1 {
            self.refresh_rate_secs -= 1;
        }
    }
}

pub struct MetricsView;

impl MetricsView {
    pub fn get_system_metrics() -> SystemMetrics {
        SystemMetrics::default()
    }

    pub fn get_agent_sessions() -> Vec<AgentSessionInfo> {
        vec![
            AgentSessionInfo {
                name: "Claude Code",
                active_sessions: 2,
                tokens_per_minute: 1250,
                status: AgentSessionStatus::Active,
            },
            AgentSessionInfo {
                name: "OpenCode",
                active_sessions: 1,
                tokens_per_minute: 850,
                status: AgentSessionStatus::Active,
            },
            AgentSessionInfo {
                name: "Codex",
                active_sessions: 0,
                tokens_per_minute: 0,
                status: AgentSessionStatus::Idle,
            },
            AgentSessionInfo {
                name: "Copilot",
                active_sessions: 0,
                tokens_per_minute: 0,
                status: AgentSessionStatus::Offline,
            },
        ]
    }

    pub fn get_historical_data(range: HistoricalRange) -> (Vec<u64>, Vec<u64>) {
        match range {
            HistoricalRange::LastHour => (
                vec![35, 42, 38, 50, 45, 52, 48, 55, 42, 45, 48, 52, 55, 48, 50],
                vec![50, 52, 55, 58, 60, 62, 58, 55, 60, 62, 65, 63, 60, 58, 62],
            ),
            HistoricalRange::LastDay => (
                vec![
                    30, 35, 40, 45, 50, 55, 48, 42, 38, 45, 52, 58, 55, 50, 45, 40, 35, 38, 42, 48,
                    52, 55, 50, 45,
                ],
                vec![
                    45, 48, 52, 55, 60, 65, 62, 58, 55, 60, 65, 70, 68, 62, 58, 55, 52, 55, 58, 62,
                    65, 68, 65, 60,
                ],
            ),
        }
    }

    pub fn get_peak_metrics(range: HistoricalRange) -> (f64, f64, &'static str, &'static str) {
        match range {
            HistoricalRange::LastHour => (72.5, 78.2, "15 min ago", "32 min ago"),
            HistoricalRange::LastDay => (85.3, 82.1, "6 hrs ago", "14 hrs ago"),
        }
    }

    pub fn get_average_metrics(range: HistoricalRange) -> (f64, f64) {
        match range {
            HistoricalRange::LastHour => (38.4, 55.1),
            HistoricalRange::LastDay => (42.8, 58.3),
        }
    }

    pub fn agent_count() -> usize {
        4
    }

    pub fn render(frame: &mut Frame, area: Rect, app: &App) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(10)])
            .split(area);

        Self::render_header(frame, chunks[0], app);
        Self::render_main_content(frame, chunks[1], app);
    }

    fn render_header(frame: &mut Frame, area: Rect, app: &App) {
        let theme = app.current_theme();
        let state = &app.metrics_state;

        let historical_ranges = [HistoricalRange::LastHour, HistoricalRange::LastDay];
        let tabs: Vec<Span> = historical_ranges
            .iter()
            .map(|r| {
                let is_selected = *r == state.historical_range;
                if is_selected {
                    Span::styled(
                        format!(" [{}] ", r.label()),
                        Style::default()
                            .fg(theme.accent())
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Span::styled(
                        format!("  {}  ", r.label()),
                        Style::default().fg(theme.foreground_dim()),
                    )
                }
            })
            .collect();

        let refresh_text = format!("Refresh: {}s", state.refresh_rate_secs);

        let block = Block::default()
            .title(" System Metrics ")
            .title_bottom(
                Line::from(vec![
                    Span::styled(" h ", Style::default().fg(theme.accent())),
                    Span::styled(
                        "toggle history  ",
                        Style::default().fg(theme.foreground_dim()),
                    ),
                    Span::styled("+/- ", Style::default().fg(theme.accent())),
                    Span::styled(
                        "refresh rate  ",
                        Style::default().fg(theme.foreground_dim()),
                    ),
                    Span::styled("r ", Style::default().fg(theme.accent())),
                    Span::styled("refresh now ", Style::default().fg(theme.foreground_dim())),
                ])
                .alignment(Alignment::Center),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border()))
            .style(Style::default().bg(theme.surface()));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let header_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(30), Constraint::Length(15)])
            .split(inner);

        let tabs_line = Line::from(tabs);
        let tabs_para = Paragraph::new(tabs_line)
            .alignment(Alignment::Left)
            .style(Style::default().bg(theme.surface()));
        frame.render_widget(tabs_para, header_layout[0]);

        let refresh_para = Paragraph::new(refresh_text)
            .alignment(Alignment::Right)
            .style(
                Style::default()
                    .fg(theme.foreground_dim())
                    .bg(theme.surface()),
            );
        frame.render_widget(refresh_para, header_layout[1]);
    }

    fn render_main_content(frame: &mut Frame, area: Rect, app: &App) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        Self::render_system_panel(frame, chunks[0], app);
        Self::render_sessions_panel(frame, chunks[1], app);
    }

    fn render_system_panel(frame: &mut Frame, area: Rect, app: &App) {
        let _theme = app.current_theme();
        let state = &app.metrics_state;
        let metrics = Self::get_system_metrics();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Min(6),
            ])
            .split(area);

        Self::render_cpu_gauge(frame, chunks[0], app, &metrics, state);
        Self::render_memory_gauge(frame, chunks[1], app, &metrics, state);
        Self::render_disk_gauge(frame, chunks[2], app, &metrics);
        Self::render_history_panel(frame, chunks[3], app, state.historical_range);
    }

    fn render_cpu_gauge(
        frame: &mut Frame,
        area: Rect,
        app: &App,
        metrics: &SystemMetrics,
        state: &MetricsViewState,
    ) {
        let theme = app.current_theme();
        let alert_level = AlertLevel::from_percentage(
            metrics.cpu_usage,
            state.cpu_warning_threshold,
            state.cpu_critical_threshold,
        );

        let gauge_color = match alert_level {
            AlertLevel::Normal => theme.success(),
            AlertLevel::Warning => theme.warning(),
            AlertLevel::Critical => theme.error(),
        };

        let alert_icon = if state.alerts_enabled && alert_level != AlertLevel::Normal {
            format!(" {} ", alert_level.icon())
        } else {
            String::new()
        };

        let title = format!(" CPU Usage{}", alert_icon);
        let label = format!("{:.1}%", metrics.cpu_usage);

        let gauge = Gauge::default()
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border()))
                    .style(Style::default().bg(theme.surface())),
            )
            .gauge_style(Style::default().fg(gauge_color).bg(theme.background()))
            .percent(metrics.cpu_usage as u16)
            .label(Span::styled(
                label,
                Style::default()
                    .fg(theme.foreground())
                    .add_modifier(Modifier::BOLD),
            ));

        frame.render_widget(gauge, area);
    }

    fn render_memory_gauge(
        frame: &mut Frame,
        area: Rect,
        app: &App,
        metrics: &SystemMetrics,
        state: &MetricsViewState,
    ) {
        let theme = app.current_theme();
        let mem_pct = metrics.memory_percentage();
        let alert_level = AlertLevel::from_percentage(
            mem_pct,
            state.memory_warning_threshold,
            state.memory_critical_threshold,
        );

        let gauge_color = match alert_level {
            AlertLevel::Normal => theme.success(),
            AlertLevel::Warning => theme.warning(),
            AlertLevel::Critical => theme.error(),
        };

        let alert_icon = if state.alerts_enabled && alert_level != AlertLevel::Normal {
            format!(" {} ", alert_level.icon())
        } else {
            String::new()
        };

        let used_gb = metrics.memory_used as f64 / 1_073_741_824.0;
        let total_gb = metrics.memory_total as f64 / 1_073_741_824.0;
        let title = format!(" Memory Usage{}", alert_icon);
        let label = format!("{:.1} / {:.1} GB ({:.1}%)", used_gb, total_gb, mem_pct);

        let gauge = Gauge::default()
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border()))
                    .style(Style::default().bg(theme.surface())),
            )
            .gauge_style(Style::default().fg(gauge_color).bg(theme.background()))
            .percent(mem_pct as u16)
            .label(Span::styled(
                label,
                Style::default()
                    .fg(theme.foreground())
                    .add_modifier(Modifier::BOLD),
            ));

        frame.render_widget(gauge, area);
    }

    fn render_disk_gauge(frame: &mut Frame, area: Rect, app: &App, metrics: &SystemMetrics) {
        let theme = app.current_theme();
        let disk_pct = metrics.disk_percentage();

        let gauge_color = if disk_pct >= 90.0 {
            theme.error()
        } else if disk_pct >= 75.0 {
            theme.warning()
        } else {
            theme.success()
        };

        let used_gb = metrics.disk_used as f64 / 1_073_741_824.0;
        let total_gb = metrics.disk_total as f64 / 1_073_741_824.0;
        let title = format!(" Disk Usage ({}) ", metrics.disk_path);
        let label = format!("{:.1} / {:.1} GB ({:.1}%)", used_gb, total_gb, disk_pct);

        let gauge = Gauge::default()
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border()))
                    .style(Style::default().bg(theme.surface())),
            )
            .gauge_style(Style::default().fg(gauge_color).bg(theme.background()))
            .percent(disk_pct as u16)
            .label(Span::styled(
                label,
                Style::default()
                    .fg(theme.foreground())
                    .add_modifier(Modifier::BOLD),
            ));

        frame.render_widget(gauge, area);
    }

    fn render_history_panel(frame: &mut Frame, area: Rect, app: &App, range: HistoricalRange) {
        let theme = app.current_theme();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let (cpu_history, _mem_history) = Self::get_historical_data(range);
        let (peak_cpu, peak_mem, cpu_time, mem_time) = Self::get_peak_metrics(range);
        let (avg_cpu, avg_mem) = Self::get_average_metrics(range);

        let cpu_sparkline = Sparkline::default()
            .block(
                Block::default()
                    .title(format!(" CPU History ({}) ", range.label()))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border()))
                    .style(Style::default().bg(theme.surface())),
            )
            .data(&cpu_history)
            .style(Style::default().fg(theme.accent()));

        frame.render_widget(cpu_sparkline, chunks[0]);

        let mem_block = Block::default()
            .title(format!(" Memory History ({}) ", range.label()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border()))
            .style(Style::default().bg(theme.surface()));
        let mem_inner = mem_block.inner(chunks[1]);
        frame.render_widget(mem_block, chunks[1]);

        let stats_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(mem_inner);

        let peak_lines = vec![
            Line::from(vec![
                Span::styled("Peak CPU:   ", Style::default().fg(theme.foreground_dim())),
                Span::styled(
                    format!("{:.1}%", peak_cpu),
                    Style::default().fg(theme.warning()),
                ),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(
                    format!("({})", cpu_time),
                    Style::default().fg(theme.foreground_dim()),
                ),
            ]),
            Line::from(vec![
                Span::styled("Peak Mem:   ", Style::default().fg(theme.foreground_dim())),
                Span::styled(
                    format!("{:.1}%", peak_mem),
                    Style::default().fg(theme.warning()),
                ),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(
                    format!("({})", mem_time),
                    Style::default().fg(theme.foreground_dim()),
                ),
            ]),
        ];

        let avg_lines = vec![
            Line::from(vec![
                Span::styled("Avg CPU:    ", Style::default().fg(theme.foreground_dim())),
                Span::styled(
                    format!("{:.1}%", avg_cpu),
                    Style::default().fg(theme.success()),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Avg Mem:    ", Style::default().fg(theme.foreground_dim())),
                Span::styled(
                    format!("{:.1}%", avg_mem),
                    Style::default().fg(theme.success()),
                ),
            ]),
        ];

        let peak_para = Paragraph::new(peak_lines).style(Style::default().bg(theme.surface()));
        let avg_para = Paragraph::new(avg_lines).style(Style::default().bg(theme.surface()));

        frame.render_widget(peak_para, stats_layout[0]);
        frame.render_widget(avg_para, stats_layout[1]);
    }

    fn render_sessions_panel(frame: &mut Frame, area: Rect, app: &App) {
        let _theme = app.current_theme();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),
                Constraint::Min(6),
                Constraint::Length(7),
            ])
            .split(area);

        Self::render_active_sessions(frame, chunks[0], app);
        Self::render_agent_breakdown(frame, chunks[1], app);
        Self::render_refresh_settings(frame, chunks[2], app);
    }

    fn render_active_sessions(frame: &mut Frame, area: Rect, app: &App) {
        let theme = app.current_theme();
        let agents = Self::get_agent_sessions();

        let total_active: u32 = agents.iter().map(|a| a.active_sessions).sum();
        let total_tokens: u64 = agents.iter().map(|a| a.tokens_per_minute).sum();

        let block = Block::default()
            .title(" Active Sessions ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border()))
            .style(Style::default().bg(theme.surface()));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let lines = vec![
            Line::from(vec![
                Span::styled(
                    "  Total Active: ",
                    Style::default().fg(theme.foreground_dim()),
                ),
                Span::styled(
                    format!("{}", total_active),
                    Style::default()
                        .fg(theme.accent())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" sessions", Style::default().fg(theme.foreground_dim())),
            ]),
            Line::from(vec![
                Span::styled(
                    "  Token Rate:   ",
                    Style::default().fg(theme.foreground_dim()),
                ),
                Span::styled(
                    format_number(total_tokens),
                    Style::default().fg(theme.foreground()),
                ),
                Span::styled("/min", Style::default().fg(theme.foreground_dim())),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Status: ", Style::default().fg(theme.foreground_dim())),
                Span::styled(
                    format!(
                        "{} active  ",
                        agents
                            .iter()
                            .filter(|a| a.status == AgentSessionStatus::Active)
                            .count()
                    ),
                    Style::default().fg(theme.success()),
                ),
                Span::styled(
                    format!(
                        "{} idle  ",
                        agents
                            .iter()
                            .filter(|a| a.status == AgentSessionStatus::Idle)
                            .count()
                    ),
                    Style::default().fg(theme.warning()),
                ),
                Span::styled(
                    format!(
                        "{} offline",
                        agents
                            .iter()
                            .filter(|a| a.status == AgentSessionStatus::Offline)
                            .count()
                    ),
                    Style::default().fg(theme.error()),
                ),
            ]),
        ];

        let para = Paragraph::new(lines).style(Style::default().bg(theme.surface()));
        frame.render_widget(para, inner);
    }

    fn render_agent_breakdown(frame: &mut Frame, area: Rect, app: &App) {
        let theme = app.current_theme();
        let agents = Self::get_agent_sessions();

        let header = Row::new(vec![
            Cell::from(Span::raw("")),
            Cell::from(Span::raw("Agent")),
            Cell::from(Span::raw("Sessions")),
            Cell::from(Span::raw("Tokens/min")),
            Cell::from(Span::raw("Status")),
        ])
        .style(
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        )
        .height(1);

        let rows: Vec<Row> = agents
            .iter()
            .map(|agent| {
                let status_color = match agent.status {
                    AgentSessionStatus::Active => theme.success(),
                    AgentSessionStatus::Idle => theme.warning(),
                    AgentSessionStatus::Offline => theme.error(),
                };

                Row::new([
                    Cell::from(Span::styled(
                        agent.status.icon(),
                        Style::default().fg(status_color),
                    )),
                    Cell::from(Span::styled(
                        agent.name,
                        Style::default().fg(theme.foreground()),
                    )),
                    Cell::from(Span::styled(
                        format!("{}", agent.active_sessions),
                        Style::default().fg(if agent.active_sessions > 0 {
                            theme.accent()
                        } else {
                            theme.foreground_dim()
                        }),
                    )),
                    Cell::from(Span::styled(
                        if agent.tokens_per_minute > 0 {
                            format_number(agent.tokens_per_minute)
                        } else {
                            "-".to_string()
                        },
                        Style::default().fg(theme.foreground_dim()),
                    )),
                    Cell::from(Span::styled(
                        agent.status.label(),
                        Style::default().fg(status_color),
                    )),
                ])
                .height(1)
            })
            .collect();

        let widths = [
            Constraint::Length(2),
            Constraint::Min(12),
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Length(10),
        ];

        let table = Table::new(rows, widths).header(header).block(
            Block::default()
                .title(" Agent Breakdown ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border()))
                .style(Style::default().bg(theme.surface())),
        );

        frame.render_widget(table, area);
    }

    fn render_refresh_settings(frame: &mut Frame, area: Rect, app: &App) {
        let theme = app.current_theme();
        let state = &app.metrics_state;

        let block = Block::default()
            .title(" Refresh Settings ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border()))
            .style(Style::default().bg(theme.surface()));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let rate_bar_len = state.refresh_rate_secs as usize;
        let rate_bar = "█".repeat(rate_bar_len) + &"░".repeat(10 - rate_bar_len);

        let lines = vec![
            Line::from(vec![
                Span::styled(
                    "  Refresh Rate: ",
                    Style::default().fg(theme.foreground_dim()),
                ),
                Span::styled(
                    format!("{} seconds", state.refresh_rate_secs),
                    Style::default().fg(theme.foreground()),
                ),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(
                    format!("[{}]", rate_bar),
                    Style::default().fg(theme.accent()),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "  Last Update:  ",
                    Style::default().fg(theme.foreground_dim()),
                ),
                Span::styled("Just now", Style::default().fg(theme.success())),
            ]),
            Line::from(vec![
                Span::styled("  Press ", Style::default().fg(theme.foreground_dim())),
                Span::styled("'r'", Style::default().fg(theme.accent())),
                Span::styled(
                    " to force refresh",
                    Style::default().fg(theme.foreground_dim()),
                ),
            ]),
        ];

        let para = Paragraph::new(lines).style(Style::default().bg(theme.surface()));
        frame.render_widget(para, inner);
    }
}

fn format_number(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        format!("{}", n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_historical_range_label() {
        assert_eq!(HistoricalRange::LastHour.label(), "Last Hour");
        assert_eq!(HistoricalRange::LastDay.label(), "Last Day");
    }

    #[test]
    fn test_historical_range_toggle() {
        assert_eq!(HistoricalRange::LastHour.toggle(), HistoricalRange::LastDay);
        assert_eq!(HistoricalRange::LastDay.toggle(), HistoricalRange::LastHour);
    }

    #[test]
    fn test_alert_level_from_percentage() {
        assert_eq!(
            AlertLevel::from_percentage(50.0, 70.0, 90.0),
            AlertLevel::Normal
        );
        assert_eq!(
            AlertLevel::from_percentage(75.0, 70.0, 90.0),
            AlertLevel::Warning
        );
        assert_eq!(
            AlertLevel::from_percentage(95.0, 70.0, 90.0),
            AlertLevel::Critical
        );
    }

    #[test]
    fn test_alert_level_icon() {
        assert_eq!(AlertLevel::Normal.icon(), "●");
        assert_eq!(AlertLevel::Warning.icon(), "◐");
        assert_eq!(AlertLevel::Critical.icon(), "▲");
    }

    #[test]
    fn test_system_metrics_memory_percentage() {
        let metrics = SystemMetrics {
            memory_used: 8_000_000_000,
            memory_total: 16_000_000_000,
            ..Default::default()
        };
        assert!((metrics.memory_percentage() - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_system_metrics_memory_percentage_zero_total() {
        let metrics = SystemMetrics {
            memory_used: 8_000_000_000,
            memory_total: 0,
            ..Default::default()
        };
        assert_eq!(metrics.memory_percentage(), 0.0);
    }

    #[test]
    fn test_system_metrics_disk_percentage() {
        let metrics = SystemMetrics {
            disk_used: 5_000_000_000,
            disk_total: 10_000_000_000,
            ..Default::default()
        };
        assert!((metrics.disk_percentage() - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_system_metrics_disk_percentage_zero_total() {
        let metrics = SystemMetrics {
            disk_used: 5_000_000_000,
            disk_total: 0,
            ..Default::default()
        };
        assert_eq!(metrics.disk_percentage(), 0.0);
    }

    #[test]
    fn test_agent_session_status_icon() {
        assert_eq!(AgentSessionStatus::Active.icon(), "●");
        assert_eq!(AgentSessionStatus::Idle.icon(), "◐");
        assert_eq!(AgentSessionStatus::Offline.icon(), "○");
    }

    #[test]
    fn test_agent_session_status_label() {
        assert_eq!(AgentSessionStatus::Active.label(), "Active");
        assert_eq!(AgentSessionStatus::Idle.label(), "Idle");
        assert_eq!(AgentSessionStatus::Offline.label(), "Offline");
    }

    #[test]
    fn test_metrics_view_state_refresh_rate() {
        let mut state = MetricsViewState::new();
        assert_eq!(state.refresh_rate_secs, 5);

        state.increase_refresh_rate();
        assert_eq!(state.refresh_rate_secs, 6);

        state.decrease_refresh_rate();
        assert_eq!(state.refresh_rate_secs, 5);

        state.refresh_rate_secs = 10;
        state.increase_refresh_rate();
        assert_eq!(state.refresh_rate_secs, 10);

        state.refresh_rate_secs = 1;
        state.decrease_refresh_rate();
        assert_eq!(state.refresh_rate_secs, 1);
    }

    #[test]
    fn test_get_agent_sessions() {
        let agents = MetricsView::get_agent_sessions();
        assert_eq!(agents.len(), 4);
        assert_eq!(agents[0].name, "Claude Code");
        assert_eq!(agents[0].status, AgentSessionStatus::Active);
    }

    #[test]
    fn test_get_historical_data() {
        let (cpu_hour, mem_hour) = MetricsView::get_historical_data(HistoricalRange::LastHour);
        let (cpu_day, mem_day) = MetricsView::get_historical_data(HistoricalRange::LastDay);

        assert!(cpu_hour.len() > 0);
        assert!(mem_hour.len() > 0);
        assert!(cpu_day.len() > cpu_hour.len());
        assert!(mem_day.len() > mem_hour.len());
    }

    #[test]
    fn test_get_peak_metrics() {
        let (peak_cpu, peak_mem, cpu_time, mem_time) =
            MetricsView::get_peak_metrics(HistoricalRange::LastHour);
        assert!(peak_cpu > 0.0);
        assert!(peak_mem > 0.0);
        assert!(!cpu_time.is_empty());
        assert!(!mem_time.is_empty());
    }

    #[test]
    fn test_get_average_metrics() {
        let (avg_cpu, avg_mem) = MetricsView::get_average_metrics(HistoricalRange::LastHour);
        assert!(avg_cpu > 0.0);
        assert!(avg_mem > 0.0);
    }

    #[test]
    fn test_agent_count() {
        assert_eq!(MetricsView::agent_count(), 4);
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(500), "500");
        assert_eq!(format_number(1_500), "1.5K");
        assert_eq!(format_number(85_432), "85.4K");
        assert_eq!(format_number(1_500_000), "1.5M");
    }
}
