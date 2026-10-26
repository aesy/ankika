use ratatui::Frame;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;

use crate::tui::hit_map::{Hit, HitMap};
use crate::tui::theme::Theme;

pub const GAP: u16 = 1;

pub struct Button {
    pub label: String,
    pub hit: Hit,
    pub enabled: bool,
    pub focused: bool,
}

impl Button {
    pub fn width(&self) -> u16 {
        u16::try_from(self.line().width()).unwrap_or(u16::MAX)
    }

    fn line(&self) -> Line<'_> {
        Line::raw(format!(" {} ", self.label))
    }
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
        let width = button.width();
        let button_area = Rect {
            x: right.saturating_sub(width),
            width,
            ..area
        }
        .intersection(area);

        render(frame, button_area, button, mouse, theme, hits);
        right = button_area.x.saturating_sub(GAP);
    }
}

/// Draws one button filling `area`.
pub fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    button: &Button,
    mouse: Option<Position>,
    theme: Theme,
    hits: &mut HitMap,
) {
    let hovered = mouse.is_some_and(|position| area.contains(position));
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

    frame.render_widget(button.line().style(style).centered(), area);
    hits.register(area, button.hit);
}
