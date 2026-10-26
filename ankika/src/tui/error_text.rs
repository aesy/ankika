use ankika_core::login::LoginError;
use ankika_core::sync::SyncError;

use crate::i18n::I18n;
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
