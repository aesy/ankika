use std::path::PathBuf;

use anki::collection::Collection;
use anki::decks::DeckId;
use ankika_core::adding;
use ankika_core::card_actions::{self, CardAction, Flag, Outcome};
use ankika_core::collection::CollectionError;
use ankika_core::deck_options::{self, DeckOptions};
use ankika_core::decks;
use ankika_core::note_edit::NoteText;
use ankika_core::overview::{self, OverviewError, Unbury};
use ankika_core::review::{self, Rating, ReviewError, Side, Sound, SoundRef, StudyCard};
use ankika_core::session::Session;
use ankika_core::stats;
use ankika_core::sync::{
    FullSyncDirection, FullSyncOutput, FullSyncRequired, NormalSyncOutput, SyncError,
};
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use futures_util::future::AbortHandle;
use ratatui::layout::Position;

use crate::config::{self, Config};
use crate::i18n::I18n;
use crate::tui::action::Action;
use crate::tui::add_form::{self, AddForm};
use crate::tui::audio::{Audio, AudioEvent, Track};
use crate::tui::browser::{Browser, Focus};
use crate::tui::choices::Choices;
use crate::tui::custom_study_form::{self, CustomStudyForm};
use crate::tui::deck_list::DeckList;
use crate::tui::deck_options_form;
use crate::tui::editor::EditorError;
use crate::tui::error_text;
use crate::tui::form::{self, Form};
use crate::tui::full_sync_prompt::{Choice, FullSyncPrompt};
use crate::tui::fuzzy_list::FuzzyList;
use crate::tui::hit_map::{Hit, HitMap};
use crate::tui::keymap::{self, Context};
use crate::tui::login_form::{Field, LoginForm};
use crate::tui::overview::DeckScreen;
use crate::tui::palette::{self, Palette};
use crate::tui::quit_prompt::{self, QuitChoice};
use crate::tui::review::{Review, Stage};
use crate::tui::settings_form;
use crate::tui::stats_screen::StatsScreen;
use crate::tui::tag_input::TagInput;
use crate::tui::tasks::{TaskResult, Tasks};
use crate::tui::unbury_prompt::{self, UnburyChoice};

pub enum Overlay {
    None,
    Help,
    Palette(Palette),
    Login(LoginForm),
    FullSync(FullSyncPrompt),
    Quit(Choices<QuitChoice>),
    Unbury(Choices<UnburyChoice>),
    CustomStudy(CustomStudyForm),
    Add(AddForm),
    ChangeDeck(FuzzyList<DeckId>),
    /// Column indexes of the browser's sortable columns.
    SortBy(FuzzyList<usize>),
    Tags {
        remove: bool,
        input: TagInput,
    },
    Settings(Form),
    /// Boxed, since the preset makes it much larger than the other overlays.
    DeckOptions(Box<DeckOptionsDialog>),
}

pub struct DeckOptionsDialog {
    form: Form,
    deck: DeckId,
    options: DeckOptions,
}

impl DeckOptionsDialog {
    pub const fn form(&self) -> &Form {
        &self.form
    }
}

/// A note for the editor, and what to do with it afterwards.
mod browsing;

pub struct EditRequest {
    pub note: NoteText,
    purpose: Purpose,
}

