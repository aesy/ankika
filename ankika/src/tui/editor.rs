use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use anki::notes::NoteId;
use ratatui::DefaultTerminal;
use tokio::task;

use crate::tui::terminal;

#[derive(Debug, thiserror::Error)]
pub enum EditorError {
    #[error("Failed to write the note to {}", path.display())]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("Failed to start the editor {editor}")]
    Launch {
        editor: String,
        #[source]
        source: io::Error,
    },
    #[error("The editor {editor} exited with {status}")]
    Failed { editor: String, status: ExitStatus },
    #[error("Failed to read the edited note from {}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("Failed to hand the terminal to the editor")]
    Terminal(#[source] io::Error),
}

/// Lets the user edit `text` in their editor, with the terminal handed over meanwhile.
/// The file lives only while the editor runs.
pub fn edit(
    terminal: &mut DefaultTerminal,
    note: NoteId,
    text: &str,
) -> Result<String, EditorError> {
    let path = env::temp_dir().join(format!("ankika-note-{note}-{}.txt", std::process::id()));

    write_private(&path, text).map_err(|source| EditorError::Write {
        path: path.clone(),
        source,
    })?;

    let result = run_editor(terminal, &path).and_then(|()| {
        fs::read_to_string(&path).map_err(|source| EditorError::Read {
            path: path.clone(),
            source,
        })
    });

    if let Err(error) = fs::remove_file(&path) {
        tracing::warn!(?path, ?error, "Failed to remove the edited note's file");
    }

    result
}

fn run_editor(terminal: &mut DefaultTerminal, path: &Path) -> Result<(), EditorError> {
    let editor = editor();
    let mut words = editor.split_whitespace();
    // Never empty, see `editor`
    let program = words.next().unwrap_or_default();
    let mut command = Command::new(program);
    command.args(words).arg(path);

    terminal::suspend().map_err(EditorError::Terminal)?;
    // The editor owns the terminal until it exits
    let status = task::block_in_place(|| command.status());
    let resumed = terminal::resume(terminal).map_err(EditorError::Terminal);
    let status = status.map_err(|source| EditorError::Launch {
        editor: editor.clone(),
        source,
    })?;

    resumed?;

    if status.success() {
        Ok(())
    } else {
        Err(EditorError::Failed { editor, status })
    }
}

/// `$VISUAL`, then `$EDITOR`, like most terminal programs.
fn editor() -> String {
    ["VISUAL", "EDITOR"]
        .into_iter()
        .filter_map(|name| env::var(name).ok())
        .find(|value| !value.trim().is_empty())
        .unwrap_or_else(|| {
            if cfg!(windows) {
                "notepad".to_owned()
            } else {
                "vi".to_owned()
            }
        })
}

/// The note may be private, so only the user can read the file.
fn write_private(path: &Path, text: &str) -> io::Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;

        options.mode(0o600);
    }

    options.open(path)?.write_all(text.as_bytes())
}
