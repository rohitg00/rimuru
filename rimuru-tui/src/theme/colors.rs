use ratatui::style::Color;

pub struct ColorPalette {
    pub background: Color,
    pub foreground: Color,
    pub foreground_dim: Color,
    pub surface: Color,
    pub border: Color,
    pub selection: Color,
    pub accent: Color,
    pub accent_secondary: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
}

impl ColorPalette {
    pub fn from_hex(
        background: u32,
        foreground: u32,
        foreground_dim: u32,
        surface: u32,
        border: u32,
        selection: u32,
        accent: u32,
        accent_secondary: u32,
        success: u32,
        warning: u32,
        error: u32,
        info: u32,
    ) -> Self {
        Self {
            background: hex_to_color(background),
            foreground: hex_to_color(foreground),
            foreground_dim: hex_to_color(foreground_dim),
            surface: hex_to_color(surface),
            border: hex_to_color(border),
            selection: hex_to_color(selection),
            accent: hex_to_color(accent),
            accent_secondary: hex_to_color(accent_secondary),
            success: hex_to_color(success),
            warning: hex_to_color(warning),
            error: hex_to_color(error),
            info: hex_to_color(info),
        }
    }
}

pub fn hex_to_color(hex: u32) -> Color {
    let r = ((hex >> 16) & 0xFF) as u8;
    let g = ((hex >> 8) & 0xFF) as u8;
    let b = (hex & 0xFF) as u8;
    Color::Rgb(r, g, b)
}
