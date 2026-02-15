use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Sparkline},
    Frame,
};

use super::agents::{AgentInfo, AgentStatus, AgentsView};
use crate::app::App;

pub struct AgentDetailsView;

impl AgentDetailsView {
    pub fn render(frame: &mut Frame, area: Rect, app: &App, agent_index: usize) {
        let _theme = app.current_theme();
        let agents = AgentsView::get_agents_list();

        let agent = agents.get(agent_index).cloned().unwrap_or(AgentInfo {
            name: "Unknown Agent",
            status: AgentStatus::Disconnected,
            active_sessions: 0,
            tokens_today: 0,
            cost_today: 0.0,
            last_activity: "N/A",
        });

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7),
                Constraint::Length(10),
                Constraint::Min(5),
            ])
            .split(area);

        Self::render_header_info(frame, chunks[0], app, &agent);
        Self::render_session_stats(frame, chunks[1], app, &agent);
        Self::render_activity_log(frame, chunks[2], app, &agent);
    }

    fn render_header_info(frame: &mut Frame, area: Rect, app: &App, agent: &AgentInfo) {
        let theme = app.current_theme();

        let status_color = match agent.status {
            AgentStatus::Connected => theme.success(),
            AgentStatus::Idle => theme.warning(),
            AgentStatus::Error => theme.error(),
            AgentStatus::Disconnected => theme.foreground_dim(),
        };

        let block = Block::default()
            .title(format!(" {} Details ", agent.name))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent()))
            .style(Style::default().bg(theme.surface()));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let info_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(inner);

        let left_info = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(theme.foreground_dim())),
                Span::styled(
                    agent.name,
                    Style::default()
                        .fg(theme.foreground())
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(theme.foreground_dim())),
                Span::styled(
                    format!("{} {}", agent.status.icon(), agent.status.label()),
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "Last Activity: ",
                    Style::default().fg(theme.foreground_dim()),
                ),
                Span::styled(agent.last_activity, Style::default().fg(theme.foreground())),
            ]),
        ];

        let right_info = vec![
            Line::from(vec![
                Span::styled(
                    "Active Sessions: ",
                    Style::default().fg(theme.foreground_dim()),
                ),
                Span::styled(
                    format!("{}", agent.active_sessions),
                    Style::default()
                        .fg(if agent.active_sessions > 0 {
                            theme.accent()
                        } else {
                            theme.foreground_dim()
                        })
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "Tokens Today: ",
                    Style::default().fg(theme.foreground_dim()),
                ),
                Span::styled(
                    format_number(agent.tokens_today),
                    Style::default().fg(theme.foreground()),
                ),
            ]),
            Line::from(vec![
                Span::styled("Cost Today: ", Style::default().fg(theme.foreground_dim())),
                Span::styled(
                    format!("${:.2}", agent.cost_today),
                    Style::default().fg(theme.success()),
                ),
            ]),
        ];

        let left_para = Paragraph::new(left_info).style(Style::default().bg(theme.surface()));
        let right_para = Paragraph::new(right_info).style(Style::default().bg(theme.surface()));

        frame.render_widget(left_para, info_chunks[0]);
        frame.render_widget(right_para, info_chunks[1]);
    }

    fn render_session_stats(frame: &mut Frame, area: Rect, app: &App, agent: &AgentInfo) {
        let theme = app.current_theme();

        let block = Block::default()
            .title(" Session Statistics ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border()))
            .style(Style::default().bg(theme.surface()));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(inner);

        let stats = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  Total Sessions (Today): ",
                    Style::default().fg(theme.foreground_dim()),
                ),
                Span::styled(
                    format!("{}", agent.active_sessions + 3),
                    Style::default()
                        .fg(theme.foreground())
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "  Avg Session Duration:   ",
                    Style::default().fg(theme.foreground_dim()),
                ),
                Span::styled("12m 34s", Style::default().fg(theme.foreground())),
            ]),
            Line::from(vec![
                Span::styled(
                    "  Total Tokens (Week):    ",
                    Style::default().fg(theme.foreground_dim()),
                ),
                Span::styled(
                    format_number(agent.tokens_today * 7),
                    Style::default().fg(theme.foreground()),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "  Total Cost (Week):      ",
                    Style::default().fg(theme.foreground_dim()),
                ),
                Span::styled(
                    format!("${:.2}", agent.cost_today * 7.0),
                    Style::default().fg(theme.success()),
                ),
            ]),
        ];

        let stats_para = Paragraph::new(stats).style(Style::default().bg(theme.surface()));
        frame.render_widget(stats_para, chunks[0]);

        let usage_data: Vec<u64> = vec![
            (agent.tokens_today / 8),
            (agent.tokens_today / 5),
            (agent.tokens_today / 3),
            (agent.tokens_today / 4),
            (agent.tokens_today / 2),
            (agent.tokens_today / 3),
            agent.tokens_today,
        ];

        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .title(" Usage (7d) ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border())),
            )
            .data(&usage_data)
            .style(Style::default().fg(theme.accent()));

        frame.render_widget(sparkline, chunks[1]);
    }

    fn render_activity_log(frame: &mut Frame, area: Rect, app: &App, _agent: &AgentInfo) {
        let theme = app.current_theme();

        let block = Block::default()
            .title(" Recent Activity ")
            .title_bottom(
                Line::from(vec![
                    Span::styled(" Esc ", Style::default().fg(theme.accent())),
                    Span::styled(
                        "back to agents ",
                        Style::default().fg(theme.foreground_dim()),
                    ),
                ])
                .alignment(Alignment::Center),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border()))
            .style(Style::default().bg(theme.surface()));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let activities = [
            (
                "2 min ago",
                "Session started",
                "Conversation about Rust TUI",
            ),
            (
                "15 min ago",
                "Session completed",
                "Code review task finished",
            ),
            ("1 hour ago", "Session started", "Documentation update"),
            ("2 hours ago", "Session completed", "Bug fix implementation"),
            (
                "Yesterday",
                "Session completed",
                "Feature planning discussion",
            ),
        ];

        let activity_lines: Vec<Line> = activities
            .iter()
            .map(|(time, action, desc)| {
                Line::from(vec![
                    Span::styled(
                        format!(" {:12} ", time),
                        Style::default().fg(theme.foreground_dim()),
                    ),
                    Span::styled(
                        format!("{:20} ", action),
                        Style::default().fg(theme.accent()),
                    ),
                    Span::styled(*desc, Style::default().fg(theme.foreground())),
                ])
            })
            .collect();

        let activity_para =
            Paragraph::new(activity_lines).style(Style::default().bg(theme.surface()));
        frame.render_widget(activity_para, inner);
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
