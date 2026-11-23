use anki::prelude::I18n;
use ankika_core::collection::{self, CollectionError};
use ankika_core::lock::{CollectionLock, LockError};
use ankika_core::media_sync;
use ankika_core::paths::Paths;
use ankika_core::session::{SessionError, SessionStore};
use ankika_core::sync::{self, FullSyncDirection, FullSyncRequired, SyncError};

#[derive(Debug, thiserror::Error)]
pub enum SyncCommandError {
    #[error("ankika is already open; it syncs on its own")]
    AlreadyOpen,
    #[error(transparent)]
    Lock(LockError),
    #[error(transparent)]
    Session(#[from] SessionError),
    #[error("Not logged in; run `ankika login` first")]
    NotLoggedIn,
    #[error(transparent)]
    Collection(#[from] CollectionError),
    #[error(transparent)]
    Sync(#[from] SyncError),
    #[error("AnkiWeb has no cards yet; open ankika to upload this collection")]
    UploadNeeded,
    #[error(
        "AnkiWeb and this device both changed in ways that can't be merged; open ankika to choose which to keep"
    )]
    FullSyncNeeded,
}

/// What happened to the collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Synced,
    /// This device was empty, so AnkiWeb's collection was downloaded, like the TUI does.
    Downloaded,
}

/// Syncs the collection and then the media, like the TUI does on start.
pub async fn run(paths: &Paths, json: bool) -> Result<(), SyncCommandError> {
    let _lock = CollectionLock::acquire(paths).map_err(|error| match error {
        LockError::AlreadyLocked => SyncCommandError::AlreadyOpen,
        error => SyncCommandError::Lock(error),
    })?;
    let sessions = SessionStore::new(paths);
    let mut session = sessions.load()?.ok_or(SyncCommandError::NotLoggedIn)?;
    let mut collection = collection::open(paths, I18n::template_only())?;
    let output = sync::normal_sync(&mut collection, &session).await?;

    if let Some(endpoint) = output.new_endpoint {
        session.account.current_endpoint = Some(endpoint);
        sessions.save_account(&session.account)?;
    }

    let outcome = match output.full_sync_required {
        None => Outcome::Synced,
        Some(FullSyncRequired::DownloadOnly) => {
            let full = sync::full_sync(collection, &session, FullSyncDirection::Download).await;

            collection = full.collection?;
            full.result?;
            Outcome::Downloaded
        },
        // Both replace a side completely, which only the TUI asks about
        Some(FullSyncRequired::UploadOnly) => {
            close(collection);
            return Err(SyncCommandError::UploadNeeded);
        },
        Some(FullSyncRequired::Conflict) => {
            close(collection);
            return Err(SyncCommandError::FullSyncNeeded);
        },
    };
    let media = media_sync::media_sync(&collection, &session)?.await;

    close(collection);
    media?;
    print(outcome, &output.server_message, json);

    Ok(())
}

fn print(outcome: Outcome, server_message: &str, json: bool) {
    let collection = match outcome {
        Outcome::Synced => "synced",
        Outcome::Downloaded => "downloaded",
    };

    if json {
        let output = serde_json::json!({
            "collection": collection,
            "media": "synced",
            "message": server_message,
        });

        println!("{output}");
        return;
    }

    match outcome {
        Outcome::Synced => println!("Synced"),
        Outcome::Downloaded => println!("Downloaded the collection from AnkiWeb"),
    }

    // AnkiWeb's own announcements, which the TUI shows in its footer
    if !server_message.is_empty() {
        println!("{server_message}");
    }
}

fn close(collection: anki::collection::Collection) {
    if let Err(error) = collection.close(None) {
        tracing::warn!(?error, "Failed to close the collection");
    }
}
