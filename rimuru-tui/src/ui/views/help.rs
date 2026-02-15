use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

pub struct HelpView;

impl HelpView {
    pub fn render(frame: &mut Frame, area: Rect, app: &App) {
        let _theme = app.current_theme();

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        Self::render_navigation_help(frame, chunks[0], app);
        Self::render_views_help(frame, chunks[1], app);
    }

    fn render_navigation_help(frame: &mut Frame, area: Rect, app: &App) {
        let theme = app.current_theme();

        let block = Block::default()
            .title(" Navigation ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border()))
            .style(Style::default().bg(theme.surface()));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let keybinds = vec![
            ("q / Esc", "Quit application"),
            ("Tab", "Next view"),
            ("Shift+Tab", "Previous view"),
            ("j / ↓", "Move down in list"),
            ("k / ↑", "Move up in list"),
            ("g", "Go to top of list"),
            ("G", "Go to bottom of list"),
            ("Enter", "Select / drill-down"),
            ("/", "Start search"),
            ("Esc", "Cancel search"),
            ("r", "Refresh data"),
            ("t", "Toggle theme"),
            ("?", "Show this help"),
            ("1-5", "Jump to view by number"),
        ];

        let lines: Vec<Line> = keybinds
            .iter()
            .map(|(key, desc)| {
                Line::from(vec![
                    Span::styled(
                        format!("  {:<15}", key),
                        Style::default()
                            .fg(theme.accent())
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(*desc, Style::default().fg(theme.foreground())),
                ])
            })
            .collect();

        let paragraph = Paragraph::new(lines).style(Style::default().bg(theme.surface()));
        frame.render_widget(paragraph, inner);
    }

    fn render_views_help(frame: &mut Frame, area: Rect, app: &App) {
        let theme = app.current_theme();

        let block = Block::default()
            .title(" Views ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border()))
            .style(Style::default().bg(theme.surface()));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let views = [
            (
                "1. Dashboard",
                "Overview with agents, metrics, and costs at a glance",
            ),
            (
                "2. Agents",
                "List all agents with status, sessions, and costs",
            ),
            (
                "3. Sessions",
                "Active and recent sessions with detailed stats",
            ),
            (
                "4. Costs",
                "Cost breakdown by agent, model, and time period",
            ),
            (
                "5. Metrics",
                "System metrics: CPU, RAM, disk, and session counts",
            ),
        ];

        let lines: Vec<Line> = views
            .iter()
            .flat_map(|(title, desc)| {
                vec![
                    Line::from(Span::styled(
                        format!("  {}", title),
                        Style::default()
                            .fg(theme.accent())
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(Span::styled(
                        format!("    {}", desc),
                        Style::default().fg(theme.foreground_dim()),
                    )),
                    Line::from(""),
                ]
            })
            .collect();

        let paragraph = Paragraph::new(lines).style(Style::default().bg(theme.surface()));
        frame.render_widget(paragraph, inner);
    }
}
