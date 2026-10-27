use std::cell::Cell;
use std::time::{Duration, Instant};

use anki::decks::DeckId;
use ankika_core::review::{Counts, StudyCard, TypeAnswer};

pub enum Stage {
    /// A sync has the collection; studying starts once it is back.
    Waiting,
    Question {
        card: StudyCard,
        typed: String,
    },
    Answer {
        card: StudyCard,
        html: String,
    },
    /// Nothing left in this deck for today.
    Finished {
        studied_today: String,
    },
}

/// Studying one deck.
pub struct Review {
    deck: DeckId,
    deck_name: String,
    counts: Option<Counts>,
    stage: Stage,
    shown_at: Instant,
    /// The deck becomes Anki's current deck once the collection is available.
    needs_deck_selection: bool,
    scroll: u16,
    /// Set by the view, which knows how tall the card is at the current width.
    max_scroll: Cell<u16>,
}

impl Review {
    pub fn new(deck: DeckId, deck_name: String) -> Self {
        Self {
            deck,
            deck_name,
            counts: None,
            stage: Stage::Waiting,
            shown_at: Instant::now(),
            needs_deck_selection: true,
            scroll: 0,
            max_scroll: Cell::new(0),
        }
    }

    pub const fn deck(&self) -> DeckId {
        self.deck
    }

    pub const fn take_deck_selection(&mut self) -> bool {
        std::mem::replace(&mut self.needs_deck_selection, false)
    }

    pub fn deck_name(&self) -> &str {
        &self.deck_name
    }

    pub const fn counts(&self) -> Option<Counts> {
        self.counts
    }

    pub const fn stage(&self) -> &Stage {
        &self.stage
    }

    pub fn scroll(&self) -> u16 {
        self.scroll.min(self.max_scroll.get())
    }

    pub fn set_max_scroll(&self, max: u16) {
        self.max_scroll.set(max);
    }

    /// Whether the question has an input line that takes the typed characters.
    pub fn is_typing(&self) -> bool {
        match &self.stage {
            Stage::Question { card, .. } => card
                .type_answer
                .as_ref()
                .is_some_and(TypeAnswer::wants_input),
            _ => false,
        }
    }

    pub fn show_question(&mut self, card: StudyCard, counts: Counts) {
        self.counts = Some(counts);
        self.stage = Stage::Question {
            card,
            typed: String::new(),
        };
        self.shown_at = Instant::now();
        self.scroll = 0;
    }

    /// The card on screen, if any.
    pub const fn card(&self) -> Option<&StudyCard> {
        match &self.stage {
            Stage::Question { card, .. } | Stage::Answer { card, .. } => Some(card),
            Stage::Waiting | Stage::Finished { .. } => None,
        }
    }

    /// The question and what was typed, for revealing the answer.
    pub fn question(&self) -> Option<(&StudyCard, &str)> {
        match &self.stage {
            Stage::Question { card, typed } => Some((card, typed)),
            _ => None,
        }
    }

    pub fn show_answer(&mut self, html: String) {
        let Stage::Question { card, .. } = std::mem::replace(&mut self.stage, Stage::Waiting)
        else {
            return;
        };

        self.stage = Stage::Answer { card, html };
        self.scroll = 0;
    }

    /// The card whose answer is showing, with how long it has been studied.
    pub fn answered_card(&self) -> Option<(&StudyCard, Duration)> {
        match &self.stage {
            Stage::Answer { card, .. } => Some((card, self.shown_at.elapsed())),
            _ => None,
        }
    }

    pub fn finish(&mut self, studied_today: String) {
        self.counts = None;
        self.stage = Stage::Finished { studied_today };
        self.scroll = 0;
    }

    pub fn wait(&mut self) {
        self.stage = Stage::Waiting;
    }

    pub fn push(&mut self, character: char) {
        if let Stage::Question { typed, .. } = &mut self.stage {
            typed.push(character);
        }
    }

    pub fn pop(&mut self) {
        if let Stage::Question { typed, .. } = &mut self.stage {
            typed.pop();
        }
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll().saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        self.scroll = self.scroll().saturating_add(1).min(self.max_scroll.get());
    }
}
