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

    render_header(f, app, chunks[0]);

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

    render_footer(f, app, chunks[2]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(22),
            Constraint::Min(0),
            Constraint::Length(10),
        ])
        .split(area);

    let logo = Paragraph::new(Line::from(vec![
        Span::styled("りむる ", Style::default().fg(theme.accent)),
        Span::styled(
            "Rimuru",
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" v0.1", Style::default().fg(theme.muted)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border)),
    );
    f.render_widget(logo, cols[0]);

    let titles: Vec<Line> = Tab::all()
        .iter()
        .enumerate()
        .map(|(i, tab)| {
            let num = if i < 9 {
                format!("{}", i + 1)
            } else {
                "0".to_string()
            };
            Line::from(vec![
                Span::styled(format!("{}", num), Style::default().fg(theme.muted)),
                Span::raw(tab.label()),
            ])
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border)),
        )
        .select(app.current_tab.index())
        .style(Style::default().fg(theme.fg))
        .highlight_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .divider(" │ ");
    f.render_widget(tabs, cols[1]);

    let now = chrono::Local::now();
    let clock = Paragraph::new(Line::from(Span::styled(
        now.format("%H:%M:%S").to_string(),
        Style::default().fg(theme.muted),
    )))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border)),
    );
    f.render_widget(clock, cols[2]);
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(30)])
        .split(area);

    let keybinds = vec![
        ("q", "Quit"),
        ("Tab", "Next"),
        ("j/k", "Nav"),
        ("t", "Theme"),
        ("r", "Refresh"),
        ("/", "Search"),
        ("?", "Help"),
    ];

    let mut spans: Vec<Span> = Vec::new();
    for (key, desc) in &keybinds {
        spans.push(Span::styled(
            format!(" {}", key),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(":{} ", desc),
            Style::default().fg(theme.muted),
        ));
    }

    if app.searching {
        spans.push(Span::styled(
            format!(" /{}", app.search_query),
            Style::default().fg(theme.warning),
        ));
    }

    if let Some(ref msg) = app.status_message {
        spans.push(Span::styled(
            format!(" {} ", msg),
            Style::default().fg(theme.warning),
        ));
    }

    f.render_widget(Paragraph::new(Line::from(spans)), cols[0]);

    let connection = if app.connected {
        Span::styled(
            "● Connected",
            Style::default().fg(theme.success),
        )
    } else {
        Span::styled(
            "○ Disconnected",
            Style::default().fg(theme.error),
        )
    };

    let right = Paragraph::new(Line::from(vec![
        connection,
        Span::styled(
            format!("  {}", app.theme().name),
            Style::default().fg(theme.muted),
        ),
    ]))
    .alignment(Alignment::Right);
    f.render_widget(right, cols[1]);
}
