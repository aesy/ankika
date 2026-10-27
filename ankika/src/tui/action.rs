use ankika_core::card_actions::{CardAction, Flag};
use ankika_core::review::Rating;

use crate::i18n::I18n;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    ToggleHelp,
    OpenPalette,
    CloseOverlay,
    LogIn,
    LogOut,
    Sync,
    OpenDeck,
    Study,
    CustomStudy,
    Unbury,
    Rebuild,
    Empty,
    CloseOverview,
    ShowAnswer,
    Answer(Rating),
    Undo,
    ReplayAudio,
    Card(CardAction),
    EditNote,
    AddNote,
    OpenBrowser,
    CloseBrowser,
    FocusSearch,
    RunSearch,
    LeaveSearch,
    ToggleNotesMode,
    TogglePreview,
    ToggleSelected,
    SelectAll,
    ToggleSuspend,
    ChangeDeck,
    AddTags,
    RemoveTags,
    SortBy,
    CloseReview,
    Quit,
}

impl Action {
    pub fn label(self, i18n: &I18n) -> String {
        match self {
            Self::ToggleHelp => i18n.anki().actions_help().into_owned(),
            Self::OpenPalette => i18n.text("action-commands"),
            Self::CloseOverlay | Self::CloseBrowser => i18n.anki().actions_close().into_owned(),
            Self::LogIn => i18n.anki().sync_log_in_button().into_owned(),
            Self::LogOut => i18n.anki().sync_log_out_button().into_owned(),
            Self::Sync => i18n.text("action-sync"),
            Self::OpenDeck => i18n.text("action-open-deck"),
            Self::Study => i18n.anki().studying_study_now().into_owned(),
            Self::CustomStudy => i18n.anki().actions_custom_study().into_owned(),
            Self::Unbury => i18n.anki().studying_unbury().into_owned(),
            Self::Rebuild => i18n.anki().actions_rebuild().into_owned(),
            Self::Empty => i18n.anki().studying_empty().into_owned(),
            Self::CloseOverview => i18n.anki().actions_decks().into_owned(),
            Self::ShowAnswer => i18n.anki().studying_show_answer().into_owned(),
            Self::Answer(Rating::Again) => i18n.anki().studying_again().into_owned(),
            Self::Answer(Rating::Hard) => i18n.anki().studying_hard().into_owned(),
            Self::Answer(Rating::Good) => i18n.anki().studying_good().into_owned(),
            Self::Answer(Rating::Easy) => i18n.anki().studying_easy().into_owned(),
            Self::Undo => i18n.anki().undo_undo().into_owned(),
            Self::ReplayAudio => i18n.anki().actions_replay_audio().into_owned(),
            Self::Card(action) => card_action_label(action, i18n),
            Self::EditNote => i18n.anki().studying_edit().into_owned(),
            Self::AddNote => i18n.anki().actions_add().into_owned(),
            Self::OpenBrowser => i18n.text("action-browse"),
            Self::FocusSearch | Self::RunSearch => i18n.anki().actions_search().into_owned(),
            Self::ToggleNotesMode => i18n
                .anki()
                .browsing_toggle_showing_cards_notes()
                .into_owned(),
            Self::TogglePreview => i18n.anki().actions_preview().into_owned(),
            Self::ToggleSelected => i18n.text("action-toggle-selected"),
            Self::SelectAll => i18n.text("action-select-all"),
            Self::ToggleSuspend => i18n.anki().browsing_toggle_suspend().into_owned(),
            Self::ChangeDeck => i18n.anki().browsing_change_deck().into_owned(),
            Self::AddTags => i18n.anki().browsing_add_tags2().into_owned(),
            Self::RemoveTags => i18n.anki().browsing_remove_tags().into_owned(),
            Self::SortBy => i18n.text("action-sort-by"),
            Self::LeaveSearch | Self::CloseReview => i18n.text("action-back"),
            Self::Quit => i18n.text("action-quit"),
        }
    }
}

fn card_action_label(action: CardAction, i18n: &I18n) -> String {
    let tr = i18n.anki();

    match action {
        CardAction::BuryCard => tr.studying_bury_card(),
        CardAction::BuryNote => tr.studying_bury_note(),
        CardAction::SuspendCard => tr.studying_suspend(),
        CardAction::SuspendNote => tr.studying_suspend_note(),
        CardAction::ToggleMark => tr.studying_mark_note(),
        CardAction::ToggleFlag(Flag::Red) => tr.actions_flag_red(),
        CardAction::ToggleFlag(Flag::Orange) => tr.actions_flag_orange(),
        CardAction::ToggleFlag(Flag::Green) => tr.actions_flag_green(),
        CardAction::ToggleFlag(Flag::Blue) => tr.actions_flag_blue(),
        CardAction::ToggleFlag(Flag::Pink) => tr.actions_flag_pink(),
        CardAction::ToggleFlag(Flag::Turquoise) => tr.actions_flag_turquoise(),
        CardAction::ToggleFlag(Flag::Purple) => tr.actions_flag_purple(),
        CardAction::DeleteNote => tr.studying_delete_note(),
    }
    .into_owned()
}
