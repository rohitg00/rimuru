use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::{App, Tab};
use crate::views;

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    render_tabs(f, app, chunks[0]);

    match app.current_tab {
        Tab::Dashboard => views::dashboard::render(f, app, chunks[1]),
        Tab::Agents => views::agents::render(f, app, chunks[1]),
        Tab::Sessions => views::sessions::render(f, app, chunks[1]),
        Tab::Costs => views::costs::render(f, app, chunks[1]),
        Tab::Models => views::models::render(f, app, chunks[1]),
        Tab::Metrics => views::metrics::render(f, app, chunks[1]),
        Tab::Plugins => views::plugins::render(f, app, chunks[1]),
        Tab::Hooks => views::hooks::render(f, app, chunks[1]),
        Tab::Mcp => views::mcp::render(f, app, chunks[1]),
        Tab::Help => views::help::render(f, app, chunks[1]),
    }

    render_status_bar(f, app, chunks[2]);
}

fn render_tabs(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();
    let titles: Vec<Line> = Tab::all()
        .iter()
        .enumerate()
        .map(|(i, tab)| {
            let num = if i < 9 { format!("{}", i + 1) } else { "0".to_string() };
            Line::from(vec![
                Span::styled(format!("{} ", num), Style::default().fg(theme.muted)),
                Span::raw(tab.label()),
            ])
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title(Span::styled(
                    " Rimuru ",
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                )),
        )
        .select(app.current_tab.index())
        .style(Style::default().fg(theme.fg))
        .highlight_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs, area);
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();

    let connection = if app.connected {
        Span::styled("● Connected", Style::default().fg(theme.success))
    } else {
        Span::styled("○ Disconnected", Style::default().fg(theme.error))
    };

    let theme_name = Span::styled(
        format!(" │ {} ", app.theme().name),
        Style::default().fg(theme.muted),
    );

    let help_hint = Span::styled(
        " │ q:Quit Tab:Switch t:Theme r:Refresh ?:Help",
        Style::default().fg(theme.muted),
    );

    let search = if app.searching {
        Span::styled(
            format!(" │ /{}", app.search_query),
            Style::default().fg(theme.warning),
        )
    } else {
        Span::raw("")
    };

    let status = if let Some(ref msg) = app.status_message {
        Span::styled(format!(" │ {}", msg), Style::default().fg(theme.warning))
    } else if let Some(ref err) = app.last_error {
        Span::styled(
            format!(" │ {}", truncate(err, 40)),
            Style::default().fg(theme.error),
        )
    } else {
        Span::raw("")
    };

    let bar = Paragraph::new(Line::from(vec![
        connection, theme_name, help_hint, search, status,
    ]))
    .style(Style::default().bg(theme.bg));

    f.render_widget(bar, area);
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
