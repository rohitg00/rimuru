use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressStyle {
    Bar,
    Blocks,
    Thin,
    Rounded,
}

pub struct ProgressBar {
    progress: f64,
    label: Option<String>,
    style: ProgressStyle,
    show_percentage: bool,
    title: Option<String>,
    use_gradient: bool,
}

impl ProgressBar {
    pub fn new(progress: f64) -> Self {
        Self {
            progress: progress.clamp(0.0, 1.0),
            label: None,
            style: ProgressStyle::Bar,
            show_percentage: true,
            title: None,
            use_gradient: false,
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_style(mut self, style: ProgressStyle) -> Self {
        self.style = style;
        self
    }

    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    pub fn with_gradient(mut self, gradient: bool) -> Self {
        self.use_gradient = gradient;
        self
    }

    fn get_progress_color(&self, theme: &dyn Theme) -> ratatui::style::Color {
        if !self.use_gradient {
            return theme.accent();
        }

        if self.progress < 0.5 {
            theme.success()
        } else if self.progress < 0.8 {
            theme.warning()
        } else {
            theme.error()
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &dyn Theme) {
        match self.style {
            ProgressStyle::Bar => self.render_bar(frame, area, theme),
            ProgressStyle::Blocks => self.render_blocks(frame, area, theme),
            ProgressStyle::Thin => self.render_thin(frame, area, theme),
            ProgressStyle::Rounded => self.render_rounded(frame, area, theme),
        }
    }

    fn render_bar(&self, frame: &mut Frame, area: Rect, theme: &dyn Theme) {
        let progress_color = self.get_progress_color(theme);

        let label = if self.show_percentage {
            self.label
                .clone()
                .unwrap_or_else(|| format!("{:.0}%", self.progress * 100.0))
        } else {
            self.label.clone().unwrap_or_default()
        };

        let mut gauge = Gauge::default()
            .percent((self.progress * 100.0) as u16)
            .gauge_style(Style::default().fg(progress_color).bg(theme.surface()))
            .label(Span::styled(
                label,
                Style::default()
                    .fg(theme.foreground())
                    .add_modifier(Modifier::BOLD),
            ));

        if let Some(ref title) = self.title {
            gauge = gauge.block(
                Block::default()
                    .title(format!(" {} ", title))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border()))
                    .style(Style::default().bg(theme.background())),
            );
        }

        frame.render_widget(gauge, area);
    }

    fn render_blocks(&self, frame: &mut Frame, area: Rect, theme: &dyn Theme) {
        let progress_color = self.get_progress_color(theme);
        let inner_width = if self.title.is_some() {
            area.width.saturating_sub(4)
        } else {
            area.width.saturating_sub(2)
        };

        let label_width = if self.show_percentage { 5 } else { 0 };
        let bar_width = inner_width.saturating_sub(label_width + 1);

        let filled = ((bar_width as f64) * self.progress) as usize;
        let empty = (bar_width as usize).saturating_sub(filled);

        let filled_char = "█";
        let empty_char = "░";

        let mut spans = vec![
            Span::styled(
                filled_char.repeat(filled),
                Style::default().fg(progress_color),
            ),
            Span::styled(
                empty_char.repeat(empty),
                Style::default().fg(theme.foreground_dim()),
            ),
        ];

        if self.show_percentage {
            spans.push(Span::styled(
                format!(" {:3.0}%", self.progress * 100.0),
                Style::default()
                    .fg(theme.foreground())
                    .add_modifier(Modifier::BOLD),
            ));
        }

        let paragraph = Paragraph::new(Line::from(spans));

        if let Some(ref title) = self.title {
            let block = Block::default()
                .title(format!(" {} ", title))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border()))
                .style(Style::default().bg(theme.background()));
            let inner = block.inner(area);
            frame.render_widget(block, area);
            frame.render_widget(paragraph, inner);
        } else {
            frame.render_widget(paragraph, area);
        }
    }

    fn render_thin(&self, frame: &mut Frame, area: Rect, theme: &dyn Theme) {
        let progress_color = self.get_progress_color(theme);
        let bar_width = area.width.saturating_sub(2);

        let filled = ((bar_width as f64) * self.progress) as usize;
        let empty = (bar_width as usize).saturating_sub(filled);

        let spans = vec![
            Span::styled("[", Style::default().fg(theme.border())),
            Span::styled("─".repeat(filled), Style::default().fg(progress_color)),
            Span::styled(
                "─".repeat(empty),
                Style::default().fg(theme.foreground_dim()),
            ),
            Span::styled("]", Style::default().fg(theme.border())),
        ];

        let paragraph = Paragraph::new(Line::from(spans));
        frame.render_widget(paragraph, area);
    }

    fn render_rounded(&self, frame: &mut Frame, area: Rect, theme: &dyn Theme) {
        let progress_color = self.get_progress_color(theme);
        let bar_width = area.width.saturating_sub(4);

        let filled = ((bar_width as f64) * self.progress) as usize;
        let empty = (bar_width as usize).saturating_sub(filled);

        let mut bar_chars = String::new();
        for i in 0..filled {
            if i == 0 {
                bar_chars.push('▐');
            } else {
                bar_chars.push('█');
            }
        }

        let mut spans = vec![
            Span::styled("(", Style::default().fg(theme.border())),
            Span::styled(bar_chars, Style::default().fg(progress_color)),
            Span::styled(
                "░".repeat(empty),
                Style::default().fg(theme.foreground_dim()),
            ),
            Span::styled(")", Style::default().fg(theme.border())),
        ];

        if self.show_percentage {
            spans.push(Span::styled(
                format!(" {:.0}%", self.progress * 100.0),
                Style::default().fg(theme.foreground_dim()),
            ));
        }

        let paragraph = Paragraph::new(Line::from(spans));
        frame.render_widget(paragraph, area);
    }
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new(0.0)
    }
}

