use std::env;

use ankika_core::card_actions::Flag;
use ratatui::style::Color;

use crate::config::ThemeName;

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub background: Color,
    pub surface: Color,
    pub border: Color,
    pub text: Color,
    pub muted: Color,
    pub accent: Color,
    pub error: Color,
    pub new_count: Color,
    pub learn_count: Color,
    pub review_count: Color,
    pub filtered: Color,
}

const DARK_TRUE_COLOR: Theme = Theme {
    background: Color::Rgb(24, 25, 33),
    surface: Color::Rgb(36, 38, 50),
    border: Color::Rgb(68, 71, 90),
    text: Color::Rgb(220, 223, 228),
    muted: Color::Rgb(128, 133, 150),
    accent: Color::Rgb(130, 170, 255),
    error: Color::Rgb(240, 113, 120),
    new_count: Color::Rgb(96, 165, 250),
    learn_count: Color::Rgb(248, 113, 113),
    review_count: Color::Rgb(74, 222, 128),
    filtered: Color::Rgb(192, 132, 252),
};

/// Closest matches in the xterm 256-color palette.
const DARK_INDEXED: Theme = Theme {
    background: Color::Indexed(234),
    surface: Color::Indexed(236),
    border: Color::Indexed(239),
    text: Color::Indexed(253),
    muted: Color::Indexed(244),
    accent: Color::Indexed(111),
    error: Color::Indexed(203),
    new_count: Color::Indexed(75),
    learn_count: Color::Indexed(210),
    review_count: Color::Indexed(114),
    filtered: Color::Indexed(177),
};

const LIGHT_TRUE_COLOR: Theme = Theme {
    background: Color::Rgb(250, 250, 252),
    surface: Color::Rgb(236, 238, 244),
    border: Color::Rgb(200, 204, 216),
    text: Color::Rgb(32, 34, 44),
    muted: Color::Rgb(110, 115, 132),
    accent: Color::Rgb(37, 99, 235),
    error: Color::Rgb(200, 40, 50),
    new_count: Color::Rgb(29, 78, 216),
    learn_count: Color::Rgb(185, 28, 28),
    review_count: Color::Rgb(21, 128, 61),
    filtered: Color::Rgb(126, 34, 206),
};

/// Closest matches in the xterm 256-color palette.
const LIGHT_INDEXED: Theme = Theme {
    background: Color::Indexed(255),
    surface: Color::Indexed(254),
    border: Color::Indexed(250),
    text: Color::Indexed(235),
    muted: Color::Indexed(243),
    accent: Color::Indexed(26),
    error: Color::Indexed(160),
    new_count: Color::Indexed(25),
    learn_count: Color::Indexed(124),
    review_count: Color::Indexed(28),
    filtered: Color::Indexed(91),
};

impl Theme {
    /// The palette for the setting, in 24-bit color when the terminal says it has it.
    pub fn new(name: ThemeName) -> Self {
        // `COLORTERM` is the de facto way terminals advertise 24-bit color
        let true_color =
            env::var("COLORTERM").is_ok_and(|value| value == "truecolor" || value == "24bit");

        match (name, true_color) {
            (ThemeName::Dark, true) => DARK_TRUE_COLOR,
            (ThemeName::Dark, false) => DARK_INDEXED,
            (ThemeName::Light, true) => LIGHT_TRUE_COLOR,
            (ThemeName::Light, false) => LIGHT_INDEXED,
        }
    }
}

/// Close to the colors of Anki's flags.
pub const fn flag_color(flag: Flag) -> Color {
    match flag {
        Flag::Red => Color::Rgb(0xff, 0x6b, 0x6b),
        Flag::Orange => Color::Rgb(0xff, 0xa9, 0x4d),
        Flag::Green => Color::Rgb(0x51, 0xcf, 0x66),
        Flag::Blue => Color::Rgb(0x4d, 0xab, 0xf7),
        Flag::Pink => Color::Rgb(0xf7, 0x83, 0xac),
        Flag::Turquoise => Color::Rgb(0x3b, 0xc9, 0xdb),
        Flag::Purple => Color::Rgb(0x97, 0x75, 0xfa),
    }
}
