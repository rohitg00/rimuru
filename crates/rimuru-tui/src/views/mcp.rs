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

    let header = Row::new(vec!["Name", "URL", "Status", "Tools"])
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
                Style::default().bg(theme.highlight).fg(theme.fg)
            } else {
                Style::default().fg(theme.fg)
            };

            let status_color = match server.status.as_str() {
                "Connected" | "Running" => theme.success,
                "Error" => theme.error,
                _ => theme.muted,
            };

            Row::new(vec![
                Cell::from(server.name.clone()),
                Cell::from(server.url.clone()),
                Cell::from(Span::styled(
                    &server.status,
                    Style::default().fg(status_color),
                )),
                Cell::from(server.tools.len().to_string()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(35),
            Constraint::Percentage(15),
            Constraint::Percentage(10),
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

    let selected_tools: Vec<ListItem> = if let Some(server) = app.mcp_servers.get(app.selected_index)
    {
        server
            .tools
            .iter()
            .map(|t| {
                ListItem::new(Span::styled(
                    format!("  {}", t),
                    Style::default().fg(theme.fg),
                ))
            })
            .collect()
    } else {
        vec![ListItem::new(Span::styled(
            "Select a server to view tools",
            Style::default().fg(theme.muted),
        ))]
    };

    let tools_list = List::new(selected_tools).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(Span::styled(
                " Server Tools ",
                Style::default().fg(theme.accent),
            )),
    );

    f.render_widget(tools_list, chunks[1]);
}