enum Purpose {
    Edit,
    Add { deck: DeckId, tags: Vec<String> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncStatus {
    Idle,
    Syncing,
    Synced,
    Failed,
}

pub struct State {
    session: Option<Session>,
    /// `None` while a sync task has it.
    collection: Option<Collection>,
    tasks: Tasks,
    audio: Audio,
    /// As in the file; command line overrides aren't saved into it.
    config: Config,
    config_path: PathBuf,
    /// The sound the audio thread reports as playing.
    playing: Option<SoundRef>,
    decks: DeckList,
    /// Shown over everything else; closing it returns to what was there.
    browser: Option<Browser>,
    /// Shown over the deck list or overview.
    stats_screen: Option<StatsScreen>,
    /// The deck overview; studying happens on top of it.
    deck_screen: Option<DeckScreen>,
    review: Option<Review>,
    /// A note waiting for the event loop to open it in the editor.
    edit: Option<EditRequest>,
    /// Collapsed states changed while a sync had the collection.
    pending_collapse: Vec<(DeckId, bool)>,
    sync: SyncStatus,
    /// Set while media syncs in the background.
    media_sync: Option<AbortHandle>,
    overlay: Overlay,
    notice: Option<String>,
    mouse: Option<Position>,
    hits: HitMap,
    /// Quit was requested and waits for the running syncs.
    quitting: bool,
    quit: bool,
    lost_collection: Option<CollectionError>,
}

impl State {
    pub fn new(
        session: Option<Session>,
        collection: Collection,
        tasks: Tasks,
        audio: Audio,
        config: Config,
        config_path: PathBuf,
    ) -> Self {
        Self {
            session,
            collection: Some(collection),
            tasks,
            audio,
            config,
            config_path,
            playing: None,
            decks: DeckList::default(),
            browser: None,
            stats_screen: None,
            deck_screen: None,
            review: None,
            edit: None,
            pending_collapse: Vec::new(),
            sync: SyncStatus::Idle,
            media_sync: None,
            overlay: Overlay::None,
            notice: None,
            mouse: None,
            hits: HitMap::default(),
            quitting: false,
            quit: false,
            lost_collection: None,
        }
    }

    pub const fn should_quit(&self) -> bool {
        self.quit
    }

    pub const fn session(&self) -> Option<&Session> {
        self.session.as_ref()
    }

    pub const fn sync_status(&self) -> SyncStatus {
        self.sync
    }

    pub const fn decks(&self) -> &DeckList {
        &self.decks
    }

    pub const fn config(&self) -> &Config {
        &self.config
    }

    pub const fn playing(&self) -> Option<SoundRef> {
        self.playing
    }

    pub const fn browser(&self) -> Option<&Browser> {
        self.browser.as_ref()
    }

    pub const fn stats_screen(&self) -> Option<&StatsScreen> {
        self.stats_screen.as_ref()
    }

    pub const fn deck_screen(&self) -> Option<&DeckScreen> {
        self.deck_screen.as_ref()
    }

    pub const fn review(&self) -> Option<&Review> {
        self.review.as_ref()
    }

    /// The deck list shows the last loaded counts while a sync has the collection.
    pub const fn decks_stale(&self) -> bool {
        self.collection.is_none()
    }

    pub const fn media_syncing(&self) -> bool {
        self.media_sync.is_some()
    }

    pub const fn overlay(&self) -> &Overlay {
        &self.overlay
    }

    /// A message to show until the next input.
    pub fn notice(&self) -> Option<&str> {
        self.notice.as_deref()
    }

    pub const fn mouse(&self) -> Option<Position> {
        self.mouse
    }

    pub fn context(&self) -> Context {
        match self.overlay {
            Overlay::None if self.browser.is_some() => {
                match self.browser.as_ref().map(Browser::focus) {
                    Some(Focus::Search) => Context::BrowserSearch,
                    _ => Context::Browser,
                }
            },
            Overlay::None if self.stats_screen.is_some() => Context::Stats,
            Overlay::None => match &self.review {
                None => match &self.deck_screen {
                    Some(screen) if screen.is_filtered() => Context::FilteredOverview,
                    Some(_) => Context::Overview,
                    None => Context::Main,
                },
                Some(review) if review.is_typing() => Context::ReviewTyping,
                Some(review) => match review.stage() {
                    Stage::Question { .. } => Context::ReviewQuestion,
                    Stage::Answer { .. } => Context::ReviewAnswer,
                    Stage::Waiting | Stage::Finished { .. } => Context::ReviewIdle,
                },
            },
            Overlay::Help => Context::Help,
            Overlay::Palette(_) | Overlay::ChangeDeck(_) | Overlay::SortBy(_) => Context::Palette,
            Overlay::Login(_) => Context::Login,
            Overlay::FullSync(_) => Context::FullSync,
            Overlay::Quit(_) => Context::Quit,
            Overlay::Unbury(_) => Context::Unbury,
            Overlay::CustomStudy(_) => Context::CustomStudy,
            Overlay::Add(_) | Overlay::Tags { .. } => Context::Add,
            Overlay::Settings(_) | Overlay::DeckOptions(_) => Context::Form,
        }
    }

    pub fn set_hits(&mut self, hits: HitMap) {
        self.hits = hits;
    }

    pub fn set_notice(&mut self, notice: String) {
        self.notice = Some(notice);
    }

    /// The collection for closing on exit, unless a sync task still has it.
    pub const fn take_collection(&mut self) -> Option<Collection> {
        self.collection.take()
    }

    pub fn into_tasks(self) -> Tasks {
        self.tasks
    }

    /// Set when a full sync could not reopen the collection; the app cannot go on.
    pub const fn take_lost_collection(&mut self) -> Option<CollectionError> {
        self.lost_collection.take()
    }

    pub fn reload_decks(&mut self, i18n: &I18n) {
        let Some(collection) = &mut self.collection else {
            return;
        };

        for (id, collapsed) in self.pending_collapse.drain(..) {
            if let Err(error) = decks::set_collapsed(collection, id, collapsed) {
                tracing::warn!(?error, "Failed to store a collapsed deck");
                self.notice = Some(error_text::decks(&error, i18n));
            }
        }

        match decks::deck_tree(collection) {
            Ok(tree) => self.decks.set_decks(tree),
            Err(error) => {
                tracing::warn!(?error, "Failed to load the deck list");
                self.notice = Some(error_text::decks(&error, i18n));
            },
        }
    }

    fn store_collapse(&mut self, change: Option<(DeckId, bool)>, i18n: &I18n) {
        let Some((id, collapsed)) = change else {
            return;
        };
        let Some(collection) = &mut self.collection else {
            self.pending_collapse.push((id, collapsed));
            return;
        };

        if let Err(error) = decks::set_collapsed(collection, id, collapsed) {
            tracing::warn!(?error, "Failed to store a collapsed deck");
            self.notice = Some(error_text::decks(&error, i18n));
        }
    }

    pub fn start_sync(&mut self) {
        let Some((session, collection)) = self.take_for_sync() else {
            return;
        };

        self.sync = SyncStatus::Syncing;
        self.tasks.sync(collection, session);
    }

    /// Hands the collection over only when there is a login to sync with.
    fn take_for_sync(&mut self) -> Option<(Session, Collection)> {
        let session = self.session.clone()?;
        let collection = self.collection.take()?;

        Some((session, collection))
    }

    pub fn handle_event(&mut self, event: &Event, i18n: &I18n) {
        match event {
            // Windows also reports releases
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                self.notice = None;
                self.handle_key(key, i18n);
            },
            Event::Mouse(mouse) => self.handle_mouse(*mouse, i18n),
            _ => {},
        }
    }

    pub fn handle_task_result(&mut self, result: TaskResult, i18n: &I18n) {
        match result {
            TaskResult::LoggedIn(Ok(session)) => {
                tracing::info!(username = session.account.username, "Logged in");
                self.session = Some(session);

                if matches!(self.overlay, Overlay::Login(_)) {
                    self.overlay = Overlay::None;
                }

                self.start_sync();
            },
            TaskResult::LoggedIn(Err(error)) => {
                tracing::warn!(?error, "Login failed");
                let text = error_text::log_in(&error, i18n);

                match &mut self.overlay {
                    Overlay::Login(form) => form.fail(text),
                    _ => self.notice = Some(text),
                }
            },
            TaskResult::LoggedOut(Ok(())) => {
                tracing::info!("Logged out");
                self.session = None;
                self.sync = SyncStatus::Idle;
                self.abort_media_sync();
            },
            TaskResult::LoggedOut(Err(error)) => {
                tracing::warn!(?error, "Logout failed");
                self.notice = Some(i18n.text("error-log-out"));
            },
            TaskResult::Synced {
                collection,
                session,
                result,
            } => {
                self.collection = Some(collection);
                self.reload_decks(i18n);
                self.refresh_deck_screen(i18n);

                // Logging out during the sync wins over the returned session
                if self.session.is_some() {
                    self.session = Some(session);
                }

                match result {
                    Ok(output) => self.finish_normal_sync(output, i18n),
                    Err(error) => {
                        tracing::warn!(?error, "Sync failed");
                        self.sync = SyncStatus::Failed;
                        self.notice = Some(error_text::sync(&error, i18n));
                    },
                }

                self.quit_if_done();
            },
            TaskResult::FullSynced(output) => self.finish_full_sync(output, i18n),
            TaskResult::Audio(AudioEvent::Playing(sound)) => self.playing = sound,
            TaskResult::Audio(AudioEvent::CannotPlay { file }) => {
                self.notice = Some(i18n.text_with("audio-cannot-play", &[("file", &file)]));
            },
            TaskResult::Audio(AudioEvent::NoOutputDevice) => {
                self.notice = Some(i18n.text("audio-no-output"));
            },
            TaskResult::MediaSynced(result) => {
                self.media_sync = None;

                match result {
                    Ok(()) => tracing::info!("Media sync complete"),
                    Err(error) => self.fail_media_sync(&error, i18n),
                }

                self.quit_if_done();
            },
        }
    }

    fn finish_normal_sync(&mut self, output: NormalSyncOutput, i18n: &I18n) {
        if !output.server_message.is_empty() {
            self.notice = Some(output.server_message);
        }

        match output.full_sync_required {
            None => {
                tracing::info!("Sync complete");
                self.sync = SyncStatus::Synced;
                self.start_media_sync(i18n);
            },
            // Nothing to lose here, but never start a full sync on the way out
            Some(_) if self.quitting => self.sync = SyncStatus::Idle,
            Some(FullSyncRequired::DownloadOnly) => {
                self.start_full_sync(FullSyncDirection::Download);
            },
            Some(required) => {
                self.sync = SyncStatus::Idle;
                self.overlay = Overlay::FullSync(FullSyncPrompt::new(required));
            },
        }
    }

    fn finish_full_sync(&mut self, output: FullSyncOutput, i18n: &I18n) {
        match output.collection {
            Ok(collection) => {
                self.collection = Some(collection);
                self.reload_decks(i18n);
                self.refresh_deck_screen(i18n);
            },
            Err(error) => {
                tracing::error!(?error, "Failed to reopen the collection after a full sync");
                self.lost_collection = Some(error);
                self.quit_now();
                return;
            },
        }

        match output.result {
            Ok(()) => {
                tracing::info!("Full sync complete");
                self.sync = SyncStatus::Synced;
                self.start_media_sync(i18n);
            },
            Err(error) => {
                tracing::warn!(?error, "Full sync failed");
                self.sync = SyncStatus::Failed;
                self.notice = Some(error_text::sync(&error, i18n));
            },
        }

        self.quit_if_done();
    }

    fn start_media_sync(&mut self, i18n: &I18n) {
        let (Some(session), Some(collection)) = (&self.session, &self.collection) else {
            return;
        };

        if self.media_sync.is_some() {
            return;
        }

        match self.tasks.media_sync(collection, session) {
            Ok(abort) => self.media_sync = Some(abort),
            Err(error) => self.fail_media_sync(&error, i18n),
        }
    }

    fn fail_media_sync(&mut self, error: &SyncError, i18n: &I18n) {
        tracing::warn!(?error, "Media sync failed");
        self.sync = SyncStatus::Failed;
        self.notice = Some(format!(
            "{} {}",
            i18n.anki().sync_media_failed(),
            error_text::sync(error, i18n)
        ));
    }

    fn abort_media_sync(&mut self) {
        if let Some(abort) = self.media_sync.take() {
            tracing::info!("Aborting media sync");
            abort.abort();
        }
    }

    fn start_full_sync(&mut self, direction: FullSyncDirection) {
        let Some((session, collection)) = self.take_for_sync() else {
            return;
        };

        tracing::info!(?direction, "Starting full sync");
        self.sync = SyncStatus::Syncing;
        self.tasks.full_sync(collection, session, direction);
    }

    fn handle_key(&mut self, key: &KeyEvent, i18n: &I18n) {
        if let Some(action) = keymap::action_for(self.context(), key) {
            self.perform(action, i18n);
            return;
        }

        let typed = match key.code {
            KeyCode::Char(character)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                Some(character)
            },
            _ => None,
        };

        match &mut self.overlay {
            Overlay::Palette(palette) => {
                if palette.handle_key(key.code, typed) {
                    self.confirm_list(i18n);
                }
            },
            Overlay::Login(_) => self.handle_login_key(key.code, typed),
            Overlay::FullSync(prompt) => {
                if let Some(choice) = navigate(prompt.choices_mut(), key.code) {
                    self.choose_full_sync(choice);
                }
            },
            Overlay::Quit(choices) => {
                if let Some(choice) = navigate(choices, key.code) {
                    self.choose_quit(choice, i18n);
                }
            },
            Overlay::Unbury(choices) => {
                if let Some(choice) = navigate(choices, key.code) {
                    self.choose_unbury(choice, i18n);
                }
            },
            Overlay::CustomStudy(form) => match key.code {
                KeyCode::Tab => form.focus_next(),
                KeyCode::BackTab => form.focus_previous(),
                KeyCode::Up | KeyCode::Left => form.step(false),
                KeyCode::Down | KeyCode::Right => form.step(true),
                KeyCode::Backspace => form.pop(),
                KeyCode::Enter if form.focus() == custom_study_form::Field::Cancel => {
                    self.overlay = Overlay::None;
                },
                KeyCode::Enter => self.submit_custom_study(i18n),
                _ => {
                    if let Some(character) = typed {
                        form.push(character);
                    }
                },
            },
            Overlay::Add(_) => self.handle_add_key(key.code, typed, i18n),
            Overlay::ChangeDeck(_) | Overlay::SortBy(_) | Overlay::Tags { .. } => {
                self.handle_browser_overlay_key(key.code, typed, i18n);
            },
            Overlay::Settings(_) | Overlay::DeckOptions(_) => {
                if let Some(form) = self.overlay_form() {
                    let outcome = form.handle_key(key.code, typed);
                    self.finish_form(outcome, i18n);
                }
            },
            Overlay::None if self.browser.is_some() => self.handle_browser_key(key.code, typed),
            Overlay::None if self.stats_screen.is_some() => {
                if let Some(screen) = &mut self.stats_screen {
                    screen.handle_key(key.code);
                }
            },
            Overlay::None if self.review.is_some() => self.handle_review_key(key.code, typed),
            Overlay::None if self.deck_screen.is_some() => {},
            Overlay::None => match key.code {
                KeyCode::Up => self.decks.select_previous(),
                KeyCode::Down => self.decks.select_next(),
                KeyCode::Home => self.decks.select_first(),
                KeyCode::End => self.decks.select_last(),
                KeyCode::Right => {
                    let change = self.decks.expand();
                    self.store_collapse(change, i18n);
                },
                KeyCode::Left => {
                    let change = self.decks.collapse();
                    self.store_collapse(change, i18n);
                },
                _ => {},
            },
            Overlay::Help => {},
        }
    }

