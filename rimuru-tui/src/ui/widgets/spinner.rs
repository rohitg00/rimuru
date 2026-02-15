use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::theme::Theme;

pub struct Spinner {
    frames: Vec<&'static str>,
    message: Option<String>,
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            message: None,
        }
    }

    pub fn dots() -> Self {
        Self {
            frames: vec!["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"],
            message: None,
        }
    }

    pub fn line() -> Self {
        Self {
            frames: vec!["-", "\\", "|", "/"],
            message: None,
        }
    }

    pub fn moon() -> Self {
        Self {
            frames: vec!["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"],
            message: None,
        }
    }

    pub fn bounce() -> Self {
        Self {
            frames: vec!["⠁", "⠂", "⠄", "⡀", "⢀", "⠠", "⠐", "⠈"],
            message: None,
        }
    }

    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    pub fn frame(&self, tick: u64) -> &'static str {
        let idx = (tick as usize) % self.frames.len();
        self.frames[idx]
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &dyn Theme, tick: u64) {
        let spinner_char = self.frame(tick);

        let spans = if let Some(ref msg) = self.message {
            vec![
                Span::styled(
                    spinner_char,
                    Style::default()
                        .fg(theme.accent())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" ", Style::default()),
                Span::styled(msg.clone(), Style::default().fg(theme.foreground_dim())),
            ]
        } else {
            vec![Span::styled(
                spinner_char,
                Style::default()
                    .fg(theme.accent())
                    .add_modifier(Modifier::BOLD),
            )]
        };

        let paragraph = Paragraph::new(Line::from(spans));
        frame.render_widget(paragraph, area);
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

pub struct LoadingIndicator {
    spinner: Spinner,
    progress: Option<f32>,
}

impl LoadingIndicator {
    pub fn new() -> Self {
        Self {
            spinner: Spinner::new(),
            progress: None,
        }
    }

    pub fn with_progress(mut self, progress: f32) -> Self {
        self.progress = Some(progress.clamp(0.0, 1.0));
        self
    }

    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.spinner = self.spinner.with_message(msg);
        self
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &dyn Theme, tick: u64) {
        if let Some(progress) = self.progress {
            let percent = (progress * 100.0) as u8;
            let filled = ((area.width as f32 - 8.0) * progress) as u16;
            let empty = area.width.saturating_sub(filled + 8);

            let bar = format!(
                "{} [{}{}] {:3}%",
                self.spinner.frame(tick),
                "█".repeat(filled as usize),
                "░".repeat(empty as usize),
                percent
            );

            let paragraph = Paragraph::new(Line::from(Span::styled(
                bar,
                Style::default().fg(theme.accent()),
            )));
            frame.render_widget(paragraph, area);
        } else {
            self.spinner.render(frame, area, theme, tick);
        }
    }
}

impl Default for LoadingIndicator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_frames() {
        let spinner = Spinner::new();
        assert_eq!(spinner.frame(0), "⠋");
        assert_eq!(spinner.frame(1), "⠙");
        assert_eq!(spinner.frame(10), "⠋");
    }

    #[test]
    fn test_spinner_variants() {
        let dots = Spinner::dots();
        assert!(dots.frames.len() > 0);

        let line = Spinner::line();
        assert_eq!(line.frame(0), "-");

        let moon = Spinner::moon();
        assert!(moon.frames.len() > 0);

        let bounce = Spinner::bounce();
        assert!(bounce.frames.len() > 0);
    }

    #[test]
    fn test_spinner_with_message() {
        let spinner = Spinner::new().with_message("Loading...");
        assert_eq!(spinner.message, Some("Loading...".to_string()));
    }

    #[test]
    fn test_loading_indicator_progress() {
        let indicator = LoadingIndicator::new().with_progress(0.5);
        assert_eq!(indicator.progress, Some(0.5));
    }

    #[test]
    fn test_loading_indicator_clamps_progress() {
        let indicator = LoadingIndicator::new().with_progress(1.5);
        assert_eq!(indicator.progress, Some(1.0));

        let indicator = LoadingIndicator::new().with_progress(-0.5);
        assert_eq!(indicator.progress, Some(0.0));
    }
}
