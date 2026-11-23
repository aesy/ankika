mod action;
mod add_form;
mod audio;
mod browser;
mod card_text;
mod choices;
mod custom_study_form;
mod deck_list;
mod deck_options_form;
mod editor;
mod error_text;
mod form;
mod full_sync_prompt;
mod fuzzy_list;
mod hit_map;
mod images;
mod keymap;
mod login_form;
mod overview;
mod package_forms;
mod palette;
mod quit_prompt;
mod replace_prompt;
mod report;
mod review;
mod settings_form;
mod state;
mod stats_screen;
mod tag_input;
mod tasks;
mod terminal;
mod theme;
mod unbury_prompt;
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
use crate::config::Config;
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

/// How the TUI ended.
pub enum Exit {
    Quit,
    /// Open this profile next.
    SwitchProfile(String),
}

/// Command line flags and environment variables, which win over the config file.
pub struct Overrides {
    pub sync_server: Option<String>,
    pub images: Option<ImageMode>,
}

pub async fn run(
    i18n: &I18n,
    paths: &Paths,
    collection: Collection,
    config: Config,
    overrides: Overrides,
) -> Result<Exit, TuiError> {
    let sync_server = overrides.sync_server.or_else(|| config.sync_server.clone());
    let image_mode = overrides.images.unwrap_or(config.images);
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
    let sync_on_start = config.sync_on_start;
    let mut state = State::new(session, collection, tasks, audio, config, paths.clone());

    if let Some(notice) = notice {
        state.set_notice(notice);
    }

    state.reload_decks(i18n);

    if sync_on_start {
        state.start_sync();
    }

    let mut terminal = terminal::init().map_err(TuiError::Init)?;
    let images = Images::detect(image_mode, paths.media_dir());
    let result = event_loop(&mut terminal, &mut state, &images, receiver, i18n).await;
    // The next profile sets the terminal up again
    terminal::discard_wakeup();
    let restored = terminal::restore().map_err(TuiError::Restore);

    close(&mut state);

    let lost_collection = state.take_lost_collection();
    let exit = state
        .take_switch_profile()
        .map_or(Exit::Quit, Exit::SwitchProfile);

    state.into_tasks().shutdown().await;

    if let Some(error) = lost_collection {
        return Err(TuiError::LostCollection(error));
    }

    result?;
    restored.map(|()| exit)
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
    let mut events = EventStream::new();

    while !state.should_quit() {
        let mut hits = HitMap::default();
        // Read each frame, so saving the settings applies it right away
        let theme = Theme::new(state.config().theme);

        state.load_browser_data();
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

        // Edits start from an input event, so the event reader has just delivered it and
        // stays idle until polled again: it can't take the editor's keystrokes. Dropping
        // it instead would leave a wake-up behind that breaks the terminal queries after.
        if let Some(request) = state.take_edit() {
            let edited = editor::edit(terminal, request.note.note_id(), &request.note.text());
            state.finish_edit(request, edited, i18n);
        }
    }

    Ok(())
}
