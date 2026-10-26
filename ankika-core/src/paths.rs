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
}
