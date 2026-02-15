use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::app::{App, View};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Header;

impl Header {
    pub fn render(frame: &mut Frame, area: Rect, app: &App) {
        let theme = app.current_theme();

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(25),
                Constraint::Min(20),
                Constraint::Length(20),
            ])
            .split(area);

        let logo = Paragraph::new(Line::from(vec![
            Span::styled("りむる ", Style::default().fg(theme.accent())),
            Span::styled(
                "Rimuru ",
                Style::default()
                    .fg(theme.foreground())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("v{}", VERSION),
                Style::default().fg(theme.foreground_dim()),
            ),
        ]))
        .block(Block::default().borders(Borders::NONE))
        .style(Style::default().bg(theme.background()));
        frame.render_widget(logo, chunks[0]);

        let tab_titles: Vec<Line> = View::all()
            .iter()
            .map(|v| {
                let style = if *v == app.current_view {
                    Style::default()
                        .fg(theme.accent())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.foreground_dim())
                };
                Line::from(Span::styled(v.name(), style))
            })
            .collect();

        let tabs = Tabs::new(tab_titles)
            .block(Block::default().borders(Borders::NONE))
            .style(Style::default().bg(theme.background()))
            .highlight_style(Style::default().fg(theme.accent()))
            .select(
                View::all()
                    .iter()
                    .position(|v| *v == app.current_view)
                    .unwrap_or(0),
            )
            .divider(Span::raw(" │ "));
        frame.render_widget(tabs, chunks[1]);

        let time = chrono::Local::now().format("%H:%M:%S").to_string();
        let time_widget = Paragraph::new(Line::from(Span::styled(
            time,
            Style::default().fg(theme.foreground_dim()),
        )))
        .alignment(ratatui::layout::Alignment::Right)
        .block(Block::default().borders(Borders::NONE))
        .style(Style::default().bg(theme.background()));
        frame.render_widget(time_widget, chunks[2]);
    }
}
