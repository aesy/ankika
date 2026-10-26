use ankika_core::sync::{FullSyncDirection, FullSyncRequired};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::widgets::{Paragraph, Wrap};

use crate::i18n::I18n;
use crate::tui::full_sync_prompt::{Choice, FullSyncPrompt};
use crate::tui::hit_map::{Hit, HitMap};
use crate::tui::state::State;
use crate::tui::theme::Theme;
use crate::tui::view::buttons::{self, Button};

const WIDTH: u16 = 70;
// Borders, margins, a blank line and the buttons around the explanation
const FIXED_HEIGHT: u16 = 6;
// Fixed, since ratatui can't measure wrapped text yet; fits Anki's English at `WIDTH`
const CONFLICT_ROWS: u16 = 14;
const EMPTY_SIDE_ROWS: u16 = 2;

pub fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    prompt: &FullSyncPrompt,
    state: &State,
    i18n: &I18n,
    theme: Theme,
    hits: &mut HitMap,
) {
    let (text, rows) = match prompt.required() {
        FullSyncRequired::Conflict => (i18n.anki().sync_conflict_explanation(), CONFLICT_ROWS),
        FullSyncRequired::UploadOnly => (i18n.anki().sync_confirm_empty_upload(), EMPTY_SIDE_ROWS),
        FullSyncRequired::DownloadOnly => {
            (i18n.anki().sync_confirm_empty_download(), EMPTY_SIDE_ROWS)
        },
    };
    let height = FIXED_HEIGHT.saturating_add(rows);
    let popup = area.centered(Constraint::Length(WIDTH), Constraint::Length(height));
    let title = i18n.text("action-sync");
    let inner = super::overlay(frame, popup, &title, theme, hits).inner(Margin::new(2, 1));
    let [text_area, _, button_area] = Layout::vertical([
        Constraint::Length(rows),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(inner);
    let buttons: Vec<Button> = prompt
        .choices()
        .iter()
        .map(|&choice| Button {
            label: label(choice, i18n),
            hit: Hit::FullSyncChoice(choice),
            enabled: true,
            focused: prompt.focused() == Some(choice),
        })
        .collect();

    frame.render_widget(Paragraph::new(text).wrap(Wrap { trim: false }), text_area);
    buttons::render_right_aligned(frame, button_area, &buttons, state.mouse(), theme, hits);
}

fn label(choice: Choice, i18n: &I18n) -> String {
    match choice {
        Choice::Sync(FullSyncDirection::Download) => i18n.anki().sync_download_from_ankiweb(),
        Choice::Sync(FullSyncDirection::Upload) => i18n.anki().sync_upload_to_ankiweb(),
        Choice::Cancel => i18n.anki().sync_cancel_button(),
    }
    .into_owned()
}
