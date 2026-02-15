use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Sparkline},
    Frame,
};

use crate::theme::Theme;

pub struct CostSparkline;

impl CostSparkline {
    pub fn render(frame: &mut Frame, area: Rect, theme: &dyn Theme, title: &str, data: &[u64]) {
        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border()))
                    .style(Style::default().bg(theme.surface())),
            )
            .data(data)
            .style(Style::default().fg(theme.accent()));

        frame.render_widget(sparkline, area);
    }
}
