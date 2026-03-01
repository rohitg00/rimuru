use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;

const SLIME: &str = r"        .-~~~-.
      .'  *    '.
     /            \
    |  \__    __/  |
    |     '--'     |
     \   Rimuru   /
      '-._____.-'";

const SAGE_TIPS: &[&str] = &[
    "<<Analysis Complete>> All systems nominal.",
    "<<Notice>> Great Sage monitoring all agents.",
    "<<Confirmed>> Predator has consumed all metrics.",
    "<<Report>> Tempest Federation networks stable.",
    "<<Advice>> Use 'r' to refresh data, Master.",
    "<<Notice>> Raphael analyzing cost patterns...",
    "<<Status>> All skills operating within parameters.",
    "<<Alert>> Veldora says hi from the stomach.",
];

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
    render_right_panel(f, app, cols[2]);
}

fn render_agents_sidebar(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();

    let items: Vec<ListItem> = if app.agents.is_empty() {
        vec![
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(Span::styled(
                "  No agents detected",
                Style::default().fg(theme.muted),
            ))),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(Span::styled(
                "  Waiting for agents",
                Style::default().fg(theme.muted),
            ))),
            ListItem::new(Line::from(Span::styled(
                "  to join Tempest...",
                Style::default().fg(theme.muted),
            ))),
        ]
    } else {
        app.agents
            .iter()
            .enumerate()
            .map(|(i, agent)| {
                let status_lower = agent.status.to_lowercase();
                let dot = match status_lower.as_str() {
                    "connected" | "active" => "●",
                    "idle" => "◐",
                    _ => "○",
                };
                let dot_color = match status_lower.as_str() {
                    "connected" | "active" => theme.success,
                    "idle" => theme.warning,
                    "error" => theme.error,
                    _ => theme.muted,
                };

                let style = if i == app.selected_index {
                    Style::default()
                        .bg(theme.selection_bg)
                        .fg(theme.selection_fg)
                } else {
                    Style::default().fg(theme.fg)
                };

                ListItem::new(Line::from(vec![
                    Span::styled(format!(" {} ", dot), Style::default().fg(dot_color)),
                    Span::styled(&agent.name, style),
                ]))
            })
            .collect()
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(Span::styled(
                " Subordinates ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )),
    );

    f.render_widget(list, area);
}

fn render_center(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    let slime_lines: Vec<Line> = SLIME
        .lines()
        .map(|l| Line::from(Span::styled(l, Style::default().fg(theme.accent))))
        .collect();

    let slime = Paragraph::new(slime_lines)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent))
                .title(Span::styled(
                    " Great Demon Lord ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                )),
        );
    f.render_widget(slime, chunks[0]);

    render_summary(f, app, chunks[1]);

    let tip_idx = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        / 5) as usize
        % SAGE_TIPS.len();

    let sage = Paragraph::new(Line::from(vec![
        Span::styled(
            " Great Sage: ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            SAGE_TIPS[tip_idx],
            Style::default().fg(theme.muted).add_modifier(Modifier::ITALIC),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border)),
    );
    f.render_widget(sage, chunks[2]);
}

fn render_summary(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();

    let (active_sessions, tokens, cost_today, total_cost) =
        if let Some(ref s) = app.stats {
            (
                s.active_sessions.to_string(),
                format_tokens(s.total_tokens),
                format!("${:.2}", s.total_cost_today),
                format!("${:.2}", s.total_cost),
            )
        } else {
            ("-".into(), "-".into(), "-".into(), "-".into())
        };

    let agents_count = if let Some(ref s) = app.stats {
        format!("{}/{}", s.active_agents, s.total_agents)
    } else {
        format!("0/{}", app.agents.len())
    };

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Active Sessions   ", Style::default().fg(theme.muted)),
            Span::styled(
                &active_sessions,
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Agents Online     ", Style::default().fg(theme.muted)),
            Span::styled(&agents_count, Style::default().fg(theme.success)),
        ]),
        Line::from(vec![
            Span::styled("  Magicules (Tokens) ", Style::default().fg(theme.muted)),
            Span::styled(&tokens, Style::default().fg(theme.fg)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Today's Tribute    ", Style::default().fg(theme.muted)),
            Span::styled(
                &cost_today,
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Total Treasury     ", Style::default().fg(theme.muted)),
            Span::styled(&total_cost, Style::default().fg(theme.fg)),
        ]),
    ];

    let summary = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(Span::styled(
                " Raphael's Report ",
                Style::default().fg(theme.accent),
            )),
    );

    f.render_widget(summary, area);
}

fn render_right_panel(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    if let Some(ref m) = app.metrics {
        let cpu = m.cpu_usage_percent;
        let cpu_ratio = (cpu / 100.0).clamp(0.0, 1.0);
        let mem_ratio = if m.memory_total_mb > 0.0 {
            (m.memory_used_mb / m.memory_total_mb).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let cpu_gauge = LineGauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(Span::styled(" CPU ", Style::default().fg(theme.muted))),
            )
            .filled_style(Style::default().fg(gauge_color(cpu_ratio, theme)))
            .unfilled_style(Style::default().fg(theme.border))
            .ratio(cpu_ratio)
            .label(format!("{:.0}%", cpu))
            .line_set(symbols::line::THICK);
        f.render_widget(cpu_gauge, chunks[0]);

        let mem_gb_used = m.memory_used_mb / 1024.0;
        let mem_gb_total = m.memory_total_mb / 1024.0;
        let mem_gauge = LineGauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(Span::styled(" RAM ", Style::default().fg(theme.muted))),
            )
            .filled_style(Style::default().fg(gauge_color(mem_ratio, theme)))
            .unfilled_style(Style::default().fg(theme.border))
            .ratio(mem_ratio)
            .label(format!("{:.0}/{:.0}G", mem_gb_used, mem_gb_total))
            .line_set(symbols::line::THICK);
        f.render_widget(mem_gauge, chunks[1]);

        let err_label = if m.error_rate > 0.0 {
            format!("{:.1}%", m.error_rate * 100.0)
        } else {
            "0%".to_string()
        };
        let err_ratio = m.error_rate.clamp(0.0, 1.0);
        let err_gauge = LineGauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(Span::styled(
                        " Errors ",
                        Style::default().fg(theme.muted),
                    )),
            )
            .filled_style(Style::default().fg(if err_ratio > 0.05 {
                theme.error
            } else {
                theme.success
            }))
            .unfilled_style(Style::default().fg(theme.border))
            .ratio(err_ratio)
            .label(err_label)
            .line_set(symbols::line::THICK);
        f.render_widget(err_gauge, chunks[2]);
    } else {
        for (i, label) in [" CPU ", " RAM ", " Errors "].iter().enumerate() {
            let gauge = LineGauge::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme.border))
                        .title(Span::styled(*label, Style::default().fg(theme.muted))),
                )
                .filled_style(Style::default().fg(theme.muted))
                .unfilled_style(Style::default().fg(theme.border))
                .ratio(0.0)
                .label("-")
                .line_set(symbols::line::THICK);
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
            " Predator (7d) ",
            Style::default().fg(theme.accent),
        ));

    if sparkline_data.is_empty() {
        let p = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                " No data consumed",
                Style::default().fg(theme.muted),
            )),
        ])
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
        theme.gauge_high
    } else if ratio > 0.7 {
        theme.gauge_mid
    } else {
        theme.gauge_low
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
