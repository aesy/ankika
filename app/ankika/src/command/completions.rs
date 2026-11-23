use std::io;

use clap::CommandFactory;
use clap_complete::{Generator, Shell};

use crate::cli::Cli;
use crate::command::output;

/// Prints the completion script for the shell.
pub fn run(shell: Shell) -> io::Result<()> {
    let mut command = Cli::command();
    let name = command.get_name().to_owned();

    // What `clap_complete::generate` does, which panics on write errors instead
    command.set_bin_name(name);
    command.build();

    output::allow_closed_pipe(shell.try_generate(&command, &mut io::stdout()))
}
