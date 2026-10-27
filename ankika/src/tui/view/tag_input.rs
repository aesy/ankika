use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;

use crate::tui::hit_map::{Hit, HitMap};
use crate::tui::tag_input::TagInput;
use crate::tui::theme::Theme;

/// The suggestions, one per row, starting `indent` columns in to line up with the input.
pub fn render_completions(
    frame: &mut Frame<'_>,
    area: Rect,
    input: &TagInput,
    indent: u16,
    theme: Theme,
    hits: &mut HitMap,
) {
    let rows = Layout::vertical(vec![Constraint::Length(1); input.completions().len()]).split(area);

    for (index, (tag, &row)) in input.completions().iter().zip(rows.iter()).enumerate() {
        let [_, tag_area] =
            Layout::horizontal([Constraint::Length(indent), Constraint::Fill(1)]).areas(row);
        let style = if index == input.completion() {
            Style::new().fg(theme.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(theme.muted)
        };

        frame.render_widget(Line::styled(format!("  {tag}"), style), tag_area);
        hits.register(tag_area, Hit::TagCompletion(index));
    }
}
