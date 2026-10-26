use std::io::{self, Write};

use ankika_core::login::{self, LoginError};
use ankika_core::paths::Paths;
use ankika_core::session::{SessionError, SessionStore};

use crate::cli::LoginArgs;

#[derive(Debug, thiserror::Error)]
pub enum LoginCommandError {
    #[error("Failed to read the email")]
    ReadEmail(#[source] io::Error),
    #[error("No email given")]
    MissingEmail,
    #[error("Failed to read the password")]
    ReadPassword(#[source] io::Error),
    #[error(transparent)]
    Login(#[from] LoginError),
    #[error(transparent)]
    Save(#[from] SessionError),
}

pub async fn run(
    paths: &Paths,
    args: LoginArgs,
    sync_server: Option<String>,
) -> Result<(), LoginCommandError> {
    let email = match args.email {
        Some(email) => email,
        None => prompt_email().map_err(LoginCommandError::ReadEmail)?,
    };

    if email.is_empty() {
        return Err(LoginCommandError::MissingEmail);
    }

    let password =
        rpassword::prompt_password("Password: ").map_err(LoginCommandError::ReadPassword)?;
    let session = login::log_in(email, password, sync_server).await?;

    SessionStore::new(paths).save(&session)?;
    println!("Logged in as {}", session.account.username);

    Ok(())
}

fn prompt_email() -> io::Result<String> {
    print!("Email: ");
    io::stdout().flush()?;

    let mut email = String::new();
    io::stdin().read_line(&mut email)?;

    Ok(email.trim().to_owned())
}
