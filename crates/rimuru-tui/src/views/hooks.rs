use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();

    let header = Row::new(vec!["ID", "Event Type", "Function", "Priority", "Enabled"])
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
                Style::default().bg(theme.highlight).fg(theme.fg)
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
                "AgentConnected" | "AgentDisconnected" => theme.accent,
                "SessionStarted" | "SessionEnded" => theme.success,
                "CostRecorded" => theme.warning,
                "HealthCheckFailed" | "ThresholdExceeded" => theme.error,
                _ => theme.fg,
            };

            Row::new(vec![
                Cell::from(truncate(&hook.id, 12)),
                Cell::from(Span::styled(
                    &hook.event_type,
                    Style::default().fg(event_color),
                )),
                Cell::from(hook.function_id.clone()),
                Cell::from(hook.priority.to_string()),
                Cell::from(Span::styled(enabled_text, Style::default().fg(enabled_color))),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(15),
            Constraint::Percentage(25),
            Constraint::Percentage(30),
            Constraint::Percentage(12),
            Constraint::Percentage(12),
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
