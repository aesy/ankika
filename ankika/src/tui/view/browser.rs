use ankika_core::browser::RowColor;
use ankika_core::review;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Paragraph};

use crate::i18n::I18n;
use crate::tui::browser::{Browser, Focus};
use crate::tui::card_text::{self, CardBlock};
use crate::tui::hit_map::{Hit, HitMap};
use crate::tui::theme::{self, Theme};

/// Wide enough for the table and a preview side by side.
const SIDE_PREVIEW_WIDTH: u16 = 120;
const MARKER_WIDTH: u16 = 2;
const SEARCH_PROMPT: &str = " / ";
/// Close to Anki's row tints, as text colors.
const SUSPENDED: Color = Color::Rgb(0xe0, 0xc0, 0x5a);
const MARKED: Color = Color::Rgb(0xc0, 0x9a, 0xf0);

pub fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    browser: &Browser,
    i18n: &I18n,
    theme: Theme,
    hits: &mut HitMap,
) {
    let tr = i18n.anki();
    let (selected, total) = (browser.selected_count(), browser.ids().len());
    let title = if browser.table().notes_mode {
        tr.browsing_window_title_notes(selected, total)
    } else {
        tr.browsing_window_title(selected, total)
    };
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme.border))
        .title(format!(" {title} "));
    let inner = block.inner(area);

    frame.render_widget(block, area);

    let [search_area, body_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(inner);

    render_search(frame, search_area, browser, i18n, theme, hits);

    if !browser.show_preview() {
        render_table(frame, body_area, browser, i18n, theme, hits);
        return;
    }

    let [table_area, preview_area] = if body_area.width >= SIDE_PREVIEW_WIDTH {
        Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)])
            .areas(body_area)
    } else {
        Layout::vertical([Constraint::Percentage(60), Constraint::Percentage(40)]).areas(body_area)
    };

    render_table(frame, table_area, browser, i18n, theme, hits);
    render_preview(frame, preview_area, browser, i18n, theme);
}

fn render_search(
    frame: &mut Frame<'_>,
    area: Rect,
    browser: &Browser,
    i18n: &I18n,
    theme: Theme,
    hits: &mut HitMap,
) {
    let editing = browser.focus() == Focus::Search;
    let prompt = Span::styled(SEARCH_PROMPT, Style::new().fg(theme.accent));
    let line = if browser.input().is_empty() && !editing {
        Line::from(vec![
            prompt,
            Span::styled(
                i18n.anki().browsing_search_bar_hint().into_owned(),
                theme.muted,
            ),
        ])
    } else {
        Line::from(vec![prompt, Span::raw(browser.input().to_owned())])
    };
    let style = if editing {
        Style::new().bg(theme.surface)
    } else {
        Style::new()
    };

    frame.render_widget(line.style(style), area);
    hits.register(area, Hit::BrowserSearch);

    if editing {
        let width = Line::raw(browser.input()).width() + SEARCH_PROMPT.len();
        let x = area
            .x
            .saturating_add(u16::try_from(width).unwrap_or(u16::MAX));

        frame.set_cursor_position(Position::new(x.min(area.right().saturating_sub(1)), area.y));
    }
}

fn render_table(
    frame: &mut Frame<'_>,
    area: Rect,
    browser: &Browser,
    i18n: &I18n,
    theme: Theme,
    hits: &mut HitMap,
) {
    let [header_area, rows_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);
    let columns = &browser.table().columns;

    browser.set_page(usize::from(rows_area.height));

    let column_areas = |row: Rect| {
        let [_, cells] =
            Layout::horizontal([Constraint::Length(MARKER_WIDTH), Constraint::Fill(1)]).areas(row);
        let constraints =
            (0..columns.len()).map(|index| Constraint::Fill(if index == 0 { 2 } else { 1 }));

        Layout::horizontal(constraints).spacing(1).split(cells)
    };

    for (index, (column, &cell)) in columns
        .iter()
        .zip(column_areas(header_area).iter())
        .enumerate()
    {
        let arrow = match &browser.table().sort {
            Some(sort) if sort.column == column.key && sort.reverse => " ▼",
            Some(sort) if sort.column == column.key => " ▲",
            _ => "",
        };

        frame.render_widget(
            Line::styled(
                format!("{}{arrow}", column.label),
                Style::new().fg(theme.muted).add_modifier(Modifier::BOLD),
            ),
            cell,
        );
        hits.register(cell, Hit::BrowserColumn(index));
    }

    if browser.ids().is_empty() {
        frame.render_widget(
            Line::styled(format!("  {}", i18n.text("list-no-matches")), theme.muted),
            rows_area,
        );
        return;
    }

    let row_areas = Layout::vertical(vec![Constraint::Length(1); usize::from(rows_area.height)])
        .split(rows_area);

    for ((position, &id), &row_area) in browser
        .ids()
        .iter()
        .enumerate()
        .skip(browser.offset())
        .zip(row_areas.iter())
    {
        let current = position == browser.cursor();
        let marker = if browser.is_selected(id) {
            "● "
        } else {
            "  "
        };

        frame.render_widget(
            Line::styled(marker, Style::new().fg(theme.accent)),
            row_area,
        );

        match browser.row(id) {
            Some(row) => {
                let style = Style::new().fg(row_color(row.color, theme));

                for (text, &cell) in row.cells.iter().zip(column_areas(row_area).iter()) {
                    frame.render_widget(Line::styled(text.as_str(), style), cell);
                }
            },
            None => frame.render_widget(Line::styled("  ...", theme.muted), row_area),
        }

        if current {
            let style = Style::new().bg(theme.border).add_modifier(Modifier::BOLD);

            frame.buffer_mut().set_style(row_area, style);
        }

        hits.register(row_area, Hit::BrowserRow(position));
    }
}

/// The answer side, which shows the question above it like in Anki's preview.
fn render_preview(frame: &mut Frame<'_>, area: Rect, browser: &Browser, i18n: &I18n, theme: Theme) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme.border))
        .title(format!(" {} ", i18n.anki().actions_preview()));
    let inner = block.inner(area);

    frame.render_widget(block, area);

    let Some(card) = browser.preview() else {
        return;
    };
    let html = review::without_type_fields(&card.answer);
    let lines: Vec<Line<'static>> =
        card_text::render(&html, inner.width, theme, &card.sounds, None)
            .into_iter()
            .map(|block| match block {
                CardBlock::Line(line) => line.line,
                CardBlock::Image(src) => Line::styled(format!("[image: {src}]"), theme.muted),
            })
            .collect();

    frame.render_widget(Paragraph::new(lines).centered(), inner);
}

const fn row_color(color: RowColor, theme: Theme) -> Color {
    match color {
        RowColor::Default => theme.text,
        RowColor::Marked => MARKED,
        RowColor::Suspended => SUSPENDED,
        RowColor::Buried => theme.muted,
        RowColor::Flag(flag) => theme::flag_color(flag),
    }
}
