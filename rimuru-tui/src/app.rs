use std::io;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind};
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::RwLock;

use crate::data::{AppData, DataLoadError, DataLoader, LoadingState};
use crate::events::{Action, CommandPalette, EventHandler, InputMode, Keybinds, ScrollDirection};
use crate::theme::{Theme, ThemeLoader, ThemeManager};
use crate::ui::layout::MainLayout;
use crate::ui::utils::SmoothScroll;
use crate::ui::views::{HooksViewState, MetricsViewState, PluginsViewState, SkillsViewState};
use crate::ui::widgets::{ConfirmDialog, DialogResult, DialogState, ToastManager};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Dashboard,
    Agents,
    AgentDetails(usize),
    Sessions,
    SessionDetails(usize),
    Costs,
    CostDetails(usize),
    Metrics,
    Skills,
    SkillDetails(usize),
    Plugins,
    PluginDetails(usize),
    Hooks,
    Help,
}

impl View {
    pub fn all() -> &'static [View] {
        &[
            View::Dashboard,
            View::Agents,
            View::Sessions,
            View::Costs,
            View::Metrics,
            View::Skills,
            View::Plugins,
            View::Hooks,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            View::Dashboard => "Dashboard",
            View::Agents => "Agents",
            View::AgentDetails(_) => "Agent Details",
            View::Sessions => "Sessions",
            View::SessionDetails(_) => "Session Details",
            View::Costs => "Costs",
            View::CostDetails(_) => "Cost Details",
            View::Metrics => "Metrics",
            View::Skills => "Skills",
            View::SkillDetails(_) => "Skill Details",
            View::Plugins => "Plugins",
            View::PluginDetails(_) => "Plugin Details",
            View::Hooks => "Hooks",
            View::Help => "Help",
        }
    }

    pub fn next(&self) -> View {
        let views = View::all();
        let idx = views.iter().position(|v| v == self).unwrap_or(0);
        views[(idx + 1) % views.len()]
    }

    pub fn prev(&self) -> View {
        let views = View::all();
        let idx = views.iter().position(|v| v == self).unwrap_or(0);
        if idx == 0 {
            views[views.len() - 1]
        } else {
            views[idx - 1]
        }
    }
}

#[derive(Default)]
pub struct AppState {
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub search_query: Option<String>,
    pub is_searching: bool,
    pub smooth_scroll: SmoothScroll,
}

pub struct App {
    pub should_quit: bool,
    pub current_view: View,
    pub theme_manager: ThemeManager,
    pub theme_loader: ThemeLoader,
    pub state: AppState,
    pub metrics_state: MetricsViewState,
    pub skills_state: SkillsViewState,
    pub plugins_state: PluginsViewState,
    pub hooks_state: HooksViewState,
    pub last_refresh: std::time::Instant,
    pub refresh_interval: Duration,
    pub status_message: Option<String>,
    pub event_handler: EventHandler,
    pub keybinds: Keybinds,
    pub command_palette: CommandPalette,
    pub terminal_size: Option<(u16, u16)>,
    pub data_loader: Arc<DataLoader>,
    pub cached_data: Arc<RwLock<AppData>>,
    pub loading_state: LoadingState,
    pub last_error: Option<String>,
    pub toast_manager: ToastManager,
    pub dialog_state: DialogState,
    pub show_help_modal: bool,
    pub help_modal_scroll: usize,
    pub animation_tick: u64,
}

impl App {
    pub async fn new() -> Result<Self> {
        let theme_loader = ThemeLoader::new();
        let theme_manager = theme_loader.initialize_theme_manager();
        let theme_name = theme_manager.current_theme_name();
        let keybinds = Keybinds::load_or_default();

        let data_loader =
            Arc::new(DataLoader::new().with_min_fetch_interval(Duration::from_millis(500)));
        let cached_data = Arc::new(RwLock::new(AppData::new()));

        Ok(Self {
            should_quit: false,
            current_view: View::Dashboard,
            theme_manager,
            theme_loader,
            state: AppState::default(),
            metrics_state: MetricsViewState::new(),
            skills_state: SkillsViewState::new(),
            plugins_state: PluginsViewState::new(),
            hooks_state: HooksViewState::new(),
            last_refresh: std::time::Instant::now(),
            refresh_interval: Duration::from_secs(5),
            status_message: Some(format!(
                "Welcome to Rimuru TUI! Theme: {}. Press '?' for help, ':' for commands.",
                theme_name
            )),
            event_handler: EventHandler::new(),
            keybinds,
            command_palette: CommandPalette::new(),
            terminal_size: None,
            data_loader,
            cached_data,
            loading_state: LoadingState::Idle,
            last_error: None,
            toast_manager: ToastManager::new(),
            dialog_state: DialogState::new(),
            show_help_modal: false,
            help_modal_scroll: 0,
            animation_tick: 0,
        })
    }

