use ankika_core::overview::Unbury;

use crate::tui::choices::Choices;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnburyChoice {
    Unbury(Unbury),
    Cancel,
}

/// Anki's question when the deck has both kinds of buried cards, in Anki's order.
pub const fn new() -> Choices<UnburyChoice> {
    Choices::focusing_first(&[
        UnburyChoice::Unbury(Unbury::Manual),
        UnburyChoice::Unbury(Unbury::Siblings),
        UnburyChoice::Unbury(Unbury::All),
        UnburyChoice::Cancel,
    ])
}
