use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::state::App;
use crate::widgets::{
    braille_sparkline, btop_block, fmt_tokens, grad_at, make_gradient, meter_bar, styled_label,
    truncate_str,
};

pub(super) fn draw(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();
    let selected = app.selected_session();
    let total_in = selected.map(|s| s.input_tokens).unwrap_or(0);
    let total_out = selected.map(|s| s.output_tokens).unwrap_or(0);
    let cache_r = selected.map(|s| s.cache_read).unwrap_or(0);
    let cache_w = selected.map(|s| s.cache_write).unwrap_or(0);
    let total = selected.map(|s| s.total_tokens).unwrap_or(0);
    let turns = selected.map(|s| s.messages as u32).unwrap_or(0);
    let avg = if turns > 0 { total / turns as u64 } else { 0 };
    let cost = selected.map(|s| s.total_cost).unwrap_or(0.0);

    let denom = if total > 0 { total } else { (total_in + total_out + cache_r + cache_w).max(1) };
    let (in_pct, out_pct, cr_pct, cw_pct) = if denom > 0 {
        (
            total_in as f64 / denom as f64 * 100.0,
            total_out as f64 / denom as f64 * 100.0,
            cache_r as f64 / denom as f64 * 100.0,
            cache_w as f64 / denom as f64 * 100.0,
        )
    } else {
        (0.0, 0.0, 0.0, 0.0)
    };

    let free_grad = make_gradient(theme.free_grad.start, theme.free_grad.mid, theme.free_grad.end);
    let used_grad = make_gradient(theme.used_grad.start, theme.used_grad.mid, theme.used_grad.end);
    let cached_grad = make_gradient(theme.cached_grad.start, theme.cached_grad.mid, theme.cached_grad.end);
    let cpu_grad = make_gradient(theme.cpu_grad.start, theme.cpu_grad.mid, theme.cpu_grad.end);

    let bar_w = (area.width as usize).saturating_sub(20).clamp(5, 15);

    let total_line = vec![
        styled_label(" Total: ", theme.graph_text),
        Span::styled(
            fmt_tokens(total),
            Style::default().fg(theme.title).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  ${:.3}", cost),
            Style::default().fg(theme.graph_text),
        ),
    ];

    let mut input_line = vec![styled_label(" Input :", theme.graph_text)];
    input_line.extend(meter_bar(in_pct, bar_w, &free_grad, theme.meter_bg));
    input_line.push(Span::styled(
        format!(" {}", fmt_tokens(total_in)),
        Style::default().fg(grad_at(&free_grad, 80.0)),
    ));

    let mut output_line = vec![styled_label(" Output:", theme.graph_text)];
    output_line.extend(meter_bar(out_pct, bar_w, &used_grad, theme.meter_bg));
    output_line.push(Span::styled(
        format!(" {}", fmt_tokens(total_out)),
        Style::default().fg(grad_at(&used_grad, 80.0)),
    ));

    let mut cr_line = vec![styled_label(" CacheR:", theme.graph_text)];
    cr_line.extend(meter_bar(cr_pct, bar_w, &cached_grad, theme.meter_bg));
    cr_line.push(Span::styled(
        format!(" {}", fmt_tokens(cache_r)),
        Style::default().fg(grad_at(&cached_grad, 80.0)),
    ));

    let mut cw_line = vec![styled_label(" CacheW:", theme.graph_text)];
    cw_line.extend(meter_bar(cw_pct, bar_w, &cached_grad, theme.meter_bg));
    cw_line.push(Span::styled(
        format!(" {}", fmt_tokens(cache_w)),
        Style::default().fg(grad_at(&cached_grad, 80.0)),
    ));

    let history: Vec<u64> = selected.map(|s| s.token_history.clone()).unwrap_or_default();
    let spark_w = (area.width as usize).saturating_sub(16).clamp(5, 20);
    let max_val = history.iter().copied().max().unwrap_or(1).max(1);
    let normalized: Vec<f64> = history.iter().map(|&v| v as f64 / max_val as f64).collect();
    let mut spark = vec![styled_label(" ", theme.graph_text)];
    spark.extend(braille_sparkline(&normalized, spark_w, &cpu_grad, theme.graph_text));
    spark.push(Span::styled(" tokens/turn", Style::default().fg(theme.graph_text)));

    let lines = vec![
        Line::from(total_line),
        Line::from(input_line),
        Line::from(output_line),
        Line::from(cr_line),
        Line::from(cw_line),
        Line::from(spark),
        Line::from(vec![
            styled_label(" Turns: ", theme.graph_text),
            Span::styled(format!("{}", turns), Style::default().fg(theme.main_fg)),
            styled_label("  Avg: ", theme.graph_text),
            Span::styled(
                format!("{}/t", fmt_tokens(avg)),
                Style::default().fg(theme.graph_text),
            ),
        ]),
    ];

    let title = match selected {
        Some(s) => format!(
            "tokens ({}/{})",
            truncate_str(&s.project_name, 12),
            truncate_str(&s.short_id, 8),
        ),
        None => "tokens".to_string(),
    };
    let block = btop_block(&title, "³", theme.mem_box, theme);
    f.render_widget(Paragraph::new(lines).block(block), area);
}
