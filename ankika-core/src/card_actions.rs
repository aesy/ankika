use anki::card::CardId;
use anki::collection::Collection;
use anki::error::AnkiError;
use anki::notes::NoteId;
use anki_proto::scheduler::bury_or_suspend_cards_request::Mode;

use crate::remote::english;

/// The tag Anki uses for marked notes.
const MARKED_TAG: &str = "marked";

#[derive(Debug, thiserror::Error)]
#[error("Failed to change the card: {}", english(source))]
pub struct CardActionError {
    #[source]
    source: AnkiError,
}

impl CardActionError {
    /// Anki's own error, which it can describe in the user's language.
    #[must_use]
    pub const fn anki_error(&self) -> &AnkiError {
        &self.source
    }
}

/// Anki's flag colors, in the order of their numbers 1 to 7.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flag {
    Red = 1,
    Orange = 2,
    Green = 3,
    Blue = 4,
    Pink = 5,
    Turquoise = 6,
    Purple = 7,
}

impl Flag {
    pub const ALL: [Self; 7] = [
        Self::Red,
        Self::Orange,
        Self::Green,
        Self::Blue,
        Self::Pink,
        Self::Turquoise,
        Self::Purple,
    ];

    #[must_use]
    pub fn from_number(number: u32) -> Option<Self> {
        Self::ALL.into_iter().find(|flag| *flag as u32 == number)
    }
}

/// What the reviewer can do to the card on screen, as Anki's reviewer does it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardAction {
    BuryCard,
    BuryNote,
    SuspendCard,
    SuspendNote,
    ToggleMark,
    /// Removes the flag when the card already has it.
    ToggleFlag(Flag),
    DeleteNote,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Buried { count: usize },
    CardSuspended,
    NoteSuspended,
    Marked(bool),
    Flagged(Option<Flag>),
    Deleted { count: usize },
}

impl Outcome {
    /// Whether the card left the queue, so the next one should be shown.
    #[must_use]
    pub const fn removes_card(self) -> bool {
        !matches!(self, Self::Marked(_) | Self::Flagged(_))
    }
}

/// The card's flag and whether its note is marked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Marks {
    pub flag: Option<Flag>,
    pub marked: bool,
}

pub fn marks(collection: &mut Collection, card: CardId) -> Result<Marks, CardActionError> {
    let card = collection
        .storage
        .get_card(card)
        .map_err(|source| CardActionError { source })?;
    let Some(card) = card else {
        return Ok(Marks::default());
    };
    let note = collection
        .storage
        .get_note(card.note_id())
        .map_err(|source| CardActionError { source })?;
    let marked = note.is_some_and(|note| {
        note.tags
            .iter()
            .any(|tag| tag.eq_ignore_ascii_case(MARKED_TAG))
    });
    // Cards only expose their flag through the protobuf form
    let flag = Flag::from_number(anki_proto::cards::Card::from(card).flags);

    Ok(Marks { flag, marked })
}

/// Every action is undoable, like in Anki.
pub fn apply(
    collection: &mut Collection,
    card: CardId,
    note: NoteId,
    marks: Marks,
    action: CardAction,
) -> Result<Outcome, CardActionError> {
    let error = |source| CardActionError { source };

    match action {
        CardAction::BuryCard => collection
            .bury_or_suspend_cards(&[card], Mode::BuryUser)
            .map(|output| Outcome::Buried {
                count: output.output,
            }),
        CardAction::BuryNote => note_cards(collection, note)
            .and_then(|cards| collection.bury_or_suspend_cards(&cards, Mode::BuryUser))
            .map(|output| Outcome::Buried {
                count: output.output,
            }),
        CardAction::SuspendCard => collection
            .bury_or_suspend_cards(&[card], Mode::Suspend)
            .map(|_| Outcome::CardSuspended),
        CardAction::SuspendNote => note_cards(collection, note)
            .and_then(|cards| collection.bury_or_suspend_cards(&cards, Mode::Suspend))
            .map(|_| Outcome::NoteSuspended),
        CardAction::ToggleMark if marks.marked => collection
            .remove_tags_from_notes(&[note], MARKED_TAG)
            .map(|_| Outcome::Marked(false)),
        CardAction::ToggleMark => collection
            .add_tags_to_notes(&[note], MARKED_TAG)
            .map(|_| Outcome::Marked(true)),
        CardAction::ToggleFlag(flag) => {
            let flag = (marks.flag != Some(flag)).then_some(flag);

            collection
                .set_card_flag(&[card], flag.map_or(0, |flag| flag as u32))
                .map(|_| Outcome::Flagged(flag))
        },
        CardAction::DeleteNote => note_cards(collection, note).and_then(|cards| {
            collection
                .remove_notes(&[note])
                .map(|_| Outcome::Deleted { count: cards.len() })
        }),
    }
    .map_err(error)
}

fn note_cards(collection: &Collection, note: NoteId) -> Result<Vec<CardId>, AnkiError> {
    collection
        .storage
        .all_card_ids_of_note_in_template_order(note)
}
