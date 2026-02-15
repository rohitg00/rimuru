use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    Connected,
    Idle,
    Error,
    Disconnected,
}

impl AgentStatus {
    pub fn icon(&self) -> &'static str {
        match self {
            AgentStatus::Connected => "●",
            AgentStatus::Idle => "◐",
            AgentStatus::Error => "✗",
            AgentStatus::Disconnected => "○",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            AgentStatus::Connected => "Connected",
            AgentStatus::Idle => "Idle",
            AgentStatus::Error => "Error",
            AgentStatus::Disconnected => "Disconnected",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub name: &'static str,
    pub status: AgentStatus,
    pub active_sessions: u32,
    pub tokens_today: u64,
    pub cost_today: f64,
    pub last_activity: &'static str,
}

pub struct AgentsView;

impl AgentsView {
    pub fn get_agents_list() -> Vec<AgentInfo> {
        Self::get_agents()
    }

    fn get_agents() -> Vec<AgentInfo> {
        vec![
            AgentInfo {
                name: "Claude Code",
                status: AgentStatus::Connected,
                active_sessions: 2,
                tokens_today: 85_432,
                cost_today: 1.82,
                last_activity: "2 min ago",
            },
            AgentInfo {
                name: "OpenCode",
                status: AgentStatus::Connected,
                active_sessions: 1,
                tokens_today: 32_100,
                cost_today: 0.43,
                last_activity: "5 min ago",
            },
            AgentInfo {
                name: "Cursor",
                status: AgentStatus::Idle,
                active_sessions: 0,
                tokens_today: 12_500,
                cost_today: 0.28,
                last_activity: "30 min ago",
            },
            AgentInfo {
                name: "Codex",
                status: AgentStatus::Disconnected,
                active_sessions: 0,
                tokens_today: 5_200,
                cost_today: 0.12,
                last_activity: "3 days ago",
            },
            AgentInfo {
                name: "Copilot",
                status: AgentStatus::Disconnected,
                active_sessions: 0,
                tokens_today: 2_700,
                cost_today: 0.08,
                last_activity: "1 week ago",
            },
            AgentInfo {
                name: "Goose",
                status: AgentStatus::Error,
                active_sessions: 0,
                tokens_today: 0,
                cost_today: 0.00,
                last_activity: "Connection failed",
            },
        ]
    }

    pub fn render(frame: &mut Frame, area: Rect, app: &App) {
        let _theme = app.current_theme();
        let agents = Self::get_agents();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(10)])
            .split(area);

