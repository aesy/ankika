use std::path::{Path, PathBuf};

use directories::ProjectDirs;

const APP_NAME: &str = "ankika";

#[derive(Debug, thiserror::Error)]
pub enum PathsError {
    #[error("Could not determine the home directory")]
    NoHomeDirectory,
}

#[derive(Debug, Clone)]
pub struct Paths {
    data: PathBuf,
    config: PathBuf,
    cache: PathBuf,
}

impl Paths {
    /// Uses the platform's standard directories for anything not overridden.
    pub fn resolve(
        data_dir: Option<PathBuf>,
        config_dir: Option<PathBuf>,
    ) -> Result<Self, PathsError> {
        let project_dirs =
            ProjectDirs::from("", "", APP_NAME).ok_or(PathsError::NoHomeDirectory)?;

        Ok(Self {
            data: data_dir.unwrap_or_else(|| project_dirs.data_dir().to_path_buf()),
            config: config_dir.unwrap_or_else(|| project_dirs.config_dir().to_path_buf()),
            cache: project_dirs.cache_dir().to_path_buf(),
        })
    }

    #[must_use]
    pub fn data_dir(&self) -> &Path {
        &self.data
    }

    #[must_use]
    pub fn config_dir(&self) -> &Path {
        &self.config
    }

    #[must_use]
    pub fn cache_dir(&self) -> &Path {
        &self.cache
    }

    #[must_use]
    pub fn collection_file(&self) -> PathBuf {
        self.data.join("collection.anki2")
    }

    #[must_use]
    pub fn media_dir(&self) -> PathBuf {
        self.data.join("collection.media")
    }

    #[must_use]
    pub fn media_db_file(&self) -> PathBuf {
        self.data.join("collection.media.db2")
    }

    #[must_use]
    pub fn lock_file(&self) -> PathBuf {
        self.data.join("ankika.lock")
    }

    #[must_use]
    pub fn account_file(&self) -> PathBuf {
        self.config.join("account.toml")
    }

    /// Only used when no OS keyring is available.
    #[must_use]
    pub fn host_key_file(&self) -> PathBuf {
        self.config.join("ankiweb-host-key")
    }
}
