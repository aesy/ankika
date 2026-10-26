mod app;
mod cli;
mod logging;

use std::error::Error;
use std::process::ExitCode;

use clap::Parser;

use crate::cli::Cli;

fn main() -> ExitCode {
    let cli = Cli::parse();

    match app::run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            report(&error);
            ExitCode::FAILURE
        },
    }
}

fn report(error: &dyn Error) {
    eprintln!("Error: {error}");

    let mut previous = error.to_string();
    let mut source = error.source();

    while let Some(cause) = source {
        let message = cause.to_string();

        // Some wrappers (like tracing's `FromEnvError`) display their source's message too
        if message != previous {
            eprintln!("  Caused by: {message}");
        }

        previous = message;
        source = cause.source();
    }
}
