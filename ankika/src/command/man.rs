use std::io;

use clap::CommandFactory;

use crate::cli::Cli;
use crate::command::output;

/// Prints the man page in roff, for `man -l -` or installing as `ankika.1`.
pub fn run() -> io::Result<()> {
    output::allow_closed_pipe(clap_mangen::Man::new(Cli::command()).render(&mut io::stdout()))
}