    fn handle_mouse(&mut self, mouse: MouseEvent, i18n: &I18n) {
        let position = Position::new(mouse.column, mouse.row);
        let hit = self.hits.at(position);
        self.mouse = Some(position);

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.notice = None;
                self.click(hit, i18n);
            },
            MouseEventKind::Moved => {
                if let (Overlay::Palette(palette), Some(Hit::ListEntry(index))) =
                    (&mut self.overlay, hit)
                {
                    palette.select(index);
                }
            },
            MouseEventKind::ScrollUp => match &mut self.overlay {
                Overlay::Palette(palette) => palette.select_previous(),
                Overlay::ChangeDeck(list) => list.select_previous(),
                Overlay::SortBy(list) => list.select_previous(),
                Overlay::None if self.browser.is_some() => {
                    if let Some(browser) = &mut self.browser {
                        browser.move_cursor(-1);
                    }
                },
                Overlay::None if self.stats_screen.is_some() => {
                    if let Some(screen) = &mut self.stats_screen {
                        screen.scroll_by(-1);
                    }
                },
                Overlay::None => match &mut self.review {
                    Some(review) => review.scroll_up(),
                    None if self.deck_screen.is_some() => {},
                    None => self.decks.select_previous(),
                },
                _ => {},
            },
            MouseEventKind::ScrollDown => match &mut self.overlay {
                Overlay::Palette(palette) => palette.select_next(),
                Overlay::ChangeDeck(list) => list.select_next(),
                Overlay::SortBy(list) => list.select_next(),
                Overlay::None if self.browser.is_some() => {
                    if let Some(browser) = &mut self.browser {
                        browser.move_cursor(1);
                    }
                },
                Overlay::None if self.stats_screen.is_some() => {
                    if let Some(screen) = &mut self.stats_screen {
                        screen.scroll_by(1);
                    }
                },
                Overlay::None => match &mut self.review {
                    Some(review) => review.scroll_down(),
                    None if self.deck_screen.is_some() => {},
                    None => self.decks.select_next(),
                },
                _ => {},
            },
            _ => {},
        }
    }

    fn click(&mut self, hit: Option<Hit>, i18n: &I18n) {
        match hit {
            Some(Hit::Action(action)) => self.perform(action, i18n),
            Some(Hit::ListEntry(index)) => self.click_list_entry(index, i18n),
            Some(Hit::AddField(field)) => self.click_add_field(field, i18n),
            Some(Hit::BrowserRow(index)) => {
                if let Some(browser) = &mut self.browser {
                    browser.focus_table();
                    browser.click(index);
                }
            },
            Some(Hit::BrowserColumn(column)) => self.sort_by(column, i18n),
            Some(Hit::Form(focus)) => {
                if let Some(form) = self.overlay_form() {
                    let outcome = form.click(focus);
                    self.finish_form(outcome, i18n);
                }
            },
            Some(Hit::BrowserSearch) => {
                if let Some(browser) = &mut self.browser {
                    browser.focus_search();
                }
            },
            Some(Hit::TagCompletion(index)) => match &mut self.overlay {
                Overlay::Add(form) => {
                    form.tags_mut().select_completion(index);
                    form.tags_mut().complete();
                },
                Overlay::Tags { input, .. } => {
                    input.select_completion(index);
                    input.complete();
                },
                _ => {},
            },
            Some(Hit::LoginField(field)) => {
                if let Overlay::Login(form) = &mut self.overlay {
                    form.set_focus(field);
                }

                match field {
                    Field::Email | Field::Password => {},
                    Field::Submit => self.submit_login(),
                    Field::Cancel => self.overlay = Overlay::None,
                }
            },
            Some(Hit::FullSyncChoice(choice)) => self.choose_full_sync(choice),
            Some(Hit::QuitChoice(choice)) => self.choose_quit(choice, i18n),
            Some(Hit::Sound(sound)) => self.play(vec![sound]),
            Some(Hit::Deck(id)) => {
                self.decks.select(id);
                self.open_deck(i18n);
            },
            Some(Hit::UnburyChoice(choice)) => self.choose_unbury(choice, i18n),
            Some(Hit::CustomStudyChoice(choice)) => {
                if let Overlay::CustomStudy(form) = &mut self.overlay {
                    form.set_focus(custom_study_form::Field::Choice);
                    form.select(choice);
                }
            },
            Some(Hit::CustomStudyKind(kind)) => {
                if let Overlay::CustomStudy(form) = &mut self.overlay {
                    form.set_focus(custom_study_form::Field::Kind);
                    form.select_kind(kind);
                }
            },
            Some(Hit::CustomStudyField(field)) => {
                if let Overlay::CustomStudy(form) = &mut self.overlay {
                    form.set_focus(field);
                }

                match field {
                    custom_study_form::Field::Ok => self.submit_custom_study(i18n),
                    custom_study_form::Field::Cancel => self.overlay = Overlay::None,
                    _ => {},
                }
            },
            Some(Hit::DeckToggle(id)) => {
                let change = self.decks.toggle(id);
                self.store_collapse(change, i18n);
            },
            Some(Hit::Overlay) => {},
            None => self.overlay = Overlay::None,
        }
    }

    fn choose_full_sync(&mut self, choice: Choice) {
        self.overlay = Overlay::None;

        if let Choice::Sync(direction) = choice {
            self.start_full_sync(direction);
        }
    }

    fn handle_review_key(&mut self, code: KeyCode, typed: Option<char>) {
        let Some(review) = &mut self.review else {
            return;
        };

        match code {
            KeyCode::Up => review.scroll_up(),
            KeyCode::Down => review.scroll_down(),
            KeyCode::Backspace if review.is_typing() => review.pop(),
            _ => {
                if let Some(character) = typed
                    && review.is_typing()
                {
                    review.push(character);
                }
            },
        }
    }

    fn start_review(&mut self, i18n: &I18n) {
        let Some(screen) = &self.deck_screen else {
            return;
        };

        self.review = Some(Review::new(screen.deck(), screen.name().to_owned()));
        self.resume_review(i18n);
    }

    fn open_deck(&mut self, i18n: &I18n) {
        let Some(deck) = self.decks.selected().and_then(|id| self.decks.deck(id)) else {
            return;
        };

        self.show_deck(deck.id, deck.name.clone(), i18n);
    }

    fn show_deck(&mut self, deck: DeckId, name: String, i18n: &I18n) {
        self.deck_screen = Some(DeckScreen::Waiting { deck, name });
        self.refresh_deck_screen(i18n);
    }

    fn open_deck_options(&mut self, i18n: &I18n) {
        let (Some(screen), Some(collection)) = (&self.deck_screen, &mut self.collection) else {
            return;
        };

        if screen.is_filtered() {
            return;
        }

        let deck = screen.deck();

        match deck_options::load(collection, deck) {
            Ok(options) => {
                let form = deck_options_form::new(screen.name(), &options, i18n);

                self.overlay = Overlay::DeckOptions(Box::new(DeckOptionsDialog {
                    form,
                    deck,
                    options,
                }));
            },
            Err(error) => {
                tracing::warn!(?error, "Failed to load the deck options");
                self.notice = Some(error_text::deck_options(&error, i18n));
            },
        }
    }

    /// The settings or deck options form, when one is open.
    const fn overlay_form(&mut self) -> Option<&mut Form> {
        match &mut self.overlay {
            Overlay::Settings(form) => Some(form),
            Overlay::DeckOptions(dialog) => Some(&mut dialog.form),
            _ => None,
        }
    }

    /// Saves or closes the settings or deck options form after a key or click.
    fn finish_form(&mut self, outcome: form::Outcome, i18n: &I18n) {
        match outcome {
            form::Outcome::Nothing => {},
            form::Outcome::Cancel => self.overlay = Overlay::None,
            form::Outcome::Save => match std::mem::replace(&mut self.overlay, Overlay::None) {
                Overlay::Settings(form) => self.save_settings(form, i18n),
                Overlay::DeckOptions(dialog) => self.save_deck_options(*dialog, i18n),
                other => self.overlay = other,
            },
        }
    }

    fn save_settings(&mut self, mut form: Form, i18n: &I18n) {
        let config = settings_form::read(&form);

        match config::save(&self.config_path, &config) {
            Ok(()) => {
                self.config = config;
                self.notice = Some(i18n.text("settings-saved"));
            },
            Err(error) => {
                tracing::warn!(?error, "Failed to save the settings");
                form.set_error(i18n.text("error-settings-save"));
                self.overlay = Overlay::Settings(form);
            },
        }
    }

    fn save_deck_options(&mut self, mut dialog: DeckOptionsDialog, i18n: &I18n) {
        if let Err(field) = deck_options_form::apply(&dialog.form, &mut dialog.options) {
            dialog
                .form
                .set_error(i18n.text_with("deck-options-invalid", &[("field", &field)]));
            self.overlay = Overlay::DeckOptions(Box::new(dialog));
            return;
        }

        let DeckOptionsDialog { deck, options, .. } = dialog;

        let Some(collection) = &mut self.collection else {
            return;
        };

        match deck_options::save(collection, deck, options) {
            Ok(()) => {
                self.notice = Some(i18n.text("deck-options-saved"));
                self.reload_decks(i18n);
                self.refresh_deck_screen(i18n);
            },
            Err(error) => {
                tracing::warn!(?error, "Failed to save the deck options");
                self.notice = Some(error_text::deck_options(&error, i18n));
            },
        }
    }

    /// Loads the statistics for the screen's scope and period, unless a sync has the
    /// collection; they load once it is back.
    fn load_stats(&mut self, i18n: &I18n) {
        let (Some(screen), Some(collection)) = (&mut self.stats_screen, &mut self.collection)
        else {
            return;
        };

        match stats::load(collection, screen.scope(), screen.period()) {
            Ok(stats) => screen.set_stats(stats),
            Err(error) => {
                tracing::warn!(?error, "Failed to load the statistics");
                self.notice = Some(error.anki_error().message(i18n.anki()));
            },
        }
    }

    /// Reloads what's on screen once the collection is available: the review if one is
    /// running, otherwise the overview.
    fn refresh_deck_screen(&mut self, i18n: &I18n) {
        self.load_stats(i18n);

        if self.review.is_some() {
            self.resume_review(i18n);
            return;
        }

        let (Some(screen), Some(collection)) = (&self.deck_screen, &mut self.collection) else {
            return;
        };

        match overview::load(collection, screen.deck(), i18n.anki()) {
            Ok(overview) => self.deck_screen = Some(DeckScreen::Ready(overview)),
            Err(error) => {
                tracing::warn!(?error, "Failed to load the deck overview");
                self.notice = Some(error_text::overview(&error, i18n));
                self.deck_screen = None;
                self.reload_decks(i18n);
            },
        }
    }

    fn close_overview(&mut self, i18n: &I18n) {
        self.deck_screen = None;
        self.reload_decks(i18n);
    }

    /// Asks which kind first when there are both, like Anki.
    fn unbury(&mut self, i18n: &I18n) {
        let Some(overview) = self.deck_screen.as_ref().and_then(DeckScreen::overview) else {
            return;
        };

        match (overview.have_user_buried, overview.have_sched_buried) {
            (true, true) => self.overlay = Overlay::Unbury(unbury_prompt::new()),
            (false, false) => {},
            _ => self.choose_unbury(UnburyChoice::Unbury(Unbury::All), i18n),
        }
    }

    fn choose_unbury(&mut self, choice: UnburyChoice, i18n: &I18n) {
        self.overlay = Overlay::None;

        let UnburyChoice::Unbury(which) = choice else {
            return;
        };

        self.change_deck(i18n, |collection, deck| {
            overview::unbury(collection, deck, which)
        });
    }

    /// Runs a change on the overview's deck, then shows the deck again.
    fn change_deck(
        &mut self,
        i18n: &I18n,
        change: impl FnOnce(&mut Collection, DeckId) -> Result<(), OverviewError>,
    ) {
        let (Some(screen), Some(collection)) = (&self.deck_screen, &mut self.collection) else {
            return;
        };

        if let Err(error) = change(collection, screen.deck()) {
            tracing::warn!(?error, "Failed to change the deck");
            self.notice = Some(error_text::overview(&error, i18n));
        }

        self.reload_decks(i18n);
        self.refresh_deck_screen(i18n);
    }

    fn open_custom_study(&mut self, i18n: &I18n) {
        let (Some(screen), Some(collection)) = (&self.deck_screen, &mut self.collection) else {
            return;
        };

        match overview::custom_study_defaults(collection, screen.deck()) {
            Ok(defaults) => {
                self.overlay = Overlay::CustomStudy(CustomStudyForm::new(screen.deck(), defaults));
            },
            Err(error) => {
                tracing::warn!(?error, "Failed to load the custom study options");
                self.notice = Some(error_text::overview(&error, i18n));
            },
        }
    }

    fn submit_custom_study(&mut self, i18n: &I18n) {
        let Overlay::CustomStudy(form) = &mut self.overlay else {
            return;
        };
        let Ok(study) = form.submit() else {
            return;
        };
        let deck = form.deck();
        let Some(collection) = &mut self.collection else {
            return;
        };

        match overview::custom_study(collection, deck, study, i18n.anki()) {
            Ok(next) => {
                self.overlay = Overlay::None;
                self.reload_decks(i18n);
                // The overview fills in the name
                self.show_deck(next, String::new(), i18n);
            },
            // The dialog stays open so the choice can be changed, like in Anki
            Err(error) => {
                tracing::warn!(?error, "Custom study failed");
                self.notice = Some(error_text::overview(&error, i18n));
            },
        }
    }

    /// Shows the next card, or waits when a sync has the collection.
    fn resume_review(&mut self, i18n: &I18n) {
        let Some(review) = &mut self.review else {
            return;
        };
        let Some(collection) = &mut self.collection else {
            review.wait();
            return;
        };

        match next_card(collection, review, i18n) {
            Ok(()) => self.play_side(false),
            Err(error) => self.fail_review(&error, i18n),
        }
    }

    fn show_answer(&mut self, i18n: &I18n) {
        self.reveal(true, i18n);
    }

    fn reveal(&mut self, play: bool, i18n: &I18n) {
        let (Some(review), Some(collection)) = (&mut self.review, &mut self.collection) else {
            return;
        };
        let Some((card, typed)) = review.question() else {
            return;
        };

        match card.answer(collection, typed) {
            Ok(html) => {
                review.show_answer(html);

                if play {
                    self.play_side(false);
                }
            },
            Err(error) => self.fail_review(&error, i18n),
        }
    }

    fn card_action(&mut self, action: CardAction, i18n: &I18n) {
        let (Some(review), Some(collection)) = (&mut self.review, &mut self.collection) else {
            return;
        };
        let Some(card) = review.card() else {
            return;
        };
        let outcome = card_actions::apply(collection, card.id, card.note_id, card.marks, action);

        let outcome = match outcome {
            Ok(outcome) => outcome,
            Err(error) => {
                tracing::warn!(?error, ?action, "Card action failed");
                self.notice = Some(error_text::card_action(&error, i18n));
                return;
            },
        };

        self.notice = outcome_notice(outcome, i18n);

        match outcome {
            Outcome::Marked(marked) => {
                if let Some(card) = review.card_mut() {
                    card.marks.marked = marked;
                }
            },
            Outcome::Flagged(flag) => {
                if let Some(card) = review.card_mut() {
                    card.marks.flag = flag;
                }
            },
            _ => match next_card(collection, review, i18n) {
                Ok(()) => self.play_side(false),
                Err(error) => self.fail_review(&error, i18n),
            },
        }
    }

    fn request_edit(&mut self, i18n: &I18n) {
        let (Some(review), Some(collection)) = (&self.review, &mut self.collection) else {
            return;
        };
        let Some(card) = review.card() else {
            return;
        };

        match NoteText::load(collection, card.note_id) {
            Ok(note) => {
                self.edit = Some(EditRequest {
                    note,
                    purpose: Purpose::Edit,
                });
            },
            Err(error) => {
                tracing::warn!(?error, "Failed to load the note for editing");
                self.notice = Some(error_text::note_edit(&error, i18n));
            },
        }
    }

    /// A note to hand to the editor, which the event loop runs.
    pub const fn take_edit(&mut self) -> Option<EditRequest> {
        self.edit.take()
    }

    /// Saves or adds what came back from the editor.
    pub fn finish_edit(
        &mut self,
        request: EditRequest,
        edited: Result<String, EditorError>,
        i18n: &I18n,
    ) {
        let text = match edited {
            Ok(text) => text,
            Err(error) => {
                tracing::warn!(?error, "Editing the note failed");
                self.notice = Some(error_text::editor(&error, i18n));
                return;
            },
        };
        let Some(collection) = &mut self.collection else {
            return;
        };

        match request.purpose {
            Purpose::Edit => match request.note.save(collection, &text) {
                Ok(true) if self.browser.is_some() => self.refresh_browser(i18n),
                Ok(true) => self.rerender(i18n),
                Ok(false) => {},
                Err(error) => {
                    tracing::warn!(?error, "Failed to save the edited note");
                    self.notice = Some(error_text::note_edit(&error, i18n));
                },
            },
            Purpose::Add { deck, tags } => match request.note.add(collection, deck, tags, &text) {
                Ok(added) => {
                    tracing::info!(cards = added.cards, "Added a note");
                    self.notice = Some(if added.duplicate {
                        i18n.text("adding-duplicate")
                    } else {
                        i18n.anki().adding_added().into_owned()
                    });
                    self.reload_decks(i18n);
                    self.refresh_browser(i18n);

                    // A review would move on to the next card if refreshed now
                    if self.review.is_none() && self.browser.is_none() {
                        self.refresh_deck_screen(i18n);
                    }
                },
                Err(error) => {
                    tracing::warn!(?error, "Failed to add the note");
                    self.notice = Some(error_text::note_edit(&error, i18n));
                },
            },
        }
    }

    fn open_add(&mut self, i18n: &I18n) {
        let Some(collection) = &mut self.collection else {
            return;
        };
        let home_deck = self
            .review
            .as_ref()
            .map(Review::deck)
            .or_else(|| self.deck_screen.as_ref().map(DeckScreen::deck))
            .unwrap_or(DeckId(1));

        match adding::choices(collection, home_deck) {
            Ok(choices) => self.overlay = Overlay::Add(AddForm::new(choices)),
            Err(error) => {
                tracing::warn!(?error, "Failed to load the note types and decks");
                self.notice = Some(error.anki_error().message(i18n.anki()));
            },
        }
    }

    /// Opens the editor with the chosen note type's empty fields.
    fn submit_add(&mut self, i18n: &I18n) {
        let (Overlay::Add(form), Some(collection)) = (&self.overlay, &mut self.collection) else {
            return;
        };

        match NoteText::new_note(collection, form.notetype()) {
            Ok(note) => {
                self.edit = Some(EditRequest {
                    note,
                    purpose: Purpose::Add {
                        deck: form.deck(),
                        tags: form.tags().tags(),
                    },
                });
            },
            Err(error) => {
                tracing::warn!(?error, "Failed to start a new note");
                self.notice = Some(error_text::note_edit(&error, i18n));
            },
        }
    }

    /// Selects the entry of the open list and acts on it, like Enter does.
    fn click_list_entry(&mut self, index: usize, i18n: &I18n) {
        match &mut self.overlay {
            Overlay::Palette(list) => list.select(index),
            Overlay::ChangeDeck(list) => list.select(index),
            Overlay::SortBy(list) => list.select(index),
            Overlay::Add(form) => {
                if let Some(picker) = form.picker_mut() {
                    picker.select(index);
                }
            },
            _ => return,
        }

        self.confirm_list(i18n);
    }

    /// What Enter does in the open list.
    pub(super) fn confirm_list(&mut self, i18n: &I18n) {
        match &mut self.overlay {
            Overlay::Palette(_) => self.run_palette_selection(i18n),
            Overlay::ChangeDeck(list) => {
                if let Some(&deck) = list.selected_value() {
                    self.move_to_deck(deck, i18n);
                }
            },
            Overlay::SortBy(list) => {
                let column = list.selected_value().copied();
                self.overlay = Overlay::None;

                if let Some(column) = column {
                    self.sort_by(column, i18n);
                }
            },
            Overlay::Add(form) => form.confirm_picker(),
            _ => {},
        }
    }

    fn handle_login_key(&mut self, code: KeyCode, typed: Option<char>) {
        let Overlay::Login(form) = &mut self.overlay else {
            return;
        };

        match code {
            KeyCode::Up | KeyCode::BackTab => form.focus_previous(),
            KeyCode::Down | KeyCode::Tab => form.focus_next(),
            KeyCode::Backspace => form.pop(),
            KeyCode::Enter => match form.focus() {
                Field::Email => form.set_focus(Field::Password),
                Field::Password | Field::Submit => self.submit_login(),
                Field::Cancel => self.overlay = Overlay::None,
            },
            _ => {
                if let Some(character) = typed {
                    form.push(character);
                }
            },
        }
    }

    fn click_add_field(&mut self, field: add_form::Field, i18n: &I18n) {
        let Overlay::Add(form) = &mut self.overlay else {
            return;
        };

        form.close_picker();
        form.set_focus(field);

        match field {
            add_form::Field::Notetype | add_form::Field::Deck => form.open_picker(),
            add_form::Field::Tags => {},
            add_form::Field::Add => self.submit_add(i18n),
            add_form::Field::Close => self.overlay = Overlay::None,
        }
    }

    fn handle_add_key(&mut self, key: KeyCode, typed: Option<char>, i18n: &I18n) {
        let Overlay::Add(form) = &mut self.overlay else {
            return;
        };

        if let Some(picker) = form.picker_mut() {
            if picker.handle_key(key, typed) {
                form.confirm_picker();
            }

            return;
        }

        let completing =
            form.focus() == add_form::Field::Tags && !form.tags().completions().is_empty();

        match key {
            KeyCode::Tab if completing => {
                form.tags_mut().complete();
            },
            KeyCode::Up if completing => form.tags_mut().previous_completion(),
            KeyCode::Down if completing => form.tags_mut().next_completion(),
            KeyCode::Tab | KeyCode::Down => form.focus_next(),
            KeyCode::BackTab | KeyCode::Up => form.focus_previous(),
            KeyCode::Backspace => form.pop(),
            KeyCode::Enter => match form.focus() {
                add_form::Field::Notetype | add_form::Field::Deck => form.open_picker(),
                add_form::Field::Tags | add_form::Field::Add => self.submit_add(i18n),
                add_form::Field::Close => self.overlay = Overlay::None,
            },
            _ => {
                if let Some(character) = typed {
                    form.push(character);
                }
            },
        }
    }

    /// Shows the current card again after its note changed, without replaying its sounds.
    fn rerender(&mut self, i18n: &I18n) {
        let (Some(review), Some(collection)) = (&mut self.review, &mut self.collection) else {
            return;
        };
        let on_answer = matches!(review.stage(), Stage::Answer { .. });

        match next_card(collection, review, i18n) {
            Ok(()) if on_answer => self.reveal(false, i18n),
            Ok(()) => {},
            Err(error) => self.fail_review(&error, i18n),
        }
    }

    fn answer(&mut self, rating: Rating, i18n: &I18n) {
        let (Some(review), Some(collection)) = (&mut self.review, &mut self.collection) else {
            return;
        };
        let Some((card, taken)) = review.answered_card() else {
            return;
        };
        let result = review::answer(collection, card, rating, taken)
            .and_then(|()| next_card(collection, review, i18n));

        match result {
            Ok(()) => self.play_side(false),
            Err(error) => self.fail_review(&error, i18n),
        }
    }

    fn undo(&mut self, i18n: &I18n) {
        let (Some(review), Some(collection)) = (&mut self.review, &mut self.collection) else {
            return;
        };
        let result = review::undo(collection).and_then(|undone| {
            if undone {
                next_card(collection, review, i18n).map(|()| true)
            } else {
                Ok(false)
            }
        });

        match result {
            Ok(true) => self.play_side(false),
            Ok(false) => {},
            Err(error) => self.fail_review(&error, i18n),
        }
    }

    /// Plays the side on screen: as Replay Audio, or automatically when the deck allows it.
    /// Anything else on screen stops the sound.
    fn play_side(&self, replay: bool) {
        let side = match self.review.as_ref().map(Review::stage) {
            Some(Stage::Question { card, .. }) => Some((card, Side::Question)),
            Some(Stage::Answer { card, .. }) => Some((card, Side::Answer)),
            _ => None,
        };
        let sounds = side.map(|(card, side)| {
            if replay {
                card.sounds.replay(side)
            } else if card.sounds.autoplay {
                card.sounds.all(side)
            } else {
                Vec::new()
            }
        });

        match sounds {
            Some(sounds) if !sounds.is_empty() => self.play(sounds),
            _ => self.audio.stop(),
        }
    }

    fn play(&self, sounds: Vec<SoundRef>) {
        let Some(card) = self.review.as_ref().and_then(Review::card) else {
            return;
        };

        self.audio.play(tracks(card, sounds));
    }

    /// Goes back to the deck list, since the card on screen may no longer be valid.
    fn fail_review(&mut self, error: &ReviewError, i18n: &I18n) {
        tracing::warn!(?error, "Review failed");
        self.audio.stop();
        self.notice = Some(error_text::review(error, i18n));
        self.review = None;
        self.reload_decks(i18n);
        self.refresh_deck_screen(i18n);
    }

    fn choose_quit(&mut self, choice: QuitChoice, i18n: &I18n) {
        self.overlay = Overlay::None;

        match choice {
            QuitChoice::Wait => self.begin_quit(i18n),
            QuitChoice::Abort => self.quit_now(),
            QuitChoice::Cancel => {},
        }
    }

    fn run_palette_selection(&mut self, i18n: &I18n) {
        let Overlay::Palette(palette) = &self.overlay else {
            return;
        };
        let action = palette.selected_value().copied();
        self.overlay = Overlay::None;

        if let Some(action) = action {
            self.perform(action, i18n);
        }
    }

    fn submit_login(&mut self) {
        let Overlay::Login(form) = &mut self.overlay else {
            return;
        };

        if form.can_submit() {
            form.start();
            self.tasks
                .log_in(form.email().to_owned(), form.password().to_owned());
        }
    }

    fn palette_actions(&self) -> Vec<Action> {
        let mut actions = vec![Action::ToggleHelp];

        if self.browser.is_some() {
            actions.extend([
                Action::FocusSearch,
                Action::ToggleNotesMode,
                Action::TogglePreview,
                Action::SortBy,
                Action::SelectAll,
                Action::EditNote,
                Action::AddNote,
                Action::ToggleSuspend,
                Action::Card(CardAction::ToggleMark),
            ]);
            actions.extend(Flag::ALL.map(|flag| Action::Card(CardAction::ToggleFlag(flag))));
            actions.extend([
                Action::ChangeDeck,
                Action::AddTags,
                Action::RemoveTags,
                Action::Card(CardAction::DeleteNote),
                Action::Undo,
                Action::CloseBrowser,
                Action::Quit,
            ]);
            return actions;
        }

        if self.review.is_none()
            && let Some(screen) = &self.deck_screen
        {
            actions.push(Action::Study);

            if screen.is_filtered() {
                actions.extend([Action::Rebuild, Action::Empty]);
            } else {
                actions.push(Action::CustomStudy);
            }

            actions.extend([Action::Unbury, Action::CloseOverview]);
        }

        if self.review.is_some() {
            actions.extend([Action::ReplayAudio, Action::EditNote]);
            actions.extend(
                [
                    CardAction::ToggleMark,
                    CardAction::BuryCard,
                    CardAction::BuryNote,
                    CardAction::SuspendCard,
                    CardAction::SuspendNote,
                ]
                .map(Action::Card),
            );
            actions.extend(Flag::ALL.map(|flag| Action::Card(CardAction::ToggleFlag(flag))));
            actions.extend([
                Action::Card(CardAction::DeleteNote),
                Action::Undo,
                Action::CloseReview,
            ]);
        } else if self.session.is_some() {
            actions.extend([Action::Sync, Action::LogOut]);
        } else {
            actions.push(Action::LogIn);
        }

        actions.push(Action::Quit);
        actions
    }

    fn quit(&mut self, i18n: &I18n) {
        // A second quit skips the remaining syncs
        if self.quitting || matches!(self.overlay, Overlay::Quit(_)) {
            self.quit_now();
        } else if self.media_sync.is_some() {
            self.overlay = Overlay::Quit(quit_prompt::new());
        } else {
            self.begin_quit(i18n);
        }
    }

    /// Syncs once more unless the settings say not to, then quits when every sync is done.
    fn begin_quit(&mut self, i18n: &I18n) {
        let can_sync =
            self.config.sync_on_quit && self.session.is_some() && self.collection.is_some();
        let syncing = self.sync == SyncStatus::Syncing;

        if !(can_sync || syncing || self.media_sync.is_some()) {
            self.quit_now();
            return;
        }

        self.quitting = true;
        self.notice = Some(i18n.text("sync-before-quit"));

        if can_sync && !syncing {
            self.start_sync();
        }
    }

    fn quit_if_done(&mut self) {
        if self.quitting && self.sync != SyncStatus::Syncing && self.media_sync.is_none() {
            self.quit = true;
        }
    }

    /// Media sync resumes where it left off, so aborting it loses nothing.
    fn quit_now(&mut self) {
        self.abort_media_sync();
        self.quit = true;
    }

    fn perform(&mut self, action: Action, i18n: &I18n) {
        if self.browser.is_some() && self.perform_in_browser(action, i18n) {
            return;
        }

        match action {
            Action::ToggleHelp => {
                self.overlay = match self.overlay {
                    Overlay::Help => Overlay::None,
                    _ => Overlay::Help,
                };
            },
            Action::OpenPalette => {
                self.overlay = Overlay::Palette(palette::new(&self.palette_actions(), i18n));
            },
            // A list open in the Add dialog closes first
            Action::CloseOverlay => match &mut self.overlay {
                Overlay::Add(form) if form.picker().is_some() => form.close_picker(),
                _ => self.overlay = Overlay::None,
            },
            Action::AddNote => self.open_add(i18n),
            Action::LogIn => self.overlay = Overlay::Login(LoginForm::new()),
            Action::LogOut => self.tasks.log_out(),
            // Studying keeps the collection; syncing waits for the deck list, like in Anki
            Action::Sync if self.review.is_some() || self.browser.is_some() => {},
            Action::Sync => {
                if self.session.is_none() {
                    self.overlay = Overlay::Login(LoginForm::new());
                } else if self.sync != SyncStatus::Syncing {
                    self.start_sync();
                }
            },
            Action::OpenDeck => self.open_deck(i18n),
            Action::Study => self.start_review(i18n),
            Action::CustomStudy => self.open_custom_study(i18n),
            Action::Unbury => self.unbury(i18n),
            Action::Rebuild
                if self
                    .deck_screen
                    .as_ref()
                    .is_some_and(DeckScreen::is_filtered) =>
            {
                self.change_deck(i18n, overview::rebuild);
            },
            Action::Empty
                if self
                    .deck_screen
                    .as_ref()
                    .is_some_and(DeckScreen::is_filtered) =>
            {
                self.change_deck(i18n, overview::empty);
            },
            Action::CloseOverview => self.close_overview(i18n),
            Action::ShowAnswer => self.show_answer(i18n),
            Action::Answer(rating) => self.answer(rating, i18n),
            Action::Undo => self.undo(i18n),
            Action::ReplayAudio => self.play_side(true),
            Action::Card(action) => self.card_action(action, i18n),
            Action::EditNote => self.request_edit(i18n),
            Action::OpenBrowser => self.open_browser(i18n),
            Action::OpenSettings => {
                self.overlay = Overlay::Settings(settings_form::new(&self.config, i18n));
            },
            Action::DeckOptions => self.open_deck_options(i18n),
            Action::OpenStats => {
                self.stats_screen = Some(StatsScreen::new());
                self.load_stats(i18n);
            },
            Action::CloseStats => self.stats_screen = None,
            Action::ToggleStatsScope => {
                if let Some(screen) = &mut self.stats_screen {
                    screen.toggle_scope();
                }

                self.load_stats(i18n);
            },
            Action::NextPeriod | Action::PreviousPeriod => {
                if let Some(screen) = &mut self.stats_screen {
                    screen.shift_period(action == Action::NextPeriod);
                }

                self.load_stats(i18n);
            },
            // Rebuild and Empty are for filtered decks; the rest only work while the browser
            // is open, which handles them first
            Action::Rebuild
            | Action::Empty
            | Action::CloseBrowser
            | Action::FocusSearch
            | Action::RunSearch
            | Action::LeaveSearch
            | Action::ToggleNotesMode
            | Action::TogglePreview
            | Action::ToggleSelected
            | Action::SelectAll
            | Action::ToggleSuspend
            | Action::ChangeDeck
            | Action::AddTags
            | Action::RemoveTags
            | Action::SortBy => {},
            Action::CloseReview => {
                self.audio.stop();
                self.review = None;
                self.reload_decks(i18n);
                self.refresh_deck_screen(i18n);
            },
            Action::Quit => self.quit(i18n),
        }
    }
}

