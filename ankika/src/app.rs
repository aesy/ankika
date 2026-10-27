use std::io;

use ankika_core::collection::{self, CollectionError};
use ankika_core::lock::{CollectionLock, LockError};
use ankika_core::paths::{DEFAULT_PROFILE, Paths, PathsError};
use ankika_core::profiles::{self, ProfileError};
use ankika_core::session::SessionError;

use crate::cli::{Cli, Command};
use crate::command::due::{self, DueCommandError};
use crate::command::export::{self, ExportCommandError};
use crate::command::import::{self, ImportCommandError};
use crate::command::login::LoginCommandError;
use crate::command::maintenance::{self, MaintenanceCommandError};
use crate::command::profile::{self, ProfileCommandError};
use crate::command::sync::{self, SyncCommandError};
use crate::command::{completions, login, logout, man};
use crate::config::{self, ConfigError};
use crate::i18n::{I18n, I18nError};
use crate::logging::{self, LoggingError};
use crate::profile_prompt;
use crate::tui::{self, Exit, Overrides, TuiError};

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
    Profile(#[from] ProfileError),
    #[error(transparent)]
    ProfileCommand(#[from] ProfileCommandError),
    #[error("There is no profile \"{name}\"; see `ankika profile list`")]
    UnknownProfile { name: String },
    #[error("Failed to read the profile choice")]
    PickProfile(#[source] io::Error),
    #[error(transparent)]
    Due(#[from] DueCommandError),
    #[error(transparent)]
    Sync(#[from] SyncCommandError),
    #[error(transparent)]
    Import(#[from] ImportCommandError),
    #[error(transparent)]
    Export(#[from] ExportCommandError),
    #[error(transparent)]
    Maintenance(#[from] MaintenanceCommandError),
    #[error("Failed to print the man page")]
    PrintManPage(#[source] io::Error),
    #[error("Failed to print the completion script")]
    PrintCompletions(#[source] io::Error),
    #[error(transparent)]
    Login(#[from] LoginCommandError),
    #[error(transparent)]
    Logout(#[from] SessionError),
}

pub async fn run(cli: Cli) -> Result<(), AppError> {
    let base = Paths::resolve(cli.data_dir, cli.config_dir)?;
    let log_file = cli
        .log_file
        .unwrap_or_else(|| base.cache_dir().join(LOG_FILE_NAME));

    logging::init(&log_file, cli.verbose)?;

    tracing::info!(
        data_dir = %base.data_dir().display(),
        config_dir = %base.config_dir().display(),
        "Starting ankika"
    );

    let config = config::load(&base.config_file())?;

    match cli.command {
        None => {
            let i18n = I18n::new()?;
            let mut profile = choose_profile(&base, cli.profile, true)?;

            // Switching profiles closes one collection and opens the next
            loop {
                let paths = base.with_profile(&profile);
                let _lock = CollectionLock::acquire(&paths)?;

                profiles::set_last_used(&base, &profile)?;
                tracing::info!(profile, "Opening profile");

                let collection = collection::open(&paths, i18n.anki().clone())?;
                let overrides = Overrides {
                    sync_server: cli.sync_server.clone(),
                    images: cli.images,
                };

                match tui::run(&i18n, &paths, collection, config.clone(), overrides).await? {
                    Exit::Quit => break,
                    Exit::SwitchProfile(next) => profile = next,
                }
            }
        },
        Some(Command::Login(args)) => {
            let paths = base.with_profile(&choose_profile(&base, cli.profile, false)?);

            login::run(&paths, args, cli.sync_server.or(config.sync_server)).await?;
        },
        Some(Command::Logout) => {
            let paths = base.with_profile(&choose_profile(&base, cli.profile, false)?);

            logout::run(&paths)?;
        },
        Some(Command::Profile(command)) => profile::run(&base, command, cli.json)?,
        Some(Command::Due(args)) => {
            let paths = base.with_profile(&choose_profile(&base, cli.profile, false)?);

            due::run(&paths, args.deck.as_deref(), args.verbose_counts, cli.json)?;
        },
        Some(Command::Sync) => {
            let paths = base.with_profile(&choose_profile(&base, cli.profile, false)?);

            sync::run(&paths, cli.json).await?;
        },
        Some(Command::Import(args)) => {
            let paths = base.with_profile(&choose_profile(&base, cli.profile, false)?);

            import::run(&paths, &args, cli.json)?;
        },
        Some(Command::Export(args)) => {
            let paths = base.with_profile(&choose_profile(&base, cli.profile, false)?);

            export::run(&paths, &args, cli.json)?;
        },
        Some(Command::CheckDatabase) => {
            let paths = base.with_profile(&choose_profile(&base, cli.profile, false)?);

            maintenance::check_database(&paths, cli.json)?;
        },
        Some(Command::CheckMedia(args)) => {
            let paths = base.with_profile(&choose_profile(&base, cli.profile, false)?);

            maintenance::check_media(&paths, &args, cli.json)?;
        },
        Some(Command::EmptyCards(args)) => {
            let paths = base.with_profile(&choose_profile(&base, cli.profile, false)?);

            maintenance::empty_cards(&paths, &args, cli.json)?;
        },
        Some(Command::Completions { shell }) => {
            completions::run(shell).map_err(AppError::PrintCompletions)?;
        },
        Some(Command::Man) => man::run().map_err(AppError::PrintManPage)?,
    }

    Ok(())
}

/// The requested profile, else the one used last, else the only one. With several and no
/// history, asks when `ask` is set and opens the default otherwise.
fn choose_profile(base: &Paths, requested: Option<String>, ask: bool) -> Result<String, AppError> {
    if let Some(name) = requested {
        return if profiles::exists(base, &name)? {
            Ok(name)
        } else {
            Err(AppError::UnknownProfile { name })
        };
    }

    if let Some(last) = profiles::last_used(base) {
        return Ok(last);
    }

    let names = profiles::list(base)?;

    if ask && names.len() > 1 {
        profile_prompt::pick(&names).map_err(AppError::PickProfile)
    } else {
        Ok(DEFAULT_PROFILE.to_owned())
    }
}
