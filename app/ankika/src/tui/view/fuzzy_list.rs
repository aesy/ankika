use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::i18n::I18n;
use crate::tui::fuzzy_list::FuzzyList;
use crate::tui::hit_map::{Hit, HitMap};
use crate::tui::theme::Theme;

const WIDTH: u16 = 60;
const MAX_ROWS: u16 = 8;
const TOP_MARGIN: u16 = 2;
const PROMPT: &str = " > ";

/// What a list overlay shows around its entries.
pub struct Labels<'a> {
    pub title: &'a str,
    /// Shown in the input before anything is typed.
    pub placeholder: &'a str,
}

/// A popup near the top with the query above the matching entries.
pub fn render<T: Clone>(
    frame: &mut Frame<'_>,
    area: Rect,
    list: &FuzzyList<T>,
    labels: &Labels<'_>,
    i18n: &I18n,
    theme: Theme,
    hits: &mut HitMap,
) {
    let rows = u16::try_from(list.matches().len())
        .unwrap_or(MAX_ROWS)
        .clamp(1, MAX_ROWS);
    // Borders, input and separator
    let height = rows.saturating_add(4);
    let popup = Rect {
        y: area.y.saturating_add(TOP_MARGIN),
        height,
        ..area.centered_horizontally(Constraint::Length(WIDTH))
    }
    .intersection(area);
    let inner = super::overlay(frame, popup, labels.title, theme, hits);
    let [input_area, separator_area, list_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(inner);

    render_input(frame, input_area, list, labels.placeholder, theme);
    frame.render_widget(
        Line::styled(
            "\u{2500}".repeat(usize::from(separator_area.width)),
            Style::new().fg(theme.border),
        ),
        separator_area,
    );
    render_matches(frame, list_area, list, i18n, theme, hits);
}

fn render_input<T: Clone>(
    frame: &mut Frame<'_>,
    area: Rect,
    list: &FuzzyList<T>,
    placeholder: &str,
    theme: Theme,
) {
    let prompt = Span::styled(PROMPT, Style::new().fg(theme.accent));
    let line = if list.query().is_empty() {
        Line::from(vec![
            prompt,
            Span::styled(placeholder.to_owned(), Style::new().fg(theme.muted)),
        ])
    } else {
        Line::from(vec![prompt, Span::raw(list.query())])
    };
    let query_width = Line::raw(list.query()).width();
    let cursor_offset = u16::try_from(PROMPT.len().saturating_add(query_width)).unwrap_or(u16::MAX);

    frame.render_widget(line, area);
    frame.set_cursor_position(Position::new(
        area.x
            .saturating_add(cursor_offset)
            .min(area.right().saturating_sub(1)),
        area.y,
    ));
}

fn render_matches<T: Clone>(
    frame: &mut Frame<'_>,
    area: Rect,
    list: &FuzzyList<T>,
    i18n: &I18n,
    theme: Theme,
    hits: &mut HitMap,
) {
    if list.matches().is_empty() {
        frame.render_widget(
            Line::styled(
                format!(" {}", i18n.text("list-no-matches")),
                Style::new().fg(theme.muted),
            ),
            area,
        );
        return;
    }

    // Keep the selection visible once it moves past the last row
    let offset = list
        .selected()
        .saturating_sub(usize::from(area.height.saturating_sub(1)));

    for ((index, entry), row) in list
        .matches()
        .iter()
        .enumerate()
        .skip(offset)
        .zip(0..area.height)
    {
        let row_area = Rect {
            y: area.y.saturating_add(row),
            height: 1,
            ..area
        };
        let style = if index == list.selected() {
            Style::new()
                .bg(theme.accent)
                .fg(theme.background)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::new()
        };

        frame.render_widget(Line::styled(format!(" {}", entry.label), style), row_area);
        hits.register(row_area, Hit::ListEntry(index));
    }
}
