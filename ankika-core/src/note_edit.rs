use anki::collection::Collection;
use anki::decks::DeckId;
use anki::error::AnkiError;
use anki::notes::{Note, NoteId};
use anki::notetype::NotetypeId;
use anki_proto::notes::note_fields_check_response::State as NoteFieldsState;

use crate::remote::english;

#[derive(Debug, thiserror::Error)]
pub enum NoteEditError {
    #[error("Failed to load the note: {}", english(source))]
    Load {
        #[source]
        source: AnkiError,
    },
    #[error("The note no longer exists")]
    Missing,
    #[error("Expected the header \"{header}\" for the next field")]
    MissingHeader { header: String },
    #[error("Failed to save the note: {}", english(source))]
    Save {
        #[source]
        source: AnkiError,
    },
    #[error("The first field is empty")]
    FirstFieldEmpty,
    #[error("The cloze note type has no cloze deletion")]
    MissingCloze,
    #[error("Cloze deletions only work on cloze note types")]
    ClozeOutsideClozeNotetype,
    #[error("Cloze deletions only work in cloze fields")]
    ClozeOutsideClozeField,
}

impl NoteEditError {
    /// Anki's own error, which it can describe in the user's language.
    #[must_use]
    pub const fn anki_error(&self) -> Option<&AnkiError> {
        match self {
            Self::Load { source } | Self::Save { source } => Some(source),
            Self::Missing
            | Self::MissingHeader { .. }
            | Self::FirstFieldEmpty
            | Self::MissingCloze
            | Self::ClozeOutsideClozeNotetype
            | Self::ClozeOutsideClozeField => None,
        }
    }
}

/// A note's fields as editable text: each field's raw HTML under a `# Name` line.
pub struct NoteText {
    note: Note,
    names: Vec<String>,
}

/// A note that was just added.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Added {
    pub cards: usize,
    /// Anki adds duplicates of the first field, but warns about them.
    pub duplicate: bool,
}

impl NoteText {
    /// An empty note of the type, for adding.
    pub fn new_note(
        collection: &mut Collection,
        notetype: NotetypeId,
    ) -> Result<Self, NoteEditError> {
        let notetype = collection
            .get_notetype(notetype)
            .map_err(|source| NoteEditError::Load { source })?
            .ok_or(NoteEditError::Missing)?;
        let names = notetype
            .fields
            .iter()
            .map(|field| field.name.clone())
            .collect();

        Ok(Self {
            note: notetype.new_note(),
            names,
        })
    }

    pub fn load(collection: &mut Collection, note: NoteId) -> Result<Self, NoteEditError> {
        let note = collection
            .storage
            .get_note(note)
            .map_err(|source| NoteEditError::Load { source })?
            .ok_or(NoteEditError::Missing)?;
        let notetype = collection
            .get_notetype(note.notetype_id)
            .map_err(|source| NoteEditError::Load { source })?
            .ok_or(NoteEditError::Missing)?;
        let names = notetype
            .fields
            .iter()
            .map(|field| field.name.clone())
            .collect();

        Ok(Self { note, names })
    }

    #[must_use]
    pub const fn note_id(&self) -> NoteId {
        self.note.id
    }

    #[must_use]
    pub fn text(&self) -> String {
        self.names
            .iter()
            .zip(self.note.fields())
            .map(|(name, value)| format!("{}\n{value}\n", header(name)))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Saves the edited text as one undoable change. Returns whether anything changed.
    pub fn save(mut self, collection: &mut Collection, text: &str) -> Result<bool, NoteEditError> {
        let fields = parse(text, &self.names)?;

        if fields == *self.note.fields() {
            return Ok(false);
        }

        for (index, value) in fields.into_iter().enumerate() {
            self.note
                .set_field(index, value)
                .map_err(|source| NoteEditError::Save { source })?;
        }

        collection
            .update_note(&mut self.note)
            .map_err(|source| NoteEditError::Save { source })?;

        Ok(true)
    }
}

impl NoteText {
    /// Adds the note with Anki's checks: an empty first field and misplaced or missing
    /// clozes are refused.
    pub fn add(
        mut self,
        collection: &mut Collection,
        deck: DeckId,
        tags: Vec<String>,
        text: &str,
    ) -> Result<Added, NoteEditError> {
        let save_error = |source| NoteEditError::Save { source };

        for (index, value) in parse(text, &self.names)?.into_iter().enumerate() {
            self.note.set_field(index, value).map_err(save_error)?;
        }

        self.note.tags = tags;

        let duplicate = match collection
            .note_fields_check(&self.note)
            .map_err(save_error)?
        {
            NoteFieldsState::Normal => false,
            NoteFieldsState::Duplicate => true,
            NoteFieldsState::Empty => return Err(NoteEditError::FirstFieldEmpty),
            NoteFieldsState::MissingCloze => return Err(NoteEditError::MissingCloze),
            NoteFieldsState::NotetypeNotCloze => {
                return Err(NoteEditError::ClozeOutsideClozeNotetype);
            },
            NoteFieldsState::FieldNotCloze => return Err(NoteEditError::ClozeOutsideClozeField),
        };
        let cards = collection
            .add_note(&mut self.note, deck)
            .map_err(save_error)?
            .output;

        Ok(Added { cards, duplicate })
    }
}

fn header(name: &str) -> String {
    format!("# {name}")
}

/// Fields are found by their headers in order, so a `# ` line that isn't the next
/// field's header stays part of the field.
fn parse(text: &str, names: &[String]) -> Result<Vec<String>, NoteEditError> {
    let mut lines = text.lines().peekable();
    let mut fields = Vec::with_capacity(names.len());

    for (index, name) in names.iter().enumerate() {
        let expected = header(name);

        if lines.next().map(str::trim_end) != Some(expected.as_str()) {
            return Err(NoteEditError::MissingHeader { header: expected });
        }

        let next = names.get(index + 1).map(|name| header(name));
        let mut value = Vec::new();

        while let Some(line) = lines.peek() {
            if next.as_deref() == Some(line.trim_end()) {
                break;
            }

            value.push(*line);
            lines.next();
        }

        // The blank line `text` puts between fields
        if next.is_some() && value.last() == Some(&"") {
            value.pop();
        }

        fields.push(value.join("\n"));
    }

    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names() -> Vec<String> {
        vec!["Front".to_owned(), "Back".to_owned()]
    }

    #[test]
    fn parses_fields_in_order() {
        let text = "# Front\nhello<br>\nworld\n\n# Back\n# not a header\nbye\n";

        assert_eq!(
            parse(text, &names()).unwrap(),
            vec!["hello<br>\nworld", "# not a header\nbye"]
        );
    }

    #[test]
    fn keeps_empty_fields() {
        assert_eq!(
            parse("# Front\n\n# Back\n", &names()).unwrap(),
            vec!["", ""]
        );
    }

    #[test]
    fn rejects_missing_headers() {
        assert!(matches!(
            parse("# Front\nx\n", &names()),
            Err(NoteEditError::MissingHeader { .. })
        ));
    }
}
