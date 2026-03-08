use ratatui::style::Color;

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub name: &'static str,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    pub border: Color,
    pub text: Color,
    pub text_dim: Color,
    pub selection: Color,
    pub gauge_low: Color,
    pub gauge_mid: Color,
    pub gauge_high: Color,
}

pub const RIMURU_SLIME: Theme = Theme {
    name: "Rimuru Slime",
    accent: Color::Rgb(100, 180, 255),
    success: Color::Rgb(80, 220, 140),
    warning: Color::Rgb(255, 200, 80),
    error: Color::Rgb(255, 100, 100),
    info: Color::Rgb(140, 180, 255),
    border: Color::Rgb(60, 80, 110),
    text: Color::Rgb(220, 230, 245),
    text_dim: Color::Rgb(100, 115, 140),
    selection: Color::Rgb(40, 60, 90),
    gauge_low: Color::Rgb(80, 220, 140),
    gauge_mid: Color::Rgb(255, 200, 80),
    gauge_high: Color::Rgb(255, 100, 100),
};

pub const GREAT_SAGE: Theme = Theme {
    name: "Great Sage",
    accent: Color::Rgb(200, 160, 255),
    success: Color::Rgb(140, 230, 160),
    warning: Color::Rgb(255, 210, 100),
    error: Color::Rgb(255, 120, 120),
    info: Color::Rgb(180, 160, 255),
    border: Color::Rgb(70, 60, 100),
    text: Color::Rgb(230, 225, 245),
    text_dim: Color::Rgb(120, 110, 150),
    selection: Color::Rgb(50, 40, 80),
    gauge_low: Color::Rgb(140, 230, 160),
    gauge_mid: Color::Rgb(255, 210, 100),
    gauge_high: Color::Rgb(255, 120, 120),
};

pub const PREDATOR: Theme = Theme {
    name: "Predator",
    accent: Color::Rgb(255, 80, 80),
    success: Color::Rgb(200, 80, 80),
    warning: Color::Rgb(255, 160, 60),
    error: Color::Rgb(255, 50, 50),
    info: Color::Rgb(255, 120, 100),
    border: Color::Rgb(100, 40, 40),
    text: Color::Rgb(245, 220, 220),
    text_dim: Color::Rgb(140, 100, 100),
    selection: Color::Rgb(80, 30, 30),
    gauge_low: Color::Rgb(200, 80, 80),
    gauge_mid: Color::Rgb(255, 160, 60),
    gauge_high: Color::Rgb(255, 50, 50),
};

pub const VELDORA: Theme = Theme {
    name: "Veldora",
    accent: Color::Rgb(255, 200, 50),
    success: Color::Rgb(100, 220, 100),
    warning: Color::Rgb(255, 180, 50),
    error: Color::Rgb(255, 80, 60),
    info: Color::Rgb(255, 220, 100),
    border: Color::Rgb(100, 80, 30),
    text: Color::Rgb(245, 240, 220),
    text_dim: Color::Rgb(140, 130, 100),
    selection: Color::Rgb(70, 60, 20),
    gauge_low: Color::Rgb(100, 220, 100),
    gauge_mid: Color::Rgb(255, 180, 50),
    gauge_high: Color::Rgb(255, 80, 60),
};

pub const SHION: Theme = Theme {
    name: "Shion",
    accent: Color::Rgb(180, 100, 255),
    success: Color::Rgb(160, 220, 160),
    warning: Color::Rgb(255, 180, 120),
    error: Color::Rgb(255, 100, 130),
    info: Color::Rgb(200, 140, 255),
    border: Color::Rgb(80, 50, 120),
    text: Color::Rgb(235, 225, 250),
    text_dim: Color::Rgb(130, 110, 160),
    selection: Color::Rgb(60, 35, 90),
    gauge_low: Color::Rgb(160, 220, 160),
    gauge_mid: Color::Rgb(255, 180, 120),
    gauge_high: Color::Rgb(255, 100, 130),
};

pub const MILIM: Theme = Theme {
    name: "Milim",
    accent: Color::Rgb(255, 120, 180),
    success: Color::Rgb(255, 180, 200),
    warning: Color::Rgb(255, 200, 100),
    error: Color::Rgb(255, 60, 100),
    info: Color::Rgb(255, 160, 200),
    border: Color::Rgb(120, 50, 80),
    text: Color::Rgb(250, 230, 240),
    text_dim: Color::Rgb(160, 110, 130),
    selection: Color::Rgb(90, 35, 60),
    gauge_low: Color::Rgb(255, 180, 200),
    gauge_mid: Color::Rgb(255, 200, 100),
    gauge_high: Color::Rgb(255, 60, 100),
};

pub const DIABLO: Theme = Theme {
    name: "Diablo",
    accent: Color::Rgb(180, 160, 140),
    success: Color::Rgb(160, 180, 160),
    warning: Color::Rgb(200, 180, 100),
    error: Color::Rgb(200, 80, 80),
    info: Color::Rgb(180, 170, 160),
    border: Color::Rgb(60, 55, 50),
    text: Color::Rgb(210, 200, 190),
    text_dim: Color::Rgb(110, 105, 100),
    selection: Color::Rgb(45, 40, 35),
    gauge_low: Color::Rgb(160, 180, 160),
    gauge_mid: Color::Rgb(200, 180, 100),
    gauge_high: Color::Rgb(200, 80, 80),
};

pub static ALL_THEMES: &[Theme] = &[
    RIMURU_SLIME,
    GREAT_SAGE,
    PREDATOR,
    VELDORA,
    SHION,
    MILIM,
    DIABLO,
];

pub fn theme_by_index(idx: usize) -> &'static Theme {
    &ALL_THEMES[idx % ALL_THEMES.len()]
}
