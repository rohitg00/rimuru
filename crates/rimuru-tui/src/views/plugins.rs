use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();

    let header = Row::new(vec!["Name", "Version", "Language", "Enabled", "Functions", "Installed"])
        .style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .plugins
        .iter()
        .enumerate()
        .map(|(i, plugin)| {
            let style = if i == app.selected_index {
                Style::default().bg(theme.highlight).fg(theme.fg)
            } else {
                Style::default().fg(theme.fg)
            };

            let enabled_text = if plugin.enabled { "Yes" } else { "No" };
            let enabled_color = if plugin.enabled {
                theme.success
            } else {
                theme.muted
            };

            Row::new(vec![
                Cell::from(plugin.name.clone()),
                Cell::from(plugin.version.clone()),
                Cell::from(plugin.language.clone()),
                Cell::from(Span::styled(enabled_text, Style::default().fg(enabled_color))),
                Cell::from(plugin.functions.len().to_string()),
                Cell::from(plugin.installed_at.format("%Y-%m-%d").to_string()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(12),
            Constraint::Percentage(13),
            Constraint::Percentage(12),
            Constraint::Percentage(13),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(Span::styled(
                format!(" Plugins ({}) ", app.plugins.len()),
                Style::default().fg(theme.accent),
            ))
            .title_bottom(Line::from(Span::styled(
                " Enter: Toggle Enable/Disable ",
                Style::default().fg(theme.muted),
            ))),
    );

    f.render_widget(table, area);
}
