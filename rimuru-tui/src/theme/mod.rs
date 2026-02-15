mod catppuccin;
mod colors;
mod dracula;
mod gruvbox;
mod loader;
mod nord;
mod styles;
mod tokyo_night;

pub use catppuccin::{CatppuccinLatte, CatppuccinMocha};
pub use colors::ColorPalette;
pub use dracula::Dracula;
pub use gruvbox::{GruvboxDark, GruvboxLight};
pub use loader::{ThemeConfig, ThemeLoader};
pub use nord::Nord;
pub use styles::{create_status_style, format_with_icon, highlight_matches, ThemedStyles};
pub use tokyo_night::TokyoNight;

use ratatui::style::Color;

pub trait Theme: Send + Sync {
    fn name(&self) -> &'static str;

    fn background(&self) -> Color;
    fn foreground(&self) -> Color;
    fn foreground_dim(&self) -> Color;

    fn surface(&self) -> Color;
    fn border(&self) -> Color;
    fn selection(&self) -> Color;

    fn accent(&self) -> Color;
    fn accent_secondary(&self) -> Color;

    fn success(&self) -> Color;
    fn warning(&self) -> Color;
    fn error(&self) -> Color;
    fn info(&self) -> Color;
}

pub struct ThemeManager {
    themes: Vec<Box<dyn Theme>>,
    current_index: usize,
}

impl ThemeManager {
    pub fn new() -> Self {
        let themes: Vec<Box<dyn Theme>> = vec![
            Box::new(TokyoNight),
            Box::new(CatppuccinMocha),
            Box::new(CatppuccinLatte),
            Box::new(Dracula),
            Box::new(Nord),
            Box::new(GruvboxDark),
            Box::new(GruvboxLight),
        ];

        Self {
            themes,
            current_index: 0,
        }
    }

    pub fn current_theme(&self) -> &dyn Theme {
        self.themes[self.current_index].as_ref()
    }

    pub fn cycle_theme(&mut self) {
        self.current_index = (self.current_index + 1) % self.themes.len();
    }

    pub fn set_theme_by_name(&mut self, name: &str) -> bool {
        if let Some(index) = self.themes.iter().position(|t| t.name() == name) {
            self.current_index = index;
            true
        } else {
            false
        }
    }

    pub fn available_themes(&self) -> Vec<&'static str> {
        self.themes.iter().map(|t| t.name()).collect()
    }

    pub fn current_theme_name(&self) -> &'static str {
        self.current_theme().name()
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}
