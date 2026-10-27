use anki::decks::DeckId;
use ankika_core::overview::Overview;

/// The deck overview, shown between the deck list and studying, like in Anki.
pub enum DeckScreen {
    /// A sync has the collection; the overview loads once it is back.
    Waiting {
        deck: DeckId,
        name: String,
    },
    Ready(Overview),
}

impl DeckScreen {
    pub const fn deck(&self) -> DeckId {
        match self {
            Self::Waiting { deck, .. } => *deck,
            Self::Ready(overview) => overview.deck,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Waiting { name, .. } => name,
            Self::Ready(overview) => &overview.name,
        }
    }

    pub const fn overview(&self) -> Option<&Overview> {
        match self {
            Self::Waiting { .. } => None,
            Self::Ready(overview) => Some(overview),
        }
    }

    pub const fn is_filtered(&self) -> bool {
        matches!(self, Self::Ready(overview) if overview.filtered)
    }
}
