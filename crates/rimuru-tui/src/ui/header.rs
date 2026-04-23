use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::state::{App, SessionStatusKind, View};

pub(super) fn draw(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();
    let session_count = app.sessions.len();
    let active = app
        .sessions
        .iter()
        .filter(|s| matches!(s.status, SessionStatusKind::Active))
        .count();

    let now = chrono::Local::now().format("%H:%M").to_string();
    let version = env!("CARGO_PKG_VERSION");
    let status_label = if app.connected { " LIVE " } else { " OFFLINE " };
    let status_color = if app.connected { theme.proc_misc } else { theme.status_fg };

    let crumb = match app.view {
        View::Home => String::new(),
        v => format!(" › {}", v.title()),
    };

    let left = format!(" rimuru v{version} ");
    let mid = format!(" agent monitor{} ", crumb);
    let right_active = format!("  {}↑", active);
    let right_total = format!(" {}●", session_count);

    let fixed = left.len() + 1 + mid.len() + now.len() + right_active.len() + right_total.len() + status_label.len() + 2;
    let remaining = (area.width as usize).saturating_sub(fixed);
    let pad = " ".repeat(remaining.max(1));

    let line = Line::from(vec![
        Span::styled(left, Style::default().fg(theme.title).add_modifier(Modifier::BOLD)),
        Span::styled("─", Style::default().fg(theme.div_line)),
        Span::styled(mid, Style::default().fg(theme.graph_text)),
        Span::styled(pad, Style::default()),
        Span::styled(now, Style::default().fg(theme.graph_text)),
        Span::styled(right_active, Style::default().fg(theme.proc_misc)),
        Span::styled(right_total, Style::default().fg(theme.main_fg)),
        Span::styled("  ", Style::default()),
        Span::styled(status_label.to_string(), Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
    ]);
    f.render_widget(Paragraph::new(line), area);
}
