use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Gauge},
    Frame,
};

use crate::theme::Theme;

pub struct MetricsGauge;

impl MetricsGauge {
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        theme: &dyn Theme,
        title: &str,
        value: f64,
        unit: &str,
    ) {
        let color = if value > 90.0 {
            theme.error()
        } else if value > 70.0 {
            theme.warning()
        } else {
            theme.success()
        };

        let gauge = Gauge::default()
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border()))
                    .style(Style::default().bg(theme.surface())),
            )
            .gauge_style(Style::default().fg(color).bg(theme.background()))
            .percent(value.min(100.0) as u16)
            .label(format!("{:.1}{}", value, unit));

        frame.render_widget(gauge, area);
    }
}
