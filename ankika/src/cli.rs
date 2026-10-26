use std::path::PathBuf;

use clap::{ArgAction, Parser};

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    /// Directory holding the collection and its media
    #[arg(long, global = true, env = "ANKIKA_DATA_DIR", value_name = "DIR")]
    pub data_dir: Option<PathBuf>,

    /// Directory holding settings and the AnkiWeb login
    #[arg(long, global = true, env = "ANKIKA_CONFIG_DIR", value_name = "DIR")]
    pub config_dir: Option<PathBuf>,

    /// File to write logs to
    #[arg(long, global = true, env = "ANKIKA_LOG_FILE", value_name = "FILE")]
    pub log_file: Option<PathBuf>,

    /// Log more detail: -v for debug, -vv for trace. `RUST_LOG` takes precedence
    #[arg(short, long, global = true, action = ArgAction::Count)]
    pub verbose: u8,
}
