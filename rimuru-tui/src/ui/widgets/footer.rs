use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

pub struct Footer;

impl Footer {
    pub fn render(frame: &mut Frame, area: Rect, app: &App) {
        let theme = app.current_theme();

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(area);

        let keybinds = [
            ("q", "Quit"),
            ("Tab", "Next View"),
            ("j/k", "Navigate"),
            ("t", "Theme"),
            ("?", "Help"),
            ("/", "Search"),
            ("r", "Refresh"),
        ];

        let keybind_spans: Vec<Span> = keybinds
            .iter()
            .flat_map(|(key, desc)| {
                vec![
                    Span::styled(
                        format!(" {key}"),
                        Style::default()
                            .fg(theme.accent())
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(":{desc} "),
                        Style::default().fg(theme.foreground_dim()),
                    ),
                ]
            })
            .collect();

        let keybinds_widget = Paragraph::new(Line::from(keybind_spans))
            .block(Block::default().borders(Borders::NONE))
            .style(Style::default().bg(theme.surface()));
        frame.render_widget(keybinds_widget, chunks[0]);

        let status = app.status_message.as_deref().unwrap_or("Ready");
        let status_widget = Paragraph::new(Line::from(Span::styled(
            status,
            Style::default().fg(theme.foreground_dim()),
        )))
        .alignment(ratatui::layout::Alignment::Right)
        .block(Block::default().borders(Borders::NONE))
        .style(Style::default().bg(theme.surface()));
        frame.render_widget(status_widget, chunks[1]);
    }
}
