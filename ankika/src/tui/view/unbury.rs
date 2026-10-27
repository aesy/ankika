use ankika_core::overview::Unbury;
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::i18n::I18n;
use crate::tui::choices::Choices;
use crate::tui::hit_map::{Hit, HitMap};
use crate::tui::state::State;
use crate::tui::theme::Theme;
use crate::tui::unbury_prompt::UnburyChoice;
use crate::tui::view::buttons::Button;
use crate::tui::view::prompt::{self, Prompt};

const WIDTH: u16 = 76;
const TEXT_ROWS: u16 = 1;

pub fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    choices: &Choices<UnburyChoice>,
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
            hit: Hit::UnburyChoice(choice),
            enabled: true,
            focused: choices.focused() == Some(choice),
        })
        .collect();
    let title = i18n.anki().studying_unbury();
    let text = i18n.anki().studying_what_would_you_like_to_unbury();
    let prompt = Prompt {
        title: &title,
        text: &text,
        width: WIDTH,
        text_rows: TEXT_ROWS,
    };

    prompt::render(frame, area, &prompt, &buttons, state.mouse(), theme, hits);
}

fn label(choice: UnburyChoice, i18n: &I18n) -> String {
    let tr = i18n.anki();

    match choice {
        UnburyChoice::Unbury(Unbury::Manual) => tr.studying_manually_buried_cards(),
        UnburyChoice::Unbury(Unbury::Siblings) => tr.studying_buried_siblings(),
        UnburyChoice::Unbury(Unbury::All) => tr.studying_all_buried_cards(),
        UnburyChoice::Cancel => tr.actions_cancel(),
    }
    .into_owned()
}
