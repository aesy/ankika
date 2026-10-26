use anki::collection::Collection;
use anki::error::{AnkiError, SyncErrorKind};
use anki::sync::collection::normal::SyncActionRequired;
use anki::sync::login::SyncAuth;
use reqwest::Url;

use crate::collection::{self, CollectionError};
use crate::remote::{self, english};
use crate::session::Session;

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("Invalid sync server URL {url}")]
    InvalidSyncServer { url: String },
    #[error("Failed to set up the HTTP client")]
    HttpClient(#[source] reqwest::Error),
    #[error("The login is no longer valid; please log in again")]
    AuthFailed {
        #[source]
        source: AnkiError,
    },
    #[error("Could not reach the sync server: {}", english(source))]
    Network {
        #[source]
        source: AnkiError,
    },
    #[error("Sync failed: {}", english(source))]
    Server {
        #[source]
        source: AnkiError,
    },
    #[error("Failed to open the media database: {}", english(source))]
    OpenMediaDatabase {
        #[source]
        source: AnkiError,
    },
}

impl SyncError {
    /// Anki's own error, which it can describe in the user's language.
    #[must_use]
    pub const fn anki_error(&self) -> Option<&AnkiError> {
        match self {
            Self::InvalidSyncServer { .. } | Self::HttpClient(_) => None,
            Self::AuthFailed { source }
            | Self::Network { source }
            | Self::Server { source }
            | Self::OpenMediaDatabase { source } => Some(source),
        }
    }
}

/// What a full sync would overwrite, as far as the server can tell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FullSyncRequired {
    /// This device has no cards, so downloading loses nothing.
    DownloadOnly,
    /// AnkiWeb has no cards, so uploading loses nothing.
    UploadOnly,
    /// Both sides have changes; one of them will be lost.
    Conflict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FullSyncDirection {
    Upload,
    Download,
}

#[derive(Debug)]
pub struct NormalSyncOutput {
    /// `None` when the sync is complete.
    pub full_sync_required: Option<FullSyncRequired>,
    /// A message from AnkiWeb meant for the user; often empty.
    pub server_message: String,
    /// Where AnkiWeb wants this account to sync from now on.
    pub new_endpoint: Option<String>,
}

pub struct FullSyncOutput {
    /// The collection is reopened whether or not the sync succeeded.
    pub collection: Result<Collection, CollectionError>,
    pub result: Result<(), SyncError>,
}

pub async fn normal_sync(
    collection: &mut Collection,
    session: &Session,
) -> Result<NormalSyncOutput, SyncError> {
    let auth = auth(session)?;
    let client = remote::http_client().map_err(SyncError::HttpClient)?;
    let output = collection
        .normal_sync(auth, client)
        .await
        .map_err(classify)?;
    let full_sync_required = match output.required {
        SyncActionRequired::NoChanges | SyncActionRequired::NormalSyncRequired => None,
        SyncActionRequired::FullSyncRequired {
            upload_ok,
            download_ok,
        } => Some(match (upload_ok, download_ok) {
            (false, true) => FullSyncRequired::DownloadOnly,
            (true, false) => FullSyncRequired::UploadOnly,
            _ => FullSyncRequired::Conflict,
        }),
    };

    Ok(NormalSyncOutput {
        full_sync_required,
        server_message: output.server_message,
        new_endpoint: output.new_endpoint,
    })
}

/// Replaces one side completely. Consumes the collection and hands back a reopened one.
pub async fn full_sync(
    collection: Collection,
    session: &Session,
    direction: FullSyncDirection,
) -> FullSyncOutput {
    let builder = collection.as_builder();
    let result = run_full_sync(collection, session, direction).await;

    FullSyncOutput {
        collection: collection::reopen(builder),
        result,
    }
}

async fn run_full_sync(
    collection: Collection,
    session: &Session,
    direction: FullSyncDirection,
) -> Result<(), SyncError> {
    let auth = auth(session)?;
    let client = remote::http_client().map_err(SyncError::HttpClient)?;
    let result = match direction {
        FullSyncDirection::Upload => collection.full_upload(auth, client).await,
        FullSyncDirection::Download => collection.full_download(auth, client).await,
    };

    result.map_err(classify)
}

pub(crate) fn auth(session: &Session) -> Result<SyncAuth, SyncError> {
    let endpoint = session
        .account
        .endpoint()
        .map(|url| {
            // Joining makes Anki's later joins work for URLs without a trailing slash
            Url::parse(url)
                .and_then(|parsed| parsed.join("./"))
                .map_err(|_| SyncError::InvalidSyncServer {
                    url: url.to_owned(),
                })
        })
        .transpose()?;

    Ok(SyncAuth {
        hkey: session.host_key.clone(),
        endpoint,
        io_timeout_secs: None,
    })
}

pub(crate) fn classify(error: AnkiError) -> SyncError {
    match &error {
        AnkiError::SyncError { source } if source.kind == SyncErrorKind::AuthFailed => {
            SyncError::AuthFailed { source: error }
        },
        AnkiError::NetworkError { .. } => SyncError::Network { source: error },
        _ => SyncError::Server { source: error },
    }
}
