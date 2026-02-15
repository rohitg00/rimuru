use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Sparkline},
    Frame,
};

use super::sessions::{SessionInfo, SessionStatus, SessionsView};
use crate::app::App;

pub struct SessionDetailsView;

impl SessionDetailsView {
    pub fn render(frame: &mut Frame, area: Rect, app: &App, session_index: usize) {
        let _theme = app.current_theme();
        let sessions = SessionsView::get_sessions();

        let session = sessions.get(session_index).cloned().unwrap_or(SessionInfo {
            id: "unknown",
            agent: "Unknown Agent",
            status: SessionStatus::Completed,
            duration_secs: 0,
            input_tokens: 0,
            output_tokens: 0,
            cost: 0.0,
            started: "N/A",
            model: "unknown",
            task_summary: "Session not found",
        });

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),
                Constraint::Length(12),
                Constraint::Min(8),
            ])
            .split(area);

        Self::render_header_info(frame, chunks[0], app, &session);
        Self::render_cost_breakdown(frame, chunks[1], app, &session);
        Self::render_messages_preview(frame, chunks[2], app, &session);
    }

    fn render_header_info(frame: &mut Frame, area: Rect, app: &App, session: &SessionInfo) {
        let theme = app.current_theme();

        let status_color = match session.status {
            SessionStatus::Active => theme.success(),
            SessionStatus::Completed => theme.foreground_dim(),
            SessionStatus::Failed => theme.error(),
        };

        let block = Block::default()
            .title(format!(" Session: {} ", session.id))
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
                Span::styled("Agent: ", Style::default().fg(theme.foreground_dim())),
                Span::styled(
                    session.agent,
                    Style::default()
                        .fg(theme.foreground())
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(theme.foreground_dim())),
                Span::styled(
                    format!("{} {}", session.status.icon(0), session.status.label()),
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Model: ", Style::default().fg(theme.foreground_dim())),
                Span::styled(session.model, Style::default().fg(theme.accent())),
            ]),
            Line::from(vec![
                Span::styled("Started: ", Style::default().fg(theme.foreground_dim())),
                Span::styled(session.started, Style::default().fg(theme.foreground())),
            ]),
        ];

        let right_info = vec![
            Line::from(vec![
                Span::styled("Duration: ", Style::default().fg(theme.foreground_dim())),
                Span::styled(
                    session.format_duration(),
                    Style::default()
                        .fg(if session.status == SessionStatus::Active {
                            theme.accent()
                        } else {
                            theme.foreground()
                        })
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "Input Tokens: ",
                    Style::default().fg(theme.foreground_dim()),
                ),
                Span::styled(
                    format_number(session.input_tokens),
                    Style::default().fg(theme.foreground()),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "Output Tokens: ",
                    Style::default().fg(theme.foreground_dim()),
                ),
                Span::styled(
                    format_number(session.output_tokens),
                    Style::default().fg(theme.foreground()),
                ),
            ]),
            Line::from(vec![
                Span::styled("Total Cost: ", Style::default().fg(theme.foreground_dim())),
                Span::styled(
                    format!("${:.4}", session.cost),
                    Style::default()
                        .fg(theme.success())
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ];

        let left_para = Paragraph::new(left_info).style(Style::default().bg(theme.surface()));
        let right_para = Paragraph::new(right_info).style(Style::default().bg(theme.surface()));

        frame.render_widget(left_para, info_chunks[0]);
        frame.render_widget(right_para, info_chunks[1]);
    }

    fn render_cost_breakdown(frame: &mut Frame, area: Rect, app: &App, session: &SessionInfo) {
        let theme = app.current_theme();

        let block = Block::default()
            .title(" Cost Breakdown ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border()))
            .style(Style::default().bg(theme.surface()));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(inner);

        let input_cost = session.cost * 0.4;
        let output_cost = session.cost * 0.6;

        let cost_info = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  Input Cost:   ",
                    Style::default().fg(theme.foreground_dim()),
                ),
                Span::styled(
                    format!("${:.4}", input_cost),
                    Style::default().fg(theme.foreground()),
                ),
                Span::styled(
                    format!(" ({} tokens)", format_number(session.input_tokens)),
                    Style::default().fg(theme.foreground_dim()),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "  Output Cost:  ",
                    Style::default().fg(theme.foreground_dim()),
                ),
                Span::styled(
                    format!("${:.4}", output_cost),
                    Style::default().fg(theme.foreground()),
                ),
                Span::styled(
                    format!(" ({} tokens)", format_number(session.output_tokens)),
                    Style::default().fg(theme.foreground_dim()),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  Total:        ",
                    Style::default().fg(theme.foreground_dim()),
                ),
                Span::styled(
                    format!("${:.4}", session.cost),
                    Style::default()
                        .fg(theme.success())
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  Rate:         ",
                    Style::default().fg(theme.foreground_dim()),
                ),
                Span::styled(
                    format!(
                        "${:.4}/min",
                        session.cost / (session.duration_secs as f64 / 60.0).max(0.1)
                    ),
                    Style::default().fg(theme.accent()),
                ),
            ]),
        ];

        let cost_para = Paragraph::new(cost_info).style(Style::default().bg(theme.surface()));
        frame.render_widget(cost_para, chunks[0]);

        let usage_data: Vec<u64> = vec![
            session.input_tokens / 4,
            session.input_tokens / 3,
            session.input_tokens / 2,
            session.input_tokens,
            session.output_tokens / 3,
            session.output_tokens / 2,
            session.output_tokens,
        ];

        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .title(" Token Flow ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border())),
            )
            .data(&usage_data)
            .style(Style::default().fg(theme.accent()));

        frame.render_widget(sparkline, chunks[1]);
    }

    fn render_messages_preview(frame: &mut Frame, area: Rect, app: &App, session: &SessionInfo) {
        let theme = app.current_theme();

        let block = Block::default()
            .title(format!(" Task: {} ", session.task_summary))
            .title_bottom(
                Line::from(vec![
                    Span::styled(" Esc ", Style::default().fg(theme.accent())),
                    Span::styled(
                        "back to sessions ",
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

        let messages = get_mock_messages(session);

        let message_lines: Vec<Line> = messages
            .iter()
            .map(|(role, content, time)| {
                let role_color = match *role {
                    "user" => theme.accent(),
                    "assistant" => theme.success(),
                    "system" => theme.warning(),
                    _ => theme.foreground_dim(),
                };
                Line::from(vec![
                    Span::styled(
                        format!(" {:8} ", time),
                        Style::default().fg(theme.foreground_dim()),
                    ),
                    Span::styled(
                        format!("[{:9}] ", role),
                        Style::default().fg(role_color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        truncate_str(content, 60),
                        Style::default().fg(theme.foreground()),
                    ),
                ])
            })
            .collect();

        let messages_para =
            Paragraph::new(message_lines).style(Style::default().bg(theme.surface()));
        frame.render_widget(messages_para, inner);
    }
}

fn get_mock_messages(session: &SessionInfo) -> Vec<(&'static str, &'static str, &'static str)> {
    match session.agent {
        "Claude Code" => vec![
            (
                "user",
                "Can you implement the TUI sessions view?",
                "14:30:21",
            ),
            (
                "assistant",
                "I'll create a comprehensive sessions view with...",
                "14:30:25",
            ),
            ("user", "Make sure to add sorting and filtering", "14:35:42"),
            (
                "assistant",
                "I've added SortColumn enum and FilterType...",
                "14:35:48",
            ),
            ("user", "Add the drill-down details view too", "14:42:15"),
            (
                "assistant",
                "Creating SessionDetailsView with cost breakdown...",
                "14:42:20",
            ),
        ],
        "OpenCode" => vec![
            (
                "user",
                "Debug the async runtime issue in the event loop",
                "16:12:34",
            ),
            (
                "assistant",
                "I see the issue - the tokio runtime is...",
                "16:12:40",
            ),
            ("user", "How do we fix the blocking call?", "16:14:22"),
            (
                "assistant",
                "We need to spawn_blocking for the IO operation...",
                "16:14:28",
            ),
        ],
        _ => vec![
            ("system", "Session started", "00:00:00"),
            ("user", "Starting task...", "00:00:05"),
            ("assistant", "Processing request...", "00:00:10"),
        ],
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
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
