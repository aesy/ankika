use anki::collection::Collection;
use ankika_core::login::{self, LoginError};
use ankika_core::session::{Session, SessionError, SessionStore};
use ankika_core::sync::{self, FullSyncDirection, FullSyncOutput, NormalSyncOutput, SyncError};
use tokio::runtime::Handle;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task;

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
}

/// Starts background work; results come back to the event loop through the channel.
pub struct Tasks {
    sender: UnboundedSender<TaskResult>,
    sessions: SessionStore,
    sync_server: Option<String>,
}

impl Tasks {
    pub const fn new(
        sender: UnboundedSender<TaskResult>,
        sessions: SessionStore,
        sync_server: Option<String>,
    ) -> Self {
        Self {
            sender,
            sessions,
            sync_server,
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

    pub fn sync(&self, mut collection: Collection, mut session: Session) {
        let sender = self.sender.clone();
        let sessions = self.sessions.clone();
        let runtime = Handle::current();

        // Anki's sync futures hold the database across awaits, so they run on their own thread
        task::spawn_blocking(move || {
            let result = runtime.block_on(sync::normal_sync(&mut collection, &session));

            if let Ok(output) = &result
                && let Some(endpoint) = &output.new_endpoint
            {
                session.account.current_endpoint = Some(endpoint.clone());

                if let Err(error) = sessions.save_account(&session.account) {
                    tracing::warn!(?error, "Failed to store the new sync endpoint");
                }
            }

            send(
                &sender,
                TaskResult::Synced {
                    collection,
                    session,
                    result,
                },
            );
        });
    }

    pub fn full_sync(
        &self,
        collection: Collection,
        session: Session,
        direction: FullSyncDirection,
    ) {
        let sender = self.sender.clone();
        let runtime = Handle::current();

        task::spawn_blocking(move || {
            let output = runtime.block_on(sync::full_sync(collection, &session, direction));

            send(&sender, TaskResult::FullSynced(output));
        });
    }
}

fn send(sender: &UnboundedSender<TaskResult>, result: TaskResult) {
    if sender.send(result).is_err() {
        tracing::debug!("Dropped a task result because the app is shutting down");
    }
}
