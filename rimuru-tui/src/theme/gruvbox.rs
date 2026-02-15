use ratatui::style::Color;

use super::{colors::hex_to_color, Theme};

pub struct GruvboxDark;

impl Theme for GruvboxDark {
    fn name(&self) -> &'static str {
        "Gruvbox Dark"
    }

    fn background(&self) -> Color {
        hex_to_color(0x282828)
    }

    fn foreground(&self) -> Color {
        hex_to_color(0xebdbb2)
    }

    fn foreground_dim(&self) -> Color {
        hex_to_color(0x928374)
    }

    fn surface(&self) -> Color {
        hex_to_color(0x3c3836)
    }

    fn border(&self) -> Color {
        hex_to_color(0x504945)
    }

    fn selection(&self) -> Color {
        hex_to_color(0x504945)
    }

    fn accent(&self) -> Color {
        hex_to_color(0xfe8019)
    }

    fn accent_secondary(&self) -> Color {
        hex_to_color(0xd3869b)
    }

    fn success(&self) -> Color {
        hex_to_color(0xb8bb26)
    }

    fn warning(&self) -> Color {
        hex_to_color(0xfabd2f)
    }

    fn error(&self) -> Color {
        hex_to_color(0xfb4934)
    }

    fn info(&self) -> Color {
        hex_to_color(0x83a598)
    }
}

pub struct GruvboxLight;

impl Theme for GruvboxLight {
    fn name(&self) -> &'static str {
        "Gruvbox Light"
    }

    fn background(&self) -> Color {
        hex_to_color(0xfbf1c7)
    }

    fn foreground(&self) -> Color {
        hex_to_color(0x3c3836)
    }

    fn foreground_dim(&self) -> Color {
        hex_to_color(0x928374)
    }

    fn surface(&self) -> Color {
        hex_to_color(0xebdbb2)
    }

    fn border(&self) -> Color {
        hex_to_color(0xd5c4a1)
    }

    fn selection(&self) -> Color {
        hex_to_color(0xd5c4a1)
    }

    fn accent(&self) -> Color {
        hex_to_color(0xaf3a03)
    }

    fn accent_secondary(&self) -> Color {
        hex_to_color(0x8f3f71)
    }

    fn success(&self) -> Color {
        hex_to_color(0x79740e)
    }

    fn warning(&self) -> Color {
        hex_to_color(0xb57614)
    }

    fn error(&self) -> Color {
        hex_to_color(0x9d0006)
    }

    fn info(&self) -> Color {
        hex_to_color(0x076678)
    }
}
