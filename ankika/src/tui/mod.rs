mod action;
mod error_text;
mod hit_map;
mod keymap;
mod login_form;
mod palette;
mod state;
mod tasks;
mod terminal;
mod theme;
mod view;

use std::io;

use ankika_core::paths::Paths;
use ankika_core::session::SessionStore;
use crossterm::event::EventStream;
use futures_util::StreamExt;
use ratatui::DefaultTerminal;
use tokio::sync::mpsc::{self, UnboundedReceiver};

use crate::i18n::I18n;
use crate::tui::hit_map::HitMap;
use crate::tui::state::State;
use crate::tui::tasks::{TaskResult, Tasks};
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

pub async fn run(i18n: &I18n, paths: &Paths, sync_server: Option<String>) -> Result<(), TuiError> {
    let sessions = SessionStore::new(paths);
    let (sender, receiver) = mpsc::unbounded_channel();
    let (session, notice) = match sessions.load() {
        Ok(session) => (session, None),
        Err(error) => {
            tracing::warn!(?error, "Failed to load the stored login");
            (None, Some(i18n.text("error-load-login")))
        },
    };
    let mut state = State::new(session, Tasks::new(sender, sessions, sync_server));

    if let Some(notice) = notice {
        state.set_notice(notice);
    }

    let mut terminal = terminal::init().map_err(TuiError::Init)?;
    let result = event_loop(&mut terminal, &mut state, receiver, i18n).await;
    let restored = terminal::restore().map_err(TuiError::Restore);

    result?;
    restored
}

async fn event_loop(
    terminal: &mut DefaultTerminal,
    state: &mut State,
    mut results: UnboundedReceiver<TaskResult>,
    i18n: &I18n,
) -> Result<(), TuiError> {
    let theme = Theme::detect();
    let mut events = EventStream::new();

    while !state.should_quit() {
        let mut hits = HitMap::default();

        terminal
            .draw(|frame| hits = view::render(frame, state, i18n, theme))
            .map_err(TuiError::Draw)?;
        state.set_hits(hits);

        tokio::select! {
            event = events.next() => {
                let Some(event) = event else {
                    break;
                };

                state.handle_event(&event.map_err(TuiError::ReadEvent)?, i18n);
            },
            Some(result) = results.recv() => state.handle_task_result(result, i18n),
        }
    }

    Ok(())
}
