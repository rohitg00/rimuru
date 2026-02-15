use ratatui::style::{Modifier, Style};

use super::Theme;

pub struct ThemedStyles<'a> {
    theme: &'a dyn Theme,
}

impl<'a> ThemedStyles<'a> {
    pub fn new(theme: &'a dyn Theme) -> Self {
        Self { theme }
    }

    pub fn base(&self) -> Style {
        Style::default()
            .bg(self.theme.background())
            .fg(self.theme.foreground())
    }

    pub fn surface(&self) -> Style {
        Style::default()
            .bg(self.theme.surface())
            .fg(self.theme.foreground())
    }

    pub fn header(&self) -> Style {
        Style::default()
            .fg(self.theme.accent())
            .add_modifier(Modifier::BOLD)
    }

    pub fn header_secondary(&self) -> Style {
        Style::default()
            .fg(self.theme.accent_secondary())
            .add_modifier(Modifier::BOLD)
    }

    pub fn border(&self) -> Style {
        Style::default().fg(self.theme.border())
    }

    pub fn border_focused(&self) -> Style {
        Style::default()
            .fg(self.theme.accent())
            .add_modifier(Modifier::BOLD)
    }

    pub fn selection(&self) -> Style {
        Style::default()
            .bg(self.theme.selection())
            .fg(self.theme.foreground())
    }

    pub fn selection_active(&self) -> Style {
        Style::default()
            .bg(self.theme.accent())
            .fg(self.theme.background())
            .add_modifier(Modifier::BOLD)
    }

    pub fn dimmed(&self) -> Style {
        Style::default().fg(self.theme.foreground_dim())
    }

    pub fn accent(&self) -> Style {
        Style::default().fg(self.theme.accent())
    }

    pub fn accent_bold(&self) -> Style {
        Style::default()
            .fg(self.theme.accent())
            .add_modifier(Modifier::BOLD)
    }

    pub fn success(&self) -> Style {
        Style::default().fg(self.theme.success())
    }

    pub fn success_bold(&self) -> Style {
        Style::default()
            .fg(self.theme.success())
            .add_modifier(Modifier::BOLD)
    }

    pub fn warning(&self) -> Style {
        Style::default().fg(self.theme.warning())
    }

    pub fn warning_bold(&self) -> Style {
        Style::default()
            .fg(self.theme.warning())
            .add_modifier(Modifier::BOLD)
    }

    pub fn error(&self) -> Style {
        Style::default().fg(self.theme.error())
    }

    pub fn error_bold(&self) -> Style {
        Style::default()
            .fg(self.theme.error())
            .add_modifier(Modifier::BOLD)
    }

    pub fn info(&self) -> Style {
        Style::default().fg(self.theme.info())
    }

    pub fn info_bold(&self) -> Style {
        Style::default()
            .fg(self.theme.info())
            .add_modifier(Modifier::BOLD)
    }

    pub fn keybind(&self) -> Style {
        Style::default()
            .fg(self.theme.accent())
            .add_modifier(Modifier::BOLD)
    }

    pub fn keybind_description(&self) -> Style {
        Style::default().fg(self.theme.foreground_dim())
    }

    pub fn link(&self) -> Style {
        Style::default()
            .fg(self.theme.info())
            .add_modifier(Modifier::UNDERLINED)
    }

    pub fn highlight(&self) -> Style {
        Style::default()
            .bg(self.theme.accent())
            .fg(self.theme.background())
            .add_modifier(Modifier::BOLD)
    }

    pub fn search_match(&self) -> Style {
        Style::default()
            .bg(self.theme.warning())
            .fg(self.theme.background())
            .add_modifier(Modifier::BOLD)
    }

    pub fn table_header(&self) -> Style {
        Style::default()
            .fg(self.theme.accent())
            .add_modifier(Modifier::BOLD)
    }

    pub fn table_row(&self) -> Style {
        Style::default().fg(self.theme.foreground())
    }

    pub fn table_row_alt(&self) -> Style {
        Style::default()
            .fg(self.theme.foreground())
            .bg(self.theme.surface())
    }

    pub fn table_row_selected(&self) -> Style {
        Style::default()
            .bg(self.theme.selection())
            .fg(self.theme.foreground())
    }

