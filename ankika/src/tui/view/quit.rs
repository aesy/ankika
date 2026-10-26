use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::widgets::{Paragraph, Wrap};

use crate::i18n::I18n;
use crate::tui::choices::Choices;
use crate::tui::hit_map::{Hit, HitMap};
use crate::tui::quit_prompt::QuitChoice;
use crate::tui::state::State;
use crate::tui::theme::Theme;
use crate::tui::view::buttons::{self, Button};

const WIDTH: u16 = 60;
// Borders, margins, a blank line and the buttons around the text
const FIXED_HEIGHT: u16 = 6;
// Fixed, since ratatui can't measure wrapped text yet; fits the English at `WIDTH`
const TEXT_ROWS: u16 = 2;

pub fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    choices: &Choices<QuitChoice>,
    state: &State,
    i18n: &I18n,
    theme: Theme,
    hits: &mut HitMap,
) {
    let popup = area.centered(
        Constraint::Length(WIDTH),
        Constraint::Length(FIXED_HEIGHT + TEXT_ROWS),
    );
    let title = i18n.text("action-quit");
    let inner = super::overlay(frame, popup, &title, theme, hits).inner(Margin::new(2, 1));
    let [text_area, _, button_area] = Layout::vertical([
        Constraint::Length(TEXT_ROWS),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(inner);
    let buttons: Vec<Button> = choices
        .items()
        .iter()
        .map(|&choice| Button {
            label: label(choice, i18n),
            hit: Hit::QuitChoice(choice),
            enabled: true,
            focused: choices.focused() == Some(choice),
        })
        .collect();
    let text = i18n.text("quit-media-syncing");

    frame.render_widget(Paragraph::new(text).wrap(Wrap { trim: false }), text_area);
    buttons::render_right_aligned(frame, button_area, &buttons, state.mouse(), theme, hits);
}

fn label(choice: QuitChoice, i18n: &I18n) -> String {
    match choice {
        QuitChoice::Wait => i18n.text("quit-wait"),
        QuitChoice::Abort => i18n.anki().sync_abort_button().into_owned(),
        QuitChoice::Cancel => i18n.anki().sync_cancel_button().into_owned(),
    }
}
