use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();

    let header = Row::new(vec!["Name", "Version", "Author", "Language", "Enabled", "Hooks"])
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
                Style::default().bg(theme.selection_bg).fg(theme.selection_fg)
            } else {
                Style::default().fg(theme.fg)
            };

            let enabled_text = if plugin.enabled { "Yes" } else { "No" };
            let enabled_color = if plugin.enabled {
                theme.success
            } else {
                theme.muted
            };

            let author = plugin.author.as_deref().unwrap_or("-");

            Row::new(vec![
                Cell::from(plugin.name.clone()),
                Cell::from(plugin.version.clone()),
                Cell::from(author.to_string()),
                Cell::from(plugin.language.clone()),
                Cell::from(Span::styled(enabled_text, Style::default().fg(enabled_color))),
                Cell::from(plugin.hooks.len().to_string()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(12),
            Constraint::Percentage(18),
            Constraint::Percentage(13),
            Constraint::Percentage(12),
            Constraint::Percentage(10),
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
