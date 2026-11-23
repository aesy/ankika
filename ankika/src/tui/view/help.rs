use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::i18n::I18n;
use crate::tui::hit_map::HitMap;
use crate::tui::keymap::{self, Context};
use crate::tui::theme::Theme;

const WIDTH: u16 = 44;
const KEY_COLUMN: usize = 14;

/// Lists the main screen's bindings.
pub fn render(frame: &mut Frame<'_>, area: Rect, i18n: &I18n, theme: Theme, hits: &mut HitMap) {
    let lines: Vec<Line<'_>> = keymap::grouped(Context::Main)
        .into_iter()
        .map(|(action, labels)| {
            Line::from(vec![
                Span::styled(
                    format!(" {:<KEY_COLUMN$}", labels.join(" / ")),
                    Style::new().fg(theme.accent).add_modifier(Modifier::BOLD),
                ),
                Span::raw(action.label(i18n)),
            ])
        })
        .collect();

    // Borders plus one blank line above and below the list
    let height = u16::try_from(lines.len())
        .unwrap_or(u16::MAX)
        .saturating_add(4);
    let popup = area.centered(Constraint::Length(WIDTH), Constraint::Length(height));
    let inner = super::overlay(frame, popup, &i18n.text("help-title"), theme, hits);
    let list = Rect {
        y: inner.y.saturating_add(1),
        height: inner.height.saturating_sub(2),
        ..inner
    };

    frame.render_widget(Paragraph::new(lines), list);
}
