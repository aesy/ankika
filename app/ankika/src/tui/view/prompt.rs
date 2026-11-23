use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Position, Rect};
use ratatui::widgets::{Paragraph, Wrap};

use crate::tui::hit_map::HitMap;
use crate::tui::theme::Theme;
use crate::tui::view::buttons::{self, Button};

// Borders, margins, a blank line and the buttons around the text
const FIXED_HEIGHT: u16 = 6;

/// A centered box with wrapped text above a row of buttons.
pub struct Prompt<'a> {
    pub title: &'a str,
    pub text: &'a str,
    pub width: u16,
    /// Fixed, since ratatui can't measure wrapped text yet; must fit the English at `width`.
    pub text_rows: u16,
}

pub fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    prompt: &Prompt<'_>,
    buttons: &[Button],
    mouse: Option<Position>,
    theme: Theme,
    hits: &mut HitMap,
) {
    let popup = area.centered(
        Constraint::Length(prompt.width),
        Constraint::Length(FIXED_HEIGHT.saturating_add(prompt.text_rows)),
    );
    let inner = super::overlay(frame, popup, prompt.title, theme, hits).inner(Margin::new(2, 1));
    let [text_area, _, button_area] = Layout::vertical([
        Constraint::Length(prompt.text_rows),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(inner);

    frame.render_widget(
        Paragraph::new(prompt.text).wrap(Wrap { trim: false }),
        text_area,
    );
    buttons::render_right_aligned(frame, button_area, buttons, mouse, theme, hits);
}
