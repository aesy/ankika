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
    Study,
    ShowAnswer,
    Answer(Rating),
    Undo,
    ReplayAudio,
    CloseReview,
    Quit,
}

impl Action {
    pub fn label(self, i18n: &I18n) -> String {
        match self {
            Self::ToggleHelp => i18n.anki().actions_help().into_owned(),
            Self::OpenPalette => i18n.text("action-commands"),
            Self::CloseOverlay => i18n.anki().actions_close().into_owned(),
            Self::LogIn => i18n.anki().sync_log_in_button().into_owned(),
            Self::LogOut => i18n.anki().sync_log_out_button().into_owned(),
            Self::Sync => i18n.text("action-sync"),
            Self::Study => i18n.anki().decks_study().into_owned(),
            Self::ShowAnswer => i18n.anki().studying_show_answer().into_owned(),
            Self::Answer(Rating::Again) => i18n.anki().studying_again().into_owned(),
            Self::Answer(Rating::Hard) => i18n.anki().studying_hard().into_owned(),
            Self::Answer(Rating::Good) => i18n.anki().studying_good().into_owned(),
            Self::Answer(Rating::Easy) => i18n.anki().studying_easy().into_owned(),
            Self::Undo => i18n.anki().undo_undo().into_owned(),
            Self::ReplayAudio => i18n.anki().actions_replay_audio().into_owned(),
            Self::CloseReview => i18n.anki().actions_decks().into_owned(),
            Self::Quit => i18n.text("action-quit"),
        }
    }
}
