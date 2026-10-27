use std::env;

use ankika_core::card_actions::Flag;
use ratatui::style::Color;

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

const TRUE_COLOR: Theme = Theme {
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
const INDEXED: Theme = Theme {
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

impl Theme {
    pub fn detect() -> Self {
        // `COLORTERM` is the de facto way terminals advertise 24-bit color
        let true_color =
            env::var("COLORTERM").is_ok_and(|value| value == "truecolor" || value == "24bit");

        if true_color { TRUE_COLOR } else { INDEXED }
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
