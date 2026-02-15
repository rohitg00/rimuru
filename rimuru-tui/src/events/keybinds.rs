use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use serde::{Deserialize, Serialize};

use super::handler::Action;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyBinding {
    pub code: SerializableKeyCode,
    #[serde(default)]
    pub modifiers: SerializableKeyModifiers,
}

impl KeyBinding {
    pub fn new(code: KeyCode) -> Self {
        Self {
            code: SerializableKeyCode(code),
            modifiers: SerializableKeyModifiers::default(),
        }
    }

    pub fn with_ctrl(code: KeyCode) -> Self {
        Self {
            code: SerializableKeyCode(code),
            modifiers: SerializableKeyModifiers(KeyModifiers::CONTROL),
        }
    }

    pub fn with_shift(code: KeyCode) -> Self {
        Self {
            code: SerializableKeyCode(code),
            modifiers: SerializableKeyModifiers(KeyModifiers::SHIFT),
        }
    }

    pub fn matches(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.code.0 == code && self.modifiers.0 == modifiers
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SerializableKeyCode(pub KeyCode);

impl Serialize for SerializableKeyCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self.0 {
            KeyCode::Char(c) => format!("Char({})", c),
            KeyCode::Tab => "Tab".to_string(),
            KeyCode::BackTab => "BackTab".to_string(),
            KeyCode::Enter => "Enter".to_string(),
            KeyCode::Esc => "Esc".to_string(),
            KeyCode::Up => "Up".to_string(),
            KeyCode::Down => "Down".to_string(),
            KeyCode::Left => "Left".to_string(),
            KeyCode::Right => "Right".to_string(),
            KeyCode::PageUp => "PageUp".to_string(),
            KeyCode::PageDown => "PageDown".to_string(),
            KeyCode::Home => "Home".to_string(),
            KeyCode::End => "End".to_string(),
            KeyCode::Delete => "Delete".to_string(),
            KeyCode::Backspace => "Backspace".to_string(),
            KeyCode::F(n) => format!("F{}", n),
            _ => format!("{:?}", self.0),
        };
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for SerializableKeyCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let code =
            parse_key_code(&s).ok_or_else(|| serde::de::Error::custom("Invalid key code"))?;
        Ok(SerializableKeyCode(code))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SerializableKeyModifiers(pub KeyModifiers);

impl Default for SerializableKeyModifiers {
    fn default() -> Self {
        Self(KeyModifiers::NONE)
    }
}

impl Serialize for SerializableKeyModifiers {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut parts = Vec::new();
        if self.0.contains(KeyModifiers::CONTROL) {
            parts.push("Ctrl");
        }
        if self.0.contains(KeyModifiers::ALT) {
            parts.push("Alt");
        }
        if self.0.contains(KeyModifiers::SHIFT) {
            parts.push("Shift");
        }
        if parts.is_empty() {
            parts.push("None");
        }
        serializer.serialize_str(&parts.join("+"))
    }
}

impl<'de> Deserialize<'de> for SerializableKeyModifiers {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let modifiers = parse_modifiers(&s);
        Ok(SerializableKeyModifiers(modifiers))
    }
}

fn parse_key_code(s: &str) -> Option<KeyCode> {
    let s = s.trim();
    if s.starts_with("Char(") && s.ends_with(')') {
        let inner = &s[5..s.len() - 1];
        let c = inner.chars().next()?;
        return Some(KeyCode::Char(c));
    }
    if s.starts_with('F') && s.len() > 1 {
        if let Ok(n) = s[1..].parse::<u8>() {
            return Some(KeyCode::F(n));
        }
    }
    match s {
        "Tab" => Some(KeyCode::Tab),
        "BackTab" => Some(KeyCode::BackTab),
        "Enter" => Some(KeyCode::Enter),
        "Esc" | "Escape" => Some(KeyCode::Esc),
        "Up" => Some(KeyCode::Up),
        "Down" => Some(KeyCode::Down),
        "Left" => Some(KeyCode::Left),
        "Right" => Some(KeyCode::Right),
        "PageUp" => Some(KeyCode::PageUp),
        "PageDown" => Some(KeyCode::PageDown),
        "Home" => Some(KeyCode::Home),
        "End" => Some(KeyCode::End),
        "Delete" => Some(KeyCode::Delete),
        "Backspace" => Some(KeyCode::Backspace),
        _ => None,
    }
}

