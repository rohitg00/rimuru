use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::ui::widgets::{CostSparkline, Logo, MetricsGauge};

pub struct DashboardView;

impl DashboardView {
    pub fn render(frame: &mut Frame, area: Rect, app: &App) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(50),
                Constraint::Percentage(25),
            ])
            .split(area);

        Self::render_left_panel(frame, chunks[0], app);
        Self::render_center_panel(frame, chunks[1], app);
        Self::render_right_panel(frame, chunks[2], app);
    }

    fn render_left_panel(frame: &mut Frame, area: Rect, app: &App) {
        let theme = app.current_theme();

        let block = Block::default()
            .title(" Agents ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border()))
            .style(Style::default().bg(theme.surface()));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let agents = [
            ("Claude Code", "●", theme.success(), "Active"),
            ("OpenCode", "●", theme.success(), "Active"),
            ("Cursor", "●", theme.warning(), "Idle"),
            ("Codex", "○", theme.foreground_dim(), "Offline"),
            ("Copilot", "○", theme.foreground_dim(), "Offline"),
            ("Goose", "○", theme.foreground_dim(), "Offline"),
        ];

        let lines: Vec<Line> = agents
            .iter()
            .enumerate()
            .map(|(idx, (name, status, color, state))| {
                let selected = idx == app.state.selected_index;
                let prefix = if selected { ">" } else { " " };
                let name_style = if selected {
                    Style::default()
                        .fg(theme.accent())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.foreground())
                };
                Line::from(vec![
                    Span::styled(prefix, Style::default().fg(theme.accent())),
                    Span::styled(format!("{status} "), Style::default().fg(*color)),
                    Span::styled(format!("{name:<12}"), name_style),
                    Span::styled(
                        format!(" ({state})"),
                        Style::default().fg(theme.foreground_dim()),
                    ),
                ])
            })
            .collect();

        let paragraph = Paragraph::new(lines).style(Style::default().bg(theme.surface()));
        frame.render_widget(paragraph, inner);
    }

    fn render_center_panel(frame: &mut Frame, area: Rect, app: &App) {
        let theme = app.current_theme();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(8), Constraint::Min(5)])
            .split(area);

        let logo = Logo::render(theme, false);
        let logo_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent()))
            .style(Style::default().bg(theme.surface()));
        let logo_inner = logo_block.inner(chunks[0]);
        frame.render_widget(logo_block, chunks[0]);
        frame.render_widget(logo.alignment(Alignment::Center), logo_inner);

        let summary_block = Block::default()
            .title(" Quick Summary ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border()))
            .style(Style::default().bg(theme.surface()));
        let summary_inner = summary_block.inner(chunks[1]);
        frame.render_widget(summary_block, chunks[1]);

        let summary_lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  Active Sessions: ",
                    Style::default().fg(theme.foreground_dim()),
                ),
                Span::styled(
                    "3",
                    Style::default()
                        .fg(theme.accent())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "  (Claude: 2, OpenCode: 1)",
                    Style::default().fg(theme.foreground_dim()),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  Today's Tokens:  ",
                    Style::default().fg(theme.foreground_dim()),
                ),
                Span::styled(
                    "125,432",
                    Style::default()
                        .fg(theme.foreground())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "  input: 98,234 / output: 27,198",
                    Style::default().fg(theme.foreground_dim()),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  Today's Cost:    ",
                    Style::default().fg(theme.foreground_dim()),
                ),
                Span::styled(
                    "$2.45",
                    Style::default()
                        .fg(theme.success())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "  (12% below yesterday)",
                    Style::default().fg(theme.success()),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  This Week:       ",
                    Style::default().fg(theme.foreground_dim()),
                ),
                Span::styled("$15.30", Style::default().fg(theme.foreground())),
                Span::styled(
                    "  | This Month: ",
                    Style::default().fg(theme.foreground_dim()),
                ),
                Span::styled("$45.80", Style::default().fg(theme.foreground())),
            ]),
        ];

        let summary = Paragraph::new(summary_lines).style(Style::default().bg(theme.surface()));
        frame.render_widget(summary, summary_inner);
    }

    fn render_right_panel(frame: &mut Frame, area: Rect, app: &App) {
        let theme = app.current_theme();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),
                Constraint::Length(4),
                Constraint::Length(4),
                Constraint::Min(5),
            ])
            .split(area);

        MetricsGauge::render(frame, chunks[0], theme, " CPU ", 45.2, "%");
        MetricsGauge::render(frame, chunks[1], theme, " RAM ", 62.8, "%");
        MetricsGauge::render(frame, chunks[2], theme, " Disk ", 34.5, "%");

        let cost_data: Vec<u64> = vec![10, 15, 8, 22, 30, 25, 28];
        CostSparkline::render(frame, chunks[3], theme, " Cost Trend (7d) ", &cost_data);
    }
}
