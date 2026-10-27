use std::io;

/// A closed pipe, like `ankika man | head`, means the reader has seen enough.
pub fn allow_closed_pipe(result: io::Result<()>) -> io::Result<()> {
    match result {
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        other => other,
    }
}
