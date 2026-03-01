use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();

    let header = Row::new(vec![
        "Model",
        "Provider",
        "Input $/M",
        "Output $/M",
        "Context",
        "Vision",
        "Tools",
    ])
    .style(
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    )
    .bottom_margin(1);

    let rows: Vec<Row> = app
        .models
        .iter()
        .enumerate()
        .map(|(i, model)| {
            let style = if i == app.selected_index {
                Style::default().bg(theme.selection_bg).fg(theme.selection_fg)
            } else {
                Style::default().fg(theme.fg)
            };

            let context = if model.context_window >= 1_000_000 {
                format!("{:.0}M", model.context_window as f64 / 1_000_000.0)
            } else {
                format!("{}K", model.context_window / 1_000)
            };

            let vision = if model.supports_vision { "Yes" } else { "-" };
            let tools = if model.supports_tools { "Yes" } else { "-" };

            Row::new(vec![
                Cell::from(model.name.clone()),
                Cell::from(model.provider.clone()),
                Cell::from(format!("${:.2}", model.input_price_per_million)),
                Cell::from(format!("${:.2}", model.output_price_per_million)),
                Cell::from(context),
                Cell::from(Span::styled(
                    vision,
                    Style::default().fg(if model.supports_vision {
                        theme.success
                    } else {
                        theme.muted
                    }),
                )),
                Cell::from(Span::styled(
                    tools,
                    Style::default().fg(if model.supports_tools {
                        theme.success
                    } else {
                        theme.muted
                    }),
                )),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(12),
            Constraint::Percentage(12),
            Constraint::Percentage(13),
            Constraint::Percentage(12),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(Span::styled(
                format!(" Models ({}) ", app.models.len()),
                Style::default().fg(theme.accent),
            ))
            .title_bottom(Line::from(Span::styled(
                " Enter: Sync Models ",
                Style::default().fg(theme.muted),
            ))),
    );

    f.render_widget(table, area);
}
