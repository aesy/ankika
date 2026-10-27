use std::io::{self, BufRead, Write};

use ankika_core::paths::Paths;
use ankika_core::profiles::{self, ProfileError};

use crate::cli::ProfileCommand;

#[derive(Debug, thiserror::Error)]
pub enum ProfileCommandError {
    #[error(transparent)]
    Profile(#[from] ProfileError),
    #[error("Failed to read the answer")]
    ReadAnswer(#[source] io::Error),
}

pub fn run(paths: &Paths, command: ProfileCommand) -> Result<(), ProfileCommandError> {
    match command {
        ProfileCommand::List => {
            let last = profiles::last_used(paths);

            for name in profiles::list(paths)? {
                let marker = if last.as_deref() == Some(name.as_str()) {
                    "*"
                } else {
                    " "
                };
                println!("{marker} {name}");
            }
        },
        ProfileCommand::Add { name } => {
            profiles::add(paths, &name)?;
            println!("Added profile {name}");
        },
        ProfileCommand::Rename { old, new } => {
            profiles::rename(paths, &old, &new)?;
            println!("Renamed profile {old} to {new}");
        },
        ProfileCommand::Remove { name, yes } => {
            let confirmed = yes
                || confirm(&format!(
                    "Remove profile {name} with its collection, media and login? [y/N] "
                ))
                .map_err(ProfileCommandError::ReadAnswer)?;

            if confirmed {
                profiles::remove(paths, &name)?;
                println!("Removed profile {name}");
            } else {
                println!("Kept profile {name}");
            }
        },
    }

    Ok(())
}

fn confirm(question: &str) -> io::Result<bool> {
    print!("{question}");
    io::stdout().flush()?;

    let mut answer = String::new();
    io::stdin().lock().read_line(&mut answer)?;

    Ok(matches!(answer.trim(), "y" | "Y" | "yes"))
}
