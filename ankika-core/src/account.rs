use std::fs;
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::paths::Paths;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    pub username: String,
    /// A self-hosted sync server; `None` means AnkiWeb.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sync_server: Option<String>,
    /// The AnkiWeb server this account was last sent to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_endpoint: Option<String>,
}

impl Account {
    /// A self-hosted server always wins over AnkiWeb's redirects, as in Anki.
    #[must_use]
    pub fn endpoint(&self) -> Option<&str> {
        self.sync_server
            .as_deref()
            .or(self.current_endpoint.as_deref())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    #[error("Failed to read {}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("Failed to parse {}", path.display())]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("Failed to serialize the account")]
    Serialize(#[source] toml::ser::Error),
    #[error("Failed to create directory {}", path.display())]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("Failed to write {}", path.display())]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("Failed to delete {}", path.display())]
    Delete {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

/// The non-secret half of a login, kept in the config directory.
#[derive(Debug, Clone)]
pub struct AccountStore {
    file: PathBuf,
}

impl AccountStore {
    #[must_use]
    pub fn new(paths: &Paths) -> Self {
        Self {
            file: paths.account_file(),
        }
    }

    pub fn load(&self) -> Result<Option<Account>, AccountError> {
        let contents = match fs::read_to_string(&self.file) {
            Ok(contents) => contents,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(source) => {
                return Err(AccountError::Read {
                    path: self.file.clone(),
                    source,
                });
            },
        };

        toml::from_str(&contents)
            .map(Some)
            .map_err(|source| AccountError::Parse {
                path: self.file.clone(),
                source,
            })
    }

    pub fn save(&self, account: &Account) -> Result<(), AccountError> {
        let contents = toml::to_string(account).map_err(AccountError::Serialize)?;

        if let Some(directory) = self.file.parent() {
            fs::create_dir_all(directory).map_err(|source| AccountError::CreateDirectory {
                path: directory.to_path_buf(),
                source,
            })?;
        }

        fs::write(&self.file, contents).map_err(|source| AccountError::Write {
            path: self.file.clone(),
            source,
        })
    }

    pub fn delete(&self) -> Result<(), AccountError> {
        match fs::remove_file(&self.file) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(source) => Err(AccountError::Delete {
                path: self.file.clone(),
                source,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[test]
    fn save_load_delete_round_trip() {
        let config_dir =
            env::temp_dir().join(format!("ankika-account-test-{}", std::process::id()));
        let paths = Paths::resolve(None, Some(config_dir.clone())).unwrap();
        let store = AccountStore::new(&paths);
        let account = Account {
            username: "user@example.com".to_owned(),
            sync_server: Some("https://sync.example.com/".to_owned()),
            current_endpoint: None,
        };

        assert_eq!(store.load().unwrap(), None);
        store.save(&account).unwrap();
        assert_eq!(store.load().unwrap(), Some(account));
        store.delete().unwrap();
        assert_eq!(store.load().unwrap(), None);

        fs::remove_dir_all(config_dir).unwrap();
    }
}
