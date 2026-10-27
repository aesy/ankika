use std::cell::Cell;

use ankika_core::stats::{Period, Scope, Stats};
use crossterm::event::KeyCode;

/// Rows Page Up and Page Down move.
const PAGE: i16 = 10;

/// Anki's statistics page for the current deck or the whole collection.
pub struct StatsScreen {
    scope: Scope,
    period: Period,
    /// `None` until loaded, or while a sync has the collection.
    stats: Option<Stats>,
    scroll: u16,
    /// Set by the view, which knows how tall the page is.
    max_scroll: Cell<u16>,
}

impl StatsScreen {
    pub const fn new() -> Self {
        Self {
            scope: Scope::Deck,
            period: Period::Month,
            stats: None,
            scroll: 0,
            max_scroll: Cell::new(0),
        }
    }

    pub const fn scope(&self) -> Scope {
        self.scope
    }

    pub const fn period(&self) -> Period {
        self.period
    }

    pub const fn stats(&self) -> Option<&Stats> {
        self.stats.as_ref()
    }

    pub fn set_stats(&mut self, stats: Stats) {
        self.stats = Some(stats);
    }

    pub const fn toggle_scope(&mut self) {
        self.scope = match self.scope {
            Scope::Deck => Scope::Collection,
            Scope::Collection => Scope::Deck,
        };
    }

    /// Moves through Anki's ranges; stops at either end.
    pub fn shift_period(&mut self, forward: bool) {
        let index = Period::ALL
            .iter()
            .position(|period| *period == self.period)
            .unwrap_or(0);
        let next = if forward {
            (index + 1).min(Period::ALL.len() - 1)
        } else {
            index.saturating_sub(1)
        };

        self.period = Period::ALL[next];
    }

    pub fn scroll(&self) -> u16 {
        self.scroll.min(self.max_scroll.get())
    }

    pub fn set_max_scroll(&self, max: u16) {
        self.max_scroll.set(max);
    }

    /// Scrolls the page with the arrow and page keys.
    pub fn handle_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Up => self.scroll_by(-1),
            KeyCode::Down => self.scroll_by(1),
            KeyCode::PageUp => self.scroll_by(-PAGE),
            KeyCode::PageDown => self.scroll_by(PAGE),
            _ => {},
        }
    }

    pub fn scroll_by(&mut self, rows: i16) {
        self.scroll = self
            .scroll()
            .saturating_add_signed(rows)
            .min(self.max_scroll.get());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn period_stops_at_the_ends() {
        let mut screen = StatsScreen::new();
        screen.shift_period(false);
        assert_eq!(screen.period(), Period::Month);

        for _ in 0..5 {
            screen.shift_period(true);
        }
        assert_eq!(screen.period(), Period::AllTime);
    }
}
