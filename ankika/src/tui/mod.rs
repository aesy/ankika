mod action;
mod hit_map;
mod keymap;
mod palette;
mod state;
mod terminal;
mod theme;
mod view;

use std::io;

use crossterm::event::EventStream;
use futures_util::StreamExt;
use ratatui::DefaultTerminal;

use crate::i18n::I18n;
use crate::tui::hit_map::HitMap;
use crate::tui::state::State;
use crate::tui::theme::Theme;

#[derive(Debug, thiserror::Error)]
pub enum TuiError {
    #[error("Failed to set up the terminal")]
    Init(#[source] io::Error),
    #[error("Failed to restore the terminal")]
    Restore(#[source] io::Error),
    #[error("Failed to draw to the terminal")]
    Draw(#[source] io::Error),
    #[error("Failed to read terminal input")]
    ReadEvent(#[source] io::Error),
}

pub async fn run(i18n: &I18n) -> Result<(), TuiError> {
    let mut terminal = terminal::init().map_err(TuiError::Init)?;
    let result = event_loop(&mut terminal, i18n).await;
    let restored = terminal::restore().map_err(TuiError::Restore);

    result?;
    restored
}

async fn event_loop(terminal: &mut DefaultTerminal, i18n: &I18n) -> Result<(), TuiError> {
    let theme = Theme::detect();
    let mut state = State::new();
    let mut events = EventStream::new();

    while !state.should_quit() {
        let mut hits = HitMap::default();

        terminal
            .draw(|frame| hits = view::render(frame, &state, i18n, theme))
            .map_err(TuiError::Draw)?;
        state.set_hits(hits);

        let Some(event) = events.next().await else {
            break;
        };

        state.handle_event(&event.map_err(TuiError::ReadEvent)?, i18n);
    }

    Ok(())
}