pub struct StepProgress {
    current_step: usize,
    total_steps: usize,
    step_labels: Vec<String>,
    show_labels: bool,
}

impl StepProgress {
    pub fn new(current: usize, total: usize) -> Self {
        Self {
            current_step: current.min(total),
            total_steps: total.max(1),
            step_labels: Vec::new(),
            show_labels: false,
        }
    }

    pub fn with_labels(mut self, labels: Vec<impl Into<String>>) -> Self {
        self.step_labels = labels.into_iter().map(|l| l.into()).collect();
        self.show_labels = true;
        self
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &dyn Theme) {
        let _step_width = area.width / (self.total_steps as u16);

        let mut spans = Vec::new();

        for i in 0..self.total_steps {
            let is_complete = i < self.current_step;
            let is_current = i == self.current_step;

            let dot = if is_complete {
                Span::styled("●", Style::default().fg(theme.success()))
            } else if is_current {
                Span::styled(
                    "◉",
                    Style::default()
                        .fg(theme.accent())
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled("○", Style::default().fg(theme.foreground_dim()))
            };

            spans.push(dot);

            if i < self.total_steps - 1 {
                let connector = if is_complete {
                    Span::styled("───".to_string(), Style::default().fg(theme.success()))
                } else {
                    Span::styled(
                        "───".to_string(),
                        Style::default().fg(theme.foreground_dim()),
                    )
                };
                spans.push(connector);
            }
        }

        let paragraph = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
    }
}

impl Default for StepProgress {
    fn default() -> Self {
        Self::new(0, 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_clamps() {
        let bar = ProgressBar::new(1.5);
        assert_eq!(bar.progress, 1.0);

        let bar = ProgressBar::new(-0.5);
        assert_eq!(bar.progress, 0.0);
    }

    #[test]
    fn test_progress_bar_builder() {
        let bar = ProgressBar::new(0.5)
            .with_label("Loading")
            .with_title("Progress")
            .with_style(ProgressStyle::Blocks)
            .show_percentage(false)
            .with_gradient(true);

        assert_eq!(bar.progress, 0.5);
        assert_eq!(bar.label, Some("Loading".to_string()));
        assert_eq!(bar.title, Some("Progress".to_string()));
        assert_eq!(bar.style, ProgressStyle::Blocks);
        assert!(!bar.show_percentage);
        assert!(bar.use_gradient);
    }

    #[test]
    fn test_step_progress() {
        let progress = StepProgress::new(2, 5);
        assert_eq!(progress.current_step, 2);
        assert_eq!(progress.total_steps, 5);
    }

    #[test]
    fn test_step_progress_clamps() {
        let progress = StepProgress::new(10, 5);
        assert_eq!(progress.current_step, 5);

        let progress = StepProgress::new(0, 0);
        assert_eq!(progress.total_steps, 1);
    }

    #[test]
    fn test_step_progress_labels() {
        let progress = StepProgress::new(1, 3).with_labels(vec!["Start", "Process", "Done"]);
        assert_eq!(progress.step_labels.len(), 3);
        assert!(progress.show_labels);
    }
}
