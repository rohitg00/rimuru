use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;

const LOGO: &str = r#"██████╗ ██╗███╗   ███╗██╗   ██╗██████╗ ██╗   ██╗
██╔══██╗██║████╗ ████║██║   ██║██╔══██╗██║   ██║
██████╔╝██║██╔████╔██║██║   ██║██████╔╝██║   ██║
██╔══██╗██║██║╚██╔╝██║██║   ██║██╔══██╗██║   ██║
██║  ██║██║██║ ╚═╝ ██║╚██████╔╝██║  ██║╚██████╔╝
╚═╝  ╚═╝╚═╝╚═╝     ╚═╝ ╚═════╝ ╚═╝  ╚═╝ ╚═════╝"#;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(area);

    render_agents_sidebar(f, app, cols[0]);
    render_center(f, app, cols[1]);
    render_gauges(f, app, cols[2]);
}

fn render_agents_sidebar(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();

    let items: Vec<ListItem> = if app.agents.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "  No agents detected",
            Style::default().fg(theme.muted),
        )))]
    } else {
        app.agents
            .iter()
            .enumerate()
            .map(|(i, agent)| {
                let prefix = if i == app.selected_index { ">" } else { " " };
                let dot = match agent.status.as_str() {
                    "Connected" | "Active" => "●",
                    "Idle" => "●",
                    _ => "○",
                };
                let dot_color = match agent.status.as_str() {
                    "Connected" | "Active" => theme.success,
                    "Idle" => theme.warning,
                    "Error" => theme.error,
                    _ => theme.muted,
                };
                let name = if agent.name.len() > 14 {
                    format!("{:.14}", agent.name)
                } else {
                    format!("{:<14}", agent.name)
                };
                let status = format!("({})", agent.status);

                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{} ", prefix),
                        Style::default().fg(theme.accent),
                    ),
                    Span::styled(format!("{} ", dot), Style::default().fg(dot_color)),
                    Span::styled(name, Style::default().fg(theme.fg)),
                    Span::styled(
                        format!(" {}", status),
                        Style::default().fg(theme.muted),
                    ),
                ]))
            })
            .collect()
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(Span::styled(
                " Agents ",
                Style::default().fg(theme.accent),
            )),
    );

    f.render_widget(list, area);
}

fn render_center(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(9), Constraint::Min(0)])
        .split(area);

    let logo_lines: Vec<Line> = LOGO
        .lines()
        .map(|l| Line::from(Span::styled(l, Style::default().fg(theme.accent))))
        .collect();

    let logo = Paragraph::new(logo_lines)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent)),
        );
    f.render_widget(logo, chunks[0]);

    render_summary(f, app, chunks[1]);
}

fn render_summary(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();

    let (active_sessions, tokens_today, cost_today, weekly, monthly) =
        if let Some(ref s) = app.stats {
            (
                s.active_sessions.to_string(),
                format_tokens(s.total_sessions * 1000),
                format!("${:.4}", s.total_cost),
                format!("${:.2}", s.total_cost * 7.0),
                format!("${:.2}", s.total_cost * 30.0),
            )
        } else {
            (
                "-".into(),
                "-".into(),
                "-".into(),
                "-".into(),
                "-".into(),
            )
        };

    let agents_count = app.agents.len();
    let connected = app
        .agents
        .iter()
        .filter(|a| a.status == "Connected" || a.status == "Active")
        .count();

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Active Sessions  ", Style::default().fg(theme.muted)),
            Span::styled(
                &active_sessions,
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Agents           ", Style::default().fg(theme.muted)),
            Span::styled(
                format!("{}/{}", connected, agents_count),
                Style::default().fg(theme.fg),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Tokens Today     ", Style::default().fg(theme.muted)),
            Span::styled(&tokens_today, Style::default().fg(theme.fg)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Today's Cost     ", Style::default().fg(theme.muted)),
            Span::styled(
                &cost_today,
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Weekly Estimate  ", Style::default().fg(theme.muted)),
            Span::styled(&weekly, Style::default().fg(theme.fg)),
        ]),
        Line::from(vec![
            Span::styled("  Monthly Estimate ", Style::default().fg(theme.muted)),
            Span::styled(&monthly, Style::default().fg(theme.fg)),
        ]),
    ];

    let summary = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(Span::styled(
                " Quick Summary ",
                Style::default().fg(theme.accent),
            )),
    );

    f.render_widget(summary, area);
}

fn render_gauges(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Min(0),
        ])
        .split(area);

    if let Some(ref m) = app.metrics {
        let cpu_ratio = (m.cpu_usage_percent / 100.0).clamp(0.0, 1.0);
        let mem_ratio = if m.memory_total_mb > 0.0 {
            (m.memory_used_mb / m.memory_total_mb).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let cpu_gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(Span::styled(" CPU ", Style::default().fg(theme.accent))),
            )
            .gauge_style(Style::default().fg(gauge_color(cpu_ratio, theme)))
            .ratio(cpu_ratio)
            .label(format!("{:.1}%", m.cpu_usage_percent));
        f.render_widget(cpu_gauge, chunks[0]);

        let mem_gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(Span::styled(" RAM ", Style::default().fg(theme.accent))),
            )
            .gauge_style(Style::default().fg(gauge_color(mem_ratio, theme)))
            .ratio(mem_ratio)
            .label(format!(
                "{:.0}/{:.0} MB",
                m.memory_used_mb, m.memory_total_mb
            ));
        f.render_widget(mem_gauge, chunks[1]);

        let req_gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(Span::styled(
                        " Requests ",
                        Style::default().fg(theme.accent),
                    )),
            )
            .gauge_style(Style::default().fg(theme.accent))
            .ratio((m.requests_per_minute / 100.0).clamp(0.0, 1.0))
            .label(format!("{:.0}/min", m.requests_per_minute));
        f.render_widget(req_gauge, chunks[2]);
    } else {
        for (i, label) in ["CPU", "RAM", "Requests"].iter().enumerate() {
            let gauge = Gauge::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme.border))
                        .title(Span::styled(
                            format!(" {} ", label),
                            Style::default().fg(theme.accent),
                        )),
                )
                .gauge_style(Style::default().fg(theme.muted))
                .ratio(0.0)
                .label("-");
            f.render_widget(gauge, chunks[i]);
        }
    }

    let sparkline_data: Vec<u64> = app
        .daily_costs
        .iter()
        .rev()
        .take(14)
        .map(|d| (d.total_cost * 10000.0) as u64)
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(Span::styled(
            " Cost Trend (7d) ",
            Style::default().fg(theme.accent),
        ));

    if sparkline_data.is_empty() {
        let p = Paragraph::new(Span::styled(" No data", Style::default().fg(theme.muted)))
            .block(block);
        f.render_widget(p, chunks[3]);
    } else {
        let sparkline = Sparkline::default()
            .block(block)
            .data(&sparkline_data)
            .style(Style::default().fg(theme.accent));
        f.render_widget(sparkline, chunks[3]);
    }
}

fn gauge_color(ratio: f64, theme: &crate::theme::Theme) -> Color {
    if ratio > 0.9 {
        theme.error
    } else if ratio > 0.7 {
        theme.warning
    } else {
        theme.success
    }
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
