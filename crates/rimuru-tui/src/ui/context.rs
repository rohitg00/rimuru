use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Cell, Paragraph, Row, Table};

use crate::state::App;
use crate::widgets::{
    braille_graph_multirow, btop_block, fmt_tokens, grad_at, make_gradient, meter_bar, truncate_str,
};

pub(super) fn draw(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();
    let cpu_grad = make_gradient(theme.cpu_grad.start, theme.cpu_grad.mid, theme.cpu_grad.end);

    let block = btop_block("context", "¹", theme.cpu_box, theme);
    f.render_widget(block, area);

    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    if inner.height < 2 || inner.width < 10 {
        return;
    }

    let halves = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(inner);

    draw_sparkline(f, app, halves[0], &cpu_grad);
    draw_bars(f, app, halves[1], &cpu_grad);
}

fn draw_sparkline(f: &mut Frame, app: &App, area: Rect, cpu_grad: &[Color; 101]) {
    let theme = app.theme();
    let avail_h = area.height as usize;
    let avail_w = area.width as usize;

    let spark_w = avail_w.saturating_sub(2).max(4);
    let rates: Vec<f64> = app.token_rates.iter().copied().collect();
    let max_rate = rates.iter().cloned().fold(1.0_f64, f64::max);
    let normalized: Vec<f64> = rates.iter().map(|&v| v / max_rate).collect();

    let mut lines: Vec<Line> = Vec::new();
    let tokens_per_sec = rates.iter().rev().take(10).sum::<f64>() / 10.0_f64.max(1.0);
    let tokens_per_min = tokens_per_sec * 60.0;
    let current_pct = normalized.last().copied().unwrap_or(0.0) * 100.0;
    let pct_color = grad_at(cpu_grad, current_pct);

    lines.push(Line::from(vec![
        Span::styled(" Token Rate", Style::default().fg(theme.graph_text)),
        Span::styled(
            format!("  {}/min", fmt_tokens(tokens_per_min as u64)),
            Style::default().fg(pct_color),
        ),
    ]));

    let graph_h = avail_h.saturating_sub(2).max(1);
    let rows = braille_graph_multirow(&normalized, spark_w, graph_h, cpu_grad, theme.graph_text);
    for row_spans in rows {
        let mut line_spans = vec![Span::styled(" ", Style::default())];
        line_spans.extend(row_spans);
        lines.push(Line::from(line_spans));
    }

    let total_tokens: u64 = app.sessions.iter().map(|s| s.total_tokens).sum();
    lines.push(Line::from(vec![
        Span::styled(format!(" {}", fmt_tokens(total_tokens)), Style::default().fg(theme.main_fg)),
        Span::styled(" total", Style::default().fg(theme.graph_text)),
    ]));

    f.render_widget(Paragraph::new(lines), area);
}

fn draw_bars(f: &mut Frame, app: &App, area: Rect, cpu_grad: &[Color; 101]) {
    let theme = app.theme();
    let header_style = Style::default().fg(theme.main_fg).add_modifier(Modifier::BOLD);

    let bar_width = (area.width as usize).saturating_sub(30).clamp(4, 20);

    let mut rows: Vec<Row> = Vec::new();
    for session in app.sessions.iter().take(area.height.saturating_sub(1) as usize) {
        let raw_pct = session.context_pct;
        let bar_pct = raw_pct.min(100.0);
        let warn = if raw_pct >= 90.0 { "⚠" } else { "" };
        let pct_color = grad_at(cpu_grad, bar_pct);

        rows.push(Row::new(vec![
            Cell::from(Span::styled(
                truncate_str(&session.project_name, 14),
                Style::default().fg(theme.title),
            )),
            Cell::from(Span::styled(
                session.short_id.clone(),
                Style::default().fg(theme.session_id),
            )),
            Cell::from(Line::from({
                let mut spans = meter_bar(bar_pct, bar_width, cpu_grad, theme.meter_bg);
                spans.push(Span::styled(
                    format!(" {:>3.0}%{}", raw_pct, warn),
                    Style::default().fg(pct_color),
                ));
                spans
            })),
        ]));
    }

    if app.sessions.is_empty() {
        rows.push(Row::new(vec![
            Cell::from(Span::styled("no active sessions", Style::default().fg(theme.inactive_fg))),
            Cell::from(""),
            Cell::from(""),
        ]));
    }

    let header = Row::new(vec![
        Cell::from(Span::styled("Project", header_style)),
        Cell::from(Span::styled("Session", header_style)),
        Cell::from(Span::styled("Context", header_style)),
    ]);

    let widths = [Constraint::Length(14), Constraint::Length(9), Constraint::Min(10)];
    let table = Table::new(rows, widths).header(header);
    f.render_widget(table, area);
}
