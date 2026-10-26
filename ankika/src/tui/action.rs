use crate::i18n::I18n;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    ToggleHelp,
    OpenPalette,
    CloseOverlay,
    LogIn,
    LogOut,
    Sync,
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
            Self::Quit => i18n.text("action-quit"),
        }
    }
}