    pub fn status_active(&self) -> Style {
        Style::default()
            .fg(self.theme.success())
            .add_modifier(Modifier::BOLD)
    }

    pub fn status_inactive(&self) -> Style {
        Style::default().fg(self.theme.foreground_dim())
    }

    pub fn status_error(&self) -> Style {
        Style::default()
            .fg(self.theme.error())
            .add_modifier(Modifier::BOLD)
    }

    pub fn status_warning(&self) -> Style {
        Style::default()
            .fg(self.theme.warning())
            .add_modifier(Modifier::BOLD)
    }

    pub fn gauge_low(&self) -> Style {
        Style::default().fg(self.theme.success())
    }

    pub fn gauge_medium(&self) -> Style {
        Style::default().fg(self.theme.warning())
    }

    pub fn gauge_high(&self) -> Style {
        Style::default().fg(self.theme.error())
    }

    pub fn gauge_for_percent(&self, percent: f32) -> Style {
        if percent < 50.0 {
            self.gauge_low()
        } else if percent < 80.0 {
            self.gauge_medium()
        } else {
            self.gauge_high()
        }
    }

    pub fn tab_active(&self) -> Style {
        Style::default()
            .fg(self.theme.accent())
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    }

    pub fn tab_inactive(&self) -> Style {
        Style::default().fg(self.theme.foreground_dim())
    }

    pub fn button(&self) -> Style {
        Style::default()
            .fg(self.theme.foreground())
            .bg(self.theme.surface())
    }

    pub fn button_focused(&self) -> Style {
        Style::default()
            .fg(self.theme.background())
            .bg(self.theme.accent())
            .add_modifier(Modifier::BOLD)
    }

    pub fn button_disabled(&self) -> Style {
        Style::default()
            .fg(self.theme.foreground_dim())
            .bg(self.theme.surface())
    }

    pub fn input(&self) -> Style {
        Style::default()
            .fg(self.theme.foreground())
            .bg(self.theme.surface())
    }

    pub fn input_focused(&self) -> Style {
        Style::default()
            .fg(self.theme.foreground())
            .bg(self.theme.surface())
    }

    pub fn input_cursor(&self) -> Style {
        Style::default()
            .fg(self.theme.background())
            .bg(self.theme.accent())
    }

    pub fn input_placeholder(&self) -> Style {
        Style::default().fg(self.theme.foreground_dim())
    }

    pub fn scrollbar(&self) -> Style {
        Style::default().fg(self.theme.foreground_dim())
    }

    pub fn scrollbar_thumb(&self) -> Style {
        Style::default().fg(self.theme.accent())
    }

    pub fn tooltip(&self) -> Style {
        Style::default()
            .fg(self.theme.foreground())
            .bg(self.theme.surface())
    }

    pub fn badge(&self) -> Style {
        Style::default()
            .fg(self.theme.background())
            .bg(self.theme.accent())
    }

    pub fn badge_success(&self) -> Style {
        Style::default()
            .fg(self.theme.background())
            .bg(self.theme.success())
    }

    pub fn badge_warning(&self) -> Style {
        Style::default()
            .fg(self.theme.background())
            .bg(self.theme.warning())
    }

    pub fn badge_error(&self) -> Style {
        Style::default()
            .fg(self.theme.background())
            .bg(self.theme.error())
    }

    pub fn code(&self) -> Style {
        Style::default()
            .fg(self.theme.accent_secondary())
            .bg(self.theme.surface())
    }

    pub fn code_keyword(&self) -> Style {
        Style::default()
            .fg(self.theme.accent())
            .add_modifier(Modifier::BOLD)
    }

    pub fn code_string(&self) -> Style {
        Style::default().fg(self.theme.success())
    }

    pub fn code_number(&self) -> Style {
        Style::default().fg(self.theme.warning())
    }

    pub fn code_comment(&self) -> Style {
        Style::default()
            .fg(self.theme.foreground_dim())
            .add_modifier(Modifier::ITALIC)
    }
}

