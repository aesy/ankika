mod buttons;
mod custom_study;
mod footer;
mod full_sync;
mod header;
mod help;
mod home;
mod login;
mod overview;
mod palette;
mod prompt;
mod quit;
mod review;
mod unbury;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Block, BorderType, Clear};

use crate::i18n::I18n;
use crate::tui::hit_map::{Hit, HitMap};
use crate::tui::images::Images;
use crate::tui::state::{Overlay, State};
use crate::tui::theme::Theme;

pub fn render(
    frame: &mut Frame<'_>,
    state: &State,
    images: &Images,
    i18n: &I18n,
    theme: Theme,
) -> HitMap {
    let mut hits = HitMap::default();
    let area = frame.area();

    frame.render_widget(
        Block::new().style(Style::new().bg(theme.background).fg(theme.text)),
        area,
    );

    let [header_area, main_area, footer_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(area);

    header::render(frame, header_area, state, i18n, theme, &mut hits);
    match state.review() {
        Some(review) => {
            let context = review::Context {
                state,
                images,
                i18n,
                theme,
            };

            review::render(frame, main_area, review, &context, &mut hits);
        },
        None => match state.deck_screen() {
            Some(screen) => {
                overview::render(frame, main_area, screen, state, i18n, theme, &mut hits);
            },
            None => home::render(frame, main_area, state, i18n, theme, &mut hits),
        },
    }
    footer::render(frame, footer_area, state, i18n, theme, &mut hits);

    match state.overlay() {
        Overlay::None => {},
        Overlay::Help => help::render(frame, area, i18n, theme, &mut hits),
        Overlay::Palette(palette) => palette::render(frame, area, palette, i18n, theme, &mut hits),
        Overlay::Login(form) => login::render(frame, area, form, state, i18n, theme, &mut hits),
        Overlay::FullSync(prompt) => {
            full_sync::render(frame, area, prompt, state, i18n, theme, &mut hits);
        },
        Overlay::Quit(choices) => quit::render(frame, area, choices, state, i18n, theme, &mut hits),
        Overlay::Unbury(choices) => {
            unbury::render(frame, area, choices, state, i18n, theme, &mut hits);
        },
        Overlay::CustomStudy(form) => {
            custom_study::render(frame, area, form, state, i18n, theme, &mut hits);
        },
    }

    hits
}

/// Clears `area` and draws the shared overlay frame, returning the inner area.
fn overlay(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &str,
    theme: Theme,
    hits: &mut HitMap,
) -> Rect {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme.accent))
        .title(format!(" {title} "))
        .style(Style::new().bg(theme.surface).fg(theme.text));
    let inner = block.inner(area);

    frame.render_widget(Clear, area);
    frame.render_widget(block, area);
    hits.register(area, Hit::Overlay);

    inner
}
