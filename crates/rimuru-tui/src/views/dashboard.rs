use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(0)])
        .split(area);

    render_stats_cards(f, app, chunks[0]);
    render_activity(f, app, chunks[1]);
}

fn render_stats_cards(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
        .split(area);

    let (agents, sessions, cost, active, uptime) = if let Some(ref s) = app.stats {
        (
            s.active_agents.to_string(),
            s.total_sessions.to_string(),
            format!("${:.2}", s.total_cost),
            s.active_sessions.to_string(),
            format_uptime(s.uptime_secs),
        )
    } else {
        ("-".into(), "-".into(), "-".into(), "-".into(), "-".into())
    };

    let cards = [
        ("Agents", &agents, theme.accent),
        ("Sessions", &sessions, theme.success),
        ("Total Cost", &cost, theme.warning),
        ("Active", &active, theme.accent),
        ("Uptime", &uptime, theme.muted),
    ];

    for (i, (title, value, color)) in cards.iter().enumerate() {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(Span::styled(
                format!(" {} ", title),
                Style::default().fg(*color),
            ));

        let p = Paragraph::new(Line::from(Span::styled(
            value.to_string(),
            Style::default()
                .fg(*color)
                .add_modifier(Modifier::BOLD),
        )))
        .block(block)
        .alignment(Alignment::Center);

        f.render_widget(p, cols[i]);
    }
}

fn render_activity(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();

    let items: Vec<ListItem> = app
        .activity
        .iter()
        .map(|evt| {
            let ts = evt
                .timestamp
                .map(|t| t.format("%H:%M:%S").to_string())
                .unwrap_or_default();
            ListItem::new(Line::from(vec![
                Span::styled(format!("{} ", ts), Style::default().fg(theme.muted)),
                Span::styled(&evt.event_type, Style::default().fg(theme.accent)),
                Span::raw(" "),
                Span::styled(&evt.description, Style::default().fg(theme.fg)),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(Span::styled(
                " Activity ",
                Style::default().fg(theme.accent),
            )),
    );

    f.render_widget(list, area);
}

fn format_uptime(secs: u64) -> String {
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m {}s", mins, secs % 60)
    }
}
