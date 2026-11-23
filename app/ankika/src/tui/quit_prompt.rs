use crate::tui::choices::Choices;

/// What to do when quitting while media is still syncing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuitChoice {
    /// Quit once the media sync is done.
    Wait,
    /// Stop the media sync and quit now; it resumes on the next start.
    Abort,
    Cancel,
}

pub const fn new() -> Choices<QuitChoice> {
    Choices::new(&[QuitChoice::Wait, QuitChoice::Abort, QuitChoice::Cancel])
}
