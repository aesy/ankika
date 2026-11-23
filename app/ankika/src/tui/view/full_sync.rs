use ankika_core::sync::{FullSyncDirection, FullSyncRequired};
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::i18n::I18n;
use crate::tui::full_sync_prompt::{Choice, FullSyncPrompt};
use crate::tui::hit_map::{Hit, HitMap};
use crate::tui::state::State;
use crate::tui::theme::Theme;
use crate::tui::view::buttons::Button;
use crate::tui::view::prompt::{self, Prompt};

const WIDTH: u16 = 70;
// Fit Anki's English at `WIDTH`
const CONFLICT_ROWS: u16 = 14;
const EMPTY_SIDE_ROWS: u16 = 2;

pub fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    full_sync: &FullSyncPrompt,
    state: &State,
    i18n: &I18n,
    theme: Theme,
    hits: &mut HitMap,
) {
    let (text, text_rows) = match full_sync.required() {
        FullSyncRequired::Conflict => (i18n.anki().sync_conflict_explanation(), CONFLICT_ROWS),
        FullSyncRequired::UploadOnly => (i18n.anki().sync_confirm_empty_upload(), EMPTY_SIDE_ROWS),
        FullSyncRequired::DownloadOnly => {
            (i18n.anki().sync_confirm_empty_download(), EMPTY_SIDE_ROWS)
        },
    };
    let choices = full_sync.choices();
    let buttons: Vec<Button> = choices
        .items()
        .iter()
        .map(|&choice| Button {
            label: label(choice, i18n),
            hit: Hit::FullSyncChoice(choice),
            enabled: true,
            focused: choices.focused() == Some(choice),
        })
        .collect();
    let title = i18n.text("action-sync");
    let prompt = Prompt {
        title: &title,
        text: &text,
        width: WIDTH,
        text_rows,
    };

    prompt::render(frame, area, &prompt, &buttons, state.mouse(), theme, hits);
}

fn label(choice: Choice, i18n: &I18n) -> String {
    match choice {
        Choice::Sync(FullSyncDirection::Download) => i18n.anki().sync_download_from_ankiweb(),
        Choice::Sync(FullSyncDirection::Upload) => i18n.anki().sync_upload_to_ankiweb(),
        Choice::Cancel => i18n.anki().sync_cancel_button(),
    }
    .into_owned()
}
