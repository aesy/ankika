mod action;
mod audio;
mod card_text;
mod choices;
mod deck_list;
mod error_text;
mod full_sync_prompt;
mod hit_map;
mod images;
mod keymap;
mod login_form;
mod palette;
mod quit_prompt;
mod review;
mod state;
mod tasks;
mod terminal;
mod theme;
mod view;

use std::io;

use anki::collection::Collection;
use ankika_core::collection::CollectionError;
use ankika_core::paths::Paths;
use ankika_core::session::SessionStore;
use crossterm::event::EventStream;
use futures_util::StreamExt;
use ratatui::DefaultTerminal;
use tokio::sync::mpsc::{self, UnboundedReceiver};

use crate::cli::ImageMode;
use crate::i18n::I18n;
use crate::tui::audio::Audio;
use crate::tui::hit_map::HitMap;
use crate::tui::images::Images;
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
    #[error(transparent)]
    LostCollection(CollectionError),
}

pub async fn run(
    i18n: &I18n,
    paths: &Paths,
    collection: Collection,
    sync_server: Option<String>,
    image_mode: ImageMode,
) -> Result<(), TuiError> {
    let sessions = SessionStore::new(paths);
    let (sender, receiver) = mpsc::unbounded_channel();
    let (session, notice) = match sessions.load() {
        Ok(session) => (session, None),
        Err(error) => {
            tracing::warn!(?error, "Failed to load the stored login");
            (None, Some(i18n.text("error-load-login")))
        },
    };
    let audio = Audio::start(paths.media_dir(), sender.clone());
    let tasks = Tasks::new(sender, sessions, sync_server);
    let mut state = State::new(session, collection, tasks, audio);

    if let Some(notice) = notice {
        state.set_notice(notice);
    }

    state.reload_decks(i18n);
    state.start_sync();

    let mut terminal = terminal::init().map_err(TuiError::Init)?;
    let images = Images::detect(image_mode, paths.media_dir());
    let result = event_loop(&mut terminal, &mut state, &images, receiver, i18n).await;
    let restored = terminal::restore().map_err(TuiError::Restore);

    close(&mut state);

    let lost_collection = state.take_lost_collection();

    state.into_tasks().shutdown().await;

    if let Some(error) = lost_collection {
        return Err(TuiError::LostCollection(error));
    }

    result?;
    restored
}

/// A sync that was skipped on quit still holds the collection; it closes when the sync stops.
fn close(state: &mut State) {
    let Some(collection) = state.take_collection() else {
        return;
    };

    if let Err(error) = collection.close(None) {
        tracing::warn!(?error, "Failed to close the collection");
    }
}

async fn event_loop(
    terminal: &mut DefaultTerminal,
    state: &mut State,
    images: &Images,
    mut results: UnboundedReceiver<TaskResult>,
    i18n: &I18n,
) -> Result<(), TuiError> {
    let theme = Theme::detect();
    let mut events = EventStream::new();

    while !state.should_quit() {
        let mut hits = HitMap::default();

        terminal
            .draw(|frame| hits = view::render(frame, state, images, i18n, theme))
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
