use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Tabs},
    Frame,
};

use crate::app::App;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PluginsTab {
    #[default]
    Installed,
    Builtin,
    Available,
}

impl PluginsTab {
    pub fn all() -> &'static [PluginsTab] {
        &[
            PluginsTab::Installed,
            PluginsTab::Builtin,
            PluginsTab::Available,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            PluginsTab::Installed => "Installed",
            PluginsTab::Builtin => "Built-in",
            PluginsTab::Available => "Available",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            PluginsTab::Installed => PluginsTab::Builtin,
            PluginsTab::Builtin => PluginsTab::Available,
            PluginsTab::Available => PluginsTab::Installed,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            PluginsTab::Installed => PluginsTab::Available,
            PluginsTab::Builtin => PluginsTab::Installed,
            PluginsTab::Available => PluginsTab::Builtin,
        }
    }

    pub fn index(&self) -> usize {
        match self {
            PluginsTab::Installed => 0,
            PluginsTab::Builtin => 1,
            PluginsTab::Available => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginStatus {
    Enabled,
    Disabled,
    Error,
    NotInstalled,
}

impl PluginStatus {
    pub fn icon(&self) -> &'static str {
        match self {
            PluginStatus::Enabled => "●",
            PluginStatus::Disabled => "○",
            PluginStatus::Error => "✗",
            PluginStatus::NotInstalled => "◌",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            PluginStatus::Enabled => "Enabled",
            PluginStatus::Disabled => "Disabled",
            PluginStatus::Error => "Error",
            PluginStatus::NotInstalled => "Not Installed",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PluginDisplayInfo {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub status: PluginStatus,
    pub capabilities: Vec<String>,
    pub is_builtin: bool,
}

impl PluginDisplayInfo {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            author: String::new(),
            description: String::new(),
            status: PluginStatus::Disabled,
            capabilities: Vec::new(),
            is_builtin: false,
        }
    }

    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = author.into();
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn with_status(mut self, status: PluginStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.capabilities = capabilities;
        self
    }

    pub fn builtin(mut self) -> Self {
        self.is_builtin = true;
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct PluginsViewState {
    pub current_tab: PluginsTab,
    pub search_query: Option<String>,
    pub is_searching: bool,
    pub installed_plugins: Vec<PluginDisplayInfo>,
    pub builtin_plugins: Vec<PluginDisplayInfo>,
    pub available_plugins: Vec<PluginDisplayInfo>,
    pub show_config_panel: bool,
    pub config_plugin_index: Option<usize>,
}

impl PluginsViewState {
    pub fn new() -> Self {
        Self {
            current_tab: PluginsTab::Installed,
            search_query: None,
            is_searching: false,
            installed_plugins: Self::get_mock_installed(),
            builtin_plugins: Self::get_mock_builtin(),
            available_plugins: Self::get_mock_available(),
            show_config_panel: false,
            config_plugin_index: None,
        }
    }

    fn get_mock_installed() -> Vec<PluginDisplayInfo> {
        vec![
            PluginDisplayInfo::new("my-custom-exporter", "1.0.0")
                .with_author("user")
                .with_description("Custom data exporter for internal analytics")
                .with_status(PluginStatus::Enabled)
                .with_capabilities(vec!["exporter".to_string()]),
            PluginDisplayInfo::new("team-notifier", "0.2.1")
                .with_author("team")
                .with_description("Send notifications to team Slack channel")
                .with_status(PluginStatus::Enabled)
                .with_capabilities(vec!["notifier".to_string()]),
            PluginDisplayInfo::new("broken-plugin", "0.1.0")
                .with_author("unknown")
                .with_description("A plugin that failed to load")
                .with_status(PluginStatus::Error)
                .with_capabilities(vec!["hook".to_string()]),
        ]
    }

    fn get_mock_builtin() -> Vec<PluginDisplayInfo> {
        vec![
            PluginDisplayInfo::new("csv-exporter", "1.0.0")
                .with_author("Rimuru Team")
                .with_description("Export sessions and costs to CSV format")
                .with_status(PluginStatus::Enabled)
                .with_capabilities(vec!["exporter".to_string()])
                .builtin(),
            PluginDisplayInfo::new("json-exporter", "1.0.0")
                .with_author("Rimuru Team")
                .with_description("Export data as JSON with optional pretty printing")
                .with_status(PluginStatus::Enabled)
                .with_capabilities(vec!["exporter".to_string()])
                .builtin(),
            PluginDisplayInfo::new("webhook-notifier", "1.0.0")
                .with_author("Rimuru Team")
                .with_description("Send events to any HTTP endpoint with retry support")
                .with_status(PluginStatus::Disabled)
                .with_capabilities(vec!["notifier".to_string()])
                .builtin(),
            PluginDisplayInfo::new("slack-notifier", "1.0.0")
                .with_author("Rimuru Team")
                .with_description("Rich Slack messages with attachments and color-coded levels")
                .with_status(PluginStatus::Disabled)
                .with_capabilities(vec!["notifier".to_string()])
                .builtin(),
            PluginDisplayInfo::new("discord-notifier", "1.0.0")
                .with_author("Rimuru Team")
                .with_description("Discord embeds with fields, role mentions, and thread support")
                .with_status(PluginStatus::Disabled)
                .with_capabilities(vec!["notifier".to_string()])
                .builtin(),
        ]
    }

    fn get_mock_available() -> Vec<PluginDisplayInfo> {
        vec![
            PluginDisplayInfo::new("datadog-exporter", "1.2.0")
                .with_author("Community")
                .with_description("Export metrics to Datadog APM")
                .with_status(PluginStatus::NotInstalled)
                .with_capabilities(vec!["exporter".to_string()]),
            PluginDisplayInfo::new("email-notifier", "0.5.0")
                .with_author("Community")
                .with_description("Send email notifications via SMTP")
                .with_status(PluginStatus::NotInstalled)
                .with_capabilities(vec!["notifier".to_string()]),
            PluginDisplayInfo::new("prometheus-exporter", "1.0.0")
                .with_author("Community")
                .with_description("Export metrics in Prometheus format")
                .with_status(PluginStatus::NotInstalled)
                .with_capabilities(vec!["exporter".to_string()]),
            PluginDisplayInfo::new("grafana-view", "0.8.0")
                .with_author("Community")
                .with_description("Custom Grafana-style dashboards")
                .with_status(PluginStatus::NotInstalled)
                .with_capabilities(vec!["view".to_string()]),
        ]
    }

    pub fn toggle_config_panel(&mut self, index: usize) {
        if self.show_config_panel && self.config_plugin_index == Some(index) {
            self.show_config_panel = false;
            self.config_plugin_index = None;
        } else {
            self.show_config_panel = true;
            self.config_plugin_index = Some(index);
        }
    }

    pub fn close_config_panel(&mut self) {
        self.show_config_panel = false;
        self.config_plugin_index = None;
    }
}

pub struct PluginsView;

impl PluginsView {
    pub fn render(frame: &mut Frame, area: Rect, app: &App, plugins_state: &PluginsViewState) {
        let _theme = app.current_theme();

        let main_chunks = if plugins_state.show_config_panel {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
                .split(area)
        } else {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(100)])
                .split(area)
        };

        let list_area = main_chunks[0];

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Min(10),
            ])
            .split(list_area);

        Self::render_tabs(frame, chunks[0], app, plugins_state);
        Self::render_quick_stats(frame, chunks[1], app, plugins_state);

        match plugins_state.current_tab {
            PluginsTab::Installed => Self::render_plugins_table(
                frame,
                chunks[2],
                app,
                &plugins_state.installed_plugins,
                "Installed Plugins",
                true,
            ),
            PluginsTab::Builtin => Self::render_plugins_table(
                frame,
                chunks[2],
                app,
                &plugins_state.builtin_plugins,
                "Built-in Plugins",
                false,
            ),
            PluginsTab::Available => Self::render_plugins_table(
                frame,
                chunks[2],
                app,
                &plugins_state.available_plugins,
                "Available Plugins",
                false,
            ),
        }

        if plugins_state.show_config_panel && main_chunks.len() > 1 {
            Self::render_config_panel(frame, main_chunks[1], app, plugins_state);
        }
    }

    fn render_tabs(frame: &mut Frame, area: Rect, app: &App, plugins_state: &PluginsViewState) {
        let theme = app.current_theme();

        let tab_titles: Vec<Line> = PluginsTab::all()
            .iter()
            .enumerate()
            .map(|(i, tab)| {
                let style = if i == plugins_state.current_tab.index() {
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
                    .title(" Plugins ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border()))
                    .style(Style::default().bg(theme.surface())),
            )
            .select(plugins_state.current_tab.index())
            .highlight_style(
                Style::default()
                    .fg(theme.accent())
                    .add_modifier(Modifier::BOLD),
            )
            .divider(" | ");

        frame.render_widget(tabs, area);
    }

    fn render_quick_stats(
        frame: &mut Frame,
        area: Rect,
        app: &App,
        plugins_state: &PluginsViewState,
    ) {
        let theme = app.current_theme();

        let installed_count = plugins_state.installed_plugins.len();
        let enabled_count = plugins_state
            .installed_plugins
            .iter()
            .chain(plugins_state.builtin_plugins.iter())
            .filter(|p| p.status == PluginStatus::Enabled)
            .count();
        let builtin_count = plugins_state.builtin_plugins.len();
        let error_count = plugins_state
            .installed_plugins
            .iter()
            .filter(|p| p.status == PluginStatus::Error)
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
            ("Installed", format!("{}", installed_count), theme.accent()),
            ("Enabled", format!("{}", enabled_count), theme.success()),
            ("Built-in", format!("{}", builtin_count), theme.foreground()),
            (
                "Errors",
                format!("{}", error_count),
                if error_count > 0 {
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

    fn render_plugins_table(
        frame: &mut Frame,
        area: Rect,
        app: &App,
        plugins: &[PluginDisplayInfo],
        title: &str,
        show_install_hint: bool,
    ) {
        let theme = app.current_theme();
        let selected_index = app
            .state
            .selected_index
            .min(plugins.len().saturating_sub(1));

        let header = Row::new(vec![
            Cell::from(""),
            Cell::from("Status"),
            Cell::from("Plugin"),
            Cell::from("Version"),
            Cell::from("Author"),
            Cell::from("Capabilities"),
        ])
        .style(
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        )
        .height(1);

        let rows: Vec<Row> = plugins
            .iter()
            .enumerate()
            .map(|(i, plugin)| {
                let is_selected = i == selected_index;
                let status_color = match plugin.status {
                    PluginStatus::Enabled => theme.success(),
                    PluginStatus::Disabled => theme.foreground_dim(),
                    PluginStatus::Error => theme.error(),
                    PluginStatus::NotInstalled => theme.warning(),
                };

                let selection_indicator = if is_selected { "▶" } else { " " };

                let row_style = if is_selected {
                    Style::default()
                        .bg(theme.selection())
                        .fg(theme.foreground())
                } else {
                    Style::default().fg(theme.foreground())
                };

                let capabilities = plugin.capabilities.join(", ");

                Row::new([
                    Cell::from(Span::styled(
                        selection_indicator,
                        Style::default().fg(theme.accent()),
                    )),
                    Cell::from(Span::styled(
                        format!("{} {}", plugin.status.icon(), plugin.status.label()),
                        Style::default().fg(status_color),
                    )),
                    Cell::from(Span::styled(
                        plugin.name.clone(),
                        if is_selected {
                            Style::default()
                                .fg(theme.accent())
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(theme.foreground())
                        },
                    )),
                    Cell::from(Span::styled(
                        plugin.version.clone(),
                        Style::default().fg(theme.foreground_dim()),
                    )),
                    Cell::from(Span::styled(
                        truncate_str(&plugin.author, 15),
                        Style::default().fg(theme.foreground_dim()),
                    )),
                    Cell::from(Span::styled(
                        capabilities,
                        Style::default().fg(theme.foreground_dim()),
                    )),
                ])
                .style(row_style)
                .height(1)
            })
            .collect();

        let widths = [
            Constraint::Length(2),
            Constraint::Length(14),
            Constraint::Min(20),
            Constraint::Length(10),
            Constraint::Length(16),
            Constraint::Length(16),
        ];

        let footer_text = if show_install_hint {
            vec![
                Span::styled(" j/k ", Style::default().fg(theme.accent())),
                Span::styled("navigate  ", Style::default().fg(theme.foreground_dim())),
                Span::styled("e ", Style::default().fg(theme.accent())),
                Span::styled("toggle  ", Style::default().fg(theme.foreground_dim())),
                Span::styled("c ", Style::default().fg(theme.accent())),
                Span::styled("configure  ", Style::default().fg(theme.foreground_dim())),
                Span::styled("i ", Style::default().fg(theme.accent())),
                Span::styled("install  ", Style::default().fg(theme.foreground_dim())),
                Span::styled("u ", Style::default().fg(theme.accent())),
                Span::styled("uninstall ", Style::default().fg(theme.foreground_dim())),
            ]
        } else {
            vec![
                Span::styled(" j/k ", Style::default().fg(theme.accent())),
                Span::styled("navigate  ", Style::default().fg(theme.foreground_dim())),
                Span::styled("e ", Style::default().fg(theme.accent())),
                Span::styled("toggle  ", Style::default().fg(theme.foreground_dim())),
                Span::styled("c ", Style::default().fg(theme.accent())),
                Span::styled("configure  ", Style::default().fg(theme.foreground_dim())),
            ]
        };

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .title(format!(" {} ", title))
                    .title_bottom(Line::from(footer_text).alignment(Alignment::Center))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border()))
                    .style(Style::default().bg(theme.surface())),
            )
            .row_highlight_style(Style::default().bg(theme.selection()));

        frame.render_widget(table, area);
    }

    fn render_config_panel(
        frame: &mut Frame,
        area: Rect,
        app: &App,
        plugins_state: &PluginsViewState,
    ) {
        let theme = app.current_theme();

        let plugin =
            plugins_state
                .config_plugin_index
                .and_then(|idx| match plugins_state.current_tab {
                    PluginsTab::Installed => plugins_state.installed_plugins.get(idx),
                    PluginsTab::Builtin => plugins_state.builtin_plugins.get(idx),
                    PluginsTab::Available => plugins_state.available_plugins.get(idx),
                });

        let content = if let Some(plugin) = plugin {
            vec![
                Line::from(vec![
                    Span::styled("Name: ", Style::default().fg(theme.foreground_dim())),
                    Span::styled(
                        plugin.name.clone(),
                        Style::default()
                            .fg(theme.accent())
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Version: ", Style::default().fg(theme.foreground_dim())),
                    Span::styled(
                        plugin.version.clone(),
                        Style::default().fg(theme.foreground()),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Author: ", Style::default().fg(theme.foreground_dim())),
                    Span::styled(
                        plugin.author.clone(),
                        Style::default().fg(theme.foreground()),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Status: ", Style::default().fg(theme.foreground_dim())),
                    Span::styled(
                        format!("{} {}", plugin.status.icon(), plugin.status.label()),
                        Style::default().fg(match plugin.status {
                            PluginStatus::Enabled => theme.success(),
                            PluginStatus::Disabled => theme.foreground_dim(),
                            PluginStatus::Error => theme.error(),
                            PluginStatus::NotInstalled => theme.warning(),
                        }),
                    ),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "Description:",
                    Style::default().fg(theme.foreground_dim()),
                )),
                Line::from(Span::styled(
                    plugin.description.clone(),
                    Style::default().fg(theme.foreground()),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Capabilities:",
                    Style::default().fg(theme.foreground_dim()),
                )),
                Line::from(Span::styled(
                    plugin.capabilities.join(", "),
                    Style::default().fg(theme.foreground()),
                )),
                Line::from(""),
                Line::from(""),
                Line::from(Span::styled(
                    "Configuration:",
                    Style::default()
                        .fg(theme.accent())
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled("  enabled: ", Style::default().fg(theme.foreground_dim())),
                    Span::styled(
                        if plugin.status == PluginStatus::Enabled {
                            "true"
                        } else {
                            "false"
                        },
                        Style::default().fg(theme.foreground()),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  priority: ", Style::default().fg(theme.foreground_dim())),
                    Span::styled("0", Style::default().fg(theme.foreground())),
                ]),
            ]
        } else {
            vec![Line::from(Span::styled(
                "No plugin selected",
                Style::default().fg(theme.foreground_dim()),
            ))]
        };

        let config_block = Block::default()
            .title(" Configuration ")
            .title_bottom(
                Line::from(vec![
                    Span::styled(" Esc ", Style::default().fg(theme.accent())),
                    Span::styled("close ", Style::default().fg(theme.foreground_dim())),
                ])
                .alignment(Alignment::Center),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent()))
            .style(Style::default().bg(theme.surface()));

        let paragraph = Paragraph::new(content).block(config_block);

        frame.render_widget(paragraph, area);
    }

    pub fn installed_count(plugins_state: &PluginsViewState) -> usize {
        plugins_state.installed_plugins.len()
    }

    pub fn builtin_count(plugins_state: &PluginsViewState) -> usize {
        plugins_state.builtin_plugins.len()
    }

    pub fn available_count(plugins_state: &PluginsViewState) -> usize {
        plugins_state.available_plugins.len()
    }

    pub fn current_list_len(plugins_state: &PluginsViewState) -> usize {
        match plugins_state.current_tab {
            PluginsTab::Installed => plugins_state.installed_plugins.len(),
            PluginsTab::Builtin => plugins_state.builtin_plugins.len(),
            PluginsTab::Available => plugins_state.available_plugins.len(),
        }
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugins_tab_navigation() {
        assert_eq!(PluginsTab::Installed.next(), PluginsTab::Builtin);
        assert_eq!(PluginsTab::Builtin.next(), PluginsTab::Available);
        assert_eq!(PluginsTab::Available.next(), PluginsTab::Installed);

        assert_eq!(PluginsTab::Installed.prev(), PluginsTab::Available);
        assert_eq!(PluginsTab::Builtin.prev(), PluginsTab::Installed);
        assert_eq!(PluginsTab::Available.prev(), PluginsTab::Builtin);
    }

    #[test]
    fn test_plugins_tab_index() {
        assert_eq!(PluginsTab::Installed.index(), 0);
        assert_eq!(PluginsTab::Builtin.index(), 1);
        assert_eq!(PluginsTab::Available.index(), 2);
    }

    #[test]
    fn test_plugin_status_icons() {
        assert_eq!(PluginStatus::Enabled.icon(), "●");
        assert_eq!(PluginStatus::Disabled.icon(), "○");
        assert_eq!(PluginStatus::Error.icon(), "✗");
        assert_eq!(PluginStatus::NotInstalled.icon(), "◌");
    }

    #[test]
    fn test_plugin_status_labels() {
        assert_eq!(PluginStatus::Enabled.label(), "Enabled");
        assert_eq!(PluginStatus::Disabled.label(), "Disabled");
        assert_eq!(PluginStatus::Error.label(), "Error");
        assert_eq!(PluginStatus::NotInstalled.label(), "Not Installed");
    }

    #[test]
    fn test_plugin_display_info_builder() {
        let plugin = PluginDisplayInfo::new("test-plugin", "1.0.0")
            .with_author("Test Author")
            .with_description("A test plugin")
            .with_status(PluginStatus::Enabled)
            .with_capabilities(vec!["exporter".to_string()])
            .builtin();

        assert_eq!(plugin.name, "test-plugin");
        assert_eq!(plugin.version, "1.0.0");
        assert_eq!(plugin.author, "Test Author");
        assert_eq!(plugin.description, "A test plugin");
        assert_eq!(plugin.status, PluginStatus::Enabled);
        assert_eq!(plugin.capabilities, vec!["exporter".to_string()]);
        assert!(plugin.is_builtin);
    }

    #[test]
    fn test_plugins_view_state_new() {
        let state = PluginsViewState::new();
        assert_eq!(state.current_tab, PluginsTab::Installed);
        assert!(!state.installed_plugins.is_empty());
        assert!(!state.builtin_plugins.is_empty());
        assert!(!state.available_plugins.is_empty());
        assert!(!state.show_config_panel);
    }

    #[test]
    fn test_toggle_config_panel() {
        let mut state = PluginsViewState::new();

        state.toggle_config_panel(0);
        assert!(state.show_config_panel);
        assert_eq!(state.config_plugin_index, Some(0));

        state.toggle_config_panel(0);
        assert!(!state.show_config_panel);
        assert_eq!(state.config_plugin_index, None);

        state.toggle_config_panel(1);
        assert!(state.show_config_panel);
        assert_eq!(state.config_plugin_index, Some(1));

        state.close_config_panel();
        assert!(!state.show_config_panel);
        assert_eq!(state.config_plugin_index, None);
    }

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("hello", 10), "hello");
        assert_eq!(truncate_str("hello world test", 10), "hello w...");
        assert_eq!(truncate_str("abc", 3), "abc");
    }

    #[test]
    fn test_current_list_len() {
        let state = PluginsViewState::new();
        assert_eq!(
            PluginsView::current_list_len(&state),
            state.installed_plugins.len()
        );
    }

    #[test]
    fn test_count_functions() {
        let state = PluginsViewState::new();
        assert_eq!(
            PluginsView::installed_count(&state),
            state.installed_plugins.len()
        );
        assert_eq!(
            PluginsView::builtin_count(&state),
            state.builtin_plugins.len()
        );
        assert_eq!(
            PluginsView::available_count(&state),
            state.available_plugins.len()
        );
    }
}
