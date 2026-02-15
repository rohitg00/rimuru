use ratatui::style::Color;

use super::{colors::hex_to_color, Theme};

pub struct Nord;

impl Theme for Nord {
    fn name(&self) -> &'static str {
        "Nord"
    }

    fn background(&self) -> Color {
        hex_to_color(0x2e3440)
    }

    fn foreground(&self) -> Color {
        hex_to_color(0xeceff4)
    }

    fn foreground_dim(&self) -> Color {
        hex_to_color(0x4c566a)
    }

    fn surface(&self) -> Color {
        hex_to_color(0x3b4252)
    }

    fn border(&self) -> Color {
        hex_to_color(0x4c566a)
    }

    fn selection(&self) -> Color {
        hex_to_color(0x434c5e)
    }

    fn accent(&self) -> Color {
        hex_to_color(0x88c0d0)
    }

    fn accent_secondary(&self) -> Color {
        hex_to_color(0x81a1c1)
    }

    fn success(&self) -> Color {
        hex_to_color(0xa3be8c)
    }

    fn warning(&self) -> Color {
        hex_to_color(0xebcb8b)
    }

    fn error(&self) -> Color {
        hex_to_color(0xbf616a)
    }

    fn info(&self) -> Color {
        hex_to_color(0x5e81ac)
    }
}
