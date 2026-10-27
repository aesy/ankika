use anki::collection::Collection;
use anki::decks::DeckId;
use anki::error::AnkiError;
use anki::notetype::NotetypeId;

use crate::remote::english;

#[derive(Debug, thiserror::Error)]
#[error("Failed to load the note types and decks: {}", english(source))]
pub struct AddingError {
    #[source]
    source: AnkiError,
}

impl AddingError {
    /// Anki's own error, which it can describe in the user's language.
    #[must_use]
    pub const fn anki_error(&self) -> &AnkiError {
        &self.source
    }
}

/// What Anki's Add window lets you pick from, and what it starts with.
pub struct AddChoices {
    pub notetypes: Vec<(NotetypeId, String)>,
    /// Normal decks only, since filtered decks can't take new notes.
    pub decks: Vec<(DeckId, String)>,
    /// Every tag in the collection, for completion.
    pub tags: Vec<String>,
    pub notetype: NotetypeId,
    pub deck: DeckId,
}

/// `home_deck` is the deck of the card being studied, which Anki uses when the current
/// deck is filtered.
pub fn choices(collection: &mut Collection, home_deck: DeckId) -> Result<AddChoices, AddingError> {
    let error = |source| AddingError { source };
    let defaults = collection.defaults_for_adding(home_deck).map_err(error)?;
    let notetypes = collection
        .get_all_notetypes()
        .map_err(error)?
        .iter()
        .map(|notetype| (notetype.id, notetype.name.clone()))
        .collect();
    let decks = normal_decks(collection)?;
    let tags = all_tags(collection)?;

    Ok(AddChoices {
        notetypes,
        decks,
        tags,
        notetype: defaults.notetype_id,
        deck: defaults.deck_id,
    })
}

/// Normal decks only, since filtered decks can't take notes or moved cards.
pub fn normal_decks(collection: &mut Collection) -> Result<Vec<(DeckId, String)>, AddingError> {
    collection
        .get_all_normal_deck_names(false)
        .map_err(|source| AddingError { source })
}

pub fn all_tags(collection: &Collection) -> Result<Vec<String>, AddingError> {
    collection
        .storage
        .all_tags()
        .map(|tags| tags.into_iter().map(|tag| tag.name).collect())
        .map_err(|source| AddingError { source })
}
