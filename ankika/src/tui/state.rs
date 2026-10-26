use ankika_core::session::Session;
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::layout::Position;

use crate::i18n::I18n;
use crate::tui::action::Action;
use crate::tui::error_text;
use crate::tui::hit_map::{Hit, HitMap};
use crate::tui::keymap::{self, Context};
use crate::tui::login_form::{Field, LoginForm};
use crate::tui::palette::Palette;
use crate::tui::tasks::{TaskResult, Tasks};

pub enum Overlay {
    None,
    Help,
    Palette(Palette),
    Login(LoginForm),
}

pub struct State {
    session: Option<Session>,
    tasks: Tasks,
    overlay: Overlay,
    notice: Option<String>,
    mouse: Option<Position>,
    hits: HitMap,
    quit: bool,
}

impl State {
    pub fn new(session: Option<Session>, tasks: Tasks) -> Self {
        Self {
            session,
            tasks,
            overlay: Overlay::None,
            notice: None,
            mouse: None,
            hits: HitMap::default(),
            quit: false,
        }
    }

    pub const fn should_quit(&self) -> bool {
        self.quit
    }

    pub const fn session(&self) -> Option<&Session> {
        self.session.as_ref()
    }

    pub const fn overlay(&self) -> &Overlay {
        &self.overlay
    }

    /// An error to show until the next input.
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
        }
    }

    pub fn set_hits(&mut self, hits: HitMap) {
        self.hits = hits;
    }

    pub fn set_notice(&mut self, notice: String) {
        self.notice = Some(notice);
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
            },
            TaskResult::LoggedOut(Err(error)) => {
                tracing::warn!(?error, "Logout failed");
                self.notice = Some(i18n.text("error-log-out"));
            },
        }
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
            Overlay::None | Overlay::Help => {},
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
            MouseEventKind::ScrollUp => {
                if let Overlay::Palette(palette) = &mut self.overlay {
                    palette.select_previous();
                }
            },
            MouseEventKind::ScrollDown => {
                if let Overlay::Palette(palette) = &mut self.overlay {
                    palette.select_next();
                }
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
            Some(Hit::Overlay) => {},
            None => self.overlay = Overlay::None,
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

    const fn palette_actions(&self) -> [Action; 3] {
        let account = if self.session.is_some() {
            Action::LogOut
        } else {
            Action::LogIn
        };

        [Action::ToggleHelp, account, Action::Quit]
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
            Action::Quit => self.quit = true,
        }
    }
}
