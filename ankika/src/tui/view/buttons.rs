use ratatui::Frame;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;

use crate::tui::hit_map::{Hit, HitMap};
use crate::tui::theme::Theme;

const GAP: u16 = 1;

pub struct Button {
    pub label: String,
    pub hit: Hit,
    pub enabled: bool,
    pub focused: bool,
}

/// Draws the buttons in order, packed against the right edge of `area`.
pub fn render_right_aligned(
    frame: &mut Frame<'_>,
    area: Rect,
    buttons: &[Button],
    mouse: Option<Position>,
    theme: Theme,
    hits: &mut HitMap,
) {
    let mut right = area.right();

    for button in buttons.iter().rev() {
        let line = Line::raw(format!(" {} ", button.label));
        let width = u16::try_from(line.width()).unwrap_or(u16::MAX);
        let button_area = Rect {
            x: right.saturating_sub(width),
            width,
            ..area
        }
        .intersection(area);
        let hovered = mouse.is_some_and(|position| button_area.contains(position));
        let style = if !button.enabled {
            Style::new().bg(theme.border).fg(theme.muted)
        } else if button.focused || hovered {
            Style::new()
                .bg(theme.accent)
                .fg(theme.background)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::new().bg(theme.border).fg(theme.text)
        };

        frame.render_widget(line.style(style), button_area);
        hits.register(button_area, button.hit);
        right = button_area.x.saturating_sub(GAP);
    }
}
