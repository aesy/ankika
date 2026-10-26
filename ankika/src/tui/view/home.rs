use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType};

use crate::i18n::I18n;
use crate::tui::deck_list::Row;
use crate::tui::hit_map::{Hit, HitMap};
use crate::tui::state::State;
use crate::tui::theme::Theme;

const COUNT_WIDTH: u16 = 7;
const INDENT: usize = 2;
const TOGGLE_WIDTH: u16 = 2;

pub fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &State,
    i18n: &I18n,
    theme: Theme,
    hits: &mut HitMap,
) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme.border))
        .title(format!(" {} ", i18n.anki().actions_decks()));
    let inner = block.inner(area);

    frame.render_widget(block, area);

    let [heading_area, list_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(inner);
    let heading = [
        format!(" {}", i18n.anki().decks_deck()).into(),
        i18n.anki().actions_new(),
        i18n.anki().decks_learn_header(),
        i18n.anki().decks_review_header(),
    ]
    .map(|text| Line::styled(text, theme.muted));

    render_columns(frame, heading_area, heading);

    let rows = state.decks().rows();
    let selected = rows
        .iter()
        .position(|row| Some(row.deck.id) == state.decks().selected());
    let height = usize::from(list_area.height);
    // Scrolls just far enough to keep the selection on screen
    let offset = selected.map_or(0, |index| (index + 1).saturating_sub(height));
    let row_areas = Layout::vertical(vec![Constraint::Length(1); height]).split(list_area);

    for ((index, row), &row_area) in rows.iter().enumerate().skip(offset).zip(row_areas.iter()) {
        let hovered = state
            .mouse()
            .is_some_and(|position| row_area.contains(position));

        render_row(frame, row_area, row, theme, hits);

        let style = if Some(index) == selected {
            Style::new().bg(theme.border).add_modifier(Modifier::BOLD)
        } else if hovered {
            Style::new().bg(theme.surface)
        } else {
            Style::new()
        };

        frame.buffer_mut().set_style(row_area, style);
    }

    if state.decks_stale() {
        frame
            .buffer_mut()
            .set_style(inner, Style::new().add_modifier(Modifier::DIM));
    }
}

fn render_row(frame: &mut Frame<'_>, area: Rect, row: &Row<'_>, theme: Theme, hits: &mut HitMap) {
    let deck = row.deck;
    let indent = " ".repeat(row.depth * INDENT);
    let toggle = match (deck.children.is_empty(), deck.collapsed) {
        (true, _) => "  ",
        (false, true) => "▸ ",
        (false, false) => "▾ ",
    };
    let name_color = if deck.filtered {
        theme.filtered
    } else {
        theme.text
    };
    let name = Line::from(vec![
        Span::raw(format!(" {indent}")),
        Span::styled(toggle, Style::new().fg(theme.muted)),
        Span::styled(deck.name.as_str(), Style::new().fg(name_color)),
    ]);

    render_columns(
        frame,
        area,
        [
            name,
            count(deck.new_count, theme.new_count, theme),
            count(deck.learn_count, theme.learn_count, theme),
            count(deck.review_count, theme.review_count, theme),
        ],
    );

    hits.register(area, Hit::Deck(deck.id));

    if !deck.children.is_empty() {
        let indent_width = u16::try_from(row.depth * INDENT + 1).unwrap_or(u16::MAX);
        let toggle_area = Rect {
            x: area.x.saturating_add(indent_width),
            width: TOGGLE_WIDTH,
            ..area
        }
        .intersection(area);

        hits.register(toggle_area, Hit::DeckToggle(deck.id));
    }
}

fn count(value: u32, color: Color, theme: Theme) -> Line<'static> {
    let color = if value == 0 { theme.muted } else { color };

    Line::styled(value.to_string(), color)
}

/// The name fills the row; the three counts are right-aligned columns.
fn render_columns(frame: &mut Frame<'_>, area: Rect, cells: [Line<'_>; 4]) {
    let [name, new, learn, review] = cells;
    let [name_area, new_area, learn_area, review_area, _] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(COUNT_WIDTH),
        Constraint::Length(COUNT_WIDTH),
        Constraint::Length(COUNT_WIDTH),
        Constraint::Length(1),
    ])
    .areas(area);

    frame.render_widget(name, name_area);
    frame.render_widget(new.right_aligned(), new_area);
    frame.render_widget(learn.right_aligned(), learn_area);
    frame.render_widget(review.right_aligned(), review_area);
}
