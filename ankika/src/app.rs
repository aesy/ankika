use ankika_core::lock::{CollectionLock, LockError};
use ankika_core::paths::{Paths, PathsError};

use crate::cli::Cli;
use crate::i18n::{I18n, I18nError};
use crate::logging::{self, LoggingError};
use crate::tui::{self, TuiError};

const LOG_FILE_NAME: &str = "ankika.log";

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Paths(#[from] PathsError),
    #[error(transparent)]
    Logging(#[from] LoggingError),
    #[error(transparent)]
    Lock(#[from] LockError),
    #[error(transparent)]
    I18n(#[from] I18nError),
    #[error(transparent)]
    Tui(#[from] TuiError),
}

pub async fn run(cli: Cli) -> Result<(), AppError> {
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

    let _lock = CollectionLock::acquire(&paths)?;
    let i18n = I18n::new()?;
    tui::run(&i18n).await?;

    Ok(())
}
