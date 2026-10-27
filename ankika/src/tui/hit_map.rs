use anki::decks::DeckId;
use ankika_core::overview::StudyKind;
use ankika_core::review::SoundRef;
use ratatui::layout::{Position, Rect};

use crate::tui::action::Action;
use crate::tui::custom_study_form;
use crate::tui::full_sync_prompt::Choice;
use crate::tui::login_form::Field;
use crate::tui::quit_prompt::QuitChoice;
use crate::tui::unbury_prompt::UnburyChoice;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hit {
    Action(Action),
    PaletteEntry(usize),
    LoginField(Field),
    FullSyncChoice(Choice),
    QuitChoice(QuitChoice),
    UnburyChoice(UnburyChoice),
    CustomStudyChoice(custom_study_form::Choice),
    CustomStudyKind(StudyKind),
    CustomStudyField(custom_study_form::Field),
    Deck(DeckId),
    Sound(SoundRef),
    /// The expand or collapse arrow in front of a deck.
    DeckToggle(DeckId),
    /// Inside an overlay but not on anything clickable, so the overlay stays open.
    Overlay,
}

/// Clickable regions of the last drawn frame. Later registrations sit on top.
#[derive(Debug, Default)]
pub struct HitMap {
    regions: Vec<(Rect, Hit)>,
}

impl HitMap {
    pub fn register(&mut self, area: Rect, hit: Hit) {
        self.regions.push((area, hit));
    }

    pub fn at(&self, position: Position) -> Option<Hit> {
        self.regions
            .iter()
            .rev()
            .find(|(area, _)| area.contains(position))
            .map(|(_, hit)| *hit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topmost_region_wins() {
        let mut hits = HitMap::default();
        hits.register(Rect::new(0, 0, 10, 10), Hit::Overlay);
        hits.register(Rect::new(2, 2, 3, 1), Hit::PaletteEntry(0));

        assert_eq!(hits.at(Position::new(3, 2)), Some(Hit::PaletteEntry(0)));
        assert_eq!(hits.at(Position::new(8, 8)), Some(Hit::Overlay));
        assert_eq!(hits.at(Position::new(20, 20)), None);
    }
}
