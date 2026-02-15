use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogButton {
    Confirm,
    Cancel,
}

impl DialogButton {
    pub fn label(&self) -> &'static str {
        match self {
            DialogButton::Confirm => "Confirm",
            DialogButton::Cancel => "Cancel",
        }
    }

    pub fn other(&self) -> Self {
        match self {
            DialogButton::Confirm => DialogButton::Cancel,
            DialogButton::Cancel => DialogButton::Confirm,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogType {
    Confirmation,
    Warning,
    Danger,
    Info,
}

impl DialogType {
    pub fn icon(&self) -> &'static str {
        match self {
            DialogType::Confirmation => "?",
            DialogType::Warning => "⚠",
            DialogType::Danger => "⚠",
            DialogType::Info => "ℹ",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfirmDialog {
    title: String,
    message: String,
    dialog_type: DialogType,
    selected_button: DialogButton,
    confirm_label: String,
    cancel_label: String,
    is_destructive: bool,
}

impl ConfirmDialog {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            dialog_type: DialogType::Confirmation,
            selected_button: DialogButton::Cancel,
            confirm_label: "Confirm".to_string(),
            cancel_label: "Cancel".to_string(),
            is_destructive: false,
        }
    }

    pub fn confirmation(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title, message).with_type(DialogType::Confirmation)
    }

    pub fn warning(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title, message).with_type(DialogType::Warning)
    }

    pub fn danger(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title, message)
            .with_type(DialogType::Danger)
            .destructive()
    }

    pub fn with_type(mut self, dialog_type: DialogType) -> Self {
        self.dialog_type = dialog_type;
        self
    }

    pub fn with_confirm_label(mut self, label: impl Into<String>) -> Self {
        self.confirm_label = label.into();
        self
    }

    pub fn with_cancel_label(mut self, label: impl Into<String>) -> Self {
        self.cancel_label = label.into();
        self
    }

    pub fn destructive(mut self) -> Self {
        self.is_destructive = true;
        self.selected_button = DialogButton::Cancel;
        self
    }

    pub fn select_next(&mut self) {
        self.selected_button = self.selected_button.other();
    }

    pub fn select_prev(&mut self) {
        self.selected_button = self.selected_button.other();
    }

    pub fn select_confirm(&mut self) {
        self.selected_button = DialogButton::Confirm;
    }

    pub fn select_cancel(&mut self) {
        self.selected_button = DialogButton::Cancel;
    }

    pub fn selected(&self) -> DialogButton {
        self.selected_button
    }

    pub fn is_confirm_selected(&self) -> bool {
        self.selected_button == DialogButton::Confirm
    }

    pub fn calculate_area(&self, screen: Rect) -> Rect {
        let width = 50u16.min(screen.width.saturating_sub(4));
        let height = 9u16.min(screen.height.saturating_sub(4));

        let x = (screen.width.saturating_sub(width)) / 2;
        let y = (screen.height.saturating_sub(height)) / 2;

        Rect::new(x, y, width, height)
    }

    pub fn render(&self, frame: &mut Frame, screen: Rect, theme: &dyn Theme) {
        let area = self.calculate_area(screen);

        frame.render_widget(Clear, area);

        let border_color = match self.dialog_type {
            DialogType::Confirmation => theme.accent(),
            DialogType::Warning => theme.warning(),
            DialogType::Danger => theme.error(),
            DialogType::Info => theme.info(),
        };

        let block = Block::default()
            .title(format!(" {} {} ", self.dialog_type.icon(), self.title))
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(border_color)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().bg(theme.surface()));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(inner);

        let message = Paragraph::new(Line::from(Span::styled(
            self.message.clone(),
            Style::default().fg(theme.foreground()),
        )))
        .alignment(Alignment::Center);
        frame.render_widget(message, chunks[0]);

        let separator = Paragraph::new(Line::from(Span::styled(
            "─".repeat(chunks[1].width as usize),
            Style::default().fg(theme.border()),
        )));
        frame.render_widget(separator, chunks[1]);

        let button_area = chunks[2];
        let button_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(button_area);

        let cancel_style = if self.selected_button == DialogButton::Cancel {
            Style::default()
                .fg(theme.background())
                .bg(theme.foreground())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.foreground_dim())
        };

        let confirm_style = if self.selected_button == DialogButton::Confirm {
            if self.is_destructive {
                Style::default()
                    .fg(theme.background())
                    .bg(theme.error())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(theme.background())
                    .bg(theme.accent())
                    .add_modifier(Modifier::BOLD)
            }
        } else if self.is_destructive {
            Style::default().fg(theme.error())
        } else {
            Style::default().fg(theme.accent())
        };

        let cancel_btn = Paragraph::new(Line::from(Span::styled(
            format!(" {} ", self.cancel_label),
            cancel_style,
        )))
        .alignment(Alignment::Center);

        let confirm_btn = Paragraph::new(Line::from(Span::styled(
            format!(" {} ", self.confirm_label),
            confirm_style,
        )))
        .alignment(Alignment::Center);

        frame.render_widget(cancel_btn, button_chunks[0]);
        frame.render_widget(confirm_btn, button_chunks[1]);
    }
}

