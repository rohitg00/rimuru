use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();

    if app.mcp_servers.is_empty() {
        let p = Paragraph::new("No MCP servers configured")
            .style(Style::default().fg(theme.muted))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(Span::styled(
                        " MCP Servers ",
                        Style::default().fg(theme.accent),
                    )),
            );
        f.render_widget(p, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let header = Row::new(vec!["Name", "Command", "Source", "Enabled"])
        .style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .mcp_servers
        .iter()
        .enumerate()
        .map(|(i, server)| {
            let style = if i == app.selected_index {
                Style::default().bg(theme.selection_bg).fg(theme.selection_fg)
            } else {
                Style::default().fg(theme.fg)
            };

            let enabled_text = if server.enabled { "Yes" } else { "No" };
            let enabled_color = if server.enabled {
                theme.success
            } else {
                theme.muted
            };

            Row::new(vec![
                Cell::from(server.name.clone()),
                Cell::from(server.command.clone()),
                Cell::from(server.source.clone()),
                Cell::from(Span::styled(enabled_text, Style::default().fg(enabled_color))),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(30),
            Constraint::Percentage(25),
            Constraint::Percentage(15),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(Span::styled(
                format!(" MCP Servers ({}) ", app.mcp_servers.len()),
                Style::default().fg(theme.accent),
            )),
    );

    f.render_widget(table, chunks[0]);

    let detail_lines: Vec<Line> = if let Some(server) = app.mcp_servers.get(app.selected_index) {
        let mut lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  Command:  ", Style::default().fg(theme.muted)),
                Span::styled(&server.command, Style::default().fg(theme.fg)),
            ]),
            Line::from(vec![
                Span::styled("  Args:     ", Style::default().fg(theme.muted)),
                Span::styled(
                    server.args.join(" "),
                    Style::default().fg(theme.fg),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Source:   ", Style::default().fg(theme.muted)),
                Span::styled(&server.source, Style::default().fg(theme.accent)),
            ]),
        ];
        if !server.id.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("  ID:       ", Style::default().fg(theme.muted)),
                Span::styled(&server.id, Style::default().fg(theme.fg)),
            ]));
        }
        lines
    } else {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "  Select a server to view details",
                Style::default().fg(theme.muted),
            )),
        ]
    };

    let detail = Paragraph::new(detail_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(Span::styled(
                " Server Details ",
                Style::default().fg(theme.accent),
            )),
    );

    f.render_widget(detail, chunks[1]);
}
