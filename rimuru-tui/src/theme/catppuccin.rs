use ratatui::style::Color;

use super::{colors::hex_to_color, Theme};

pub struct CatppuccinMocha;

impl Theme for CatppuccinMocha {
    fn name(&self) -> &'static str {
        "Catppuccin Mocha"
    }

    fn background(&self) -> Color {
        hex_to_color(0x1e1e2e)
    }

    fn foreground(&self) -> Color {
        hex_to_color(0xcdd6f4)
    }

    fn foreground_dim(&self) -> Color {
        hex_to_color(0x6c7086)
    }

    fn surface(&self) -> Color {
        hex_to_color(0x313244)
    }

    fn border(&self) -> Color {
        hex_to_color(0x45475a)
    }

    fn selection(&self) -> Color {
        hex_to_color(0x585b70)
    }

    fn accent(&self) -> Color {
        hex_to_color(0xcba6f7)
    }

    fn accent_secondary(&self) -> Color {
        hex_to_color(0xf5c2e7)
    }

    fn success(&self) -> Color {
        hex_to_color(0xa6e3a1)
    }

    fn warning(&self) -> Color {
        hex_to_color(0xf9e2af)
    }

    fn error(&self) -> Color {
        hex_to_color(0xf38ba8)
    }

    fn info(&self) -> Color {
        hex_to_color(0x89b4fa)
    }
}

pub struct CatppuccinLatte;

impl Theme for CatppuccinLatte {
    fn name(&self) -> &'static str {
        "Catppuccin Latte"
    }

    fn background(&self) -> Color {
        hex_to_color(0xeff1f5)
    }

    fn foreground(&self) -> Color {
        hex_to_color(0x4c4f69)
    }

    fn foreground_dim(&self) -> Color {
        hex_to_color(0x9ca0b0)
    }

    fn surface(&self) -> Color {
        hex_to_color(0xe6e9ef)
    }

    fn border(&self) -> Color {
        hex_to_color(0xbcc0cc)
    }

    fn selection(&self) -> Color {
        hex_to_color(0xdce0e8)
    }

    fn accent(&self) -> Color {
        hex_to_color(0x8839ef)
    }

    fn accent_secondary(&self) -> Color {
        hex_to_color(0xea76cb)
    }

    fn success(&self) -> Color {
        hex_to_color(0x40a02b)
    }

    fn warning(&self) -> Color {
        hex_to_color(0xdf8e1d)
    }

    fn error(&self) -> Color {
        hex_to_color(0xd20f39)
    }

    fn info(&self) -> Color {
        hex_to_color(0x1e66f5)
    }
}
