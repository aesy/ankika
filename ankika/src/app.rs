use ankika_core::paths::{Paths, PathsError};

use crate::cli::Cli;
use crate::logging::{self, LoggingError};

const LOG_FILE_NAME: &str = "ankika.log";

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Paths(#[from] PathsError),
    #[error(transparent)]
    Logging(#[from] LoggingError),
}

pub fn run(cli: Cli) -> Result<(), AppError> {
    let paths = Paths::resolve(cli.data_dir, cli.config_dir)?;
    let log_file = cli
        .log_file
        .unwrap_or_else(|| paths.cache_dir().join(LOG_FILE_NAME));

    logging::init(&log_file, cli.verbose)?;

    tracing::info!(
        data_dir = %paths.data_dir().display(),
        config_dir = %paths.config_dir().display(),
        "Starting ankika"
    );

    Ok(())
}
