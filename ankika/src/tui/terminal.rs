use std::io;
use std::panic;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use ratatui::DefaultTerminal;

pub fn init() -> io::Result<DefaultTerminal> {
    let terminal = ratatui::try_init()?;
    set_panic_hook();
    execute!(io::stdout(), EnableMouseCapture)?;

    Ok(terminal)
}

pub fn restore() -> io::Result<()> {
    execute!(io::stdout(), DisableMouseCapture)?;
    ratatui::try_restore()
}

/// Runs before ratatui's own hook, which restores everything except mouse capture.
fn set_panic_hook() {
    let hook = panic::take_hook();

    panic::set_hook(Box::new(move |info| {
        // Already panicking; nothing useful to do if this fails
        let _ = execute!(io::stdout(), DisableMouseCapture);
        hook(info);
    }));
}