/// Moves the focus with arrows and Tab; returns the choice made with Enter.
fn navigate<T: Copy>(choices: &mut Choices<T>, code: KeyCode) -> Option<T> {
    match code {
        KeyCode::Left | KeyCode::Up | KeyCode::BackTab => choices.focus_previous(),
        KeyCode::Right | KeyCode::Down | KeyCode::Tab => choices.focus_next(),
        KeyCode::Enter => return choices.focused(),
        _ => {},
    }

    None
}

fn next_card(
    collection: &mut Collection,
    review: &mut Review,
    i18n: &I18n,
) -> Result<(), ReviewError> {
    if review.take_deck_selection() {
        review::select_deck(collection, review.deck())?;
    }

    match review::next_card(collection, i18n.anki())? {
        Some(next) => review.show_question(next.card, next.counts),
        None => review.finish(review::studied_today(collection)?),
    }

    Ok(())
}

/// Files only; speech isn't supported yet.
fn tracks(card: &StudyCard, sounds: Vec<SoundRef>) -> Vec<Track> {
    sounds
        .into_iter()
        .filter_map(|sound| match card.sounds.get(sound)? {
            Sound::File(file) => Some(Track {
                sound,
                file: file.clone(),
            }),
            Sound::Tts(_) => None,
        })
        .collect()
}

/// Anki's tooltips for the actions that take the card away; marks show on the card.
fn outcome_notice(outcome: Outcome, i18n: &I18n) -> Option<String> {
    let tr = i18n.anki();
    let text = match outcome {
        Outcome::Buried { count } => tr.studying_cards_buried(count),
        Outcome::CardSuspended => tr.studying_card_suspended(),
        Outcome::NoteSuspended => tr.studying_note_suspended(),
        Outcome::Deleted { count } => tr.browsing_cards_deleted(count),
        Outcome::Marked(_) | Outcome::Flagged(_) => return None,
    };

    Some(text.into_owned())
}
