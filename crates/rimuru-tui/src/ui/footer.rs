use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::state::{App, View};

pub(super) fn draw(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();

    let mut spans: Vec<Span> = Vec::new();
    if app.view == View::Home {
        push_key(&mut spans, "↑↓", " select ", theme);
        push_key(&mut spans, "1-9", " drill ", theme);
    } else {
        push_key(&mut spans, "esc", " home ", theme);
        push_key(&mut spans, "j/k", " scroll ", theme);
    }
    push_key(&mut spans, "t", " theme ", theme);
    push_key(&mut spans, "r", " refresh ", theme);
    push_key(&mut spans, "q", " quit ", theme);

    let status_text = app
        .status_msg
        .as_ref()
        .filter(|(_, when)| when.elapsed().as_secs() < 3)
        .map(|(m, _)| m.as_str());
    if let Some(m) = status_text {
        spans.push(Span::styled(format!(" {} ", m), Style::default().fg(theme.status_fg)));
    } else {
        spans.push(Span::styled("3s auto", Style::default().fg(theme.inactive_fg)));
    }

    let used: usize = spans.iter().map(|s| s.content.len()).sum();
    let remaining = (area.width as usize).saturating_sub(used + 2);
    let right = format!("{} sessions", app.sessions.len());
    spans.push(Span::styled(
        format!("{:>width$}", right, width = remaining),
        Style::default().fg(theme.graph_text),
    ));

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn push_key<'a>(spans: &mut Vec<Span<'a>>, key: &'a str, label: &'a str, theme: &crate::theme::Theme) {
    spans.push(Span::styled(format!(" {}", key), Style::default().fg(theme.hi_fg)));
    spans.push(Span::styled(label, Style::default().fg(theme.main_fg)));
}
