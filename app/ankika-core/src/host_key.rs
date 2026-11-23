use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use keyring::Entry;

use crate::paths::{DEFAULT_PROFILE, Paths};

const KEYRING_SERVICE: &str = "ankika";
const KEYRING_ACCOUNT: &str = "ankiweb-host-key";

#[derive(Debug, thiserror::Error)]
pub enum HostKeyError {
    #[error("Failed to access the system keyring")]
    Keyring(#[source] keyring::Error),
    #[error("Failed to create directory {}", path.display())]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("Failed to read {}", path.display())]
    ReadFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("Failed to write {}", path.display())]
    WriteFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("Failed to delete {}", path.display())]
    DeleteFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

/// The AnkiWeb host key, kept in the OS keyring.
///
/// Falls back to a file readable only by the user when the platform has no keyring at all
/// (for example a headless Linux box without Secret Service). A keyring that exists but
/// refuses access is an error, never a reason to fall back to plain text.
#[derive(Debug, Clone)]
pub struct HostKeyStore {
    file: PathBuf,
    /// The keyring entry's account, one per profile.
    account: String,
}

impl HostKeyStore {
    #[must_use]
    pub fn new(paths: &Paths) -> Self {
        // The default profile keeps the entry from before profiles existed
        let account = if paths.profile() == DEFAULT_PROFILE {
            KEYRING_ACCOUNT.to_owned()
        } else {
            format!("{KEYRING_ACCOUNT}:{}", paths.profile())
        };

        Self {
            file: paths.host_key_file(),
            account,
        }
    }

    pub fn load(&self) -> Result<Option<String>, HostKeyError> {
        let Some(entry) = keyring_entry(&self.account)? else {
            return self.load_file();
        };

        match entry.get_password() {
            Ok(key) => Ok(Some(key)),
            // Saved while no keyring was available
            Err(keyring::Error::NoEntry) => self.load_file(),
            Err(error) => Err(HostKeyError::Keyring(error)),
        }
    }

    pub fn save(&self, key: &str) -> Result<(), HostKeyError> {
        keyring_entry(&self.account)?.map_or_else(
            || self.save_file(key),
            |entry| entry.set_password(key).map_err(HostKeyError::Keyring),
        )
    }

    pub fn delete(&self) -> Result<(), HostKeyError> {
        if let Some(entry) = keyring_entry(&self.account)? {
            match entry.delete_credential() {
                Ok(()) | Err(keyring::Error::NoEntry) => {},
                Err(error) => return Err(HostKeyError::Keyring(error)),
            }
        }

        match fs::remove_file(&self.file) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(source) => Err(HostKeyError::DeleteFile {
                path: self.file.clone(),
                source,
            }),
        }
    }

    fn load_file(&self) -> Result<Option<String>, HostKeyError> {
        match fs::read_to_string(&self.file) {
            Ok(key) => Ok(Some(key.trim().to_owned())),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(source) => Err(HostKeyError::ReadFile {
                path: self.file.clone(),
                source,
            }),
        }
    }

    fn save_file(&self, key: &str) -> Result<(), HostKeyError> {
        if let Some(directory) = self.file.parent() {
            fs::create_dir_all(directory).map_err(|source| HostKeyError::CreateDirectory {
                path: directory.to_path_buf(),
                source,
            })?;
        }

        write_private(&self.file, key).map_err(|source| HostKeyError::WriteFile {
            path: self.file.clone(),
            source,
        })
    }
}

/// `None` when the platform has no keyring to use.
fn keyring_entry(account: &str) -> Result<Option<Entry>, HostKeyError> {
    match Entry::new(KEYRING_SERVICE, account) {
        Ok(entry) => Ok(Some(entry)),
        Err(keyring::Error::NoDefaultStore) => {
            tracing::warn!("No system keyring available, storing the AnkiWeb host key in a file");
            Ok(None)
        },
        Err(error) => Err(HostKeyError::Keyring(error)),
    }
}

fn write_private(path: &Path, contents: &str) -> io::Result<()> {
    let mut file = File::create(path)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        // Also tightens a file that already existed with wider permissions
        file.set_permissions(fs::Permissions::from_mode(0o600))?;
    }

    file.write_all(contents.as_bytes())
}
