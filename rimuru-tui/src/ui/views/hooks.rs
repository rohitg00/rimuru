use chrono::{DateTime, Utc};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Tabs},
    Frame,
};
use uuid::Uuid;

use crate::app::App;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HooksTab {
    #[default]
    ByType,
    Handlers,
    ExecutionLog,
}

impl HooksTab {
    pub fn all() -> &'static [HooksTab] {
        &[HooksTab::ByType, HooksTab::Handlers, HooksTab::ExecutionLog]
    }

    pub fn name(&self) -> &'static str {
        match self {
            HooksTab::ByType => "By Type",
            HooksTab::Handlers => "Handlers",
            HooksTab::ExecutionLog => "Execution Log",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            HooksTab::ByType => HooksTab::Handlers,
            HooksTab::Handlers => HooksTab::ExecutionLog,
            HooksTab::ExecutionLog => HooksTab::ByType,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            HooksTab::ByType => HooksTab::ExecutionLog,
            HooksTab::Handlers => HooksTab::ByType,
            HooksTab::ExecutionLog => HooksTab::Handlers,
        }
    }

    pub fn index(&self) -> usize {
        match self {
            HooksTab::ByType => 0,
            HooksTab::Handlers => 1,
            HooksTab::ExecutionLog => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookType {
    PreSessionStart,
    PostSessionEnd,
    OnCostRecorded,
    OnMetricsCollected,
    OnAgentConnect,
    OnAgentDisconnect,
    OnSyncComplete,
    OnPluginLoaded,
    OnPluginUnloaded,
    OnConfigChanged,
    OnError,
    Custom,
}

impl HookType {
    pub fn name(&self) -> &'static str {
        match self {
            HookType::PreSessionStart => "pre_session_start",
            HookType::PostSessionEnd => "post_session_end",
            HookType::OnCostRecorded => "on_cost_recorded",
            HookType::OnMetricsCollected => "on_metrics_collected",
            HookType::OnAgentConnect => "on_agent_connect",
            HookType::OnAgentDisconnect => "on_agent_disconnect",
            HookType::OnSyncComplete => "on_sync_complete",
            HookType::OnPluginLoaded => "on_plugin_loaded",
            HookType::OnPluginUnloaded => "on_plugin_unloaded",
            HookType::OnConfigChanged => "on_config_changed",
            HookType::OnError => "on_error",
            HookType::Custom => "custom",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            HookType::PreSessionStart => "Triggered before a session starts",
            HookType::PostSessionEnd => "Triggered after a session ends",
            HookType::OnCostRecorded => "Triggered when cost is recorded",
            HookType::OnMetricsCollected => "Triggered when metrics are collected",
            HookType::OnAgentConnect => "Triggered when an agent connects",
            HookType::OnAgentDisconnect => "Triggered when an agent disconnects",
            HookType::OnSyncComplete => "Triggered after sync completes",
            HookType::OnPluginLoaded => "Triggered when a plugin loads",
            HookType::OnPluginUnloaded => "Triggered when a plugin unloads",
            HookType::OnConfigChanged => "Triggered when configuration changes",
            HookType::OnError => "Triggered on errors",
            HookType::Custom => "User-defined custom hook",
        }
    }

    pub fn data_type(&self) -> &'static str {
        match self {
            HookType::PreSessionStart | HookType::PostSessionEnd => "Session",
            HookType::OnCostRecorded => "Cost",
            HookType::OnMetricsCollected => "Metrics",
            HookType::OnAgentConnect | HookType::OnAgentDisconnect => "Agent",
            HookType::OnSyncComplete => "Sync",
            HookType::OnPluginLoaded | HookType::OnPluginUnloaded => "Plugin",
            HookType::OnConfigChanged => "Config",
            HookType::OnError => "Error",
            HookType::Custom => "Custom",
        }
    }

    pub fn all_standard() -> Vec<Self> {
        vec![
            HookType::PreSessionStart,
            HookType::PostSessionEnd,
            HookType::OnCostRecorded,
            HookType::OnMetricsCollected,
            HookType::OnAgentConnect,
            HookType::OnAgentDisconnect,
            HookType::OnSyncComplete,
            HookType::OnPluginLoaded,
            HookType::OnPluginUnloaded,
            HookType::OnConfigChanged,
            HookType::OnError,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct HookTypeInfo {
    pub hook_type: HookType,
    pub handler_count: usize,
    pub enabled: bool,
    pub last_triggered: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct HookHandlerInfo {
    pub name: String,
    pub hook_type: HookType,
    pub priority: i32,
    pub enabled: bool,
    pub plugin_id: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStatus {
    Success,
    Failed,
    Aborted,
    Skipped,
}

impl ExecutionStatus {
    pub fn icon(&self) -> &'static str {
        match self {
            ExecutionStatus::Success => "✓",
            ExecutionStatus::Failed => "✗",
            ExecutionStatus::Aborted => "⊘",
            ExecutionStatus::Skipped => "⊳",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ExecutionStatus::Success => "Success",
            ExecutionStatus::Failed => "Failed",
            ExecutionStatus::Aborted => "Aborted",
            ExecutionStatus::Skipped => "Skipped",
        }
    }
}

#[derive(Debug, Clone)]
pub struct HookExecutionEntry {
    pub id: Uuid,
    pub hook_type: HookType,
    pub handler_name: String,
    pub status: ExecutionStatus,
    pub executed_at: DateTime<Utc>,
    pub duration_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct HooksViewState {
    pub current_tab: HooksTab,
    pub search_query: Option<String>,
    pub is_searching: bool,
    pub hook_types: Vec<HookTypeInfo>,
    pub handlers: Vec<HookHandlerInfo>,
    pub execution_log: Vec<HookExecutionEntry>,
    pub selected_hook_type: Option<HookType>,
}

impl HooksViewState {
    pub fn new() -> Self {
        Self {
            current_tab: HooksTab::ByType,
            search_query: None,
            is_searching: false,
            hook_types: Self::get_mock_hook_types(),
            handlers: Self::get_mock_handlers(),
            execution_log: Self::get_mock_execution_log(),
            selected_hook_type: None,
        }
    }

    fn get_mock_hook_types() -> Vec<HookTypeInfo> {
        use chrono::Duration;

        vec![
            HookTypeInfo {
                hook_type: HookType::OnCostRecorded,
                handler_count: 2,
                enabled: true,
                last_triggered: Some(Utc::now() - Duration::minutes(5)),
            },
            HookTypeInfo {
                hook_type: HookType::PostSessionEnd,
                handler_count: 1,
                enabled: true,
                last_triggered: Some(Utc::now() - Duration::minutes(15)),
            },
            HookTypeInfo {
                hook_type: HookType::OnMetricsCollected,
                handler_count: 1,
                enabled: true,
                last_triggered: Some(Utc::now() - Duration::seconds(30)),
            },
            HookTypeInfo {
                hook_type: HookType::PreSessionStart,
                handler_count: 1,
                enabled: true,
                last_triggered: Some(Utc::now() - Duration::hours(1)),
            },
            HookTypeInfo {
                hook_type: HookType::OnAgentConnect,
                handler_count: 0,
                enabled: true,
                last_triggered: Some(Utc::now() - Duration::hours(2)),
            },
            HookTypeInfo {
                hook_type: HookType::OnAgentDisconnect,
                handler_count: 0,
                enabled: true,
                last_triggered: None,
            },
            HookTypeInfo {
                hook_type: HookType::OnSyncComplete,
                handler_count: 0,
                enabled: false,
                last_triggered: None,
            },
            HookTypeInfo {
                hook_type: HookType::OnError,
                handler_count: 1,
                enabled: true,
                last_triggered: Some(Utc::now() - Duration::days(1)),
            },
        ]
    }

    fn get_mock_handlers() -> Vec<HookHandlerInfo> {
        vec![
            HookHandlerInfo {
                name: "cost_alert".to_string(),
                hook_type: HookType::OnCostRecorded,
                priority: 10,
                enabled: true,
                plugin_id: Some("builtin".to_string()),
                description: "Alerts when cost exceeds threshold".to_string(),
            },
            HookHandlerInfo {
                name: "cost_webhook".to_string(),
                hook_type: HookType::OnCostRecorded,
                priority: 0,
                enabled: true,
                plugin_id: Some("webhook-notifier".to_string()),
                description: "Sends cost data to webhook endpoint".to_string(),
            },
            HookHandlerInfo {
                name: "session_logger".to_string(),
                hook_type: HookType::PostSessionEnd,
                priority: 0,
                enabled: true,
                plugin_id: Some("builtin".to_string()),
                description: "Logs session data to file".to_string(),
            },
            HookHandlerInfo {
                name: "metrics_exporter".to_string(),
                hook_type: HookType::OnMetricsCollected,
                priority: 5,
                enabled: true,
                plugin_id: Some("builtin".to_string()),
                description: "Exports metrics to external service".to_string(),
            },
            HookHandlerInfo {
                name: "session_start_log".to_string(),
                hook_type: HookType::PreSessionStart,
                priority: 0,
                enabled: true,
                plugin_id: None,
                description: "Logs session start events".to_string(),
            },
            HookHandlerInfo {
                name: "error_notifier".to_string(),
                hook_type: HookType::OnError,
                priority: 100,
                enabled: true,
                plugin_id: Some("slack-notifier".to_string()),
                description: "Sends error notifications to Slack".to_string(),
            },
        ]
    }

    fn get_mock_execution_log() -> Vec<HookExecutionEntry> {
        use chrono::Duration;

        vec![
            HookExecutionEntry {
                id: Uuid::new_v4(),
                hook_type: HookType::OnMetricsCollected,
                handler_name: "metrics_exporter".to_string(),
                status: ExecutionStatus::Success,
                executed_at: Utc::now() - Duration::seconds(30),
                duration_ms: 12,
                error: None,
            },
            HookExecutionEntry {
                id: Uuid::new_v4(),
                hook_type: HookType::OnCostRecorded,
                handler_name: "cost_alert".to_string(),
                status: ExecutionStatus::Success,
                executed_at: Utc::now() - Duration::minutes(5),
                duration_ms: 8,
                error: None,
            },
            HookExecutionEntry {
                id: Uuid::new_v4(),
                hook_type: HookType::OnCostRecorded,
                handler_name: "cost_webhook".to_string(),
                status: ExecutionStatus::Success,
                executed_at: Utc::now() - Duration::minutes(5),
                duration_ms: 145,
                error: None,
            },
            HookExecutionEntry {
                id: Uuid::new_v4(),
                hook_type: HookType::PostSessionEnd,
                handler_name: "session_logger".to_string(),
                status: ExecutionStatus::Success,
                executed_at: Utc::now() - Duration::minutes(15),
                duration_ms: 3,
                error: None,
            },
            HookExecutionEntry {
                id: Uuid::new_v4(),
                hook_type: HookType::OnError,
                handler_name: "error_notifier".to_string(),
                status: ExecutionStatus::Failed,
                executed_at: Utc::now() - Duration::days(1),
                duration_ms: 5032,
                error: Some("Connection timeout to Slack API".to_string()),
            },
            HookExecutionEntry {
                id: Uuid::new_v4(),
                hook_type: HookType::PreSessionStart,
                handler_name: "session_start_log".to_string(),
                status: ExecutionStatus::Success,
                executed_at: Utc::now() - Duration::hours(1),
                duration_ms: 1,
                error: None,
            },
            HookExecutionEntry {
                id: Uuid::new_v4(),
                hook_type: HookType::OnAgentConnect,
                handler_name: "agent_webhook".to_string(),
                status: ExecutionStatus::Skipped,
                executed_at: Utc::now() - Duration::hours(2),
                duration_ms: 0,
                error: None,
            },
        ]
    }
}

pub struct HooksView;

impl HooksView {
    pub fn render(frame: &mut Frame, area: Rect, app: &App, hooks_state: &HooksViewState) {
        let _theme = app.current_theme();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Min(10),
            ])
            .split(area);

        Self::render_tabs(frame, chunks[0], app, hooks_state);
        Self::render_quick_stats(frame, chunks[1], app, hooks_state);

        match hooks_state.current_tab {
            HooksTab::ByType => Self::render_by_type_tab(frame, chunks[2], app, hooks_state),
            HooksTab::Handlers => Self::render_handlers_tab(frame, chunks[2], app, hooks_state),
            HooksTab::ExecutionLog => {
                Self::render_execution_log_tab(frame, chunks[2], app, hooks_state)
            }
        }
    }

    fn render_tabs(frame: &mut Frame, area: Rect, app: &App, hooks_state: &HooksViewState) {
        let theme = app.current_theme();

        let tab_titles: Vec<Line> = HooksTab::all()
            .iter()
            .enumerate()
            .map(|(i, tab)| {
                let style = if i == hooks_state.current_tab.index() {
                    Style::default()
                        .fg(theme.accent())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.foreground_dim())
                };
                Line::styled(tab.name(), style)
            })
            .collect();

        let tabs = Tabs::new(tab_titles)
            .block(
                Block::default()
                    .title(" Hooks ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border()))
                    .style(Style::default().bg(theme.surface())),
            )
            .select(hooks_state.current_tab.index())
            .highlight_style(
                Style::default()
                    .fg(theme.accent())
                    .add_modifier(Modifier::BOLD),
            )
            .divider(" | ");

        frame.render_widget(tabs, area);
    }

    fn render_quick_stats(frame: &mut Frame, area: Rect, app: &App, hooks_state: &HooksViewState) {
        let theme = app.current_theme();

        let total_hooks = hooks_state.hook_types.len();
        let active_handlers = hooks_state.handlers.iter().filter(|h| h.enabled).count();
        let total_executions = hooks_state.execution_log.len();
        let failed_executions = hooks_state
            .execution_log
            .iter()
            .filter(|e| e.status == ExecutionStatus::Failed)
            .count();

        let stats_block = Block::default()
            .title(" Quick Stats ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border()))
            .style(Style::default().bg(theme.surface()));

        let inner = stats_block.inner(area);
        frame.render_widget(stats_block, area);

        let stat_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(inner);

        let stat_items = [
            ("Hook Types", format!("{}", total_hooks), theme.accent()),
            (
                "Active Handlers",
                format!("{}", active_handlers),
                theme.success(),
            ),
            (
                "Executions",
                format!("{}", total_executions),
                theme.foreground(),
            ),
            (
                "Failed",
                format!("{}", failed_executions),
                if failed_executions > 0 {
                    theme.error()
                } else {
                    theme.foreground_dim()
                },
            ),
        ];

        for (i, (label, value, color)) in stat_items.iter().enumerate() {
            let stat_lines = vec![
                Line::from(Span::styled(
                    *label,
                    Style::default().fg(theme.foreground_dim()),
                )),
                Line::from(Span::styled(
                    value.clone(),
                    Style::default().fg(*color).add_modifier(Modifier::BOLD),
                )),
            ];
            let paragraph = Paragraph::new(stat_lines)
                .alignment(Alignment::Center)
                .style(Style::default().bg(theme.surface()));
            frame.render_widget(paragraph, stat_chunks[i]);
        }
    }

    fn render_by_type_tab(frame: &mut Frame, area: Rect, app: &App, hooks_state: &HooksViewState) {
        let theme = app.current_theme();
        let selected_index = app
            .state
            .selected_index
            .min(hooks_state.hook_types.len().saturating_sub(1));

        let header = Row::new(vec![
            Cell::from(""),
            Cell::from("Hook Type"),
            Cell::from("Data Type"),
            Cell::from("Handlers"),
            Cell::from("Enabled"),
            Cell::from("Last Triggered"),
        ])
        .style(
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        )
        .height(1);

        let rows: Vec<Row> = hooks_state
            .hook_types
            .iter()
            .enumerate()
            .map(|(i, hook_info)| {
                let is_selected = i == selected_index;

                let selection_indicator = if is_selected { "▶" } else { " " };

                let row_style = if is_selected {
                    Style::default()
                        .bg(theme.selection())
                        .fg(theme.foreground())
                } else {
                    Style::default().fg(theme.foreground())
                };

                let enabled_icon = if hook_info.enabled { "●" } else { "○" };
                let enabled_color = if hook_info.enabled {
                    theme.success()
                } else {
                    theme.foreground_dim()
                };

                let handler_color = if hook_info.handler_count > 0 {
                    theme.accent()
                } else {
                    theme.foreground_dim()
                };

                let last_triggered = hook_info
                    .last_triggered
                    .map(format_relative_time)
                    .unwrap_or_else(|| "Never".to_string());

                Row::new([
                    Cell::from(Span::styled(
                        selection_indicator,
                        Style::default().fg(theme.accent()),
                    )),
                    Cell::from(Span::styled(
                        hook_info.hook_type.name(),
                        if is_selected {
                            Style::default()
                                .fg(theme.accent())
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(theme.foreground())
                        },
                    )),
                    Cell::from(Span::styled(
                        hook_info.hook_type.data_type(),
                        Style::default().fg(theme.foreground_dim()),
                    )),
                    Cell::from(Span::styled(
                        format!("{}", hook_info.handler_count),
                        Style::default().fg(handler_color),
                    )),
                    Cell::from(Span::styled(
                        format!(
                            "{} {}",
                            enabled_icon,
                            if hook_info.enabled { "Yes" } else { "No" }
                        ),
                        Style::default().fg(enabled_color),
                    )),
                    Cell::from(Span::styled(
                        last_triggered,
                        Style::default().fg(theme.foreground_dim()),
                    )),
                ])
                .style(row_style)
                .height(1)
            })
            .collect();

        let widths = [
            Constraint::Length(2),
            Constraint::Min(20),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(16),
        ];

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .title(" Hooks by Type ")
                    .title_bottom(
                        Line::from(vec![
                            Span::styled(" j/k ", Style::default().fg(theme.accent())),
                            Span::styled("navigate  ", Style::default().fg(theme.foreground_dim())),
                            Span::styled("Enter ", Style::default().fg(theme.accent())),
                            Span::styled(
                                "view handlers  ",
                                Style::default().fg(theme.foreground_dim()),
                            ),
                            Span::styled("t ", Style::default().fg(theme.accent())),
                            Span::styled("trigger ", Style::default().fg(theme.foreground_dim())),
                        ])
                        .alignment(Alignment::Center),
                    )
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border()))
                    .style(Style::default().bg(theme.surface())),
            )
            .row_highlight_style(Style::default().bg(theme.selection()));

        frame.render_widget(table, area);
    }

    fn render_handlers_tab(frame: &mut Frame, area: Rect, app: &App, hooks_state: &HooksViewState) {
        let theme = app.current_theme();
        let selected_index = app
            .state
            .selected_index
            .min(hooks_state.handlers.len().saturating_sub(1));

        let header = Row::new(vec![
            Cell::from(""),
            Cell::from("Status"),
            Cell::from("Handler"),
            Cell::from("Hook Type"),
            Cell::from("Priority"),
            Cell::from("Plugin"),
        ])
        .style(
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        )
        .height(1);

        let rows: Vec<Row> = hooks_state
            .handlers
            .iter()
            .enumerate()
            .map(|(i, handler)| {
                let is_selected = i == selected_index;

                let selection_indicator = if is_selected { "▶" } else { " " };

                let row_style = if is_selected {
                    Style::default()
                        .bg(theme.selection())
                        .fg(theme.foreground())
                } else {
                    Style::default().fg(theme.foreground())
                };

                let status_icon = if handler.enabled { "●" } else { "○" };
                let status_color = if handler.enabled {
                    theme.success()
                } else {
                    theme.foreground_dim()
                };

                let plugin_name = handler.plugin_id.as_deref().unwrap_or("(custom)");

                Row::new([
                    Cell::from(Span::styled(
                        selection_indicator,
                        Style::default().fg(theme.accent()),
                    )),
                    Cell::from(Span::styled(
                        format!(
                            "{} {}",
                            status_icon,
                            if handler.enabled {
                                "Enabled"
                            } else {
                                "Disabled"
                            }
                        ),
                        Style::default().fg(status_color),
                    )),
                    Cell::from(Span::styled(
                        handler.name.clone(),
                        if is_selected {
                            Style::default()
                                .fg(theme.accent())
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(theme.foreground())
                        },
                    )),
                    Cell::from(Span::styled(
                        handler.hook_type.name(),
                        Style::default().fg(theme.foreground_dim()),
                    )),
                    Cell::from(Span::styled(
                        format!("{}", handler.priority),
                        Style::default().fg(theme.foreground()),
                    )),
                    Cell::from(Span::styled(
                        plugin_name,
                        Style::default().fg(theme.foreground_dim()),
                    )),
                ])
                .style(row_style)
                .height(1)
            })
            .collect();

        let widths = [
            Constraint::Length(2),
            Constraint::Length(12),
            Constraint::Min(18),
            Constraint::Length(20),
            Constraint::Length(10),
            Constraint::Length(18),
        ];

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .title(" Hook Handlers ")
                    .title_bottom(
                        Line::from(vec![
                            Span::styled(" j/k ", Style::default().fg(theme.accent())),
                            Span::styled("navigate  ", Style::default().fg(theme.foreground_dim())),
                            Span::styled("e ", Style::default().fg(theme.accent())),
                            Span::styled("toggle  ", Style::default().fg(theme.foreground_dim())),
                            Span::styled("Enter ", Style::default().fg(theme.accent())),
                            Span::styled("details ", Style::default().fg(theme.foreground_dim())),
                        ])
                        .alignment(Alignment::Center),
                    )
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border()))
                    .style(Style::default().bg(theme.surface())),
            )
            .row_highlight_style(Style::default().bg(theme.selection()));

        frame.render_widget(table, area);
    }

    fn render_execution_log_tab(
        frame: &mut Frame,
        area: Rect,
        app: &App,
        hooks_state: &HooksViewState,
    ) {
        let theme = app.current_theme();
        let selected_index = app
            .state
            .selected_index
            .min(hooks_state.execution_log.len().saturating_sub(1));

        let header = Row::new(vec![
            Cell::from(""),
            Cell::from("Status"),
            Cell::from("Handler"),
            Cell::from("Hook Type"),
            Cell::from("Duration"),
            Cell::from("Executed"),
        ])
        .style(
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        )
        .height(1);

        let rows: Vec<Row> = hooks_state
            .execution_log
            .iter()
            .enumerate()
            .map(|(i, execution)| {
                let is_selected = i == selected_index;

                let selection_indicator = if is_selected { "▶" } else { " " };

                let row_style = if is_selected {
                    Style::default()
                        .bg(theme.selection())
                        .fg(theme.foreground())
                } else {
                    Style::default().fg(theme.foreground())
                };

                let status_color = match execution.status {
                    ExecutionStatus::Success => theme.success(),
                    ExecutionStatus::Failed => theme.error(),
                    ExecutionStatus::Aborted => theme.warning(),
                    ExecutionStatus::Skipped => theme.foreground_dim(),
                };

                let duration = if execution.duration_ms > 1000 {
                    format!("{:.1}s", execution.duration_ms as f64 / 1000.0)
                } else {
                    format!("{}ms", execution.duration_ms)
                };

                Row::new([
                    Cell::from(Span::styled(
                        selection_indicator,
                        Style::default().fg(theme.accent()),
                    )),
                    Cell::from(Span::styled(
                        format!("{} {}", execution.status.icon(), execution.status.label()),
                        Style::default().fg(status_color),
                    )),
                    Cell::from(Span::styled(
                        execution.handler_name.clone(),
                        if is_selected {
                            Style::default()
                                .fg(theme.accent())
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(theme.foreground())
                        },
                    )),
                    Cell::from(Span::styled(
                        execution.hook_type.name(),
                        Style::default().fg(theme.foreground_dim()),
                    )),
                    Cell::from(Span::styled(
                        duration,
                        Style::default().fg(theme.foreground()),
                    )),
                    Cell::from(Span::styled(
                        format_relative_time(execution.executed_at),
                        Style::default().fg(theme.foreground_dim()),
                    )),
                ])
                .style(row_style)
                .height(1)
            })
            .collect();

        let widths = [
            Constraint::Length(2),
            Constraint::Length(12),
            Constraint::Min(18),
            Constraint::Length(20),
            Constraint::Length(10),
            Constraint::Length(14),
        ];

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .title(" Execution Log ")
                    .title_bottom(
                        Line::from(vec![
                            Span::styled(" j/k ", Style::default().fg(theme.accent())),
                            Span::styled("navigate  ", Style::default().fg(theme.foreground_dim())),
                            Span::styled("Enter ", Style::default().fg(theme.accent())),
                            Span::styled("details  ", Style::default().fg(theme.foreground_dim())),
                            Span::styled("r ", Style::default().fg(theme.accent())),
                            Span::styled("refresh ", Style::default().fg(theme.foreground_dim())),
                        ])
                        .alignment(Alignment::Center),
                    )
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border()))
                    .style(Style::default().bg(theme.surface())),
            )
            .row_highlight_style(Style::default().bg(theme.selection()));

        frame.render_widget(table, area);
    }

    pub fn hook_types_count(hooks_state: &HooksViewState) -> usize {
        hooks_state.hook_types.len()
    }

    pub fn handlers_count(hooks_state: &HooksViewState) -> usize {
        hooks_state.handlers.len()
    }

    pub fn execution_log_count(hooks_state: &HooksViewState) -> usize {
        hooks_state.execution_log.len()
    }

    pub fn current_list_len(hooks_state: &HooksViewState) -> usize {
        match hooks_state.current_tab {
            HooksTab::ByType => hooks_state.hook_types.len(),
            HooksTab::Handlers => hooks_state.handlers.len(),
            HooksTab::ExecutionLog => hooks_state.execution_log.len(),
        }
    }
}

