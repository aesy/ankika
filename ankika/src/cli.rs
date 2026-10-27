use std::path::PathBuf;

use ankika_core::packages::UpdateCondition;
use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};
use serde::Deserialize;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Profile to open; each has its own collection and AnkiWeb login. Defaults to the
    /// one used last
    #[arg(long, global = true, env = "ANKIKA_PROFILE", value_name = "NAME")]
    pub profile: Option<String>,

    /// Directory holding the collection and its media
    #[arg(long, global = true, env = "ANKIKA_DATA_DIR", value_name = "DIR")]
    pub data_dir: Option<PathBuf>,

    /// Directory holding settings and the AnkiWeb login
    #[arg(long, global = true, env = "ANKIKA_CONFIG_DIR", value_name = "DIR")]
    pub config_dir: Option<PathBuf>,

    /// File to write logs to
    #[arg(long, global = true, env = "ANKIKA_LOG_FILE", value_name = "FILE")]
    pub log_file: Option<PathBuf>,

    /// Self-hosted sync server to log in to instead of AnkiWeb. Defaults to the config
    /// file's `sync_server`
    #[arg(long, global = true, env = "ANKIKA_SYNC_SERVER", value_name = "URL")]
    pub sync_server: Option<String>,

    /// How to draw card images; `auto` asks the terminal what it supports. Defaults to the
    /// config file's `images`, then `auto`
    #[arg(long, global = true, env = "ANKIKA_IMAGES", value_name = "MODE")]
    pub images: Option<ImageMode>,

    /// Print JSON instead of text, for scripts
    #[arg(long, global = true)]
    pub json: bool,

    /// Log more detail: -v for debug, -vv for trace. `RUST_LOG` takes precedence
    #[arg(short, long, global = true, action = ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageMode {
    Auto,
    Kitty,
    Sixel,
    Iterm2,
    /// Colored Unicode half blocks, which work in any terminal
    Halfblocks,
    /// Placeholders with the file name instead of images
    Off,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Log in to AnkiWeb, or to the server given with --sync-server
    Login(LoginArgs),
    /// Forget the stored login
    Logout,
    /// List, add, rename or remove profiles
    #[command(subcommand)]
    Profile(ProfileCommand),
    /// Print how many cards are due today, for status bars
    Due(DueArgs),
    /// Sync the collection and media without opening the TUI
    Sync,
    /// Import an .apkg (decks) or .colpkg (a whole collection, replacing this one)
    Import(ImportArgs),
    /// Export the collection or a deck as .apkg, or the whole collection as .colpkg
    Export(ExportArgs),
    /// Check the collection for problems and fix them, like Anki's Check Database
    CheckDatabase,
    /// Report missing and unused media files
    CheckMedia(CheckMediaArgs),
    /// Report cards whose front is empty, and optionally delete them
    EmptyCards(EmptyCardsArgs),
    /// Print a shell completion script
    Completions { shell: clap_complete::Shell },
    /// Print the man page
    Man,
}

#[derive(Debug, Args)]
pub struct ImportArgs {
    pub file: PathBuf,
    #[command(flatten)]
    pub options: ImportFlags,
    /// Replace the collection without asking (.colpkg)
    #[arg(long)]
    pub yes: bool,
}

/// Anki's `.apkg` import options.
#[derive(Debug, Args)]
pub struct ImportFlags {
    /// Import learning progress (.apkg)
    #[arg(long)]
    pub with_scheduling: bool,
    /// Import deck presets (.apkg)
    #[arg(long)]
    pub with_presets: bool,
    /// Merge note types instead of keeping both (.apkg)
    #[arg(long)]
    pub merge_notetypes: bool,
    /// When to update existing notes (.apkg)
    #[arg(long, value_name = "WHEN", default_value = "if-newer")]
    pub update_notes: UpdateWhen,
    /// When to update existing note types (.apkg)
    #[arg(long, value_name = "WHEN", default_value = "if-newer")]
    pub update_notetypes: UpdateWhen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum UpdateWhen {
    IfNewer,
    Always,
    Never,
}

impl From<UpdateWhen> for UpdateCondition {
    fn from(when: UpdateWhen) -> Self {
        match when {
            UpdateWhen::IfNewer => Self::IfNewer,
            UpdateWhen::Always => Self::Always,
            UpdateWhen::Never => Self::Never,
        }
    }
}

#[derive(Debug, Args)]
pub struct ExportArgs {
    pub file: PathBuf,
    /// Only this deck and its subdecks, by full name (.apkg); the whole collection otherwise
    #[arg(long, value_name = "NAME")]
    pub deck: Option<String>,
    #[command(flatten)]
    pub contents: ContentFlags,
    /// The older format, for Anki before 2.1.50
    #[arg(long)]
    pub legacy: bool,
}

/// What an export leaves out or adds.
#[derive(Debug, Args)]
pub struct ContentFlags {
    /// Leave out learning progress (.apkg)
    #[arg(long)]
    pub no_scheduling: bool,
    /// Leave out media
    #[arg(long)]
    pub no_media: bool,
    /// Include deck presets (.apkg)
    #[arg(long)]
    pub with_presets: bool,
}

#[derive(Debug, Args)]
pub struct CheckMediaArgs {
    /// Move the unused files to Anki's media trash
    #[arg(long)]
    pub delete_unused: bool,
    /// Delete the files in the media trash for good
    #[arg(long, conflicts_with = "restore_trash")]
    pub empty_trash: bool,
    /// Put the files in the media trash back
    #[arg(long)]
    pub restore_trash: bool,
}

#[derive(Debug, Args)]
pub struct EmptyCardsArgs {
    /// Delete the empty cards
    #[arg(long)]
    pub delete: bool,
    /// Also delete notes left without cards, instead of keeping their first card
    #[arg(long)]
    pub no_keep_notes: bool,
}

#[derive(Debug, Args)]
pub struct DueArgs {
    /// Only this deck, by its full name like `Languages::Spanish`
    #[arg(long, value_name = "NAME")]
    pub deck: Option<String>,
    /// Also show new, learning and review counts
    #[arg(long)]
    pub verbose_counts: bool,
}

#[derive(Debug, Subcommand)]
pub enum ProfileCommand {
    /// List the profiles; the one used last is marked with *
    List,
    /// Add an empty profile
    Add { name: String },
    /// Rename a profile, keeping its collection and login
    Rename { old: String, new: String },
    /// Remove a profile with its collection, media and login
    Remove {
        name: String,
        /// Don't ask for confirmation
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Debug, Args)]
pub struct LoginArgs {
    /// AnkiWeb email; asked for when left out
    #[arg(long, value_name = "EMAIL")]
    pub email: Option<String>,
}
