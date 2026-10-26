use ankika_core::login::{self, LoginError};
use ankika_core::session::{Session, SessionError, SessionStore};
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
}

fn send(sender: &UnboundedSender<TaskResult>, result: TaskResult) {
    if sender.send(result).is_err() {
        tracing::debug!("Dropped a task result because the app is shutting down");
    }
}
