use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();

    let header = Row::new(vec!["Name", "Type", "Status", "Sessions", "Cost", "Last Seen"])
        .style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .agents
        .iter()
        .enumerate()
        .map(|(i, agent)| {
            let status_color = match agent.status.as_str() {
                "Connected" | "Active" => theme.success,
                "Idle" => theme.warning,
                "Error" => theme.error,
                _ => theme.muted,
            };

            let style = if i == app.selected_index {
                Style::default().bg(theme.highlight).fg(theme.fg)
            } else {
                Style::default().fg(theme.fg)
            };

            let last_seen = agent
                .last_seen
                .map(|t| t.format("%H:%M:%S").to_string())
                .unwrap_or_else(|| "-".to_string());

            Row::new(vec![
                Cell::from(agent.name.clone()),
                Cell::from(agent.agent_type.clone()),
                Cell::from(Span::styled(&agent.status, Style::default().fg(status_color))),
                Cell::from(agent.session_count.to_string()),
                Cell::from(format!("${:.2}", agent.total_cost)),
                Cell::from(last_seen),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(20),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(Span::styled(
                format!(" Agents ({}) ", app.agents.len()),
                Style::default().fg(theme.accent),
            ))
            .title_bottom(Line::from(Span::styled(
                " Enter: Toggle Connect  j/k: Navigate ",
                Style::default().fg(theme.muted),
            ))),
    );

    f.render_widget(table, area);
}
