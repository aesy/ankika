use ratatui::Frame;
use ratatui::layout::Rect;

use crate::i18n::I18n;
use crate::tui::hit_map::{Hit, HitMap};
use crate::tui::replace_prompt::{ReplaceChoice, ReplacePrompt};
use crate::tui::state::State;
use crate::tui::theme::Theme;
use crate::tui::view::buttons::Button;
use crate::tui::view::prompt::{self, Prompt};

const WIDTH: u16 = 70;
const TEXT_ROWS: u16 = 3;

pub fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    replace: &ReplacePrompt,
    state: &State,
    i18n: &I18n,
    theme: Theme,
    hits: &mut HitMap,
) {
    let choices = &replace.choices;
    let buttons: Vec<Button> = choices
        .items()
        .iter()
        .map(|&choice| Button {
            label: match choice {
                ReplaceChoice::Replace => i18n.text("package-replace"),
                ReplaceChoice::Cancel => i18n.anki().actions_cancel().into_owned(),
            },
            hit: Hit::ReplaceChoice(choice),
            enabled: true,
            focused: choices.focused() == Some(choice),
        })
        .collect();
    let title = i18n.anki().actions_import();
    let text = i18n.text_with(
        "package-replace-question",
        &[("file", &replace.file.display().to_string())],
    );
    let prompt = Prompt {
        title: &title,
        text: &text,
        width: WIDTH,
        text_rows: TEXT_ROWS,
    };

    prompt::render(frame, area, &prompt, &buttons, state.mouse(), theme, hits);
}
