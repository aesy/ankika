use anki::prelude::I18n;
use ankika_core::collection::{self, CollectionError};
use ankika_core::decks::{self, DeckError};
use ankika_core::due::{self, DueCounts, DueError, DueSnapshot};
use ankika_core::lock::{CollectionLock, LockError};
use ankika_core::paths::Paths;
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum DueCommandError {
    #[error(transparent)]
    Lock(LockError),
    #[error(transparent)]
    Collection(#[from] CollectionError),
    #[error(transparent)]
    Decks(#[from] DeckError),
    #[error(transparent)]
    Cache(#[from] DueError),
    #[error("ankika is open and hasn't saved any counts yet")]
    NoCounts,
    #[error("There is no deck \"{name}\"")]
    UnknownDeck { name: String },
}

/// Prints today's due count. While the TUI has the collection, uses what it last showed.
pub fn run(
    paths: &Paths,
    deck: Option<&str>,
    breakdown: bool,
    json: bool,
) -> Result<(), DueCommandError> {
    let snapshot = match CollectionLock::acquire(paths) {
        Ok(_lock) => {
            let mut collection = collection::open(paths, I18n::template_only())?;
            let snapshot = DueSnapshot::new(&decks::deck_tree(&mut collection)?);

            if let Err(error) = collection.close(None) {
                tracing::warn!(?error, "Failed to close the collection");
            }

            snapshot
        },
        Err(LockError::AlreadyLocked) => {
            due::read_cache(paths)?.ok_or(DueCommandError::NoCounts)?
        },
        Err(error) => return Err(DueCommandError::Lock(error)),
    };
    let counts = snapshot
        .counts(deck)
        .ok_or_else(|| DueCommandError::UnknownDeck {
            name: deck.unwrap_or_default().to_owned(),
        })?;

    println!("{}", format(counts, breakdown, json));

    Ok(())
}

/// In this field order, which `json!` would sort.
#[derive(Serialize)]
struct JsonCounts {
    new: u32,
    learning: u32,
    review: u32,
    total: u32,
}

fn format(counts: DueCounts, breakdown: bool, json: bool) -> String {
    if json {
        let output = JsonCounts {
            new: counts.new,
            learning: counts.learning,
            review: counts.review,
            total: counts.total(),
        };

        // Plain numbers always serialize
        serde_json::to_string(&output).unwrap_or_default()
    } else if breakdown {
        format!(
            "{} ({} new, {} learning, {} review)",
            counts.total(),
            counts.new,
            counts.learning,
            counts.review
        )
    } else {
        counts.total().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const COUNTS: DueCounts = DueCounts {
        new: 3,
        learning: 2,
        review: 7,
    };

    #[test]
    fn formats_each_style() {
        assert_eq!(format(COUNTS, false, false), "12");
        assert_eq!(
            format(COUNTS, true, false),
            "12 (3 new, 2 learning, 7 review)"
        );
        assert_eq!(
            format(COUNTS, false, true),
            r#"{"new":3,"learning":2,"review":7,"total":12}"#
        );
    }
}
