use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalSize {
    Small,
    Medium,
    Large,
    Custom {
        width_percent: u16,
        height_percent: u16,
    },
}

impl ModalSize {
    pub fn dimensions(&self) -> (u16, u16) {
        match self {
            ModalSize::Small => (40, 30),
            ModalSize::Medium => (60, 50),
            ModalSize::Large => (80, 70),
            ModalSize::Custom {
                width_percent,
                height_percent,
            } => (*width_percent, *height_percent),
        }
    }
}

pub struct Modal {
    title: String,
    content: Vec<Line<'static>>,
    size: ModalSize,
    footer_hints: Vec<(String, String)>,
    scrollable: bool,
    scroll_offset: usize,
}

impl Modal {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            content: Vec::new(),
            size: ModalSize::Medium,
            footer_hints: Vec::new(),
            scrollable: false,
            scroll_offset: 0,
        }
    }

    pub fn with_size(mut self, size: ModalSize) -> Self {
        self.size = size;
        self
    }

    pub fn with_content(mut self, content: Vec<Line<'static>>) -> Self {
        self.content = content;
        self
    }

    pub fn with_paragraph(mut self, text: impl Into<String>) -> Self {
        let text: String = text.into();
        self.content = text
            .lines()
            .map(|line| Line::from(line.to_string()))
            .collect();
        self
    }

    pub fn add_line(mut self, line: Line<'static>) -> Self {
        self.content.push(line);
        self
    }

    pub fn add_text(mut self, text: impl Into<String>) -> Self {
        self.content.push(Line::from(text.into()));
        self
    }

    pub fn add_keybind(
        mut self,
        key: impl Into<String>,
        desc: impl Into<String>,
        theme: &dyn Theme,
    ) -> Self {
        let line = Line::from(vec![
            Span::styled(
                format!("  {:<15}", key.into()),
                Style::default()
                    .fg(theme.accent())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(desc.into(), Style::default().fg(theme.foreground())),
        ]);
        self.content.push(line);
        self
    }

    pub fn with_footer_hint(mut self, key: impl Into<String>, desc: impl Into<String>) -> Self {
        self.footer_hints.push((key.into(), desc.into()));
        self
    }

    pub fn scrollable(mut self, scrollable: bool) -> Self {
        self.scrollable = scrollable;
        self
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn scroll_down(&mut self, max_visible: usize) {
        if self.content.len() > max_visible {
            let max_offset = self.content.len().saturating_sub(max_visible);
            self.scroll_offset = (self.scroll_offset + 1).min(max_offset);
        }
    }

    pub fn calculate_area(&self, screen: Rect) -> Rect {
        let (width_percent, height_percent) = self.size.dimensions();

        let modal_width = (screen.width as u32 * width_percent as u32 / 100) as u16;
        let modal_height = (screen.height as u32 * height_percent as u32 / 100) as u16;

        let modal_width = modal_width.max(20).min(screen.width.saturating_sub(4));
        let modal_height = modal_height.max(10).min(screen.height.saturating_sub(4));

        let x = (screen.width.saturating_sub(modal_width)) / 2;
        let y = (screen.height.saturating_sub(modal_height)) / 2;

        Rect::new(x, y, modal_width, modal_height)
    }

    pub fn render(&self, frame: &mut Frame, screen: Rect, theme: &dyn Theme) {
        let area = self.calculate_area(screen);

        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(format!(" {} ", self.title))
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(theme.accent())
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().bg(theme.surface()));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let footer_height = if self.footer_hints.is_empty() { 0 } else { 2 };
        let content_height = inner.height.saturating_sub(footer_height);

        let content_area = Rect::new(inner.x, inner.y, inner.width, content_height);
        let footer_area = if footer_height > 0 {
            Some(Rect::new(
                inner.x,
                inner.y + content_height,
                inner.width,
                footer_height,
            ))
        } else {
            None
        };

        let visible_lines = if self.scrollable {
            let end = (self.scroll_offset + content_height as usize).min(self.content.len());
            self.content[self.scroll_offset..end].to_vec()
        } else {
            self.content.clone()
        };

        let content = Paragraph::new(visible_lines)
            .style(Style::default().bg(theme.surface()))
            .wrap(Wrap { trim: false });

        frame.render_widget(content, content_area);

        if self.scrollable && self.content.len() > content_height as usize {
            let scroll_indicator = format!(
                " {}/{} ",
                self.scroll_offset + 1,
                self.content.len().saturating_sub(content_height as usize) + 1
            );
            let indicator_para = Paragraph::new(Line::from(Span::styled(
                scroll_indicator,
                Style::default().fg(theme.foreground_dim()),
            )))
            .alignment(Alignment::Right);

            let indicator_area = Rect::new(
                content_area.x,
                content_area.y + content_area.height.saturating_sub(1),
                content_area.width,
                1,
            );
            frame.render_widget(indicator_para, indicator_area);
        }

        if let Some(footer_area) = footer_area {
            let hint_spans: Vec<Span> = self
                .footer_hints
                .iter()
                .flat_map(|(key, desc)| {
                    vec![
                        Span::styled(
                            format!(" {} ", key),
                            Style::default()
                                .fg(theme.accent())
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!("{} ", desc),
                            Style::default().fg(theme.foreground_dim()),
                        ),
                    ]
                })
                .collect();

            let separator = Paragraph::new(Line::from(Span::styled(
                "─".repeat(footer_area.width as usize),
                Style::default().fg(theme.border()),
            )));
            frame.render_widget(
                separator,
                Rect::new(footer_area.x, footer_area.y, footer_area.width, 1),
            );

            let hints = Paragraph::new(Line::from(hint_spans)).alignment(Alignment::Center);
            frame.render_widget(
                hints,
                Rect::new(footer_area.x, footer_area.y + 1, footer_area.width, 1),
            );
        }
    }
}

