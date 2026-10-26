use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::{FromEnvError, LevelFilter};

#[derive(Debug, thiserror::Error)]
pub enum LoggingError {
    #[error("Failed to create log directory {}", path.display())]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("Failed to open log file {}", path.display())]
    OpenFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("Invalid RUST_LOG filter")]
    InvalidFilter(#[source] FromEnvError),
}

/// Logs go to a file because stdout and stderr belong to the TUI.
pub fn init(log_file: &Path, verbosity: u8) -> Result<(), LoggingError> {
    if let Some(directory) = log_file.parent() {
        fs::create_dir_all(directory).map_err(|source| LoggingError::CreateDirectory {
            path: directory.to_path_buf(),
            source,
        })?;
    }

    let file = File::options()
        .create(true)
        .append(true)
        .open(log_file)
        .map_err(|source| LoggingError::OpenFile {
            path: log_file.to_path_buf(),
            source,
        })?;

    let filter = EnvFilter::builder()
        .with_default_directive(level(verbosity).into())
        .from_env()
        .map_err(LoggingError::InvalidFilter)?;

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(Mutex::new(file))
        .with_ansi(false)
        .init();

    Ok(())
}

const fn level(verbosity: u8) -> LevelFilter {
    match verbosity {
        0 => LevelFilter::INFO,
        1 => LevelFilter::DEBUG,
        _ => LevelFilter::TRACE,
    }
}
