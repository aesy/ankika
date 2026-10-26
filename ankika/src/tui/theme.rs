use std::env;

use ratatui::style::Color;

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub background: Color,
    pub surface: Color,
    pub border: Color,
    pub text: Color,
    pub muted: Color,
    pub accent: Color,
}

const TRUE_COLOR: Theme = Theme {
    background: Color::Rgb(24, 25, 33),
    surface: Color::Rgb(36, 38, 50),
    border: Color::Rgb(68, 71, 90),
    text: Color::Rgb(220, 223, 228),
    muted: Color::Rgb(128, 133, 150),
    accent: Color::Rgb(130, 170, 255),
};

/// Closest matches in the xterm 256-color palette.
const INDEXED: Theme = Theme {
    background: Color::Indexed(234),
    surface: Color::Indexed(236),
    border: Color::Indexed(239),
    text: Color::Indexed(253),
    muted: Color::Indexed(244),
    accent: Color::Indexed(111),
};

impl Theme {
    pub fn detect() -> Self {
        // `COLORTERM` is the de facto way terminals advertise 24-bit color
        let true_color =
            env::var("COLORTERM").is_ok_and(|value| value == "truecolor" || value == "24bit");

        if true_color { TRUE_COLOR } else { INDEXED }
    }
}
