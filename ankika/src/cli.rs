use std::path::PathBuf;

use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Directory holding the collection and its media
    #[arg(long, global = true, env = "ANKIKA_DATA_DIR", value_name = "DIR")]
    pub data_dir: Option<PathBuf>,

    /// Directory holding settings and the AnkiWeb login
    #[arg(long, global = true, env = "ANKIKA_CONFIG_DIR", value_name = "DIR")]
    pub config_dir: Option<PathBuf>,

    /// File to write logs to
    #[arg(long, global = true, env = "ANKIKA_LOG_FILE", value_name = "FILE")]
    pub log_file: Option<PathBuf>,

    /// Self-hosted sync server to log in to instead of AnkiWeb
    #[arg(long, global = true, env = "ANKIKA_SYNC_SERVER", value_name = "URL")]
    pub sync_server: Option<String>,

    /// How to draw card images; `auto` asks the terminal what it supports
    #[arg(
        long,
        global = true,
        env = "ANKIKA_IMAGES",
        value_name = "MODE",
        default_value = "auto"
    )]
    pub images: ImageMode,

    /// Log more detail: -v for debug, -vv for trace. `RUST_LOG` takes precedence
    #[arg(short, long, global = true, action = ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
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
}

#[derive(Debug, Args)]
pub struct LoginArgs {
    /// AnkiWeb email; asked for when left out
    #[arg(long, value_name = "EMAIL")]
    pub email: Option<String>,
}
