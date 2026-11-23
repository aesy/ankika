use std::future::Future;

use anki::collection::Collection;

use crate::remote;
use crate::session::Session;
use crate::sync::{self, SyncError};

/// Prepares a media sync. The returned future works on the media folder and its own
/// database only, so the collection stays usable while it runs.
pub fn media_sync(
    collection: &Collection,
    session: &Session,
) -> Result<impl Future<Output = Result<(), SyncError>> + use<>, SyncError> {
    let auth = sync::auth(session)?;
    let client = remote::http_client().map_err(SyncError::HttpClient)?;
    let manager = collection
        .media()
        .map_err(|source| SyncError::OpenMediaDatabase { source })?;
    let progress = collection.new_progress_handler();

    Ok(async move {
        manager
            .sync_media(progress, auth, client, None)
            .await
            .map_err(sync::classify)
    })
}
