use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::decks::DeckNode;
use crate::paths::Paths;

#[derive(Debug, thiserror::Error)]
pub enum DueError {
    #[error("Failed to write {}", path.display())]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("Failed to read {}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("Failed to parse {}", path.display())]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
}

/// Cards due today, with Anki's daily limits applied.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DueCounts {
    pub new: u32,
    pub learning: u32,
    pub review: u32,
}

impl DueCounts {
    #[must_use]
    pub const fn total(self) -> u32 {
        self.new + self.learning + self.review
    }
}

/// Today's counts for the whole collection and each deck, by full name like `A::B`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DueSnapshot {
    pub total: DueCounts,
    pub decks: BTreeMap<String, DueCounts>,
}

impl DueSnapshot {
    #[must_use]
    pub fn new(decks: &[DeckNode]) -> Self {
        let mut snapshot = Self::default();

        for deck in decks {
            let counts = counts(deck);

            snapshot.total.new += counts.new;
            snapshot.total.learning += counts.learning;
            snapshot.total.review += counts.review;
            add_decks(&mut snapshot.decks, deck, "");
        }

        snapshot
    }

    /// The whole collection, or one deck matched like Anki matches names, ignoring case.
    #[must_use]
    pub fn counts(&self, deck: Option<&str>) -> Option<DueCounts> {
        let Some(deck) = deck else {
            return Some(self.total);
        };

        self.decks
            .iter()
            .find(|(name, _)| name.to_lowercase() == deck.to_lowercase())
            .map(|(_, counts)| *counts)
    }
}

/// What the TUI last showed, for `ankika due` while the TUI holds the collection.
pub fn write_cache(paths: &Paths, snapshot: &DueSnapshot) -> Result<(), DueError> {
    let path = paths.due_cache_file();
    let write_error = |source| DueError::Write {
        path: path.clone(),
        source,
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(write_error)?;
    }

    // Serializing plain counts can't fail
    let json = serde_json::to_vec(snapshot).unwrap_or_default();

    fs::write(&path, json).map_err(write_error)
}

pub fn read_cache(paths: &Paths) -> Result<Option<DueSnapshot>, DueError> {
    let path = paths.due_cache_file();
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(DueError::Read { path, source }),
    };

    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|source| DueError::Parse { path, source })
}

const fn counts(deck: &DeckNode) -> DueCounts {
    DueCounts {
        new: deck.new_count,
        learning: deck.learn_count,
        review: deck.review_count,
    }
}

fn add_decks(decks: &mut BTreeMap<String, DueCounts>, deck: &DeckNode, parent: &str) {
    let name = if parent.is_empty() {
        deck.name.clone()
    } else {
        format!("{parent}::{}", deck.name)
    };

    for child in &deck.children {
        add_decks(decks, child, &name);
    }

    decks.insert(name, counts(deck));
}

#[cfg(test)]
mod tests {
    use anki::decks::DeckId;

    use super::*;

    fn deck(name: &str, new: u32, children: Vec<DeckNode>) -> DeckNode {
        DeckNode {
            id: DeckId(1),
            name: name.to_owned(),
            collapsed: false,
            filtered: false,
            new_count: new,
            learn_count: 0,
            review_count: 1,
            children,
        }
    }

    #[test]
    fn totals_top_level_decks_and_names_children() {
        let snapshot = DueSnapshot::new(&[
            deck("A", 5, vec![deck("B", 2, vec![])]),
            deck("C", 1, vec![]),
        ]);

        assert_eq!(snapshot.total.total(), 8);
        assert_eq!(snapshot.counts(Some("a::b")).map(DueCounts::total), Some(3));
        assert_eq!(snapshot.counts(Some("nope")), None);
    }
}
