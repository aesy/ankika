use ankika_core::login::LoginError;

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
