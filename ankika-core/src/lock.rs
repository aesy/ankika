use std::fs::{self, File, TryLockError};
use std::io;
use std::path::PathBuf;

use crate::paths::Paths;

#[derive(Debug, thiserror::Error)]
pub enum LockError {
    #[error("Failed to create directory {}", path.display())]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("Failed to open lock file {}", path.display())]
    Open {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("Another ankika instance is using this collection")]
    AlreadyLocked,
    #[error("Failed to lock {}", path.display())]
    Lock {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

/// Exclusive access to the collection until dropped. The OS releases it if the process dies.
#[derive(Debug)]
pub struct CollectionLock {
    _file: File,
}

impl CollectionLock {
    pub fn acquire(paths: &Paths) -> Result<Self, LockError> {
        let directory = paths.data_dir();

        fs::create_dir_all(directory).map_err(|source| LockError::CreateDirectory {
            path: directory.to_path_buf(),
            source,
        })?;

        let path = paths.lock_file();
        let file = File::options()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&path)
            .map_err(|source| LockError::Open {
                path: path.clone(),
                source,
            })?;

        match file.try_lock() {
            Ok(()) => Ok(Self { _file: file }),
            Err(TryLockError::WouldBlock) => Err(LockError::AlreadyLocked),
            Err(TryLockError::Error(source)) => Err(LockError::Lock { path, source }),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[test]
    fn second_lock_is_refused_until_first_is_dropped() {
        let data_dir = env::temp_dir().join(format!("ankika-lock-test-{}", std::process::id()));
        let paths = Paths::resolve(Some(data_dir.clone()), None).unwrap();

        let first = CollectionLock::acquire(&paths).unwrap();
        assert!(matches!(
            CollectionLock::acquire(&paths),
            Err(LockError::AlreadyLocked)
        ));

        drop(first);
        assert!(CollectionLock::acquire(&paths).is_ok());

        fs::remove_dir_all(data_dir).unwrap();
    }
}