impl Default for ConfirmDialog {
    fn default() -> Self {
        Self::new("Confirm", "Are you sure?")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogResult {
    Pending,
    Confirmed,
    Cancelled,
}

pub struct DialogState {
    pub dialog: Option<ConfirmDialog>,
    pub result: DialogResult,
}

impl DialogState {
    pub fn new() -> Self {
        Self {
            dialog: None,
            result: DialogResult::Pending,
        }
    }

    pub fn show(&mut self, dialog: ConfirmDialog) {
        self.dialog = Some(dialog);
        self.result = DialogResult::Pending;
    }

    pub fn close(&mut self) {
        self.dialog = None;
        self.result = DialogResult::Pending;
    }

    pub fn confirm(&mut self) {
        if self.dialog.is_some() {
            self.result = DialogResult::Confirmed;
            self.dialog = None;
        }
    }

    pub fn cancel(&mut self) {
        if self.dialog.is_some() {
            self.result = DialogResult::Cancelled;
            self.dialog = None;
        }
    }

    pub fn is_open(&self) -> bool {
        self.dialog.is_some()
    }

    pub fn take_result(&mut self) -> DialogResult {
        let result = self.result;
        self.result = DialogResult::Pending;
        result
    }

    pub fn select_next(&mut self) {
        if let Some(ref mut dialog) = self.dialog {
            dialog.select_next();
        }
    }

    pub fn select_prev(&mut self) {
        if let Some(ref mut dialog) = self.dialog {
            dialog.select_prev();
        }
    }

    pub fn execute_selected(&mut self) {
        if let Some(ref dialog) = self.dialog {
            match dialog.selected() {
                DialogButton::Confirm => self.confirm(),
                DialogButton::Cancel => self.cancel(),
            }
        }
    }
}

impl Default for DialogState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialog_button_other() {
        assert_eq!(DialogButton::Confirm.other(), DialogButton::Cancel);
        assert_eq!(DialogButton::Cancel.other(), DialogButton::Confirm);
    }

    #[test]
    fn test_dialog_type_icons() {
        assert_eq!(DialogType::Confirmation.icon(), "?");
        assert_eq!(DialogType::Warning.icon(), "⚠");
        assert_eq!(DialogType::Danger.icon(), "⚠");
        assert_eq!(DialogType::Info.icon(), "ℹ");
    }

    #[test]
    fn test_confirm_dialog_creation() {
        let dialog = ConfirmDialog::new("Test", "Test message");
        assert_eq!(dialog.title, "Test");
        assert_eq!(dialog.message, "Test message");
        assert_eq!(dialog.dialog_type, DialogType::Confirmation);
        assert_eq!(dialog.selected_button, DialogButton::Cancel);
    }

    #[test]
    fn test_confirm_dialog_variants() {
        let confirm = ConfirmDialog::confirmation("Confirm", "message");
        assert_eq!(confirm.dialog_type, DialogType::Confirmation);

        let warning = ConfirmDialog::warning("Warning", "message");
        assert_eq!(warning.dialog_type, DialogType::Warning);

        let danger = ConfirmDialog::danger("Danger", "message");
        assert_eq!(danger.dialog_type, DialogType::Danger);
        assert!(danger.is_destructive);
    }

    #[test]
    fn test_confirm_dialog_labels() {
        let dialog = ConfirmDialog::new("Test", "message")
            .with_confirm_label("Yes, delete")
            .with_cancel_label("No, keep");

        assert_eq!(dialog.confirm_label, "Yes, delete");
        assert_eq!(dialog.cancel_label, "No, keep");
    }

    #[test]
    fn test_confirm_dialog_selection() {
        let mut dialog = ConfirmDialog::new("Test", "message");

        assert!(!dialog.is_confirm_selected());

        dialog.select_confirm();
        assert!(dialog.is_confirm_selected());

        dialog.select_cancel();
        assert!(!dialog.is_confirm_selected());

        dialog.select_next();
        assert!(dialog.is_confirm_selected());

        dialog.select_prev();
        assert!(!dialog.is_confirm_selected());
    }

    #[test]
    fn test_dialog_state() {
        let mut state = DialogState::new();
        assert!(!state.is_open());

        state.show(ConfirmDialog::new("Test", "message"));
        assert!(state.is_open());
        assert_eq!(state.result, DialogResult::Pending);

        state.confirm();
        assert!(!state.is_open());

        let result = state.take_result();
        assert_eq!(result, DialogResult::Confirmed);
        assert_eq!(state.result, DialogResult::Pending);
    }

    #[test]
    fn test_dialog_state_cancel() {
        let mut state = DialogState::new();
        state.show(ConfirmDialog::new("Test", "message"));
        state.cancel();

        assert!(!state.is_open());
        assert_eq!(state.take_result(), DialogResult::Cancelled);
    }

    #[test]
    fn test_dialog_state_navigation() {
        let mut state = DialogState::new();
        state.show(ConfirmDialog::new("Test", "message"));

        state.select_next();
        if let Some(ref dialog) = state.dialog {
            assert!(dialog.is_confirm_selected());
        }

        state.select_prev();
        if let Some(ref dialog) = state.dialog {
            assert!(!dialog.is_confirm_selected());
        }
    }

    #[test]
    fn test_dialog_calculate_area() {
        let dialog = ConfirmDialog::new("Test", "message");
        let screen = Rect::new(0, 0, 100, 50);

        let area = dialog.calculate_area(screen);

        assert!(area.width > 0);
        assert!(area.height > 0);
        assert!(area.x > 0);
        assert!(area.y > 0);
        assert!(area.x + area.width <= screen.width);
        assert!(area.y + area.height <= screen.height);
    }
}
