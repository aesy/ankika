use std::path::PathBuf;

use crate::tui::choices::Choices;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplaceChoice {
    Replace,
    Cancel,
}

/// Asked before a `.colpkg` replaces the collection; Cancel is focused, since replacing
/// can't be undone.
pub struct ReplacePrompt {
    pub file: PathBuf,
    pub choices: Choices<ReplaceChoice>,
}

impl ReplacePrompt {
    pub const fn new(file: PathBuf) -> Self {
        Self {
            file,
            choices: Choices::new(&[ReplaceChoice::Replace, ReplaceChoice::Cancel]),
        }
    }
}
