use ankika_core::collection::{self, CollectionError};
use ankika_core::lock::{CollectionLock, LockError};
use ankika_core::paths::{Paths, PathsError};
use ankika_core::session::SessionError;

use crate::cli::{Cli, Command};
use crate::command::login::LoginCommandError;
use crate::command::{login, logout};
use crate::config::{self, ConfigError};
use crate::i18n::{I18n, I18nError};
use crate::logging::{self, LoggingError};
use crate::tui::{self, Overrides, TuiError};

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
    Collection(#[from] CollectionError),
    #[error(transparent)]
    I18n(#[from] I18nError),
    #[error(transparent)]
    Tui(#[from] TuiError),
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Login(#[from] LoginCommandError),
    #[error(transparent)]
    Logout(#[from] SessionError),
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

    let config = config::load(&paths.config_file())?;

    match cli.command {
        None => {
            let _lock = CollectionLock::acquire(&paths)?;
            let i18n = I18n::new()?;
            let collection = collection::open(&paths, i18n.anki().clone())?;
            let overrides = Overrides {
                sync_server: cli.sync_server,
                images: cli.images,
            };

            tui::run(&i18n, &paths, collection, config, overrides).await?;
        },
        Some(Command::Login(args)) => {
            login::run(&paths, args, cli.sync_server.or(config.sync_server)).await?;
        },
        Some(Command::Logout) => logout::run(&paths)?,
    }

    Ok(())
}
