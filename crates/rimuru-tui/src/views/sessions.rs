use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();

    let header = Row::new(vec![
        "Agent", "Status", "Model", "Tokens", "Cost", "Started", "Duration",
    ])
    .style(
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    )
    .bottom_margin(1);

    let rows: Vec<Row> = app
        .sessions
        .iter()
        .enumerate()
        .map(|(i, session)| {
            let status_lower = session.status.to_lowercase();
            let status_color = match status_lower.as_str() {
                "active" => theme.success,
                "completed" => theme.muted,
                "abandoned" => theme.warning,
                "error" => theme.error,
                _ => theme.fg,
            };

            let style = if i == app.selected_index {
                Style::default().bg(theme.highlight).fg(theme.fg)
            } else {
                Style::default().fg(theme.fg)
            };

            let model = session.model.as_deref().unwrap_or("-");
            let started = session.started_at.format("%m/%d %H:%M").to_string();
            let duration = session
                .ended_at
                .map(|end| {
                    let dur = end - session.started_at;
                    format_duration(dur.num_seconds().unsigned_abs())
                })
                .unwrap_or_else(|| {
                    let dur = chrono::Utc::now() - session.started_at;
                    format!("{}*", format_duration(dur.num_seconds().unsigned_abs()))
                });

            Row::new(vec![
                Cell::from(session.agent_type.clone()),
                Cell::from(Span::styled(
                    &session.status,
                    Style::default().fg(status_color),
                )),
                Cell::from(model.to_string()),
                Cell::from(format_tokens(session.total_tokens)),
                Cell::from(format!("${:.4}", session.total_cost)),
                Cell::from(started),
                Cell::from(duration),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(12),
            Constraint::Percentage(12),
            Constraint::Percentage(18),
            Constraint::Percentage(14),
            Constraint::Percentage(12),
            Constraint::Percentage(16),
            Constraint::Percentage(16),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(Span::styled(
                format!(" Sessions ({}) ", app.sessions.len()),
                Style::default().fg(theme.accent),
            )),
    );

    f.render_widget(table, area);
}

fn format_tokens(tokens: u64) -> String {
    if tokens >= 1_000_000 {
        format!("{:.1}M", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{:.1}K", tokens as f64 / 1_000.0)
    } else {
        tokens.to_string()
    }
}

fn format_duration(secs: u64) -> String {
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    let s = secs % 60;
    if hours > 0 {
        format!("{}h{}m", hours, mins)
    } else if mins > 0 {
        format!("{}m{}s", mins, s)
    } else {
        format!("{}s", s)
    }
}
