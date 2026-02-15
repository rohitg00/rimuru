use ratatui::style::Color;

use super::{colors::hex_to_color, Theme};

pub struct TokyoNight;

impl Theme for TokyoNight {
    fn name(&self) -> &'static str {
        "Tokyo Night"
    }

    fn background(&self) -> Color {
        hex_to_color(0x1a1b26)
    }

    fn foreground(&self) -> Color {
        hex_to_color(0xc0caf5)
    }

    fn foreground_dim(&self) -> Color {
        hex_to_color(0x565f89)
    }

    fn surface(&self) -> Color {
        hex_to_color(0x24283b)
    }

    fn border(&self) -> Color {
        hex_to_color(0x414868)
    }

    fn selection(&self) -> Color {
        hex_to_color(0x364a82)
    }

    fn accent(&self) -> Color {
        hex_to_color(0x7aa2f7)
    }

    fn accent_secondary(&self) -> Color {
        hex_to_color(0xbb9af7)
    }

    fn success(&self) -> Color {
        hex_to_color(0x9ece6a)
    }

    fn warning(&self) -> Color {
        hex_to_color(0xe0af68)
    }

    fn error(&self) -> Color {
        hex_to_color(0xf7768e)
    }

    fn info(&self) -> Color {
        hex_to_color(0x7dcfff)
    }
}
