use std::time::{Duration, Instant};

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl ToastLevel {
    pub fn icon(&self) -> &'static str {
        match self {
            ToastLevel::Info => "ℹ",
            ToastLevel::Success => "✓",
            ToastLevel::Warning => "⚠",
            ToastLevel::Error => "✗",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub level: ToastLevel,
    pub created_at: Instant,
    pub duration: Duration,
    pub dismissible: bool,
    id: u64,
}

impl Toast {
    pub fn new(message: impl Into<String>, level: ToastLevel) -> Self {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

        Self {
            message: message.into(),
            level,
            created_at: Instant::now(),
            duration: Duration::from_secs(3),
            dismissible: true,
            id: COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
        }
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self::new(message, ToastLevel::Info)
    }

    pub fn success(message: impl Into<String>) -> Self {
        Self::new(message, ToastLevel::Success)
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(message, ToastLevel::Warning)
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new(message, ToastLevel::Error)
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn persistent(mut self) -> Self {
        self.duration = Duration::from_secs(u64::MAX);
        self.dismissible = false;
        self
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() >= self.duration
    }

    pub fn remaining_time(&self) -> Duration {
        self.duration.saturating_sub(self.created_at.elapsed())
    }

    pub fn progress(&self) -> f32 {
        let elapsed = self.created_at.elapsed().as_secs_f32();
        let total = self.duration.as_secs_f32();
        1.0 - (elapsed / total).clamp(0.0, 1.0)
    }

    pub fn id(&self) -> u64 {
        self.id
    }
}

#[derive(Debug, Default)]
pub struct ToastManager {
    toasts: Vec<Toast>,
    max_visible: usize,
}

impl ToastManager {
    pub fn new() -> Self {
        Self {
            toasts: Vec::new(),
            max_visible: 5,
        }
    }

    pub fn with_max_visible(mut self, max: usize) -> Self {
        self.max_visible = max;
        self
    }

    pub fn push(&mut self, toast: Toast) {
        self.toasts.push(toast);
    }

    pub fn info(&mut self, message: impl Into<String>) {
        self.push(Toast::info(message));
    }

    pub fn success(&mut self, message: impl Into<String>) {
        self.push(Toast::success(message));
    }

    pub fn warning(&mut self, message: impl Into<String>) {
        self.push(Toast::warning(message));
    }

    pub fn error(&mut self, message: impl Into<String>) {
        self.push(Toast::error(message));
    }

    pub fn dismiss(&mut self, id: u64) {
        self.toasts.retain(|t| t.id() != id);
    }

    pub fn clear(&mut self) {
        self.toasts.clear();
    }

    pub fn cleanup(&mut self) {
        self.toasts.retain(|t| !t.is_expired());
    }

    pub fn visible_toasts(&self) -> impl Iterator<Item = &Toast> {
        self.toasts.iter().rev().take(self.max_visible)
    }

    pub fn count(&self) -> usize {
        self.toasts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.toasts.is_empty()
    }

    pub fn render(&self, frame: &mut Frame, screen_area: Rect, theme: &dyn Theme) {
        let toasts: Vec<_> = self.visible_toasts().collect();
        if toasts.is_empty() {
            return;
        }

        let toast_width = 40u16.min(screen_area.width.saturating_sub(4));
        let toast_height = 3u16;
        let margin = 2u16;

        let start_x = screen_area.width.saturating_sub(toast_width + margin);
        let mut start_y = margin + 3;

        for toast in toasts {
            if start_y + toast_height > screen_area.height {
                break;
            }

            let area = Rect::new(start_x, start_y, toast_width, toast_height);
            Self::render_toast(frame, area, toast, theme);
            start_y += toast_height + 1;
        }
    }

    fn render_toast(frame: &mut Frame, area: Rect, toast: &Toast, theme: &dyn Theme) {
        frame.render_widget(Clear, area);

        let (border_color, icon_color) = match toast.level {
            ToastLevel::Info => (theme.info(), theme.info()),
            ToastLevel::Success => (theme.success(), theme.success()),
            ToastLevel::Warning => (theme.warning(), theme.warning()),
            ToastLevel::Error => (theme.error(), theme.error()),
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(theme.surface()));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let progress = toast.progress();
        let progress_width = ((inner.width as f32) * progress) as u16;

        if progress_width > 0 {
            let progress_area = Rect::new(
                inner.x,
                inner.y + inner.height.saturating_sub(1),
                progress_width,
                1,
            );
            let progress_line = Paragraph::new(Line::from(Span::styled(
                "─".repeat(progress_width as usize),
                Style::default().fg(icon_color),
            )));
            frame.render_widget(progress_line, progress_area);
        }

        let msg_area = Rect::new(
            inner.x,
            inner.y,
            inner.width,
            inner.height.saturating_sub(1),
        );

        let truncated_msg = if toast.message.len() > (inner.width as usize).saturating_sub(4) {
            format!(
                "{}...",
                &toast.message[..(inner.width as usize).saturating_sub(7)]
            )
        } else {
            toast.message.clone()
        };

        let content = Line::from(vec![
            Span::styled(
                format!("{} ", toast.level.icon()),
                Style::default().fg(icon_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(truncated_msg, Style::default().fg(theme.foreground())),
        ]);

        let paragraph = Paragraph::new(content);
        frame.render_widget(paragraph, msg_area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toast_levels() {
        assert_eq!(ToastLevel::Info.icon(), "ℹ");
        assert_eq!(ToastLevel::Success.icon(), "✓");
        assert_eq!(ToastLevel::Warning.icon(), "⚠");
        assert_eq!(ToastLevel::Error.icon(), "✗");
    }

    #[test]
    fn test_toast_creation() {
        let toast = Toast::info("Test message");
        assert_eq!(toast.message, "Test message");
        assert_eq!(toast.level, ToastLevel::Info);
        assert!(!toast.is_expired());
    }

    #[test]
    fn test_toast_variants() {
        let info = Toast::info("info");
        assert_eq!(info.level, ToastLevel::Info);

        let success = Toast::success("success");
        assert_eq!(success.level, ToastLevel::Success);

        let warning = Toast::warning("warning");
        assert_eq!(warning.level, ToastLevel::Warning);

        let error = Toast::error("error");
        assert_eq!(error.level, ToastLevel::Error);
    }

    #[test]
    fn test_toast_duration() {
        let toast = Toast::info("test").with_duration(Duration::from_millis(100));
        assert_eq!(toast.duration, Duration::from_millis(100));
    }

    #[test]
    fn test_toast_persistent() {
        let toast = Toast::info("test").persistent();
        assert!(!toast.dismissible);
    }

    #[test]
    fn test_toast_progress() {
        let toast = Toast::info("test").with_duration(Duration::from_secs(10));
        let progress = toast.progress();
        assert!(progress > 0.9);
        assert!(progress <= 1.0);
    }

    #[test]
    fn test_toast_manager() {
        let mut manager = ToastManager::new();
        assert!(manager.is_empty());

        manager.info("info");
        manager.success("success");
        manager.warning("warning");
        manager.error("error");

        assert_eq!(manager.count(), 4);
        assert!(!manager.is_empty());
    }

    #[test]
    fn test_toast_manager_cleanup() {
        let mut manager = ToastManager::new();
        manager.push(Toast::info("test").with_duration(Duration::from_millis(1)));

        std::thread::sleep(Duration::from_millis(10));
        manager.cleanup();

        assert!(manager.is_empty());
    }

    #[test]
    fn test_toast_manager_dismiss() {
        let mut manager = ToastManager::new();
        let toast = Toast::info("test");
        let id = toast.id();
        manager.push(toast);

        assert_eq!(manager.count(), 1);
        manager.dismiss(id);
        assert!(manager.is_empty());
    }

    #[test]
    fn test_toast_manager_max_visible() {
        let manager = ToastManager::new().with_max_visible(3);
        assert_eq!(manager.max_visible, 3);
    }

    #[test]
    fn test_unique_toast_ids() {
        let toast1 = Toast::info("test1");
        let toast2 = Toast::info("test2");
        assert_ne!(toast1.id(), toast2.id());
    }
}