    pub async fn initialize_data(&mut self) -> Result<()> {
        self.loading_state = LoadingState::Loading;
        self.status_message = Some("Loading data...".to_string());

        match self.data_loader.refresh().await {
            Ok(()) => {
                let data = self.data_loader.data().await;
                *self.cached_data.write().await = data;
                self.loading_state = LoadingState::Success;
                self.last_error = None;
                self.status_message = Some("Data loaded successfully".to_string());
            }
            Err(e) => {
                self.loading_state = LoadingState::Error;
                let error: DataLoadError = e;
                self.last_error = Some(error.to_string());
                self.status_message = Some(format!("Error loading data: {}", error));
            }
        }

        Ok(())
    }

    pub async fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<()> {
        let tick_rate = Duration::from_millis(250);

        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(io::stdout(), crossterm::event::EnableMouseCapture)?;

        self.initialize_data().await?;

        loop {
            self.animation_tick = self.animation_tick.wrapping_add(1);

            self.toast_manager.cleanup();

            self.state.smooth_scroll.update();

            terminal.draw(|frame| {
                MainLayout::render(frame, self);
            })?;

            if event::poll(tick_rate)? {
                let evt = event::read()?;
                self.handle_event(evt);
            }

            if self.should_quit {
                break;
            }

            if self.last_refresh.elapsed() >= self.refresh_interval {
                self.refresh_data().await?;
                self.last_refresh = std::time::Instant::now();
            }
        }

        crossterm::execute!(io::stdout(), crossterm::event::DisableMouseCapture)?;

        Ok(())
    }

    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                self.handle_key_event(key.code, key.modifiers);
            }
            Event::Mouse(mouse) => {
                self.handle_mouse_event(mouse);
            }
            Event::Resize(width, height) => {
                self.handle_resize_event(width, height);
            }
            _ => {}
        }
    }

    fn handle_key_event(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        if self.dialog_state.is_open() {
            self.handle_dialog_key(key);
            return;
        }

        if self.show_help_modal {
            self.handle_help_modal_key(key);
            return;
        }

        if self.command_palette.is_open() {
            self.handle_command_palette_key(key, modifiers);
            return;
        }

        if self.state.is_searching {
            self.handle_search_key(key, modifiers);
            return;
        }

        if let Some(action) = self.keybinds.get(key, modifiers) {
            self.execute_action(action.clone());
            return;
        }

        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Esc
                if self.current_view == View::Plugins && self.plugins_state.show_config_panel =>
            {
                self.plugins_state.close_config_panel();
                self.status_message = Some("Configuration panel closed".to_string());
            }
            KeyCode::Esc => self.handle_back(),
            KeyCode::Tab => {
                self.current_view = self.current_view.next();
                self.state.selected_index = 0;
            }
            KeyCode::BackTab => {
                self.current_view = self.current_view.prev();
                self.state.selected_index = 0;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                let max_index = self.get_max_selection_index();
                if self.state.selected_index < max_index {
                    self.state.selected_index += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.state.selected_index = self.state.selected_index.saturating_sub(1);
            }
            KeyCode::Char('g') => {
                self.state.selected_index = 0;
            }
            KeyCode::Char('G') => {
                self.state.selected_index = self.get_max_selection_index();
            }
            KeyCode::PageUp => {
                self.state.selected_index = self.state.selected_index.saturating_sub(10);
            }
            KeyCode::PageDown => {
                let max_index = self.get_max_selection_index();
                self.state.selected_index = (self.state.selected_index + 10).min(max_index);
            }
            KeyCode::Char('u') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.state.selected_index = self.state.selected_index.saturating_sub(10);
            }
            KeyCode::Char('d') if modifiers.contains(KeyModifiers::CONTROL) => {
                let max_index = self.get_max_selection_index();
                self.state.selected_index = (self.state.selected_index + 10).min(max_index);
            }
            KeyCode::Enter => self.handle_select(),
            KeyCode::Char('t')
                if self.current_view == View::Skills
                    || matches!(self.current_view, View::SkillDetails(_)) =>
            {
                self.status_message =
                    Some("Translate skill: Select target agent... (simulated)".to_string());
            }
            KeyCode::Char('t') if self.current_view == View::Hooks => {
                self.status_message = Some("Triggering hook manually... (simulated)".to_string());
            }
            KeyCode::Char('t') => self.toggle_theme(),
            KeyCode::Char('/') => {
                self.state.is_searching = true;
                self.state.search_query = Some(String::new());
                self.event_handler.set_input_mode(InputMode::Search);
            }
            KeyCode::Char(':') => {
                self.command_palette.open();
                self.event_handler.set_input_mode(InputMode::Command);
            }
            KeyCode::Char('r') => {
                self.request_refresh();
            }
            KeyCode::Char('h') if self.current_view == View::Metrics => {
                self.metrics_state.historical_range = self.metrics_state.historical_range.toggle();
                self.status_message = Some(format!(
                    "Historical range: {}",
                    self.metrics_state.historical_range.label()
                ));
            }
            KeyCode::Char('+') | KeyCode::Char('=') if self.current_view == View::Metrics => {
                self.metrics_state.increase_refresh_rate();
                self.refresh_interval =
                    Duration::from_secs(self.metrics_state.refresh_rate_secs as u64);
                self.status_message = Some(format!(
                    "Refresh rate: {} seconds",
                    self.metrics_state.refresh_rate_secs
                ));
            }
            KeyCode::Char('-') if self.current_view == View::Metrics => {
                self.metrics_state.decrease_refresh_rate();
                self.refresh_interval =
                    Duration::from_secs(self.metrics_state.refresh_rate_secs as u64);
                self.status_message = Some(format!(
                    "Refresh rate: {} seconds",
                    self.metrics_state.refresh_rate_secs
                ));
            }
            KeyCode::Char('?') => {
                self.show_help_modal = !self.show_help_modal;
                self.help_modal_scroll = 0;
            }
            KeyCode::Char('1') => self.current_view = View::Dashboard,
            KeyCode::Char('2') => self.current_view = View::Agents,
            KeyCode::Char('3') => self.current_view = View::Sessions,
            KeyCode::Char('4') => self.current_view = View::Costs,
            KeyCode::Char('5') => self.current_view = View::Metrics,
            KeyCode::Char('6') => self.current_view = View::Skills,
            KeyCode::Char('7') => self.current_view = View::Plugins,
            KeyCode::Char('8') => self.current_view = View::Hooks,
            KeyCode::Char('i')
                if self.current_view == View::Skills
                    || matches!(self.current_view, View::SkillDetails(_)) =>
            {
                self.status_message = Some("Installing skill... (simulated)".to_string());
            }
            KeyCode::Left if self.current_view == View::Skills => {
                self.skills_state.current_tab = self.skills_state.current_tab.prev();
                self.state.selected_index = 0;
            }
            KeyCode::Right if self.current_view == View::Skills => {
                self.skills_state.current_tab = self.skills_state.current_tab.next();
                self.state.selected_index = 0;
            }
            KeyCode::Char('[') if self.current_view == View::Skills => {
                self.skills_state.current_tab = self.skills_state.current_tab.prev();
                self.state.selected_index = 0;
            }
            KeyCode::Char(']') if self.current_view == View::Skills => {
                self.skills_state.current_tab = self.skills_state.current_tab.next();
                self.state.selected_index = 0;
            }
            KeyCode::Char('e')
                if self.current_view == View::Plugins
                    || matches!(self.current_view, View::PluginDetails(_)) =>
            {
                self.status_message = Some("Toggling plugin... (simulated)".to_string());
            }
            KeyCode::Char('c')
                if self.current_view == View::Plugins
                    || matches!(self.current_view, View::PluginDetails(_)) =>
            {
                self.plugins_state
                    .toggle_config_panel(self.state.selected_index);
                if self.plugins_state.show_config_panel {
                    self.status_message = Some("Plugin configuration panel opened".to_string());
                } else {
                    self.status_message = Some("Plugin configuration panel closed".to_string());
                }
            }
            KeyCode::Char('i')
                if self.current_view == View::Plugins
                    || matches!(self.current_view, View::PluginDetails(_)) =>
            {
                self.status_message = Some("Installing plugin... (simulated)".to_string());
            }
            KeyCode::Char('u') if self.current_view == View::Plugins => {
                self.status_message = Some("Uninstalling plugin... (simulated)".to_string());
            }
            KeyCode::Left if self.current_view == View::Plugins => {
                self.plugins_state.current_tab = self.plugins_state.current_tab.prev();
                self.state.selected_index = 0;
            }
            KeyCode::Right if self.current_view == View::Plugins => {
                self.plugins_state.current_tab = self.plugins_state.current_tab.next();
                self.state.selected_index = 0;
            }
            KeyCode::Char('[') if self.current_view == View::Plugins => {
                self.plugins_state.current_tab = self.plugins_state.current_tab.prev();
                self.state.selected_index = 0;
            }
            KeyCode::Char(']') if self.current_view == View::Plugins => {
                self.plugins_state.current_tab = self.plugins_state.current_tab.next();
                self.state.selected_index = 0;
            }
            KeyCode::Char('e') if self.current_view == View::Hooks => {
                self.status_message = Some("Toggling hook handler... (simulated)".to_string());
            }
            KeyCode::Left if self.current_view == View::Hooks => {
                self.hooks_state.current_tab = self.hooks_state.current_tab.prev();
                self.state.selected_index = 0;
            }
            KeyCode::Right if self.current_view == View::Hooks => {
                self.hooks_state.current_tab = self.hooks_state.current_tab.next();
                self.state.selected_index = 0;
            }
            KeyCode::Char('[') if self.current_view == View::Hooks => {
                self.hooks_state.current_tab = self.hooks_state.current_tab.prev();
                self.state.selected_index = 0;
            }
            KeyCode::Char(']') if self.current_view == View::Hooks => {
                self.hooks_state.current_tab = self.hooks_state.current_tab.next();
                self.state.selected_index = 0;
            }
            _ => {}
        }
    }

    fn handle_search_key(&mut self, key: KeyCode, _modifiers: KeyModifiers) {
        match key {
            KeyCode::Esc => {
                self.state.is_searching = false;
                self.state.search_query = None;
                self.event_handler.set_input_mode(InputMode::Normal);
            }
            KeyCode::Enter => {
                self.state.is_searching = false;
                self.event_handler.set_input_mode(InputMode::Normal);
            }
            KeyCode::Backspace => {
                if let Some(ref mut query) = self.state.search_query {
                    query.pop();
                }
            }
            KeyCode::Char(c) => {
                self.state
                    .search_query
                    .get_or_insert_with(String::new)
                    .push(c);
            }
            _ => {}
        }
    }

    fn handle_dialog_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                self.dialog_state.cancel();
            }
            KeyCode::Enter => {
                self.dialog_state.execute_selected();
            }
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.dialog_state.confirm();
            }
            KeyCode::Tab
            | KeyCode::Left
            | KeyCode::Right
            | KeyCode::Char('h')
            | KeyCode::Char('l') => {
                self.dialog_state.select_next();
            }
            _ => {}
        }
    }

    fn handle_help_modal_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                self.show_help_modal = false;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.help_modal_scroll = self.help_modal_scroll.saturating_add(1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.help_modal_scroll = self.help_modal_scroll.saturating_sub(1);
            }
            KeyCode::PageDown | KeyCode::Char('d') => {
                self.help_modal_scroll = self.help_modal_scroll.saturating_add(10);
            }
            KeyCode::PageUp | KeyCode::Char('u') => {
                self.help_modal_scroll = self.help_modal_scroll.saturating_sub(10);
            }
            KeyCode::Char('g') => {
                self.help_modal_scroll = 0;
            }
            KeyCode::Char('G') => {
                self.help_modal_scroll = 100;
            }
            _ => {}
        }
    }

    fn handle_command_palette_key(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match key {
            KeyCode::Esc => {
                self.command_palette.close();
                self.event_handler.set_input_mode(InputMode::Normal);
            }
            KeyCode::Enter => {
                if let Some(action) = self.command_palette.execute_input() {
                    self.event_handler.set_input_mode(InputMode::Normal);
                    self.execute_action(action);
                }
            }
            KeyCode::Backspace => {
                self.command_palette.delete_char();
            }
            KeyCode::Delete => {
                self.command_palette.delete_char_forward();
            }
            KeyCode::Left => {
                self.command_palette.move_cursor_left();
            }
            KeyCode::Right => {
                self.command_palette.move_cursor_right();
            }
            KeyCode::Home => {
                self.command_palette.move_cursor_start();
            }
            KeyCode::End => {
                self.command_palette.move_cursor_end();
            }
            KeyCode::Up => {
                if modifiers.contains(KeyModifiers::CONTROL) {
                    self.command_palette.history_prev();
                } else {
                    self.command_palette.select_prev();
                }
            }
            KeyCode::Down => {
                if modifiers.contains(KeyModifiers::CONTROL) {
                    self.command_palette.history_next();
                } else {
                    self.command_palette.select_next();
                }
            }
            KeyCode::Tab => {
                self.command_palette.select_next();
            }
            KeyCode::BackTab => {
                self.command_palette.select_prev();
            }
            KeyCode::Char('u') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.command_palette.clear_input();
            }
            KeyCode::Char('n') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.command_palette.select_next();
            }
            KeyCode::Char('p') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.command_palette.select_prev();
            }
            KeyCode::Char(c) => {
                self.command_palette.insert_char(c);
            }
            _ => {}
        }
    }

    fn handle_mouse_event(&mut self, mouse: crossterm::event::MouseEvent) {
        match mouse.kind {
            MouseEventKind::ScrollUp => {
                self.state.selected_index = self.state.selected_index.saturating_sub(3);
            }
            MouseEventKind::ScrollDown => {
                let max_index = self.get_max_selection_index();
                self.state.selected_index = (self.state.selected_index + 3).min(max_index);
            }
            MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                self.status_message = Some(format!("Click at ({}, {})", mouse.column, mouse.row));
            }
            _ => {}
        }
    }

    fn handle_resize_event(&mut self, width: u16, height: u16) {
        self.terminal_size = Some((width, height));
        self.event_handler.handle_resize(width, height);
    }

    fn execute_action(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::NextView => {
                self.current_view = self.current_view.next();
                self.state.selected_index = 0;
            }
            Action::PrevView => {
                self.current_view = self.current_view.prev();
                self.state.selected_index = 0;
            }
            Action::Up => {
                self.state.selected_index = self.state.selected_index.saturating_sub(1);
            }
            Action::Down => {
                let max_index = self.get_max_selection_index();
                if self.state.selected_index < max_index {
                    self.state.selected_index += 1;
                }
            }
            Action::Left | Action::Right => {}
            Action::Top => {
                self.state.selected_index = 0;
            }
            Action::Bottom => {
                self.state.selected_index = self.get_max_selection_index();
            }
            Action::PageUp => {
                self.state.selected_index = self.state.selected_index.saturating_sub(10);
            }
            Action::PageDown => {
                let max_index = self.get_max_selection_index();
                self.state.selected_index = (self.state.selected_index + 10).min(max_index);
            }
            Action::Select => self.handle_select(),
            Action::Back => self.handle_back(),
            Action::Search => {
                self.state.is_searching = true;
                self.state.search_query = Some(String::new());
                self.event_handler.set_input_mode(InputMode::Search);
            }
            Action::Refresh => {
                self.request_refresh();
            }
            Action::ToggleTheme => self.toggle_theme(),
            Action::Help => {
                self.current_view = View::Help;
            }
            Action::GoToView(idx) => {
                let views = View::all();
                if idx < views.len() {
                    self.current_view = views[idx];
                    self.state.selected_index = 0;
                }
            }
            Action::OpenCommandPalette => {
                self.command_palette.open();
                self.event_handler.set_input_mode(InputMode::Command);
            }
            Action::CancelCommand => {
                self.command_palette.close();
                self.state.is_searching = false;
                self.state.search_query = None;
                self.event_handler.set_input_mode(InputMode::Normal);
            }
            Action::SortColumn => {
                self.status_message = Some("Sort toggled".to_string());
            }
            Action::FilterToggle => {
                self.status_message = Some("Filter toggled".to_string());
            }
            Action::ToggleHistorical => {
                if self.current_view == View::Metrics {
                    self.metrics_state.historical_range =
                        self.metrics_state.historical_range.toggle();
                    self.status_message = Some(format!(
                        "Historical range: {}",
                        self.metrics_state.historical_range.label()
                    ));
                }
            }
            Action::IncreaseRefreshRate => {
                if self.current_view == View::Metrics {
                    self.metrics_state.increase_refresh_rate();
                    self.refresh_interval =
                        Duration::from_secs(self.metrics_state.refresh_rate_secs as u64);
                    self.status_message = Some(format!(
                        "Refresh rate: {} seconds",
                        self.metrics_state.refresh_rate_secs
                    ));
                }
            }
            Action::DecreaseRefreshRate => {
                if self.current_view == View::Metrics {
                    self.metrics_state.decrease_refresh_rate();
                    self.refresh_interval =
                        Duration::from_secs(self.metrics_state.refresh_rate_secs as u64);
                    self.status_message = Some(format!(
                        "Refresh rate: {} seconds",
                        self.metrics_state.refresh_rate_secs
                    ));
                }
            }
            Action::MouseClick { x, y } => {
                self.status_message = Some(format!("Click at ({}, {})", x, y));
            }
            Action::MouseScroll { direction } => match direction {
                ScrollDirection::Up => {
                    self.state.selected_index = self.state.selected_index.saturating_sub(3);
                }
                ScrollDirection::Down => {
                    let max_index = self.get_max_selection_index();
                    self.state.selected_index = (self.state.selected_index + 3).min(max_index);
                }
            },
            Action::Resize { width, height } => {
                self.terminal_size = Some((width, height));
            }
            Action::ExecuteCommand(cmd) => {
                self.status_message = Some(format!("Executed: {}", cmd));
            }
            Action::None => {}
        }
    }

    fn handle_select(&mut self) {
        match self.current_view {
            View::Agents => {
                self.current_view = View::AgentDetails(self.state.selected_index);
                self.status_message =
                    Some("Viewing agent details. Press Esc to go back.".to_string());
            }
            View::Sessions => {
                self.current_view = View::SessionDetails(self.state.selected_index);
                self.status_message =
                    Some("Viewing session details. Press Esc to go back.".to_string());
            }
            View::Costs => {
                self.current_view = View::CostDetails(self.state.selected_index);
                self.status_message =
                    Some("Viewing cost details. Press Esc to go back.".to_string());
            }
            View::Skills => {
                self.current_view = View::SkillDetails(self.state.selected_index);
                self.status_message =
                    Some("Viewing skill details. Press Esc to go back.".to_string());
            }
            View::Plugins => {
                self.current_view = View::PluginDetails(self.state.selected_index);
                self.status_message =
                    Some("Viewing plugin details. Press Esc to go back.".to_string());
            }
            _ => {}
        }
    }

    fn handle_back(&mut self) {
        match self.current_view {
            View::AgentDetails(_) => {
                self.current_view = View::Agents;
            }
            View::SessionDetails(_) => {
                self.current_view = View::Sessions;
            }
            View::CostDetails(_) => {
                self.current_view = View::Costs;
            }
            View::SkillDetails(_) => {
                self.current_view = View::Skills;
            }
            View::PluginDetails(_) => {
                self.current_view = View::Plugins;
                self.plugins_state.close_config_panel();
            }
            View::Help => {
                self.current_view = View::Dashboard;
            }
            View::Plugins if self.plugins_state.show_config_panel => {
                self.plugins_state.close_config_panel();
            }
            _ => self.should_quit = true,
        }
    }

    fn toggle_theme(&mut self) {
        self.theme_manager.cycle_theme();
        let theme_name = self.theme_manager.current_theme_name();
        if let Err(e) = self.theme_loader.save_theme_name(theme_name) {
            tracing::warn!("Failed to save theme preference: {}", e);
        }
        self.status_message = Some(format!("Theme changed to: {}", theme_name));
    }

    async fn refresh_data(&mut self) -> Result<()> {
        if !self.data_loader.should_refresh().await {
            return Ok(());
        }

        self.loading_state = LoadingState::Loading;

        match self.data_loader.refresh().await {
            Ok(()) => {
                let data = self.data_loader.data().await;
                *self.cached_data.write().await = data;
                self.loading_state = LoadingState::Success;
                self.last_error = None;

                self.metrics_state.last_update_tick = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
            }
            Err(e) => {
                self.loading_state = LoadingState::Error;
                self.last_error = Some(self.format_error(&e));
                self.status_message = Some(self.format_error(&e));
            }
        }

        Ok(())
    }

    fn format_error(&self, error: &DataLoadError) -> String {
        match error {
            DataLoadError::DatabaseError(msg) => {
                format!("Database unavailable: {}. Using cached data.", msg)
            }
            DataLoadError::ConnectionError(msg) => {
                format!("Connection lost: {}. Retrying...", msg)
            }
            DataLoadError::TimeoutError => {
                "Request timed out. Check your network connection.".to_string()
            }
            DataLoadError::Other(msg) => {
                format!("Error: {}", msg)
            }
        }
    }

    pub async fn force_refresh(&mut self) -> Result<()> {
        self.status_message = Some("Refreshing data...".to_string());
        self.loading_state = LoadingState::Loading;

        match self.data_loader.force_refresh().await {
            Ok(()) => {
                let data = self.data_loader.data().await;
                *self.cached_data.write().await = data;
                self.loading_state = LoadingState::Success;
                self.last_error = None;
                self.status_message = Some("Data refreshed successfully".to_string());
            }
            Err(e) => {
                self.loading_state = LoadingState::Error;
                let error_msg = self.format_error(&e);
                self.last_error = Some(error_msg.clone());
                self.status_message = Some(error_msg);
            }
        }

        self.last_refresh = std::time::Instant::now();
        Ok(())
    }

    pub fn data(&self) -> &Arc<RwLock<AppData>> {
        &self.cached_data
    }

    pub fn is_loading(&self) -> bool {
        self.loading_state.is_loading()
    }

    pub fn has_error(&self) -> bool {
        self.loading_state.is_error()
    }

    pub fn loading_indicator(&self) -> &'static str {
        self.loading_state.indicator()
    }

    pub fn request_refresh(&mut self) {
        self.status_message = Some("Refreshing data...".to_string());
        self.loading_state = LoadingState::Loading;
        self.last_refresh = std::time::Instant::now() - self.refresh_interval;
        self.metrics_state.last_update_tick = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    pub fn current_theme(&self) -> &dyn Theme {
        self.theme_manager.current_theme()
    }

    fn get_max_selection_index(&self) -> usize {
        use crate::ui::views::{HooksView, PluginsView, SkillsView};

        if let Ok(data) = self.cached_data.try_read() {
            match self.current_view {
                View::Agents => data.agents.len().saturating_sub(1),
                View::Sessions => data.sessions.len().saturating_sub(1),
                View::Dashboard => data.agents.len().saturating_sub(1),
                View::Costs => data.costs.by_agent.len().saturating_sub(1),
                View::Skills => SkillsView::current_list_len(&self.skills_state).saturating_sub(1),
                View::Plugins => {
                    PluginsView::current_list_len(&self.plugins_state).saturating_sub(1)
                }
                View::Hooks => HooksView::current_list_len(&self.hooks_state).saturating_sub(1),
                _ => 10,
            }
        } else {
            match self.current_view {
                View::Agents => 5,
                View::Sessions => 10,
                View::Dashboard => 5,
                View::Costs => 3,
                View::Skills => SkillsView::current_list_len(&self.skills_state).saturating_sub(1),
                View::Plugins => {
                    PluginsView::current_list_len(&self.plugins_state).saturating_sub(1)
                }
                View::Hooks => HooksView::current_list_len(&self.hooks_state).saturating_sub(1),
                _ => 10,
            }
        }
    }

    pub fn input_mode(&self) -> InputMode {
        self.event_handler.input_mode()
    }

    pub fn is_command_palette_open(&self) -> bool {
        self.command_palette.is_open()
    }

    pub fn show_toast_info(&mut self, message: impl Into<String>) {
        self.toast_manager.info(message);
    }

    pub fn show_toast_success(&mut self, message: impl Into<String>) {
        self.toast_manager.success(message);
    }

    pub fn show_toast_warning(&mut self, message: impl Into<String>) {
        self.toast_manager.warning(message);
    }

    pub fn show_toast_error(&mut self, message: impl Into<String>) {
        self.toast_manager.error(message);
    }

    pub fn show_confirm_dialog(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.dialog_state
            .show(ConfirmDialog::confirmation(title, message));
    }

    pub fn show_warning_dialog(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.dialog_state
            .show(ConfirmDialog::warning(title, message));
    }

    pub fn show_danger_dialog(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.dialog_state
            .show(ConfirmDialog::danger(title, message));
    }

    pub fn take_dialog_result(&mut self) -> DialogResult {
        self.dialog_state.take_result()
    }

    pub fn is_dialog_open(&self) -> bool {
        self.dialog_state.is_open()
    }

    pub fn is_help_modal_open(&self) -> bool {
        self.show_help_modal
    }

    pub fn animation_frame(&self) -> u64 {
        self.animation_tick
    }
}
