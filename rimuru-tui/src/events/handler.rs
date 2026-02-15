use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::layout::Rect;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Tick,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Quit,
    NextView,
    PrevView,
    Up,
    Down,
    Left,
    Right,
    Top,
    Bottom,
    PageUp,
    PageDown,
    Select,
    Back,
    Search,
    Refresh,
    ToggleTheme,
    Help,
    GoToView(usize),
    OpenCommandPalette,
    ExecuteCommand(String),
    CancelCommand,
    MouseClick { x: u16, y: u16 },
    MouseScroll { direction: ScrollDirection },
    Resize { width: u16, height: u16 },
    SortColumn,
    FilterToggle,
    ToggleHistorical,
    IncreaseRefreshRate,
    DecreaseRefreshRate,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Search,
    Command,
}

pub struct EventHandler {
    input_mode: InputMode,
    terminal_size: Option<(u16, u16)>,
    clickable_regions: Vec<ClickableRegion>,
}

#[derive(Debug, Clone)]
pub struct ClickableRegion {
    pub area: Rect,
    pub action: Action,
    pub label: String,
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl EventHandler {
    pub fn new() -> Self {
        Self {
            input_mode: InputMode::Normal,
            terminal_size: None,
            clickable_regions: Vec::new(),
        }
    }

    pub fn input_mode(&self) -> InputMode {
        self.input_mode
    }

    pub fn set_input_mode(&mut self, mode: InputMode) {
        self.input_mode = mode;
    }

    pub fn terminal_size(&self) -> Option<(u16, u16)> {
        self.terminal_size
    }

    pub fn register_clickable_region(&mut self, region: ClickableRegion) {
        self.clickable_regions.push(region);
    }

    pub fn clear_clickable_regions(&mut self) {
        self.clickable_regions.clear();
    }

    pub fn handle_event(&mut self, event: Event) -> Option<Action> {
        match event {
            Event::Key(key) => self.handle_key(key),
            Event::Mouse(mouse) => self.handle_mouse(mouse),
            Event::Resize(width, height) => self.handle_resize(width, height),
            Event::FocusGained | Event::FocusLost | Event::Paste(_) => None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<Action> {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_mode_key(key),
            InputMode::Search => self.handle_search_mode_key(key),
            InputMode::Command => self.handle_command_mode_key(key),
        }
    }

    fn handle_normal_mode_key(&mut self, key: KeyEvent) -> Option<Action> {
        let ctrl_pressed = key.modifiers.contains(KeyModifiers::CONTROL);

        match (key.code, ctrl_pressed) {
            (KeyCode::Char('c'), true) => Some(Action::Quit),
            (KeyCode::Char('q'), false) => Some(Action::Quit),
            (KeyCode::Esc, _) => Some(Action::Back),
            (KeyCode::Tab, _) => Some(Action::NextView),
            (KeyCode::BackTab, _) => Some(Action::PrevView),
            (KeyCode::Char('j'), false) | (KeyCode::Down, _) => Some(Action::Down),
            (KeyCode::Char('k'), false) | (KeyCode::Up, _) => Some(Action::Up),
            (KeyCode::Char('h'), false) | (KeyCode::Left, _) => Some(Action::Left),
            (KeyCode::Char('l'), false) | (KeyCode::Right, _) => Some(Action::Right),
            (KeyCode::Char('g'), false) => Some(Action::Top),
            (KeyCode::Char('G'), false) => Some(Action::Bottom),
            (KeyCode::PageUp, _) | (KeyCode::Char('u'), true) => Some(Action::PageUp),
            (KeyCode::PageDown, _) | (KeyCode::Char('d'), true) => Some(Action::PageDown),
            (KeyCode::Enter, _) => Some(Action::Select),
            (KeyCode::Char('/'), false) => {
                self.input_mode = InputMode::Search;
                Some(Action::Search)
            }
            (KeyCode::Char(':'), false) => {
                self.input_mode = InputMode::Command;
                Some(Action::OpenCommandPalette)
            }
            (KeyCode::Char('r'), false) => Some(Action::Refresh),
            (KeyCode::Char('t'), false) => Some(Action::ToggleTheme),
            (KeyCode::Char('?'), false) => Some(Action::Help),
            (KeyCode::Char('s'), false) => Some(Action::SortColumn),
            (KeyCode::Char('f'), false) => Some(Action::FilterToggle),
            (KeyCode::Char('H'), false) => Some(Action::ToggleHistorical),
            (KeyCode::Char('+'), false) | (KeyCode::Char('='), false) => {
                Some(Action::IncreaseRefreshRate)
            }
            (KeyCode::Char('-'), false) => Some(Action::DecreaseRefreshRate),
            (KeyCode::Char('1'), false) => Some(Action::GoToView(0)),
            (KeyCode::Char('2'), false) => Some(Action::GoToView(1)),
            (KeyCode::Char('3'), false) => Some(Action::GoToView(2)),
            (KeyCode::Char('4'), false) => Some(Action::GoToView(3)),
            (KeyCode::Char('5'), false) => Some(Action::GoToView(4)),
            _ => None,
        }
    }

    fn handle_search_mode_key(&mut self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                Some(Action::CancelCommand)
            }
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
                Some(Action::Select)
            }
            _ => None,
        }
    }

    fn handle_command_mode_key(&mut self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                Some(Action::CancelCommand)
            }
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
                None
            }
            _ => None,
        }
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent) -> Option<Action> {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let x = mouse.column;
                let y = mouse.row;

                for region in &self.clickable_regions {
                    if x >= region.area.x
                        && x < region.area.x + region.area.width
                        && y >= region.area.y
                        && y < region.area.y + region.area.height
                    {
                        return Some(region.action.clone());
                    }
                }

                Some(Action::MouseClick { x, y })
            }
            MouseEventKind::ScrollUp => Some(Action::MouseScroll {
                direction: ScrollDirection::Up,
            }),
            MouseEventKind::ScrollDown => Some(Action::MouseScroll {
                direction: ScrollDirection::Down,
            }),
            _ => None,
        }
    }

    pub fn handle_resize(&mut self, width: u16, height: u16) -> Option<Action> {
        self.terminal_size = Some((width, height));
        Some(Action::Resize { width, height })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn key_event_ctrl(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::CONTROL)
    }

    #[test]
    fn test_normal_mode_navigation() {
        let mut handler = EventHandler::new();

        assert_eq!(
            handler.handle_key(key_event(KeyCode::Char('j'))),
            Some(Action::Down)
        );
        assert_eq!(
            handler.handle_key(key_event(KeyCode::Down)),
            Some(Action::Down)
        );
        assert_eq!(
            handler.handle_key(key_event(KeyCode::Char('k'))),
            Some(Action::Up)
        );
        assert_eq!(handler.handle_key(key_event(KeyCode::Up)), Some(Action::Up));
        assert_eq!(
            handler.handle_key(key_event(KeyCode::Char('g'))),
            Some(Action::Top)
        );
        assert_eq!(
            handler.handle_key(key_event(KeyCode::Char('G'))),
            Some(Action::Bottom)
        );
    }

    #[test]
    fn test_normal_mode_quit() {
        let mut handler = EventHandler::new();

        assert_eq!(
            handler.handle_key(key_event(KeyCode::Char('q'))),
            Some(Action::Quit)
        );
        assert_eq!(
            handler.handle_key(key_event_ctrl(KeyCode::Char('c'))),
            Some(Action::Quit)
        );
    }

    #[test]
    fn test_view_switching() {
        let mut handler = EventHandler::new();

        assert_eq!(
            handler.handle_key(key_event(KeyCode::Tab)),
            Some(Action::NextView)
        );
        assert_eq!(
            handler.handle_key(key_event(KeyCode::BackTab)),
            Some(Action::PrevView)
        );
        assert_eq!(
            handler.handle_key(key_event(KeyCode::Char('1'))),
            Some(Action::GoToView(0))
        );
        assert_eq!(
            handler.handle_key(key_event(KeyCode::Char('5'))),
            Some(Action::GoToView(4))
        );
    }

    #[test]
    fn test_search_mode_transition() {
        let mut handler = EventHandler::new();
        assert_eq!(handler.input_mode(), InputMode::Normal);

        handler.handle_key(key_event(KeyCode::Char('/')));
        assert_eq!(handler.input_mode(), InputMode::Search);

        handler.handle_key(key_event(KeyCode::Esc));
        assert_eq!(handler.input_mode(), InputMode::Normal);
    }

    #[test]
    fn test_command_mode_transition() {
        let mut handler = EventHandler::new();
        assert_eq!(handler.input_mode(), InputMode::Normal);

        handler.handle_key(key_event(KeyCode::Char(':')));
        assert_eq!(handler.input_mode(), InputMode::Command);

        handler.handle_key(key_event(KeyCode::Esc));
        assert_eq!(handler.input_mode(), InputMode::Normal);
    }

    #[test]
    fn test_mouse_scroll() {
        let mut handler = EventHandler::new();

        let scroll_up = MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        };
        assert_eq!(
            handler.handle_mouse(scroll_up),
            Some(Action::MouseScroll {
                direction: ScrollDirection::Up
            })
        );

        let scroll_down = MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        };
        assert_eq!(
            handler.handle_mouse(scroll_down),
            Some(Action::MouseScroll {
                direction: ScrollDirection::Down
            })
        );
    }

    #[test]
    fn test_clickable_regions() {
        let mut handler = EventHandler::new();

        let region = ClickableRegion {
            area: Rect::new(10, 10, 20, 5),
            action: Action::Help,
            label: "Help".to_string(),
        };
        handler.register_clickable_region(region);

        let click_inside = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 15,
            row: 12,
            modifiers: KeyModifiers::NONE,
        };
        assert_eq!(handler.handle_mouse(click_inside), Some(Action::Help));

        let click_outside = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 5,
            row: 5,
            modifiers: KeyModifiers::NONE,
        };
        assert_eq!(
            handler.handle_mouse(click_outside),
            Some(Action::MouseClick { x: 5, y: 5 })
        );
    }

    #[test]
    fn test_resize() {
        let mut handler = EventHandler::new();

        assert_eq!(handler.terminal_size(), None);

        let action = handler.handle_resize(120, 40);
        assert_eq!(
            action,
            Some(Action::Resize {
                width: 120,
                height: 40
            })
        );
        assert_eq!(handler.terminal_size(), Some((120, 40)));
    }

    #[test]
    fn test_page_navigation() {
        let mut handler = EventHandler::new();

        assert_eq!(
            handler.handle_key(key_event(KeyCode::PageUp)),
            Some(Action::PageUp)
        );
        assert_eq!(
            handler.handle_key(key_event(KeyCode::PageDown)),
            Some(Action::PageDown)
        );
        assert_eq!(
            handler.handle_key(key_event_ctrl(KeyCode::Char('u'))),
            Some(Action::PageUp)
        );
        assert_eq!(
            handler.handle_key(key_event_ctrl(KeyCode::Char('d'))),
            Some(Action::PageDown)
        );
    }

    #[test]
    fn test_actions() {
        let mut handler = EventHandler::new();

        assert_eq!(
            handler.handle_key(key_event(KeyCode::Char('r'))),
            Some(Action::Refresh)
        );
        assert_eq!(
            handler.handle_key(key_event(KeyCode::Char('t'))),
            Some(Action::ToggleTheme)
        );
        assert_eq!(
            handler.handle_key(key_event(KeyCode::Char('?'))),
            Some(Action::Help)
        );
        assert_eq!(
            handler.handle_key(key_event(KeyCode::Char('s'))),
            Some(Action::SortColumn)
        );
        assert_eq!(
            handler.handle_key(key_event(KeyCode::Char('f'))),
            Some(Action::FilterToggle)
        );
    }
}
