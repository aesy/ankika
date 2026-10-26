use crate::account::{Account, AccountError, AccountStore};
use crate::host_key::{HostKeyError, HostKeyStore};
use crate::paths::Paths;

/// A logged-in AnkiWeb (or self-hosted) account.
#[derive(Debug, Clone)]
pub struct Session {
    pub account: Account,
    pub host_key: String,
}

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error(transparent)]
    Account(#[from] AccountError),
    #[error(transparent)]
    HostKey(#[from] HostKeyError),
}

/// Keeps the account in the config directory and the host key in the keyring.
///
/// Keyring access blocks, so async callers should run these off the runtime threads.
#[derive(Debug, Clone)]
pub struct SessionStore {
    accounts: AccountStore,
    host_keys: HostKeyStore,
}

impl SessionStore {
    #[must_use]
    pub fn new(paths: &Paths) -> Self {
        Self {
            accounts: AccountStore::new(paths),
            host_keys: HostKeyStore::new(paths),
        }
    }

    /// `None` unless both halves of the session are present.
    pub fn load(&self) -> Result<Option<Session>, SessionError> {
        let Some(account) = self.accounts.load()? else {
            return Ok(None);
        };
        let Some(host_key) = self.host_keys.load()? else {
            tracing::warn!("Account found without a host key, treating as logged out");
            return Ok(None);
        };

        Ok(Some(Session { account, host_key }))
    }

    pub fn save(&self, session: &Session) -> Result<(), SessionError> {
        self.host_keys.save(&session.host_key)?;
        self.accounts.save(&session.account)?;

        Ok(())
    }

    pub fn clear(&self) -> Result<(), SessionError> {
        self.accounts.delete()?;
        self.host_keys.delete()?;

        Ok(())
    }
}
