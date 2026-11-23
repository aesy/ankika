use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::host_key::{HostKeyError, HostKeyStore};
use crate::lock::{CollectionLock, LockError};
use crate::paths::{DEFAULT_PROFILE, Paths};

/// Keeps names usable as folder names everywhere.
const MAX_NAME_LENGTH: usize = 64;

#[derive(Debug, thiserror::Error)]
pub enum ProfileError {
    #[error("\"{name}\" is not a valid profile name; use letters, digits, - and _")]
    InvalidName { name: String },
    #[error("The profile \"{name}\" already exists")]
    Exists { name: String },
    #[error("There is no profile \"{name}\"")]
    Missing { name: String },
    #[error("The default profile can't be renamed or removed")]
    Default,
    #[error("The profile \"{name}\" is open in another ankika")]
    InUse {
        name: String,
        #[source]
        source: LockError,
    },
    #[error("Failed to change {}", path.display())]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error(transparent)]
    HostKey(#[from] HostKeyError),
}

/// The default profile first, then the others by name.
pub fn list(paths: &Paths) -> Result<Vec<String>, ProfileError> {
    let (data, _) = paths.profiles_dirs();
    let mut names = Vec::new();

    match fs::read_dir(&data) {
        Ok(entries) => {
            for entry in entries {
                let entry = entry.map_err(|source| io_error(&data, source))?;

                if entry.path().is_dir()
                    && let Some(name) = entry.file_name().to_str()
                {
                    names.push(name.to_owned());
                }
            }
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound => {},
        Err(source) => return Err(io_error(&data, source)),
    }

    names.sort();
    names.insert(0, DEFAULT_PROFILE.to_owned());

    Ok(names)
}

pub fn exists(paths: &Paths, name: &str) -> Result<bool, ProfileError> {
    Ok(list(paths)?.iter().any(|profile| profile == name))
}

pub fn add(paths: &Paths, name: &str) -> Result<(), ProfileError> {
    validate(name)?;

    if exists(paths, name)? {
        return Err(ProfileError::Exists {
            name: name.to_owned(),
        });
    }

    let profile = paths.with_profile(name);

    create_dir(&profile.profile_data_dir())?;
    create_dir(&profile.profile_config_dir())
}

/// Moves the profile's folders and its stored login.
pub fn rename(paths: &Paths, old: &str, new: &str) -> Result<(), ProfileError> {
    let from = changeable(paths, old)?;

    validate(new)?;

    if exists(paths, new)? {
        return Err(ProfileError::Exists {
            name: new.to_owned(),
        });
    }

    let to = paths.with_profile(new);
    let old_key = HostKeyStore::new(&from);

    move_dir(&from.profile_data_dir(), &to.profile_data_dir())?;
    move_dir(&from.profile_config_dir(), &to.profile_config_dir())?;

    if let Some(key) = old_key.load()? {
        HostKeyStore::new(&to).save(&key)?;
        old_key.delete()?;
    }

    Ok(())
}

/// Deletes the profile's collection, media, account and stored login.
pub fn remove(paths: &Paths, name: &str) -> Result<(), ProfileError> {
    let profile = changeable(paths, name)?;

    HostKeyStore::new(&profile).delete()?;
    remove_dir(&profile.profile_data_dir())?;
    remove_dir(&profile.profile_config_dir())
}

/// The profile used last, if it still exists.
#[must_use]
pub fn last_used(paths: &Paths) -> Option<String> {
    let name = fs::read_to_string(paths.last_profile_file()).ok()?;
    let name = name.trim();

    exists(paths, name).ok()?.then(|| name.to_owned())
}

pub fn set_last_used(paths: &Paths, name: &str) -> Result<(), ProfileError> {
    let file = paths.last_profile_file();

    if let Some(parent) = file.parent() {
        create_dir(parent)?;
    }

    fs::write(&file, name).map_err(|source| io_error(&file, source))
}

/// A profile other than the default that exists and isn't open anywhere.
fn changeable(paths: &Paths, name: &str) -> Result<Paths, ProfileError> {
    if name == DEFAULT_PROFILE {
        return Err(ProfileError::Default);
    }

    if !exists(paths, name)? {
        return Err(ProfileError::Missing {
            name: name.to_owned(),
        });
    }

    let profile = paths.with_profile(name);

    // Only checks: the lock file is inside the folder that is about to change
    drop(
        CollectionLock::acquire(&profile).map_err(|source| ProfileError::InUse {
            name: name.to_owned(),
            source,
        })?,
    );

    Ok(profile)
}

fn validate(name: &str) -> Result<(), ProfileError> {
    let valid = !name.is_empty()
        && name.len() <= MAX_NAME_LENGTH
        && name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'));

    if valid {
        Ok(())
    } else {
        Err(ProfileError::InvalidName {
            name: name.to_owned(),
        })
    }
}

fn create_dir(path: &Path) -> Result<(), ProfileError> {
    fs::create_dir_all(path).map_err(|source| io_error(path, source))
}

/// A missing source is fine: a new profile may never have stored an account.
fn move_dir(from: &Path, to: &Path) -> Result<(), ProfileError> {
    if !from.exists() {
        return Ok(());
    }

    fs::rename(from, to).map_err(|source| io_error(from, source))
}

fn remove_dir(path: &Path) -> Result<(), ProfileError> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(io_error(path, source)),
    }
}

fn io_error(path: &Path, source: io::Error) -> ProfileError {
    ProfileError::Io {
        path: path.to_owned(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_names() {
        assert!(validate("work_2").is_ok());
        assert!(validate("").is_err());
        assert!(validate("../x").is_err());
        assert!(validate("a b").is_err());
    }
}
