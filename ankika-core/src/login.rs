use anki::error::{AnkiError, SyncErrorKind};
use anki::prelude::I18n;
use anki::sync::login::sync_login;
use reqwest::Client;

use crate::account::Account;
use crate::session::Session;

#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error("Failed to set up the HTTP client")]
    HttpClient(#[source] reqwest::Error),
    #[error("Email or password was incorrect")]
    InvalidCredentials {
        #[source]
        source: AnkiError,
    },
    #[error("Could not reach the sync server: {}", english(source))]
    Network {
        #[source]
        source: AnkiError,
    },
    #[error("The sync server refused the login: {}", english(source))]
    Server {
        #[source]
        source: AnkiError,
    },
}

impl LoginError {
    /// Anki's own error, which it can describe in the user's language.
    #[must_use]
    pub const fn anki_error(&self) -> Option<&AnkiError> {
        match self {
            Self::HttpClient(_) => None,
            Self::InvalidCredentials { source }
            | Self::Network { source }
            | Self::Server { source } => Some(source),
        }
    }
}

/// Exchanges the credentials for a host key. Nothing is stored.
pub async fn log_in(
    username: String,
    password: String,
    sync_server: Option<String>,
) -> Result<Session, LoginError> {
    // Same client setup as Anki desktop uses for syncing
    let client = Client::builder()
        .http1_only()
        .build()
        .map_err(LoginError::HttpClient)?;
    let auth = sync_login(username.clone(), password, sync_server.clone(), client)
        .await
        .map_err(classify)?;

    Ok(Session {
        account: Account {
            username,
            sync_server,
        },
        host_key: auth.hkey,
    })
}

/// Anki's `Display` only names the variant; its message has the details.
fn english(error: &AnkiError) -> String {
    error.message(&I18n::template_only())
}

fn classify(error: AnkiError) -> LoginError {
    match &error {
        AnkiError::SyncError { source } if source.kind == SyncErrorKind::AuthFailed => {
            LoginError::InvalidCredentials { source: error }
        },
        AnkiError::NetworkError { .. } => LoginError::Network { source: error },
        _ => LoginError::Server { source: error },
    }
}
