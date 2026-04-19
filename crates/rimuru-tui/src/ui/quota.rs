use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::state::App;
use crate::widgets::{btop_block, fmt_dollars, grad_at, make_gradient, remaining_bar, styled_label};

pub(super) fn draw(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();
    let cpu_grad = make_gradient(theme.cpu_grad.start, theme.cpu_grad.mid, theme.cpu_grad.end);

    let block = btop_block("quota(left)", "²", theme.cpu_box, theme);

    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    if inner.width < 8 || inner.height < 2 {
        f.render_widget(block, area);
        return;
    }

    let bar_w = (inner.width as usize).saturating_sub(12).clamp(3, 20);
    let bs = &app.budget_status;

    let daily_limit = bs.get("daily_limit").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let daily_used = bs.get("daily_spent").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let monthly_limit = bs.get("monthly_limit").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let monthly_used = bs.get("monthly_spent").and_then(|v| v.as_f64()).unwrap_or(0.0);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        " BUDGET",
        Style::default().fg(theme.title).add_modifier(Modifier::BOLD),
    )));

    if daily_limit <= 0.0 && monthly_limit <= 0.0 {
        lines.push(Line::from(Span::styled(
            "  — no limits set",
            Style::default().fg(theme.inactive_fg),
        )));
        lines.push(Line::from(Span::styled(
            "  set via /budget/set",
            Style::default().fg(theme.graph_text),
        )));
    } else {
        if daily_limit > 0.0 {
            let used_pct = ((daily_used / daily_limit) * 100.0).clamp(0.0, 100.0);
            let remaining = 100.0 - used_pct;
            let c = grad_at(&cpu_grad, used_pct);
            let mut s = vec![styled_label(" day ", theme.graph_text)];
            s.extend(remaining_bar(remaining, bar_w, &cpu_grad, theme.meter_bg));
            s.push(Span::styled(
                format!(" {:>3.0}%", remaining),
                Style::default().fg(c),
            ));
            lines.push(Line::from(s));
            lines.push(Line::from(Span::styled(
                format!("  {} / {}", fmt_dollars(daily_used), fmt_dollars(daily_limit)),
                Style::default().fg(theme.graph_text),
            )));
        }
        if monthly_limit > 0.0 {
            let used_pct = ((monthly_used / monthly_limit) * 100.0).clamp(0.0, 100.0);
            let remaining = 100.0 - used_pct;
            let c = grad_at(&cpu_grad, used_pct);
            let mut s = vec![styled_label(" mo  ", theme.graph_text)];
            s.extend(remaining_bar(remaining, bar_w, &cpu_grad, theme.meter_bg));
            s.push(Span::styled(
                format!(" {:>3.0}%", remaining),
                Style::default().fg(c),
            ));
            lines.push(Line::from(s));
            lines.push(Line::from(Span::styled(
                format!("  {} / {}", fmt_dollars(monthly_used), fmt_dollars(monthly_limit)),
                Style::default().fg(theme.graph_text),
            )));
        }
    }

    let total_cost: f64 = app.sessions.iter().map(|s| s.total_cost).sum();
    while lines.len() < (inner.height as usize).saturating_sub(1) {
        lines.push(Line::from(""));
    }
    lines.push(Line::from(vec![
        Span::styled(
            format!(" {}", fmt_dollars(total_cost)),
            Style::default().fg(theme.main_fg),
        ),
        Span::styled(
            format!("  saved {}", fmt_dollars(app.total_savings)),
            Style::default().fg(theme.proc_misc),
        ),
    ]));

    f.render_widget(Paragraph::new(lines).block(block), area);
}
