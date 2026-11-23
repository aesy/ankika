use ankika_core::paths::Paths;
use ankika_core::session::{SessionError, SessionStore};

pub fn run(paths: &Paths) -> Result<(), SessionError> {
    SessionStore::new(paths).clear()?;
    println!("Logged out");

    Ok(())
}
