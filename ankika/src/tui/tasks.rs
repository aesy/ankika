use std::future::Future;
use std::path::PathBuf;

use anki::collection::Collection;
use ankika_core::collection::CollectionError;
use ankika_core::login::{self, LoginError};
use ankika_core::media_sync;
use ankika_core::packages::{
    self, Contents, ExportScope, Format, ImportOptions, ImportSummary, PackageError,
};
use ankika_core::paths::Paths;
use ankika_core::session::{Session, SessionError, SessionStore};
use ankika_core::sync::{self, FullSyncDirection, FullSyncOutput, NormalSyncOutput, SyncError};
use futures_util::future::{AbortHandle, Abortable};
use tokio::runtime::Handle;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::watch;

use crate::tui::audio::AudioEvent;
use tokio::task::{self, JoinSet};

#[derive(Debug, thiserror::Error)]
pub enum LogInTaskError {
    #[error(transparent)]
    Login(#[from] LoginError),
    #[error(transparent)]
    Save(#[from] SessionError),
}

pub enum TaskResult {
    LoggedIn(Result<Session, LogInTaskError>),
    LoggedOut(Result<(), SessionError>),
    /// Hands the collection back, with the session updated if AnkiWeb moved the account.
    Synced {
        collection: Collection,
        session: Session,
        result: Result<NormalSyncOutput, SyncError>,
    },
    FullSynced(FullSyncOutput),
    /// Not sent when the media sync was aborted.
    MediaSynced(Result<(), SyncError>),
    Audio(AudioEvent),
    /// The collection comes back reopened after the jobs that close it.
    Package {
        collection: Result<Collection, CollectionError>,
        result: Result<PackageDone, PackageError>,
    },
}

pub enum PackageJob {
    ImportApkg {
        file: PathBuf,
        options: ImportOptions,
    },
    ImportColpkg {
        file: PathBuf,
    },
    ExportApkg {
        file: PathBuf,
        scope: ExportScope,
        contents: Contents,
        format: Format,
    },
    ExportColpkg {
        file: PathBuf,
        media: bool,
        format: Format,
    },
}

pub enum PackageDone {
    Imported(ImportSummary),
    Replaced,
    ExportedNotes(usize),
    ExportedCollection,
}

/// Starts background work; results come back to the event loop through the channel.
pub struct Tasks {
    sender: UnboundedSender<TaskResult>,
    sessions: SessionStore,
    sync_server: Option<String>,
    syncs: JoinSet<()>,
    shutdown: watch::Sender<bool>,
}

impl Tasks {
    pub fn new(
        sender: UnboundedSender<TaskResult>,
        sessions: SessionStore,
        sync_server: Option<String>,
    ) -> Self {
        Self {
            sender,
            sessions,
            sync_server,
            syncs: JoinSet::new(),
            shutdown: watch::Sender::new(false),
        }
    }

    pub fn log_in(&self, email: String, password: String) {
        let sender = self.sender.clone();
        let sessions = self.sessions.clone();
        let sync_server = self.sync_server.clone();

        tokio::spawn(async move {
            let result: Result<Session, LogInTaskError> = async {
                let session = login::log_in(email, password, sync_server).await?;
                // Keyring access blocks
                task::block_in_place(|| sessions.save(&session))?;

                Ok(session)
            }
            .await;

            send(&sender, TaskResult::LoggedIn(result));
        });
    }

    pub fn log_out(&self) {
        let sender = self.sender.clone();
        let sessions = self.sessions.clone();

        tokio::spawn(async move {
            let result = task::block_in_place(|| sessions.clear());

            send(&sender, TaskResult::LoggedOut(result));
        });
    }

    pub fn sync(&mut self, mut collection: Collection, mut session: Session) {
        let sessions = self.sessions.clone();

        self.spawn_sync(move || async move {
            let result = sync::normal_sync(&mut collection, &session).await;

            if let Ok(output) = &result
                && let Some(endpoint) = &output.new_endpoint
            {
                session.account.current_endpoint = Some(endpoint.clone());

                if let Err(error) = sessions.save_account(&session.account) {
                    tracing::warn!(?error, "Failed to store the new sync endpoint");
                }
            }

            Some(TaskResult::Synced {
                collection,
                session,
                result,
            })
        });
    }

    pub fn full_sync(
        &mut self,
        collection: Collection,
        session: Session,
        direction: FullSyncDirection,
    ) {
        self.spawn_sync(move || async move {
            let output = sync::full_sync(collection, &session, direction).await;

            Some(TaskResult::FullSynced(output))
        });
    }

    /// Imports or exports with the collection on its own thread, like a sync.
    pub fn package(&mut self, mut collection: Collection, paths: Paths, job: PackageJob) {
        self.spawn_sync(move || async move {
            let (collection, result) = match job {
                PackageJob::ImportApkg { file, options } => {
                    let result = packages::import_apkg(&mut collection, &file, options)
                        .map(PackageDone::Imported);

                    (Ok(collection), result)
                },
                PackageJob::ImportColpkg { file } => {
                    let reopened = packages::import_colpkg(collection, &paths, &file);

                    (
                        reopened.collection,
                        reopened.result.map(|()| PackageDone::Replaced),
                    )
                },
                PackageJob::ExportApkg {
                    file,
                    scope,
                    contents,
                    format,
                } => {
                    let result =
                        packages::export_apkg(&mut collection, &file, scope, contents, format)
                            .map(PackageDone::ExportedNotes);

                    (Ok(collection), result)
                },
                PackageJob::ExportColpkg {
                    file,
                    media,
                    format,
                } => {
                    let reopened = packages::export_colpkg(collection, &file, media, format);

                    (
                        reopened.collection,
                        reopened.result.map(|()| PackageDone::ExportedCollection),
                    )
                },
            };

            Some(TaskResult::Package { collection, result })
        });
    }

    pub fn media_sync(
        &mut self,
        collection: &Collection,
        session: &Session,
    ) -> Result<AbortHandle, SyncError> {
        let sync = media_sync::media_sync(collection, session)?;
        let (abort, registration) = AbortHandle::new_pair();

        self.spawn_sync(move || async move {
            let result = Abortable::new(sync, registration).await.ok()?;

            Some(TaskResult::MediaSynced(result))
        });

        Ok(abort)
    }

    /// Stops the running syncs at their next await and waits for them, so none of them
    /// outlives the runtime.
    pub async fn shutdown(mut self) {
        self.shutdown.send_replace(true);

        while self.syncs.join_next().await.is_some() {}
    }

    /// Anki's sync futures hold the database across awaits, so each runs on its own thread.
    fn spawn_sync<F, Fut>(&mut self, start: F)
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Option<TaskResult>>,
    {
        let sender = self.sender.clone();
        let runtime = Handle::current();
        let mut shutdown = self.shutdown.subscribe();

        while self.syncs.try_join_next().is_some() {}

        self.syncs.spawn_blocking(move || {
            let result = runtime.block_on(async {
                tokio::select! {
                    result = start() => result,
                    _ = shutdown.wait_for(|&stopping| stopping) => None,
                }
            });

            if let Some(result) = result {
                send(&sender, result);
            }
        });
    }
}

fn send(sender: &UnboundedSender<TaskResult>, result: TaskResult) {
    if sender.send(result).is_err() {
        tracing::debug!("Dropped a task result because the app is shutting down");
    }
}
