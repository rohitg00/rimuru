use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(9), Constraint::Min(0)])
        .split(area);

    render_gauges(f, app, chunks[0]);
    render_history(f, app, chunks[1]);
}

fn render_gauges(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
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
        f.render_widget(cpu_gauge, cols[0]);

        let mem_gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(Span::styled(" Memory ", Style::default().fg(theme.accent))),
            )
            .gauge_style(Style::default().fg(gauge_color(mem_ratio, theme)))
            .ratio(mem_ratio)
            .label(format!(
                "{:.0}/{:.0} MB",
                m.memory_used_mb, m.memory_total_mb
            ));
        f.render_widget(mem_gauge, cols[1]);

        let stats_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(Span::styled(
                " Performance ",
                Style::default().fg(theme.accent),
            ));

        let stats = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Req/min: ", Style::default().fg(theme.muted)),
                Span::styled(
                    format!("{:.1}", m.requests_per_minute),
                    Style::default().fg(theme.fg),
                ),
            ]),
            Line::from(vec![
                Span::styled("Avg RT:  ", Style::default().fg(theme.muted)),
                Span::styled(
                    format!("{:.1}ms", m.avg_response_time_ms),
                    Style::default().fg(theme.fg),
                ),
            ]),
            Line::from(vec![
                Span::styled("Errors:  ", Style::default().fg(theme.muted)),
                Span::styled(
                    format!("{:.2}%", m.error_rate * 100.0),
                    Style::default().fg(if m.error_rate > 0.05 {
                        theme.error
                    } else {
                        theme.success
                    }),
                ),
            ]),
        ])
        .block(stats_block);
        f.render_widget(stats, cols[2]);

        let info_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(Span::styled(" Status ", Style::default().fg(theme.accent)));

        let uptime = format_uptime(m.uptime_secs);
        let info = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Agents:   ", Style::default().fg(theme.muted)),
                Span::styled(
                    m.active_agents.to_string(),
                    Style::default().fg(theme.accent),
                ),
            ]),
            Line::from(vec![
                Span::styled("Sessions: ", Style::default().fg(theme.muted)),
                Span::styled(
                    m.active_sessions.to_string(),
                    Style::default().fg(theme.success),
                ),
            ]),
            Line::from(vec![
                Span::styled("Uptime:   ", Style::default().fg(theme.muted)),
                Span::styled(uptime, Style::default().fg(theme.fg)),
            ]),
        ])
        .block(info_block);
        f.render_widget(info, cols[3]);
    } else {
        let p = Paragraph::new("Waiting for metrics data...")
            .style(Style::default().fg(theme.muted))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border)),
            );
        f.render_widget(p, area);
    }
}

fn render_history(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(Span::styled(
            " Metrics History ",
            Style::default().fg(theme.accent),
        ));

    if let Some(ref history) = app.metrics_history {
        if history.entries.is_empty() {
            let p = Paragraph::new("No history data yet")
                .style(Style::default().fg(theme.muted))
                .alignment(Alignment::Center)
                .block(block);
            f.render_widget(p, area);
            return;
        }

        let cpu_data: Vec<u64> = history
            .entries
            .iter()
            .map(|e| e.cpu_usage_percent as u64)
            .collect();

        let sparkline = Sparkline::default()
            .block(block)
            .data(&cpu_data)
            .style(Style::default().fg(theme.accent));

        f.render_widget(sparkline, area);
    } else {
        let p = Paragraph::new("No history available")
            .style(Style::default().fg(theme.muted))
            .alignment(Alignment::Center)
            .block(block);
        f.render_widget(p, area);
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

fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    if days > 0 {
        format!("{}d {}h", days, hours)
    } else if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m {}s", mins, secs % 60)
    }
}
