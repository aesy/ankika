mod add;
mod browser;
mod buttons;
mod custom_study;
mod footer;
mod full_sync;
mod fuzzy_list;
mod header;
mod help;
mod home;
mod login;
mod overview;
mod prompt;
mod quit;
mod review;
mod tag_input;
mod tags;
mod unbury;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Block, BorderType, Clear};

use crate::i18n::I18n;
use crate::tui::action::Action;
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

    if let Some(browser) = state.browser() {
        browser::render(frame, main_area, browser, i18n, theme, &mut hits);
    } else {
        render_main(frame, main_area, state, images, i18n, theme, &mut hits);
    }

    footer::render(frame, footer_area, state, i18n, theme, &mut hits);
    render_overlay(frame, area, state, i18n, theme, &mut hits);

    hits
}

fn render_main(
    frame: &mut Frame<'_>,
    main_area: Rect,
    state: &State,
    images: &Images,
    i18n: &I18n,
    theme: Theme,
    hits: &mut HitMap,
) {
    match state.review() {
        Some(review) => {
            let context = review::Context {
                state,
                images,
                i18n,
                theme,
            };

            review::render(frame, main_area, review, &context, hits);
        },
        None => match state.deck_screen() {
            Some(screen) => {
                overview::render(frame, main_area, screen, state, i18n, theme, hits);
            },
            None => home::render(frame, main_area, state, i18n, theme, hits),
        },
    }
}

fn render_overlay(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &State,
    i18n: &I18n,
    theme: Theme,
    hits: &mut HitMap,
) {
    match state.overlay() {
        Overlay::None => {},
        Overlay::Help => help::render(frame, area, i18n, theme, hits),
        Overlay::Palette(palette) => {
            let title = Action::OpenPalette.label(i18n);
            let placeholder = i18n.text("palette-placeholder");
            let labels = fuzzy_list::Labels {
                title: &title,
                placeholder: &placeholder,
            };

            fuzzy_list::render(frame, area, palette, &labels, i18n, theme, hits);
        },
        Overlay::Login(form) => login::render(frame, area, form, state, i18n, theme, hits),
        Overlay::FullSync(prompt) => {
            full_sync::render(frame, area, prompt, state, i18n, theme, hits);
        },
        Overlay::Quit(choices) => quit::render(frame, area, choices, state, i18n, theme, hits),
        Overlay::Unbury(choices) => {
            unbury::render(frame, area, choices, state, i18n, theme, hits);
        },
        Overlay::CustomStudy(form) => {
            custom_study::render(frame, area, form, state, i18n, theme, hits);
        },
        Overlay::Add(form) => add::render(frame, area, form, state, i18n, theme, hits),
        Overlay::ChangeDeck(list) => {
            let title = Action::ChangeDeck.label(i18n);
            let placeholder = i18n.text("picker-placeholder");
            let labels = fuzzy_list::Labels {
                title: &title,
                placeholder: &placeholder,
            };

            fuzzy_list::render(frame, area, list, &labels, i18n, theme, hits);
        },
        Overlay::SortBy(list) => {
            let title = Action::SortBy.label(i18n);
            let placeholder = i18n.text("picker-placeholder");
            let labels = fuzzy_list::Labels {
                title: &title,
                placeholder: &placeholder,
            };

            fuzzy_list::render(frame, area, list, &labels, i18n, theme, hits);
        },
        Overlay::Tags { remove, input } => {
            tags::render(frame, area, *remove, input, i18n, theme, hits);
        },
    }
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
