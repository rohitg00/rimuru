use ratatui::style::Color;

use super::{colors::hex_to_color, Theme};

pub struct Dracula;

impl Theme for Dracula {
    fn name(&self) -> &'static str {
        "Dracula"
    }

    fn background(&self) -> Color {
        hex_to_color(0x282a36)
    }

    fn foreground(&self) -> Color {
        hex_to_color(0xf8f8f2)
    }

    fn foreground_dim(&self) -> Color {
        hex_to_color(0x6272a4)
    }

    fn surface(&self) -> Color {
        hex_to_color(0x44475a)
    }

    fn border(&self) -> Color {
        hex_to_color(0x6272a4)
    }

    fn selection(&self) -> Color {
        hex_to_color(0x44475a)
    }

    fn accent(&self) -> Color {
        hex_to_color(0xbd93f9)
    }

    fn accent_secondary(&self) -> Color {
        hex_to_color(0xff79c6)
    }

    fn success(&self) -> Color {
        hex_to_color(0x50fa7b)
    }

    fn warning(&self) -> Color {
        hex_to_color(0xf1fa8c)
    }

    fn error(&self) -> Color {
        hex_to_color(0xff5555)
    }

    fn info(&self) -> Color {
        hex_to_color(0x8be9fd)
    }
}
