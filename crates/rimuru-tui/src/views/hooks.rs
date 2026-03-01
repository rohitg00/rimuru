use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();

    let header = Row::new(vec!["Name", "Event", "Matcher", "Plugin", "Enabled"])
        .style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .hooks
        .iter()
        .enumerate()
        .map(|(i, hook)| {
            let style = if i == app.selected_index {
                Style::default().bg(theme.selection_bg).fg(theme.selection_fg)
            } else {
                Style::default().fg(theme.fg)
            };

            let enabled_text = if hook.enabled { "Yes" } else { "No" };
            let enabled_color = if hook.enabled {
                theme.success
            } else {
                theme.muted
            };

            let event_color = match hook.event_type.as_str() {
                "PreToolUse" | "PostToolUse" => theme.accent,
                "Stop" | "SessionStart" | "SessionEnd" => theme.success,
                "UserPromptSubmit" => theme.warning,
                "Notification" | "PreCompact" => theme.muted,
                _ => theme.fg,
            };

            let display_name = if hook.name.is_empty() {
                truncate(&hook.id, 30)
            } else {
                truncate(&hook.name, 30)
            };

            let matcher = hook.matcher.as_deref().unwrap_or("-");
            let plugin = hook
                .plugin_id
                .as_deref()
                .map(|p| truncate(p, 20))
                .unwrap_or_else(|| "-".to_string());

            Row::new(vec![
                Cell::from(display_name),
                Cell::from(Span::styled(
                    &hook.event_type,
                    Style::default().fg(event_color),
                )),
                Cell::from(matcher.to_string()),
                Cell::from(plugin),
                Cell::from(Span::styled(enabled_text, Style::default().fg(enabled_color))),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(20),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
            Constraint::Percentage(10),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(Span::styled(
                format!(" Hooks ({}) ", app.hooks.len()),
                Style::default().fg(theme.accent),
            )),
    );

    f.render_widget(table, area);
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
