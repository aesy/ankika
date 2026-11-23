use anki::collection::Collection;
use anki::decks::DeckId;
use anki::error::AnkiError;
use anki::timestamp::TimestampSecs;
use anki_proto::decks::DeckTreeNode;
use anki_proto::decks::set_deck_collapsed_request::Scope;

use crate::remote::english;

#[derive(Debug, thiserror::Error)]
pub enum DeckError {
    #[error("Failed to load the deck list: {}", english(source))]
    Load {
        #[source]
        source: AnkiError,
    },
    #[error("Failed to store whether a deck is collapsed: {}", english(source))]
    SetCollapsed {
        #[source]
        source: AnkiError,
    },
    #[error("Failed to look up the deck: {}", english(source))]
    Find {
        #[source]
        source: AnkiError,
    },
}

impl DeckError {
    /// Anki's own error, which it can describe in the user's language.
    #[must_use]
    pub const fn anki_error(&self) -> &AnkiError {
        match self {
            Self::Load { source } | Self::SetCollapsed { source } | Self::Find { source } => source,
        }
    }
}

/// A deck with today's due counts, which include its subdecks and the daily limits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeckNode {
    pub id: DeckId,
    /// The last part of the full name.
    pub name: String,
    pub collapsed: bool,
    pub filtered: bool,
    pub new_count: u32,
    pub learn_count: u32,
    pub review_count: u32,
    pub children: Vec<Self>,
}

impl DeckNode {
    #[must_use]
    pub const fn due_count(&self) -> u32 {
        self.new_count + self.learn_count + self.review_count
    }
}

/// The top-level decks. Anki leaves out the Default deck when it is empty and others exist.
pub fn deck_tree(collection: &mut Collection) -> Result<Vec<DeckNode>, DeckError> {
    let root = collection
        .deck_tree(Some(TimestampSecs::now()))
        .map_err(|source| DeckError::Load { source })?;

    Ok(root.children.into_iter().map(DeckNode::from).collect())
}

/// A deck by its full name like `A::B`, ignoring case like Anki.
pub fn find(collection: &Collection, name: &str) -> Result<Option<DeckId>, DeckError> {
    collection
        .get_deck_id(name)
        .map_err(|source| DeckError::Find { source })
}

/// Stored in the collection like Anki does, so it syncs to other devices.
pub fn set_collapsed(
    collection: &mut Collection,
    id: DeckId,
    collapsed: bool,
) -> Result<(), DeckError> {
    collection
        .set_deck_collapsed(id, collapsed, Scope::Reviewer)
        .map(|_| ())
        .map_err(|source| DeckError::SetCollapsed { source })
}

impl From<DeckTreeNode> for DeckNode {
    fn from(node: DeckTreeNode) -> Self {
        Self {
            id: DeckId(node.deck_id),
            name: node.name,
            collapsed: node.collapsed,
            filtered: node.filtered,
            new_count: node.new_count,
            learn_count: node.learn_count,
            review_count: node.review_count,
            children: node.children.into_iter().map(Self::from).collect(),
        }
    }
}
