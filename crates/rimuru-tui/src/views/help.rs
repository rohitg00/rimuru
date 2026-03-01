use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;
use crate::theme::THEMES;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_keybinds(f, app, chunks[0]);
    render_themes(f, app, chunks[1]);
}

fn render_keybinds(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();

    let keybinds = vec![
        ("Navigation", vec![
            ("Tab / BackTab", "Next / Previous tab"),
            ("1-9, 0", "Jump to tab by number"),
            ("j / Down", "Move selection down"),
            ("k / Up", "Move selection up"),
            ("Enter", "Activate selected item"),
        ]),
        ("Actions", vec![
            ("r", "Refresh current view"),
            ("t", "Cycle through themes"),
            ("/", "Search (Esc to cancel)"),
            ("?", "Show this help screen"),
            ("q / Ctrl+C", "Quit"),
        ]),
        ("Tab-Specific", vec![
            ("Agents", "Enter toggles connect/disconnect"),
            ("Plugins", "Enter toggles enable/disable"),
            ("Models", "Enter syncs model pricing"),
        ]),
    ];

    let mut items: Vec<ListItem> = Vec::new();
    for (section, binds) in keybinds {
        items.push(ListItem::new(Line::from(Span::styled(
            format!("  {}", section),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ))));
        items.push(ListItem::new(Line::from("")));
        for (key, desc) in binds {
            items.push(ListItem::new(Line::from(vec![
                Span::styled(
                    format!("    {:16}", key),
                    Style::default()
                        .fg(theme.warning)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(desc, Style::default().fg(theme.fg)),
            ])));
        }
        items.push(ListItem::new(Line::from("")));
    }

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(Span::styled(
                " Keyboard Shortcuts ",
                Style::default().fg(theme.accent),
            )),
    );

    f.render_widget(list, area);
}

fn render_themes(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();

    let items: Vec<ListItem> = THEMES
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let marker = if i == app.theme_index { " *" } else { "  " };
            let style = if i == app.theme_index {
                Style::default()
                    .fg(t.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.fg)
            };

            ListItem::new(Line::from(vec![
                Span::styled(marker, Style::default().fg(theme.accent)),
                Span::styled(format!(" {}", t.name), style),
                Span::raw("  "),
                Span::styled("|||", Style::default().fg(t.accent)),
                Span::styled("|||", Style::default().fg(t.success)),
                Span::styled("|||", Style::default().fg(t.warning)),
                Span::styled("|||", Style::default().fg(t.error)),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(Span::styled(
                " Themes (t to cycle) ",
                Style::default().fg(theme.accent),
            )),
    );

    f.render_widget(list, area);
}
