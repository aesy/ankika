use anki::collection::Collection;
use anki::error::AnkiError;
use anki::services::{CardRenderingService, CardsService, CollectionService, MediaService};
use anki_proto::card_rendering::empty_cards_report::NoteWithEmptyCards;
use anki_proto::cards::RemoveCardsRequest;
use anki_proto::media::TrashMediaFilesRequest;

use crate::remote::english;

const NOTE_LINK: &str = "[anki:nid:";

#[derive(Debug, thiserror::Error)]
pub enum MaintenanceError {
    #[error("Failed to check the database: {}", english(source))]
    CheckDatabase {
        #[source]
        source: AnkiError,
    },
    #[error("Failed to check the media: {}", english(source))]
    CheckMedia {
        #[source]
        source: AnkiError,
    },
    #[error("Failed to change the media trash: {}", english(source))]
    Trash {
        #[source]
        source: AnkiError,
    },
    #[error("Failed to find empty cards: {}", english(source))]
    EmptyCards {
        #[source]
        source: AnkiError,
    },
    #[error("Failed to delete the empty cards: {}", english(source))]
    DeleteCards {
        #[source]
        source: AnkiError,
    },
}

impl MaintenanceError {
    /// Anki's own error, which it can describe in the user's language.
    #[must_use]
    pub const fn anki_error(&self) -> &AnkiError {
        match self {
            Self::CheckDatabase { source }
            | Self::CheckMedia { source }
            | Self::Trash { source }
            | Self::EmptyCards { source }
            | Self::DeleteCards { source } => source,
        }
    }
}

/// What Anki's Check Database found and fixed; empty when all was well, which Anki
/// reports as rebuilt and optimized.
pub fn check_database(collection: &mut Collection) -> Result<Vec<String>, MaintenanceError> {
    collection
        .check_database()
        .map(|response| response.problems)
        .map_err(|source| MaintenanceError::CheckDatabase { source })
}

pub struct MediaReport {
    /// Anki's report, in plain text.
    pub report: String,
    /// Files in the media folder that no note uses.
    pub unused: Vec<String>,
    pub missing: Vec<String>,
    pub have_trash: bool,
}

pub fn check_media(collection: &mut Collection) -> Result<MediaReport, MaintenanceError> {
    let response = collection
        .check_media()
        .map_err(|source| MaintenanceError::CheckMedia { source })?;

    Ok(MediaReport {
        report: response.report,
        unused: response.unused,
        missing: response.missing,
        have_trash: response.have_trash,
    })
}

/// Moves the files into Anki's media trash, from where they can be restored.
pub fn trash_files(
    collection: &mut Collection,
    files: Vec<String>,
) -> Result<(), MaintenanceError> {
    collection
        .trash_media_files(TrashMediaFilesRequest { fnames: files })
        .map_err(|source| MaintenanceError::Trash { source })
}

pub fn empty_trash(collection: &mut Collection) -> Result<(), MaintenanceError> {
    MediaService::empty_trash(collection).map_err(|source| MaintenanceError::Trash { source })
}

pub fn restore_trash(collection: &mut Collection) -> Result<(), MaintenanceError> {
    MediaService::restore_trash(collection).map_err(|source| MaintenanceError::Trash { source })
}

pub struct EmptyCards {
    /// Anki's report, in HTML, without the note links only Anki's window can follow.
    pub report: String,
    notes: Vec<NoteWithEmptyCards>,
}

impl EmptyCards {
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.notes.is_empty()
    }

    /// The cards to delete. Keeping notes leaves the first card of notes that would
    /// otherwise lose all of them, like Anki's checkbox.
    #[must_use]
    pub fn cards(&self, keep_notes: bool) -> Vec<i64> {
        self.notes
            .iter()
            .flat_map(|note| {
                let skip = usize::from(keep_notes && note.will_delete_note);

                note.card_ids.iter().skip(skip).copied()
            })
            .collect()
    }
}

pub fn empty_cards(collection: &mut Collection) -> Result<EmptyCards, MaintenanceError> {
    let report = collection
        .get_empty_cards()
        .map_err(|source| MaintenanceError::EmptyCards { source })?;

    Ok(EmptyCards {
        report: without_note_links(&report.report),
        notes: report.notes,
    })
}

/// Anki marks each line with `[anki:nid:<id>]` for its window to turn into a link.
fn without_note_links(report: &str) -> String {
    let mut text = String::with_capacity(report.len());
    let mut rest = report;

    while let Some(start) = rest.find(NOTE_LINK) {
        text.push_str(&rest[..start]);
        rest = &rest[start..];

        let end = rest.find(']').map_or(rest.len(), |end| end + 1);
        let after = &rest[end..];

        rest = after.strip_prefix(' ').unwrap_or(after);
    }

    text.push_str(rest);
    text
}

/// Deletes the cards, and notes left without any; undoable. Returns how many cards went.
pub fn delete_cards(
    collection: &mut Collection,
    cards: Vec<i64>,
) -> Result<usize, MaintenanceError> {
    let count = cards.len();

    CardsService::remove_cards(collection, RemoveCardsRequest { card_ids: cards })
        .map(|_| count)
        .map_err(|source| MaintenanceError::DeleteCards { source })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeping_notes_spares_their_first_card() {
        let empty = EmptyCards {
            report: String::new(),
            notes: vec![
                NoteWithEmptyCards {
                    note_id: 1,
                    card_ids: vec![10, 11],
                    will_delete_note: true,
                },
                NoteWithEmptyCards {
                    note_id: 2,
                    card_ids: vec![20],
                    will_delete_note: false,
                },
            ],
        };

        assert_eq!(empty.cards(true), vec![11, 20]);
        assert_eq!(empty.cards(false), vec![10, 11, 20]);
    }

    #[test]
    fn note_links_are_removed() {
        let report = "<ol><li class=>[anki:nid:12] 1 of 2</li><li>[anki:nid:3] 2 of 2</li></ol>";

        assert_eq!(
            without_note_links(report),
            "<ol><li class=>1 of 2</li><li>2 of 2</li></ol>"
        );
    }
}
