use std::io;
use std::panic;
use std::sync::atomic::{AtomicBool, Ordering};

use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
    PushKeyboardEnhancementFlags,
};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{execute, terminal::supports_keyboard_enhancement};
use ratatui::DefaultTerminal;

/// Set when the terminal reports modified keys like `Ctrl+1`, which otherwise arrive as
/// plain digits.
static KEYBOARD_ENHANCED: AtomicBool = AtomicBool::new(false);

pub fn init() -> io::Result<DefaultTerminal> {
    let terminal = ratatui::try_init()?;
    set_panic_hook();
    execute!(io::stdout(), EnableMouseCapture)?;
    enhance_keyboard()?;

    Ok(terminal)
}

pub fn restore() -> io::Result<()> {
    reset_keyboard()?;
    execute!(io::stdout(), DisableMouseCapture)?;
    ratatui::try_restore()
}

/// Hands the terminal to another program, like an editor.
pub fn suspend() -> io::Result<()> {
    reset_keyboard()?;
    execute!(io::stdout(), DisableMouseCapture, LeaveAlternateScreen)?;
    terminal::disable_raw_mode()
}

/// Takes the terminal back after `suspend`, redrawing everything.
pub fn resume(terminal: &mut DefaultTerminal) -> io::Result<()> {
    terminal::enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    enhance_keyboard()?;
    terminal.clear()
}

fn enhance_keyboard() -> io::Result<()> {
    // Asks the terminal, so it must run before events are read
    if supports_keyboard_enhancement()? {
        execute!(
            io::stdout(),
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
        )?;
        KEYBOARD_ENHANCED.store(true, Ordering::Relaxed);
    }

    Ok(())
}

fn reset_keyboard() -> io::Result<()> {
    if KEYBOARD_ENHANCED.swap(false, Ordering::Relaxed) {
        execute!(io::stdout(), PopKeyboardEnhancementFlags)?;
    }

    Ok(())
}

/// Runs before ratatui's own hook, which restores everything except mouse capture and
/// the keyboard mode.
fn set_panic_hook() {
    let hook = panic::take_hook();

    panic::set_hook(Box::new(move |info| {
        // Already panicking; nothing useful to do if this fails
        let _ = reset_keyboard();
        let _ = execute!(io::stdout(), DisableMouseCapture);
        hook(info);
    }));
}
