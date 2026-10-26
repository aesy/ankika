use ankika_core::review::Rating;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Paragraph};

use crate::i18n::I18n;
use crate::tui::action::Action;
use crate::tui::card_text;
use crate::tui::hit_map::{Hit, HitMap};
use crate::tui::review::{Review, Stage};
use crate::tui::state::State;
use crate::tui::theme::Theme;
use crate::tui::view::buttons::{self, Button, GAP};

const BOTTOM_HEIGHT: u16 = 3;
const INPUT_WIDTH: u16 = 50;

pub fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    review: &Review,
    state: &State,
    i18n: &I18n,
    theme: Theme,
    hits: &mut HitMap,
) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme.border))
        .title(format!(" {} ", review.deck_name()));
    let inner = block.inner(area).inner(Margin::new(2, 1));

    frame.render_widget(block, area);

    let [content_area, bottom_area] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(BOTTOM_HEIGHT)]).areas(inner);

    match review.stage() {
        Stage::Waiting => {
            let text = Line::styled(i18n.anki().sync_syncing().into_owned(), theme.muted);

            frame.render_widget(Paragraph::new(text).centered(), content_area);
        },
        Stage::Question { card, typed } => {
            render_card(frame, content_area, &card.question, review, theme);

            let [input_area, _, button_area] = Layout::vertical([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .areas(bottom_area);

            if review.is_typing() {
                render_input(frame, input_area, typed, theme);
            }

            let show = Button {
                label: Action::ShowAnswer.label(i18n),
                hit: Hit::Action(Action::ShowAnswer),
                enabled: true,
                focused: true,
            };

            render_centered(frame, button_area, &[(show, None)], state, theme, hits);
        },
        Stage::Answer { card, html } => {
            render_card(frame, content_area, html, review, theme);

            let buttons: Vec<(Button, Option<&str>)> = Rating::ALL
                .iter()
                .zip(&card.intervals)
                .map(|(&rating, interval)| {
                    let action = Action::Answer(rating);
                    let button = Button {
                        label: action.label(i18n),
                        hit: Hit::Action(action),
                        enabled: true,
                        focused: rating == Rating::Good,
                    };

                    (button, Some(interval.as_str()))
                })
                .collect();
            let [_, button_area] =
                Layout::vertical([Constraint::Length(1), Constraint::Length(2)]).areas(bottom_area);

            render_centered(frame, button_area, &buttons, state, theme, hits);
        },
        Stage::Finished { studied_today } => {
            let lines = vec![
                Line::styled(
                    i18n.anki()
                        .scheduling_congratulations_finished()
                        .into_owned(),
                    Style::new().fg(theme.accent).add_modifier(Modifier::BOLD),
                ),
                Line::default(),
                Line::styled(studied_today.clone(), theme.muted),
            ];
            let [_, text_area] =
                Layout::vertical([Constraint::Percentage(30), Constraint::Fill(1)])
                    .areas(content_area);
            let back = Button {
                label: Action::CloseReview.label(i18n),
                hit: Hit::Action(Action::CloseReview),
                enabled: true,
                focused: true,
            };
            let [_, _, button_area] = Layout::vertical([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .areas(bottom_area);

            frame.render_widget(Paragraph::new(lines).centered(), text_area);
            render_centered(frame, button_area, &[(back, None)], state, theme, hits);
        },
    }
}

fn render_card(frame: &mut Frame<'_>, area: Rect, html: &str, review: &Review, theme: Theme) {
    let lines = card_text::render(html, area.width, theme);
    let height = u16::try_from(lines.len()).unwrap_or(u16::MAX);

    review.set_max_scroll(height.saturating_sub(area.height));

    let paragraph = Paragraph::new(lines)
        .centered()
        .scroll((review.scroll(), 0));

    frame.render_widget(paragraph, area);
}

fn render_input(frame: &mut Frame<'_>, area: Rect, typed: &str, theme: Theme) {
    let input_area = area.centered_horizontally(Constraint::Length(INPUT_WIDTH));
    let line = Line::from(vec![
        Span::raw(format!(" {typed}")),
        Span::styled("_", Style::new().fg(theme.accent)),
    ]);

    frame.render_widget(
        Paragraph::new(line).style(Style::new().bg(theme.surface).fg(theme.text)),
        input_area,
    );
}

/// Buttons side by side in the middle of the last row, each with an optional caption
/// centered above it.
fn render_centered(
    frame: &mut Frame<'_>,
    area: Rect,
    buttons: &[(Button, Option<&str>)],
    state: &State,
    theme: Theme,
    hits: &mut HitMap,
) {
    let widths: Vec<u16> = buttons
        .iter()
        .map(|(button, caption)| {
            let caption = caption.map_or(0, |text| {
                u16::try_from(Line::raw(text).width()).unwrap_or(u16::MAX)
            });

            button.width().max(caption.saturating_add(2))
        })
        .collect();
    let gaps = u16::try_from(buttons.len().saturating_sub(1)).unwrap_or(0) * GAP * 2;
    let total = widths.iter().sum::<u16>().saturating_add(gaps);
    let mut x = area.x + area.width.saturating_sub(total) / 2;
    let button_y = area.bottom().saturating_sub(1);

    for ((button, caption), width) in buttons.iter().zip(widths) {
        let button_area = Rect::new(x, button_y, width, 1).intersection(area);

        if let Some(caption) = caption {
            let caption_area =
                Rect::new(x, button_y.saturating_sub(1), width, 1).intersection(area);

            frame.render_widget(Line::styled(*caption, theme.muted).centered(), caption_area);
        }

        buttons::render(frame, button_area, button, state.mouse(), theme, hits);
        x = x.saturating_add(width).saturating_add(GAP * 2);
    }
}
