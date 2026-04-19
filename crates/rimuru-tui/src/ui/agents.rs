use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::state::App;
use crate::widgets::{btop_block, truncate_str};

pub(super) fn draw(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();
    let inner_w = area.width.saturating_sub(2) as usize;
    let name_w = inner_w.saturating_sub(10);

    let mut lines: Vec<Line> = Vec::new();
    for agent in app.agents.iter().take(area.height.saturating_sub(2) as usize) {
        let name = agent.get("name").and_then(|v| v.as_str()).unwrap_or("?");
        let status = agent.get("status").and_then(|v| v.as_str()).unwrap_or("");
        let sessions = agent.get("session_count").and_then(|v| v.as_u64()).unwrap_or(0);

        let (dot, dot_color) = match status {
            "connected" | "active" => ("●", theme.proc_misc),
            "idle" => ("○", theme.inactive_fg),
            "disconnected" | "" => ("·", theme.inactive_fg),
            _ => ("◌", theme.warning_fg),
        };

        lines.push(Line::from(vec![
            Span::styled(format!(" {} ", dot), Style::default().fg(dot_color)),
            Span::styled(
                truncate_str(name, name_w),
                Style::default().fg(theme.title).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("   ", Style::default()),
            Span::styled(
                format!("{} · {} session{}", status, sessions, if sessions == 1 { "" } else { "s" }),
                Style::default().fg(theme.graph_text),
            ),
        ]));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            " no agents connected",
            Style::default().fg(theme.inactive_fg),
        )));
    }

    let block = btop_block("agents", "", theme.mem_box, theme);
    f.render_widget(Paragraph::new(lines).block(block), area);
}
