use anki::collection::Collection;
use anki::decks::DeckId;
use ankika_core::collection::CollectionError;
use ankika_core::decks;
use ankika_core::session::Session;
use ankika_core::sync::{
    FullSyncDirection, FullSyncOutput, FullSyncRequired, NormalSyncOutput, SyncError,
};
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use futures_util::future::AbortHandle;
use ratatui::layout::Position;

use crate::i18n::I18n;
use crate::tui::action::Action;
use crate::tui::choices::Choices;
use crate::tui::deck_list::DeckList;
use crate::tui::error_text;
use crate::tui::full_sync_prompt::{Choice, FullSyncPrompt};
use crate::tui::hit_map::{Hit, HitMap};
use crate::tui::keymap::{self, Context};
use crate::tui::login_form::{Field, LoginForm};
use crate::tui::palette::Palette;
use crate::tui::quit_prompt::{self, QuitChoice};
use crate::tui::tasks::{TaskResult, Tasks};

pub enum Overlay {
    None,
    Help,
    Palette(Palette),
    Login(LoginForm),
    FullSync(FullSyncPrompt),
    Quit(Choices<QuitChoice>),
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
    decks: DeckList,
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
    pub fn new(session: Option<Session>, collection: Collection, tasks: Tasks) -> Self {
        Self {
            session,
            collection: Some(collection),
            tasks,
            decks: DeckList::default(),
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

    pub const fn context(&self) -> Context {
        match self.overlay {
            Overlay::None => Context::Main,
            Overlay::Help => Context::Help,
            Overlay::Palette(_) => Context::Palette,
            Overlay::Login(_) => Context::Login,
            Overlay::FullSync(_) => Context::FullSync,
            Overlay::Quit(_) => Context::Quit,
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
            Overlay::Palette(palette) => match key.code {
                KeyCode::Up => palette.select_previous(),
                KeyCode::Down => palette.select_next(),
                KeyCode::Backspace => palette.pop(),
                KeyCode::Enter => self.run_palette_selection(i18n),
                _ => {
                    if let Some(character) = typed {
                        palette.push(character);
                    }
                },
            },
            Overlay::Login(form) => match key.code {
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
            },
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
                if let (Overlay::Palette(palette), Some(Hit::PaletteEntry(index))) =
                    (&mut self.overlay, hit)
                {
                    palette.select(index);
                }
            },
            MouseEventKind::ScrollUp => match &mut self.overlay {
                Overlay::Palette(palette) => palette.select_previous(),
                Overlay::None => self.decks.select_previous(),
                _ => {},
            },
            MouseEventKind::ScrollDown => match &mut self.overlay {
                Overlay::Palette(palette) => palette.select_next(),
                Overlay::None => self.decks.select_next(),
                _ => {},
            },
            _ => {},
        }
    }

    fn click(&mut self, hit: Option<Hit>, i18n: &I18n) {
        match hit {
            Some(Hit::Action(action)) => self.perform(action, i18n),
            Some(Hit::PaletteEntry(index)) => {
                if let Overlay::Palette(palette) = &mut self.overlay {
                    palette.select(index);
                }

                self.run_palette_selection(i18n);
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
            Some(Hit::Deck(id)) => self.decks.select(id),
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
        let action = palette.selected_action();
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

        if self.session.is_some() {
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

    /// Syncs once more, then quits when every sync is done.
    fn begin_quit(&mut self, i18n: &I18n) {
        let can_sync = self.session.is_some() && self.collection.is_some();
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
        match action {
            Action::ToggleHelp => {
                self.overlay = match self.overlay {
                    Overlay::Help => Overlay::None,
                    _ => Overlay::Help,
                };
            },
            Action::OpenPalette => {
                self.overlay = Overlay::Palette(Palette::new(&self.palette_actions(), i18n));
            },
            Action::CloseOverlay => self.overlay = Overlay::None,
            Action::LogIn => self.overlay = Overlay::Login(LoginForm::new()),
            Action::LogOut => self.tasks.log_out(),
            Action::Sync => {
                if self.session.is_none() {
                    self.overlay = Overlay::Login(LoginForm::new());
                } else if self.sync != SyncStatus::Syncing {
                    self.start_sync();
                }
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