fn format_relative_time(time: DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff = now.signed_duration_since(time);

    if diff.num_seconds() < 60 {
        format!("{}s ago", diff.num_seconds())
    } else if diff.num_minutes() < 60 {
        format!("{}m ago", diff.num_minutes())
    } else if diff.num_hours() < 24 {
        format!("{}h ago", diff.num_hours())
    } else {
        format!("{}d ago", diff.num_days())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hooks_tab_navigation() {
        assert_eq!(HooksTab::ByType.next(), HooksTab::Handlers);
        assert_eq!(HooksTab::Handlers.next(), HooksTab::ExecutionLog);
        assert_eq!(HooksTab::ExecutionLog.next(), HooksTab::ByType);

        assert_eq!(HooksTab::ByType.prev(), HooksTab::ExecutionLog);
        assert_eq!(HooksTab::Handlers.prev(), HooksTab::ByType);
        assert_eq!(HooksTab::ExecutionLog.prev(), HooksTab::Handlers);
    }

    #[test]
    fn test_hooks_tab_index() {
        assert_eq!(HooksTab::ByType.index(), 0);
        assert_eq!(HooksTab::Handlers.index(), 1);
        assert_eq!(HooksTab::ExecutionLog.index(), 2);
    }

    #[test]
    fn test_hook_type_names() {
        assert_eq!(HookType::PreSessionStart.name(), "pre_session_start");
        assert_eq!(HookType::PostSessionEnd.name(), "post_session_end");
        assert_eq!(HookType::OnCostRecorded.name(), "on_cost_recorded");
        assert_eq!(HookType::OnError.name(), "on_error");
    }

    #[test]
    fn test_hook_type_data_types() {
        assert_eq!(HookType::PreSessionStart.data_type(), "Session");
        assert_eq!(HookType::OnCostRecorded.data_type(), "Cost");
        assert_eq!(HookType::OnMetricsCollected.data_type(), "Metrics");
        assert_eq!(HookType::OnAgentConnect.data_type(), "Agent");
    }

    #[test]
    fn test_execution_status_icons() {
        assert_eq!(ExecutionStatus::Success.icon(), "✓");
        assert_eq!(ExecutionStatus::Failed.icon(), "✗");
        assert_eq!(ExecutionStatus::Aborted.icon(), "⊘");
        assert_eq!(ExecutionStatus::Skipped.icon(), "⊳");
    }

    #[test]
    fn test_execution_status_labels() {
        assert_eq!(ExecutionStatus::Success.label(), "Success");
        assert_eq!(ExecutionStatus::Failed.label(), "Failed");
        assert_eq!(ExecutionStatus::Aborted.label(), "Aborted");
        assert_eq!(ExecutionStatus::Skipped.label(), "Skipped");
    }

    #[test]
    fn test_hooks_view_state_new() {
        let state = HooksViewState::new();
        assert_eq!(state.current_tab, HooksTab::ByType);
        assert!(!state.hook_types.is_empty());
        assert!(!state.handlers.is_empty());
        assert!(!state.execution_log.is_empty());
    }

    #[test]
    fn test_current_list_len() {
        let state = HooksViewState::new();
        assert_eq!(HooksView::current_list_len(&state), state.hook_types.len());
    }

    #[test]
    fn test_count_functions() {
        let state = HooksViewState::new();
        assert_eq!(HooksView::hook_types_count(&state), state.hook_types.len());
        assert_eq!(HooksView::handlers_count(&state), state.handlers.len());
        assert_eq!(
            HooksView::execution_log_count(&state),
            state.execution_log.len()
        );
    }

    #[test]
    fn test_format_relative_time() {
        use chrono::Duration;

        let now = Utc::now();
        assert!(format_relative_time(now - Duration::seconds(30)).contains("s ago"));
        assert!(format_relative_time(now - Duration::minutes(5)).contains("m ago"));
        assert!(format_relative_time(now - Duration::hours(2)).contains("h ago"));
        assert!(format_relative_time(now - Duration::days(3)).contains("d ago"));
    }

    #[test]
    fn test_all_standard_hook_types() {
        let types = HookType::all_standard();
        assert_eq!(types.len(), 11);
        assert!(types.contains(&HookType::PreSessionStart));
        assert!(types.contains(&HookType::OnError));
    }
}