        Self::render_quick_stats(frame, chunks[0], app, &agents);
        Self::render_agents_table(frame, chunks[1], app, &agents);
    }

    fn render_quick_stats(frame: &mut Frame, area: Rect, app: &App, agents: &[AgentInfo]) {
        let theme = app.current_theme();

        let total_sessions: u32 = agents.iter().map(|a| a.active_sessions).sum();
        let total_tokens: u64 = agents.iter().map(|a| a.tokens_today).sum();
        let total_cost: f64 = agents.iter().map(|a| a.cost_today).sum();
        let connected_count = agents
            .iter()
            .filter(|a| a.status == AgentStatus::Connected)
            .count();
        let idle_count = agents
            .iter()
            .filter(|a| a.status == AgentStatus::Idle)
            .count();
        let error_count = agents
            .iter()
            .filter(|a| a.status == AgentStatus::Error)
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
            (
                "Active Sessions",
                format!("{}", total_sessions),
                theme.accent(),
            ),
            (
                "Tokens Today",
                format_number(total_tokens).to_string(),
                theme.foreground(),
            ),
            ("Cost Today", format!("${:.2}", total_cost), theme.success()),
            (
                "Status",
                format!("{}● {}◐ {}✗", connected_count, idle_count, error_count),
                theme.foreground_dim(),
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

    fn render_agents_table(frame: &mut Frame, area: Rect, app: &App, agents: &[AgentInfo]) {
        let theme = app.current_theme();
        let selected_index = app.state.selected_index.min(agents.len().saturating_sub(1));

        let header = Row::new(vec![
            Cell::from(""),
            Cell::from("Status"),
            Cell::from("Agent"),
            Cell::from("Sessions"),
            Cell::from("Tokens Today"),
            Cell::from("Cost Today"),
            Cell::from("Last Activity"),
        ])
        .style(
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        )
        .height(1);

        let rows: Vec<Row> = agents
            .iter()
            .enumerate()
            .map(|(i, agent)| {
                let is_selected = i == selected_index;
                let status_color = match agent.status {
                    AgentStatus::Connected => theme.success(),
                    AgentStatus::Idle => theme.warning(),
                    AgentStatus::Error => theme.error(),
                    AgentStatus::Disconnected => theme.foreground_dim(),
                };

                let selection_indicator = if is_selected { "▶" } else { " " };

                let row_style = if is_selected {
                    Style::default()
                        .bg(theme.selection())
                        .fg(theme.foreground())
                } else {
                    Style::default().fg(theme.foreground())
                };

                Row::new([
                    Cell::from(Span::styled(
                        selection_indicator,
                        Style::default().fg(theme.accent()),
                    )),
                    Cell::from(Span::styled(
                        format!("{} {}", agent.status.icon(), agent.status.label()),
                        Style::default().fg(status_color),
                    )),
                    Cell::from(Span::styled(
                        agent.name,
                        if is_selected {
                            Style::default()
                                .fg(theme.accent())
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(theme.foreground())
                        },
                    )),
                    Cell::from(Span::styled(
                        format!("{}", agent.active_sessions),
                        if agent.active_sessions > 0 {
                            Style::default()
                                .fg(theme.accent())
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(theme.foreground_dim())
                        },
                    )),
                    Cell::from(format_number(agent.tokens_today)),
                    Cell::from(Span::styled(
                        format!("${:.2}", agent.cost_today),
                        if agent.cost_today > 0.0 {
                            Style::default().fg(theme.success())
                        } else {
                            Style::default().fg(theme.foreground_dim())
                        },
                    )),
                    Cell::from(Span::styled(
                        agent.last_activity,
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
            Constraint::Min(12),
            Constraint::Length(10),
            Constraint::Length(14),
            Constraint::Length(12),
            Constraint::Length(18),
        ];

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .title(" Agents ")
                    .title_bottom(
                        Line::from(vec![
                            Span::styled(" j/k ", Style::default().fg(theme.accent())),
                            Span::styled("navigate  ", Style::default().fg(theme.foreground_dim())),
                            Span::styled("Enter ", Style::default().fg(theme.accent())),
                            Span::styled("details  ", Style::default().fg(theme.foreground_dim())),
                            Span::styled("/ ", Style::default().fg(theme.accent())),
                            Span::styled("search ", Style::default().fg(theme.foreground_dim())),
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

    pub fn agents_count() -> usize {
        Self::get_agents().len()
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
    fn test_agent_status_icons() {
        assert_eq!(AgentStatus::Connected.icon(), "●");
        assert_eq!(AgentStatus::Idle.icon(), "◐");
        assert_eq!(AgentStatus::Error.icon(), "✗");
        assert_eq!(AgentStatus::Disconnected.icon(), "○");
    }

    #[test]
    fn test_agent_status_labels() {
        assert_eq!(AgentStatus::Connected.label(), "Connected");
        assert_eq!(AgentStatus::Idle.label(), "Idle");
        assert_eq!(AgentStatus::Error.label(), "Error");
        assert_eq!(AgentStatus::Disconnected.label(), "Disconnected");
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(500), "500");
        assert_eq!(format_number(1_500), "1.5K");
        assert_eq!(format_number(85_432), "85.4K");
        assert_eq!(format_number(1_500_000), "1.5M");
    }

    #[test]
    fn test_get_agents_returns_data() {
        let agents = AgentsView::get_agents();
        assert!(!agents.is_empty());
        assert!(agents.len() >= 4);
    }

    #[test]
    fn test_agents_count() {
        let count = AgentsView::agents_count();
        assert_eq!(count, AgentsView::get_agents().len());
    }
}
