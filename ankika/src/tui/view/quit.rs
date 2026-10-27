use ratatui::Frame;
use ratatui::layout::Rect;

use crate::i18n::I18n;
use crate::tui::choices::Choices;
use crate::tui::hit_map::{Hit, HitMap};
use crate::tui::quit_prompt::QuitChoice;
use crate::tui::state::State;
use crate::tui::theme::Theme;
use crate::tui::view::buttons::Button;
use crate::tui::view::prompt::{self, Prompt};

const WIDTH: u16 = 60;
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
    let title = i18n.text("action-quit");
    let text = i18n.text("quit-media-syncing");
    let prompt = Prompt {
        title: &title,
        text: &text,
        width: WIDTH,
        text_rows: TEXT_ROWS,
    };

    prompt::render(frame, area, &prompt, &buttons, state.mouse(), theme, hits);
}

fn label(choice: QuitChoice, i18n: &I18n) -> String {
    match choice {
        QuitChoice::Wait => i18n.text("quit-wait"),
        QuitChoice::Abort => i18n.anki().sync_abort_button().into_owned(),
        QuitChoice::Cancel => i18n.anki().sync_cancel_button().into_owned(),
    }
}