fn parse_modifiers(s: &str) -> KeyModifiers {
    let mut modifiers = KeyModifiers::NONE;
    for part in s.split('+') {
        match part.trim() {
            "Ctrl" | "Control" => modifiers |= KeyModifiers::CONTROL,
            "Alt" => modifiers |= KeyModifiers::ALT,
            "Shift" => modifiers |= KeyModifiers::SHIFT,
            _ => {}
        }
    }
    modifiers
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindConfig {
    pub quit: Vec<KeyBinding>,
    pub quit_all: Vec<KeyBinding>,
    pub next_view: Vec<KeyBinding>,
    pub prev_view: Vec<KeyBinding>,
    pub up: Vec<KeyBinding>,
    pub down: Vec<KeyBinding>,
    pub left: Vec<KeyBinding>,
    pub right: Vec<KeyBinding>,
    pub top: Vec<KeyBinding>,
    pub bottom: Vec<KeyBinding>,
    pub page_up: Vec<KeyBinding>,
    pub page_down: Vec<KeyBinding>,
    pub select: Vec<KeyBinding>,
    pub back: Vec<KeyBinding>,
    pub search: Vec<KeyBinding>,
    pub command: Vec<KeyBinding>,
    pub refresh: Vec<KeyBinding>,
    pub toggle_theme: Vec<KeyBinding>,
    pub help: Vec<KeyBinding>,
    pub sort: Vec<KeyBinding>,
    pub filter: Vec<KeyBinding>,
    pub view_dashboard: Vec<KeyBinding>,
    pub view_agents: Vec<KeyBinding>,
    pub view_sessions: Vec<KeyBinding>,
    pub view_costs: Vec<KeyBinding>,
    pub view_metrics: Vec<KeyBinding>,
}

impl Default for KeybindConfig {
    fn default() -> Self {
        Self {
            quit: vec![KeyBinding::new(KeyCode::Char('q'))],
            quit_all: vec![KeyBinding::with_ctrl(KeyCode::Char('c'))],
            next_view: vec![KeyBinding::new(KeyCode::Tab)],
            prev_view: vec![KeyBinding::new(KeyCode::BackTab)],
            up: vec![
                KeyBinding::new(KeyCode::Char('k')),
                KeyBinding::new(KeyCode::Up),
            ],
            down: vec![
                KeyBinding::new(KeyCode::Char('j')),
                KeyBinding::new(KeyCode::Down),
            ],
            left: vec![
                KeyBinding::new(KeyCode::Char('h')),
                KeyBinding::new(KeyCode::Left),
            ],
            right: vec![
                KeyBinding::new(KeyCode::Char('l')),
                KeyBinding::new(KeyCode::Right),
            ],
            top: vec![KeyBinding::new(KeyCode::Char('g'))],
            bottom: vec![KeyBinding::with_shift(KeyCode::Char('G'))],
            page_up: vec![
                KeyBinding::new(KeyCode::PageUp),
                KeyBinding::with_ctrl(KeyCode::Char('u')),
            ],
            page_down: vec![
                KeyBinding::new(KeyCode::PageDown),
                KeyBinding::with_ctrl(KeyCode::Char('d')),
            ],
            select: vec![KeyBinding::new(KeyCode::Enter)],
            back: vec![KeyBinding::new(KeyCode::Esc)],
            search: vec![KeyBinding::new(KeyCode::Char('/'))],
            command: vec![KeyBinding::new(KeyCode::Char(':'))],
            refresh: vec![KeyBinding::new(KeyCode::Char('r'))],
            toggle_theme: vec![KeyBinding::new(KeyCode::Char('t'))],
            help: vec![KeyBinding::new(KeyCode::Char('?'))],
            sort: vec![KeyBinding::new(KeyCode::Char('s'))],
            filter: vec![KeyBinding::new(KeyCode::Char('f'))],
            view_dashboard: vec![KeyBinding::new(KeyCode::Char('1'))],
            view_agents: vec![KeyBinding::new(KeyCode::Char('2'))],
            view_sessions: vec![KeyBinding::new(KeyCode::Char('3'))],
            view_costs: vec![KeyBinding::new(KeyCode::Char('4'))],
            view_metrics: vec![KeyBinding::new(KeyCode::Char('5'))],
        }
    }
}

pub struct Keybinds {
    config: KeybindConfig,
    bindings: HashMap<(KeyCode, KeyModifiers), Action>,
}

impl Default for Keybinds {
    fn default() -> Self {
        Self::new()
    }
}

impl Keybinds {
    pub fn new() -> Self {
        let config = KeybindConfig::default();
        let mut keybinds = Self {
            config: config.clone(),
            bindings: HashMap::new(),
        };
        keybinds.rebuild_bindings(&config);
        keybinds
    }

    pub fn from_config(config: KeybindConfig) -> Self {
        let mut keybinds = Self {
            config: config.clone(),
            bindings: HashMap::new(),
        };
        keybinds.rebuild_bindings(&config);
        keybinds
    }

    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: KeybindConfig = toml::from_str(&content)?;
        Ok(Self::from_config(config))
    }

    pub fn load_or_default() -> Self {
        let config_path = Self::config_path();
        if config_path.exists() {
            Self::load_from_file(&config_path).unwrap_or_else(|e| {
                tracing::warn!("Failed to load keybinds config: {}. Using defaults.", e);
                Self::new()
            })
        } else {
            Self::new()
        }
    }

    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(&self.config)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path();
        self.save_to_file(&config_path)
    }

    fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rimuru")
            .join("keybinds.toml")
    }

    fn rebuild_bindings(&mut self, config: &KeybindConfig) {
        self.bindings.clear();

        for kb in &config.quit {
            self.bindings
                .insert((kb.code.0, kb.modifiers.0), Action::Quit);
        }
        for kb in &config.quit_all {
            self.bindings
                .insert((kb.code.0, kb.modifiers.0), Action::Quit);
        }
        for kb in &config.next_view {
            self.bindings
                .insert((kb.code.0, kb.modifiers.0), Action::NextView);
        }
        for kb in &config.prev_view {
            self.bindings
                .insert((kb.code.0, kb.modifiers.0), Action::PrevView);
        }
        for kb in &config.up {
            self.bindings
                .insert((kb.code.0, kb.modifiers.0), Action::Up);
        }
        for kb in &config.down {
            self.bindings
                .insert((kb.code.0, kb.modifiers.0), Action::Down);
        }
        for kb in &config.left {
            self.bindings
                .insert((kb.code.0, kb.modifiers.0), Action::Left);
        }
        for kb in &config.right {
            self.bindings
                .insert((kb.code.0, kb.modifiers.0), Action::Right);
        }
        for kb in &config.top {
            self.bindings
                .insert((kb.code.0, kb.modifiers.0), Action::Top);
        }
        for kb in &config.bottom {
            self.bindings
                .insert((kb.code.0, kb.modifiers.0), Action::Bottom);
        }
        for kb in &config.page_up {
            self.bindings
                .insert((kb.code.0, kb.modifiers.0), Action::PageUp);
        }
        for kb in &config.page_down {
            self.bindings
                .insert((kb.code.0, kb.modifiers.0), Action::PageDown);
        }
        for kb in &config.select {
            self.bindings
                .insert((kb.code.0, kb.modifiers.0), Action::Select);
        }
        for kb in &config.back {
            self.bindings
                .insert((kb.code.0, kb.modifiers.0), Action::Back);
        }
        for kb in &config.search {
            self.bindings
                .insert((kb.code.0, kb.modifiers.0), Action::Search);
        }
        for kb in &config.command {
            self.bindings
                .insert((kb.code.0, kb.modifiers.0), Action::OpenCommandPalette);
        }
        for kb in &config.refresh {
            self.bindings
                .insert((kb.code.0, kb.modifiers.0), Action::Refresh);
        }
        for kb in &config.toggle_theme {
            self.bindings
                .insert((kb.code.0, kb.modifiers.0), Action::ToggleTheme);
        }
        for kb in &config.help {
            self.bindings
                .insert((kb.code.0, kb.modifiers.0), Action::Help);
        }
        for kb in &config.sort {
            self.bindings
                .insert((kb.code.0, kb.modifiers.0), Action::SortColumn);
        }
        for kb in &config.filter {
            self.bindings
                .insert((kb.code.0, kb.modifiers.0), Action::FilterToggle);
        }
        for kb in &config.view_dashboard {
            self.bindings
                .insert((kb.code.0, kb.modifiers.0), Action::GoToView(0));
        }
        for kb in &config.view_agents {
            self.bindings
                .insert((kb.code.0, kb.modifiers.0), Action::GoToView(1));
        }
        for kb in &config.view_sessions {
            self.bindings
                .insert((kb.code.0, kb.modifiers.0), Action::GoToView(2));
        }
        for kb in &config.view_costs {
            self.bindings
                .insert((kb.code.0, kb.modifiers.0), Action::GoToView(3));
        }
        for kb in &config.view_metrics {
            self.bindings
                .insert((kb.code.0, kb.modifiers.0), Action::GoToView(4));
        }
    }

    pub fn get(&self, code: KeyCode, modifiers: KeyModifiers) -> Option<&Action> {
        self.bindings.get(&(code, modifiers))
    }

    pub fn config(&self) -> &KeybindConfig {
        &self.config
    }

    pub fn set_binding(&mut self, action_name: &str, bindings: Vec<KeyBinding>) {
        match action_name {
            "quit" => self.config.quit = bindings,
            "quit_all" => self.config.quit_all = bindings,
            "next_view" => self.config.next_view = bindings,
            "prev_view" => self.config.prev_view = bindings,
            "up" => self.config.up = bindings,
            "down" => self.config.down = bindings,
            "left" => self.config.left = bindings,
            "right" => self.config.right = bindings,
            "top" => self.config.top = bindings,
            "bottom" => self.config.bottom = bindings,
            "page_up" => self.config.page_up = bindings,
            "page_down" => self.config.page_down = bindings,
            "select" => self.config.select = bindings,
            "back" => self.config.back = bindings,
            "search" => self.config.search = bindings,
            "command" => self.config.command = bindings,
            "refresh" => self.config.refresh = bindings,
            "toggle_theme" => self.config.toggle_theme = bindings,
            "help" => self.config.help = bindings,
            "sort" => self.config.sort = bindings,
            "filter" => self.config.filter = bindings,
            "view_dashboard" => self.config.view_dashboard = bindings,
            "view_agents" => self.config.view_agents = bindings,
            "view_sessions" => self.config.view_sessions = bindings,
            "view_costs" => self.config.view_costs = bindings,
            "view_metrics" => self.config.view_metrics = bindings,
            _ => {}
        }
        self.rebuild_bindings(&self.config.clone());
    }

    pub fn all_bindings(&self) -> Vec<(String, String, Action)> {
        vec![
            (
                Self::keybindings_to_string(&self.config.quit),
                "Quit".to_string(),
                Action::Quit,
            ),
            (
                Self::keybindings_to_string(&self.config.next_view),
                "Next view".to_string(),
                Action::NextView,
            ),
            (
                Self::keybindings_to_string(&self.config.prev_view),
                "Previous view".to_string(),
                Action::PrevView,
            ),
            (
                Self::keybindings_to_string(&self.config.up),
                "Move up".to_string(),
                Action::Up,
            ),
            (
                Self::keybindings_to_string(&self.config.down),
                "Move down".to_string(),
                Action::Down,
            ),
            (
                Self::keybindings_to_string(&self.config.top),
                "Go to top".to_string(),
                Action::Top,
            ),
            (
                Self::keybindings_to_string(&self.config.bottom),
                "Go to bottom".to_string(),
                Action::Bottom,
            ),
            (
                Self::keybindings_to_string(&self.config.select),
                "Select/Enter".to_string(),
                Action::Select,
            ),
            (
                Self::keybindings_to_string(&self.config.back),
                "Back/Cancel".to_string(),
                Action::Back,
            ),
            (
                Self::keybindings_to_string(&self.config.search),
                "Search".to_string(),
                Action::Search,
            ),
            (
                Self::keybindings_to_string(&self.config.command),
                "Command palette".to_string(),
                Action::OpenCommandPalette,
            ),
            (
                Self::keybindings_to_string(&self.config.refresh),
                "Refresh".to_string(),
                Action::Refresh,
            ),
            (
                Self::keybindings_to_string(&self.config.toggle_theme),
                "Toggle theme".to_string(),
                Action::ToggleTheme,
            ),
            (
                Self::keybindings_to_string(&self.config.help),
                "Help".to_string(),
                Action::Help,
            ),
        ]
    }

    fn keybinding_to_string(kb: &KeyBinding) -> String {
        let mut parts = Vec::new();
        if kb.modifiers.0.contains(KeyModifiers::CONTROL) {
            parts.push("Ctrl".to_string());
        }
        if kb.modifiers.0.contains(KeyModifiers::ALT) {
            parts.push("Alt".to_string());
        }
        if kb.modifiers.0.contains(KeyModifiers::SHIFT) {
            parts.push("Shift".to_string());
        }
        parts.push(Self::keycode_to_string(&kb.code.0));
        parts.join("+")
    }

    fn keybindings_to_string(bindings: &[KeyBinding]) -> String {
        bindings
            .iter()
            .map(Self::keybinding_to_string)
            .collect::<Vec<_>>()
            .join(" / ")
    }

    fn keycode_to_string(key: &KeyCode) -> String {
        match key {
            KeyCode::Char(c) => c.to_string(),
            KeyCode::Tab => "Tab".to_string(),
            KeyCode::BackTab => "Shift+Tab".to_string(),
            KeyCode::Enter => "Enter".to_string(),
            KeyCode::Esc => "Esc".to_string(),
            KeyCode::Up => "↑".to_string(),
            KeyCode::Down => "↓".to_string(),
            KeyCode::Left => "←".to_string(),
            KeyCode::Right => "→".to_string(),
            KeyCode::PageUp => "PgUp".to_string(),
            KeyCode::PageDown => "PgDn".to_string(),
            KeyCode::Home => "Home".to_string(),
            KeyCode::End => "End".to_string(),
            KeyCode::F(n) => format!("F{}", n),
            _ => format!("{:?}", key),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_keybinds() {
        let keybinds = Keybinds::new();

        assert_eq!(
            keybinds.get(KeyCode::Char('q'), KeyModifiers::NONE),
            Some(&Action::Quit)
        );
        assert_eq!(
            keybinds.get(KeyCode::Char('c'), KeyModifiers::CONTROL),
            Some(&Action::Quit)
        );
        assert_eq!(
            keybinds.get(KeyCode::Tab, KeyModifiers::NONE),
            Some(&Action::NextView)
        );
        assert_eq!(
            keybinds.get(KeyCode::Char('j'), KeyModifiers::NONE),
            Some(&Action::Down)
        );
        assert_eq!(
            keybinds.get(KeyCode::Down, KeyModifiers::NONE),
            Some(&Action::Down)
        );
    }

    #[test]
    fn test_keybinding_serialization() {
        let kb = KeyBinding::new(KeyCode::Char('q'));
        let serialized = toml::to_string(&kb).unwrap();
        assert!(serialized.contains("Char(q)"));

        let kb_ctrl = KeyBinding::with_ctrl(KeyCode::Char('c'));
        let serialized_ctrl = toml::to_string(&kb_ctrl).unwrap();
        assert!(serialized_ctrl.contains("Ctrl"));
    }

    #[test]
    fn test_keybind_config_serialization() {
        let config = KeybindConfig::default();
        let serialized = toml::to_string_pretty(&config).unwrap();

        let deserialized: KeybindConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.quit.len(), config.quit.len());
    }

    #[test]
    fn test_parse_key_code() {
        assert_eq!(parse_key_code("Char(q)"), Some(KeyCode::Char('q')));
        assert_eq!(parse_key_code("Tab"), Some(KeyCode::Tab));
        assert_eq!(parse_key_code("Enter"), Some(KeyCode::Enter));
        assert_eq!(parse_key_code("Esc"), Some(KeyCode::Esc));
        assert_eq!(parse_key_code("Up"), Some(KeyCode::Up));
        assert_eq!(parse_key_code("F1"), Some(KeyCode::F(1)));
        assert_eq!(parse_key_code("F12"), Some(KeyCode::F(12)));
    }

    #[test]
    fn test_parse_modifiers() {
        assert_eq!(parse_modifiers("None"), KeyModifiers::NONE);
        assert_eq!(parse_modifiers("Ctrl"), KeyModifiers::CONTROL);
        assert_eq!(parse_modifiers("Alt"), KeyModifiers::ALT);
        assert_eq!(parse_modifiers("Shift"), KeyModifiers::SHIFT);
        assert_eq!(
            parse_modifiers("Ctrl+Shift"),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT
        );
    }

    #[test]
    fn test_set_binding() {
        let mut keybinds = Keybinds::new();

        keybinds.set_binding("quit", vec![KeyBinding::new(KeyCode::Char('x'))]);

        assert_eq!(
            keybinds.get(KeyCode::Char('x'), KeyModifiers::NONE),
            Some(&Action::Quit)
        );
        assert_eq!(keybinds.get(KeyCode::Char('q'), KeyModifiers::NONE), None);
    }

    #[test]
    fn test_all_bindings() {
        let keybinds = Keybinds::new();
        let all = keybinds.all_bindings();

        assert!(!all.is_empty());
        assert!(all.iter().any(|(key, desc, _)| desc == "Quit"));
        assert!(all.iter().any(|(key, desc, _)| desc == "Help"));
    }

    #[test]
    fn test_keybinding_to_string() {
        let kb = KeyBinding::new(KeyCode::Char('q'));
        assert_eq!(Keybinds::keybinding_to_string(&kb), "q");

        let kb_ctrl = KeyBinding::with_ctrl(KeyCode::Char('c'));
        assert_eq!(Keybinds::keybinding_to_string(&kb_ctrl), "Ctrl+c");

        let kb_tab = KeyBinding::new(KeyCode::Tab);
        assert_eq!(Keybinds::keybinding_to_string(&kb_tab), "Tab");
    }
}