impl Default for Modal {
    fn default() -> Self {
        Self::new("Modal")
    }
}

pub struct HelpModal;

impl HelpModal {
    pub fn create(theme: &dyn Theme) -> Modal {
        Modal::new("Keyboard Shortcuts")
            .with_size(ModalSize::Large)
            .add_text("")
            .add_line(Line::from(Span::styled(
                "  Navigation",
                Style::default()
                    .fg(theme.accent_secondary())
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )))
            .add_text("")
            .add_keybind("q / Esc", "Quit application", theme)
            .add_keybind("Tab", "Next view", theme)
            .add_keybind("Shift+Tab", "Previous view", theme)
            .add_keybind("j / ↓", "Move down in list", theme)
            .add_keybind("k / ↑", "Move up in list", theme)
            .add_keybind("g", "Go to top of list", theme)
            .add_keybind("G", "Go to bottom of list", theme)
            .add_keybind("Ctrl+d", "Page down", theme)
            .add_keybind("Ctrl+u", "Page up", theme)
            .add_keybind("Enter", "Select / drill-down", theme)
            .add_text("")
            .add_line(Line::from(Span::styled(
                "  Actions",
                Style::default()
                    .fg(theme.accent_secondary())
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )))
            .add_text("")
            .add_keybind("/", "Start search", theme)
            .add_keybind(":", "Open command palette", theme)
            .add_keybind("r", "Refresh data", theme)
            .add_keybind("t", "Toggle theme", theme)
            .add_keybind("?", "Show/hide help", theme)
            .add_text("")
            .add_line(Line::from(Span::styled(
                "  Views",
                Style::default()
                    .fg(theme.accent_secondary())
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )))
            .add_text("")
            .add_keybind("1", "Dashboard", theme)
            .add_keybind("2", "Agents", theme)
            .add_keybind("3", "Sessions", theme)
            .add_keybind("4", "Costs", theme)
            .add_keybind("5", "Metrics", theme)
            .add_keybind("6", "Skills", theme)
            .add_keybind("7", "Plugins", theme)
            .add_keybind("8", "Hooks", theme)
            .add_text("")
            .add_line(Line::from(Span::styled(
                "  View-Specific",
                Style::default()
                    .fg(theme.accent_secondary())
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )))
            .add_text("")
            .add_keybind("[/]", "Switch tabs (Skills/Plugins/Hooks)", theme)
            .add_keybind("h", "Toggle historical data (Metrics)", theme)
            .add_keybind("+/-", "Adjust refresh rate (Metrics)", theme)
            .add_keybind("i", "Install (Skills/Plugins)", theme)
            .add_keybind("e", "Enable/disable (Plugins/Hooks)", theme)
            .add_keybind("c", "Configure (Plugins)", theme)
            .scrollable(true)
            .with_footer_hint("Esc", "Close")
            .with_footer_hint("j/k", "Scroll")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modal_size_dimensions() {
        assert_eq!(ModalSize::Small.dimensions(), (40, 30));
        assert_eq!(ModalSize::Medium.dimensions(), (60, 50));
        assert_eq!(ModalSize::Large.dimensions(), (80, 70));
        assert_eq!(
            ModalSize::Custom {
                width_percent: 90,
                height_percent: 85
            }
            .dimensions(),
            (90, 85)
        );
    }

    #[test]
    fn test_modal_creation() {
        let modal = Modal::new("Test");
        assert_eq!(modal.title, "Test");
        assert!(modal.content.is_empty());
        assert!(modal.footer_hints.is_empty());
    }

    #[test]
    fn test_modal_builder() {
        let modal = Modal::new("Test")
            .with_size(ModalSize::Large)
            .add_text("Line 1")
            .add_text("Line 2")
            .with_footer_hint("Esc", "Close")
            .scrollable(true);

        assert_eq!(modal.size, ModalSize::Large);
        assert_eq!(modal.content.len(), 2);
        assert_eq!(modal.footer_hints.len(), 1);
        assert!(modal.scrollable);
    }

    #[test]
    fn test_modal_scroll() {
        let mut modal = Modal::new("Test")
            .add_text("Line 1")
            .add_text("Line 2")
            .add_text("Line 3")
            .add_text("Line 4")
            .add_text("Line 5")
            .scrollable(true);

        assert_eq!(modal.scroll_offset, 0);

        modal.scroll_down(2);
        assert_eq!(modal.scroll_offset, 1);

        modal.scroll_up();
        assert_eq!(modal.scroll_offset, 0);

        modal.scroll_up();
        assert_eq!(modal.scroll_offset, 0);
    }

    #[test]
    fn test_modal_calculate_area() {
        let modal = Modal::new("Test").with_size(ModalSize::Medium);
        let screen = Rect::new(0, 0, 100, 50);

        let area = modal.calculate_area(screen);

        assert!(area.width > 0);
        assert!(area.height > 0);
        assert!(area.x > 0);
        assert!(area.y > 0);
        assert!(area.x + area.width <= screen.width);
        assert!(area.y + area.height <= screen.height);
    }

    #[test]
    fn test_modal_with_paragraph() {
        let modal = Modal::new("Test").with_paragraph("Line 1\nLine 2\nLine 3");
        assert_eq!(modal.content.len(), 3);
    }
}
