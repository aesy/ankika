use crate::i18n::I18n;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    ToggleHelp,
    OpenPalette,
    CloseOverlay,
    Quit,
}

impl Action {
    /// Actions offered in the command palette.
    pub const PALETTE: [Self; 2] = [Self::ToggleHelp, Self::Quit];

    pub fn label(self, i18n: &I18n) -> String {
        match self {
            Self::ToggleHelp => i18n.anki().actions_help().into_owned(),
            Self::OpenPalette => i18n.text("action-commands"),
            Self::CloseOverlay => i18n.anki().actions_close().into_owned(),
            Self::Quit => i18n.text("action-quit"),
        }
    }
}
