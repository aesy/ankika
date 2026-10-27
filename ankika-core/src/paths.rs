use std::path::{Path, PathBuf};

use directories::ProjectDirs;

const APP_NAME: &str = "ankika";
/// The profile that lives directly in the data and config directories, where ankika
/// kept everything before it had profiles.
pub const DEFAULT_PROFILE: &str = "default";
const PROFILES_DIR: &str = "profiles";

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
    profile: String,
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
            profile: DEFAULT_PROFILE.to_owned(),
        })
    }

    /// The same directories with another profile's collection and account.
    #[must_use]
    pub fn with_profile(&self, profile: &str) -> Self {
        Self {
            profile: profile.to_owned(),
            ..self.clone()
        }
    }

    #[must_use]
    pub fn profile(&self) -> &str {
        &self.profile
    }

    /// Where the other profiles' folders are, in the data and config directories.
    #[must_use]
    pub fn profiles_dirs(&self) -> (PathBuf, PathBuf) {
        (self.data.join(PROFILES_DIR), self.config.join(PROFILES_DIR))
    }

    /// The profile's collection and media.
    #[must_use]
    pub fn profile_data_dir(&self) -> PathBuf {
        if self.profile == DEFAULT_PROFILE {
            self.data.clone()
        } else {
            self.data.join(PROFILES_DIR).join(&self.profile)
        }
    }

    /// The profile's account.
    #[must_use]
    pub fn profile_config_dir(&self) -> PathBuf {
        if self.profile == DEFAULT_PROFILE {
            self.config.clone()
        } else {
            self.config.join(PROFILES_DIR).join(&self.profile)
        }
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
        self.profile_data_dir().join("collection.anki2")
    }

    #[must_use]
    pub fn media_dir(&self) -> PathBuf {
        self.profile_data_dir().join("collection.media")
    }

    #[must_use]
    pub fn media_db_file(&self) -> PathBuf {
        self.profile_data_dir().join("collection.media.db2")
    }

    #[must_use]
    pub fn lock_file(&self) -> PathBuf {
        self.profile_data_dir().join("ankika.lock")
    }

    /// ankika's own settings, shared by all profiles; Anki's live in the collection.
    #[must_use]
    pub fn config_file(&self) -> PathBuf {
        self.config.join("config.toml")
    }

    /// The profile opened last, for the next start.
    #[must_use]
    pub fn last_profile_file(&self) -> PathBuf {
        self.config.join("last-profile")
    }

    #[must_use]
    pub fn account_file(&self) -> PathBuf {
        self.profile_config_dir().join("account.toml")
    }

    /// Only used when no OS keyring is available.
    #[must_use]
    pub fn host_key_file(&self) -> PathBuf {
        self.profile_config_dir().join("ankiweb-host-key")
    }
}
