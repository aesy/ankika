use ankika_core::card_actions::CardActionError;
use ankika_core::decks::DeckError;
use ankika_core::login::LoginError;
use ankika_core::note_edit::NoteEditError;
use ankika_core::review::ReviewError;
use ankika_core::sync::SyncError;

use crate::i18n::I18n;
use crate::tui::editor::EditorError;
use crate::tui::tasks::LogInTaskError;

/// Anki's own localized description where Anki produced the error.
pub fn log_in(error: &LogInTaskError, i18n: &I18n) -> String {
    match error {
        LogInTaskError::Login(LoginError::HttpClient(_)) => i18n.text("error-http-client"),
        LogInTaskError::Login(error) => error.anki_error().map_or_else(
            || error.to_string(),
            |anki_error| anki_error.message(i18n.anki()),
        ),
        LogInTaskError::Save(_) => i18n.text("error-save-login"),
    }
}

pub fn sync(error: &SyncError, i18n: &I18n) -> String {
    match error {
        SyncError::InvalidSyncServer { .. } => i18n.text("error-sync-server-url"),
        SyncError::HttpClient(_) => i18n.text("error-http-client"),
        SyncError::AuthFailed { source }
        | SyncError::Network { source }
        | SyncError::Server { source }
        | SyncError::OpenMediaDatabase { source } => source.message(i18n.anki()),
    }
}

pub fn decks(error: &DeckError, i18n: &I18n) -> String {
    error.anki_error().message(i18n.anki())
}

pub fn review(error: &ReviewError, i18n: &I18n) -> String {
    error.anki_error().message(i18n.anki())
}

pub fn card_action(error: &CardActionError, i18n: &I18n) -> String {
    error.anki_error().message(i18n.anki())
}

pub fn note_edit(error: &NoteEditError, i18n: &I18n) -> String {
    match error {
        NoteEditError::Missing => i18n.text("error-note-missing"),
        NoteEditError::MissingHeader { header } => {
            i18n.text_with("error-note-header", &[("header", header)])
        },
        NoteEditError::Load { source } | NoteEditError::Save { source } => {
            source.message(i18n.anki())
        },
    }
}

/// The editor's own messages are in English, so the details stay in the log.
pub fn editor(error: &EditorError, i18n: &I18n) -> String {
    match error {
        EditorError::Launch { editor, .. } => {
            i18n.text_with("error-editor-launch", &[("editor", editor)])
        },
        EditorError::Failed { editor, .. } => {
            i18n.text_with("error-editor-failed", &[("editor", editor)])
        },
        EditorError::Write { .. } | EditorError::Read { .. } | EditorError::Terminal(_) => {
            i18n.text("error-editor-file")
        },
    }
}
