use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(7), Constraint::Min(0)])
        .split(area);

    render_summary(f, app, chunks[0]);
    render_daily(f, app, chunks[1]);
}

fn render_summary(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    let (total, input, output, records) = if let Some(ref cs) = app.cost_summary {
        (
            format!("${:.4}", cs.total_cost),
            format_tokens(cs.total_input_tokens),
            format_tokens(cs.total_output_tokens),
            cs.total_records.to_string(),
        )
    } else {
        ("-".into(), "-".into(), "-".into(), "-".into())
    };

    let cards = [
        ("Total Cost", &total, theme.warning),
        ("Input Tokens", &input, theme.accent),
        ("Output Tokens", &output, theme.success),
        ("Records", &records, theme.muted),
    ];

    for (i, (title, value, color)) in cards.iter().enumerate() {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(Span::styled(
                format!(" {} ", title),
                Style::default().fg(*color),
            ));

        let p = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                value.to_string(),
                Style::default()
                    .fg(*color)
                    .add_modifier(Modifier::BOLD),
            )),
        ])
        .block(block)
        .alignment(Alignment::Center);

        f.render_widget(p, cols[i]);
    }
}

fn render_daily(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();

    if app.daily_costs.is_empty() {
        let p = Paragraph::new("No daily cost data")
            .style(Style::default().fg(theme.muted))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(Span::styled(" Daily Costs ", Style::default().fg(theme.accent))),
            );
        f.render_widget(p, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    let header = Row::new(vec!["Date", "Cost", "Input", "Output", "Records"])
        .style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .daily_costs
        .iter()
        .enumerate()
        .map(|(i, dc)| {
            let style = if i == app.selected_index {
                Style::default().bg(theme.selection_bg).fg(theme.selection_fg)
            } else {
                Style::default().fg(theme.fg)
            };

            Row::new(vec![
                Cell::from(dc.date.to_string()),
                Cell::from(format!("${:.4}", dc.total_cost)),
                Cell::from(format_tokens(dc.total_input_tokens)),
                Cell::from(format_tokens(dc.total_output_tokens)),
                Cell::from(dc.record_count.to_string()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(15),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(Span::styled(
                " Daily Breakdown ",
                Style::default().fg(theme.accent),
            )),
    );

    f.render_widget(table, chunks[0]);

    let max_cost = app
        .daily_costs
        .iter()
        .map(|d| d.total_cost)
        .fold(0.0_f64, f64::max);

    let theme = app.theme();

    let chart_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(Span::styled(
            " Cost Chart ",
            Style::default().fg(theme.accent),
        ));

    let sparkline_data: Vec<u64> = app
        .daily_costs
        .iter()
        .rev()
        .take(20)
        .map(|d| {
            if max_cost > 0.0 {
                ((d.total_cost / max_cost) * 100.0) as u64
            } else {
                0
            }
        })
        .collect();

    let sparkline = Sparkline::default()
        .block(chart_block)
        .data(&sparkline_data)
        .style(Style::default().fg(theme.warning));

    f.render_widget(sparkline, chunks[1]);
}

fn format_tokens(tokens: u64) -> String {
    if tokens >= 1_000_000 {
        format!("{:.1}M", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{:.1}K", tokens as f64 / 1_000.0)
    } else {
        tokens.to_string()
    }
}
