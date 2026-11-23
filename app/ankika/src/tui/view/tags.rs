use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::style::Style;
use ratatui::text::Line;

use crate::i18n::I18n;
use crate::tui::hit_map::HitMap;
use crate::tui::tag_input::TagInput;
use crate::tui::theme::Theme;
use crate::tui::view::tag_input;

const WIDTH: u16 = 60;
// Borders, margins and the input
const FIXED_HEIGHT: u16 = 5;

/// Anki's Add Tags and Remove Tags dialogs: one input, Enter applies.
pub fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    remove: bool,
    input: &TagInput,
    i18n: &I18n,
    theme: Theme,
    hits: &mut HitMap,
) {
    let completions = u16::try_from(input.completions().len()).unwrap_or(0);
    let popup = area.centered(
        Constraint::Length(WIDTH),
        Constraint::Length(FIXED_HEIGHT.saturating_add(completions)),
    );
    let title = if remove {
        i18n.anki().browsing_remove_tags()
    } else {
        i18n.anki().browsing_add_tags2()
    };
    let inner = super::overlay(frame, popup, &title, theme, hits).inner(Margin::new(2, 1));
    let [input_area, completion_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Length(completions)]).areas(inner);

    frame.render_widget(
        Line::raw(format!(" {}_", input.text())).style(Style::new().bg(theme.border)),
        input_area,
    );
    tag_input::render_completions(frame, completion_area, input, 0, theme, hits);
}
