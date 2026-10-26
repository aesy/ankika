use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::layout::Position;

use crate::i18n::I18n;
use crate::tui::action::Action;
use crate::tui::hit_map::{Hit, HitMap};
use crate::tui::keymap::{self, Context};
use crate::tui::palette::Palette;

pub enum Overlay {
    None,
    Help,
    Palette(Palette),
}

pub struct State {
    overlay: Overlay,
    mouse: Option<Position>,
    hits: HitMap,
    quit: bool,
}

impl State {
    pub fn new() -> Self {
        Self {
            overlay: Overlay::None,
            mouse: None,
            hits: HitMap::default(),
            quit: false,
        }
    }

    pub const fn should_quit(&self) -> bool {
        self.quit
    }

    pub const fn overlay(&self) -> &Overlay {
        &self.overlay
    }

    pub const fn mouse(&self) -> Option<Position> {
        self.mouse
    }

    pub const fn context(&self) -> Context {
        match self.overlay {
            Overlay::None => Context::Main,
            Overlay::Help => Context::Help,
            Overlay::Palette(_) => Context::Palette,
        }
    }

    pub fn set_hits(&mut self, hits: HitMap) {
        self.hits = hits;
    }

    pub fn handle_event(&mut self, event: &Event, i18n: &I18n) {
        match event {
            // Windows also reports releases
            Event::Key(key) if key.kind == KeyEventKind::Press => self.handle_key(key, i18n),
            Event::Mouse(mouse) => self.handle_mouse(*mouse, i18n),
            _ => {},
        }
    }

    fn handle_key(&mut self, key: &KeyEvent, i18n: &I18n) {
        if let Some(action) = keymap::action_for(self.context(), key) {
            self.perform(action, i18n);
            return;
        }

        let Overlay::Palette(palette) = &mut self.overlay else {
            return;
        };

        match key.code {
            KeyCode::Up => palette.select_previous(),
            KeyCode::Down => palette.select_next(),
            KeyCode::Backspace => palette.pop(),
            KeyCode::Enter => self.run_palette_selection(i18n),
            KeyCode::Char(character)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                palette.push(character);
            },
            _ => {},
        }
    }

    fn handle_mouse(&mut self, mouse: MouseEvent, i18n: &I18n) {
        let position = Position::new(mouse.column, mouse.row);
        let hit = self.hits.at(position);
        self.mouse = Some(position);

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => self.click(hit, i18n),
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

    fn perform(&mut self, action: Action, i18n: &I18n) {
        match action {
            Action::ToggleHelp => {
                self.overlay = match self.overlay {
                    Overlay::Help => Overlay::None,
                    _ => Overlay::Help,
                };
            },
            Action::OpenPalette => self.overlay = Overlay::Palette(Palette::new(i18n)),
            Action::CloseOverlay => self.overlay = Overlay::None,
            Action::Quit => self.quit = true,
        }
    }
}