pub fn highlight_matches(
    text: &str,
    query: &str,
    base_style: Style,
    highlight_style: Style,
) -> Vec<(String, Style)> {
    if query.is_empty() {
        return vec![(text.to_string(), base_style)];
    }

    let query_lower = query.to_lowercase();
    let text_lower = text.to_lowercase();

    let mut result = Vec::new();
    let mut last_end = 0;

    for (start, _) in text_lower.match_indices(&query_lower) {
        if start > last_end {
            result.push((text[last_end..start].to_string(), base_style));
        }
        result.push((
            text[start..start + query.len()].to_string(),
            highlight_style,
        ));
        last_end = start + query.len();
    }

    if last_end < text.len() {
        result.push((text[last_end..].to_string(), base_style));
    }

    if result.is_empty() {
        result.push((text.to_string(), base_style));
    }

    result
}

pub fn create_status_style(theme: &dyn Theme, status: &str) -> Style {
    match status.to_lowercase().as_str() {
        "active" | "running" | "connected" | "online" | "enabled" => Style::default()
            .fg(theme.success())
            .add_modifier(Modifier::BOLD),
        "completed" | "finished" | "done" | "success" => Style::default().fg(theme.success()),
        "pending" | "queued" | "waiting" | "idle" => Style::default().fg(theme.warning()),
        "failed" | "error" | "disconnected" | "offline" | "disabled" => Style::default()
            .fg(theme.error())
            .add_modifier(Modifier::BOLD),
        "warning" | "degraded" | "partial" => Style::default().fg(theme.warning()),
        _ => Style::default().fg(theme.foreground_dim()),
    }
}

pub fn format_with_icon(status: &str) -> (&'static str, &str) {
    match status.to_lowercase().as_str() {
        "active" | "running" | "connected" | "online" | "enabled" => ("●", status),
        "completed" | "finished" | "done" | "success" => ("✓", status),
        "pending" | "queued" | "waiting" => ("◐", status),
        "idle" => ("◯", status),
        "failed" | "error" => ("✗", status),
        "disconnected" | "offline" | "disabled" => ("○", status),
        "warning" | "degraded" | "partial" => ("⚠", status),
        _ => ("·", status),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::TokyoNight;

    #[test]
    fn test_themed_styles_creation() {
        let theme = TokyoNight;
        let styles = ThemedStyles::new(&theme);

        let base = styles.base();
        assert!(base.bg.is_some());
        assert!(base.fg.is_some());
    }

    #[test]
    fn test_gauge_for_percent() {
        let theme = TokyoNight;
        let styles = ThemedStyles::new(&theme);

        let low = styles.gauge_for_percent(25.0);
        let medium = styles.gauge_for_percent(65.0);
        let high = styles.gauge_for_percent(90.0);

        assert_eq!(low.fg, styles.gauge_low().fg);
        assert_eq!(medium.fg, styles.gauge_medium().fg);
        assert_eq!(high.fg, styles.gauge_high().fg);
    }

    #[test]
    fn test_highlight_matches() {
        let base = Style::default();
        let highlight = Style::default().add_modifier(Modifier::BOLD);

        let result = highlight_matches("Hello World", "world", base, highlight);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "Hello ");
        assert_eq!(result[1].0, "World");

        let result = highlight_matches("Test test TEST", "test", base, highlight);
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_highlight_matches_empty_query() {
        let base = Style::default();
        let highlight = Style::default().add_modifier(Modifier::BOLD);

        let result = highlight_matches("Hello World", "", base, highlight);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "Hello World");
    }

    #[test]
    fn test_highlight_matches_no_match() {
        let base = Style::default();
        let highlight = Style::default().add_modifier(Modifier::BOLD);

        let result = highlight_matches("Hello World", "xyz", base, highlight);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "Hello World");
    }

    #[test]
    fn test_create_status_style() {
        let theme = TokyoNight;

        let active = create_status_style(&theme, "active");
        assert_eq!(active.fg, Some(theme.success()));

        let failed = create_status_style(&theme, "failed");
        assert_eq!(failed.fg, Some(theme.error()));

        let pending = create_status_style(&theme, "pending");
        assert_eq!(pending.fg, Some(theme.warning()));
    }

    #[test]
    fn test_format_with_icon() {
        assert_eq!(format_with_icon("active").0, "●");
        assert_eq!(format_with_icon("completed").0, "✓");
        assert_eq!(format_with_icon("failed").0, "✗");
        assert_eq!(format_with_icon("pending").0, "◐");
        assert_eq!(format_with_icon("idle").0, "◯");
        assert_eq!(format_with_icon("warning").0, "⚠");
    }
}
