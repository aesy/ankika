use std::str::FromStr;

use anki::browser_table::Column;
use anki::card::CardId;
use anki::collection::Collection;
use anki::config::{BoolKey, StringKey};
use anki::decks::DeckId;
use anki::error::AnkiError;
use anki::notes::NoteId;
use anki::search::SortMode;
use anki::services::{ConfigService, SearchService};
use anki_proto::config::SetConfigJsonRequest;
use anki_proto::generic::{String as ProtoString, StringList};
use anki_proto::scheduler::bury_or_suspend_cards_request::Mode;
use anki_proto::search::browser_columns::Sorting;
use anki_proto::search::browser_row::Color;

use crate::remote::english;

#[derive(Debug, thiserror::Error)]
pub enum BrowserError {
    #[error("Search failed: {}", english(source))]
    Search {
        #[source]
        source: AnkiError,
    },
    #[error("Failed to load the browser: {}", english(source))]
    Load {
        #[source]
        source: AnkiError,
    },
    #[error("Failed to change the cards: {}", english(source))]
    Change {
        #[source]
        source: AnkiError,
    },
    #[error("Failed to encode the browser setting {key}")]
    EncodeSetting {
        key: String,
        #[source]
        source: serde_json::Error,
    },
}

impl BrowserError {
    /// Anki's own error, which it can describe in the user's language.
    #[must_use]
    pub const fn anki_error(&self) -> Option<&AnkiError> {
        match self {
            Self::Search { source } | Self::Load { source } | Self::Change { source } => {
                Some(source)
            },
            Self::EncodeSetting { .. } => None,
        }
    }
}

/// Anki desktop's browser settings, stored in the collection so both stay in step.
const CARD_COLUMNS_KEY: &str = "activeCols";
const NOTE_COLUMNS_KEY: &str = "activeNoteCols";
const CARD_SORT_KEY: &str = "sortType";
const NOTE_SORT_KEY: &str = "noteSortType";
const CARD_BACKWARDS_KEY: &str = "sortBackwards";
const NOTE_BACKWARDS_KEY: &str = "browserNoteSortBackwards";
const DEFAULT_CARD_COLUMNS: [&str; 4] = ["noteFld", "template", "cardDue", "deck"];
const DEFAULT_NOTE_COLUMNS: [&str; 4] = ["noteFld", "note", "template", "noteTags"];
/// What Anki searches when the default search setting is empty.
const FALLBACK_SEARCH: &str = "deck:current";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnInfo {
    pub key: String,
    pub label: String,
    pub sortable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sort {
    pub column: String,
    pub reverse: bool,
}

/// What the table shows. Loading it also tells rslib which columns rows need.
pub struct Table {
    pub notes_mode: bool,
    pub columns: Vec<ColumnInfo>,
    pub sort: Option<Sort>,
}

/// The browser's row colors, in Anki's order of precedence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowColor {
    Default,
    Marked,
    Suspended,
    Buried,
    Flag(crate::card_actions::Flag),
}

pub struct Row {
    pub cells: Vec<String>,
    pub color: RowColor,
}

pub fn notes_mode(collection: &Collection) -> bool {
    collection.get_config_bool(BoolKey::BrowserTableShowNotesMode)
}

pub fn table(collection: &mut Collection, notes_mode: bool) -> Result<Table, BrowserError> {
    let load_error = |source| BrowserError::Load { source };
    let (columns_key, sort_key, backwards_key, defaults) = keys(notes_mode);
    let keys: Vec<String> = config(collection, columns_key)
        .unwrap_or_else(|| defaults.iter().map(|&key| key.to_owned()).collect());

    collection
        .set_active_browser_columns(StringList { vals: keys.clone() })
        .map_err(load_error)?;

    let all = collection
        .all_browser_columns()
        .map_err(load_error)?
        .columns;
    let columns = keys
        .iter()
        .filter_map(|key| {
            let column = all.iter().find(|column| column.key == *key)?;
            let (label, sorting) = if notes_mode {
                (&column.notes_mode_label, column.sorting_notes())
            } else {
                (&column.cards_mode_label, column.sorting_cards())
            };

            Some(ColumnInfo {
                key: key.clone(),
                label: label.clone(),
                sortable: sorting != Sorting::None,
            })
        })
        .collect();
    let sort = config::<String>(collection, sort_key).map(|column| Sort {
        column,
        reverse: config(collection, backwards_key).unwrap_or(false),
    });

    Ok(Table {
        notes_mode,
        columns,
        sort,
    })
}

pub fn set_notes_mode(collection: &mut Collection, notes_mode: bool) -> Result<(), BrowserError> {
    collection
        .set_config_bool(BoolKey::BrowserTableShowNotesMode, notes_mode, false)
        .map(|_| ())
        .map_err(|source| BrowserError::Load { source })
}

/// Remembered like in Anki desktop.
pub fn set_sort(
    collection: &mut Collection,
    notes_mode: bool,
    sort: &Sort,
) -> Result<(), BrowserError> {
    let (_, sort_key, backwards_key, _) = keys(notes_mode);

    set_config(collection, sort_key, &sort.column)?;
    set_config(collection, backwards_key, &sort.reverse)
}

pub fn default_search(collection: &Collection) -> String {
    let search = collection.get_config_string(StringKey::DefaultSearchText);

    if search.trim().is_empty() {
        FALLBACK_SEARCH.to_owned()
    } else {
        search
    }
}

/// Card ids, or note ids in notes mode.
pub fn search(
    collection: &mut Collection,
    query: &str,
    table: &Table,
) -> Result<Vec<i64>, BrowserError> {
    let order = table
        .sort
        .as_ref()
        .and_then(|sort| {
            Column::from_str(&sort.column)
                .ok()
                .map(|column| SortMode::Builtin {
                    column,
                    reverse: sort.reverse,
                })
        })
        .unwrap_or(SortMode::NoOrder);
    let error = |source| BrowserError::Search { source };

    if table.notes_mode {
        let ids = collection.search_notes(query, order).map_err(error)?;

        Ok(ids.into_iter().map(|id| id.0).collect())
    } else {
        let ids = collection.search_cards(query, order).map_err(error)?;

        Ok(ids.into_iter().map(|id| id.0).collect())
    }
}

pub fn row(collection: &mut Collection, id: i64) -> Result<Row, BrowserError> {
    let row = collection
        .browser_row_for_id(id)
        .map_err(|source| BrowserError::Load { source })?;
    let color = match row.color() {
        Color::Default => RowColor::Default,
        Color::Marked => RowColor::Marked,
        Color::Suspended => RowColor::Suspended,
        Color::Buried => RowColor::Buried,
        flag => crate::card_actions::Flag::from_number(flag as u32 - Color::FlagRed as u32 + 1)
            .map_or(RowColor::Default, RowColor::Flag),
    };

    Ok(Row {
        cells: row.cells.into_iter().map(|cell| cell.text).collect(),
        color,
    })
}

/// The cards behind table ids: the ids themselves, or every card of the notes.
pub fn cards(
    collection: &mut Collection,
    notes_mode: bool,
    ids: &[i64],
) -> Result<Vec<CardId>, BrowserError> {
    if !notes_mode {
        return Ok(ids.iter().map(|&id| CardId(id)).collect());
    }

    collection
        .search_cards(format!("nid:{}", join(ids)).as_str(), SortMode::NoOrder)
        .map_err(|source| BrowserError::Load { source })
}

/// The notes behind table ids.
pub fn notes(
    collection: &mut Collection,
    notes_mode: bool,
    ids: &[i64],
) -> Result<Vec<NoteId>, BrowserError> {
    if notes_mode {
        return Ok(ids.iter().map(|&id| NoteId(id)).collect());
    }

    collection
        .search_notes(format!("cid:{}", join(ids)).as_str(), SortMode::NoOrder)
        .map_err(|source| BrowserError::Load { source })
}

pub fn is_suspended(collection: &mut Collection, card: CardId) -> Result<bool, BrowserError> {
    collection
        .search_cards(
            format!("cid:{card} is:suspended").as_str(),
            SortMode::NoOrder,
        )
        .map(|found| !found.is_empty())
        .map_err(|source| BrowserError::Load { source })
}

pub fn set_suspended(
    collection: &mut Collection,
    cards: &[CardId],
    suspended: bool,
) -> Result<(), BrowserError> {
    let result = if suspended {
        collection
            .bury_or_suspend_cards(cards, Mode::Suspend)
            .map(|_| ())
    } else {
        collection.unbury_or_unsuspend_cards(cards).map(|_| ())
    };

    result.map_err(|source| BrowserError::Change { source })
}

/// Number 0 removes the flag.
pub fn set_flag(
    collection: &mut Collection,
    cards: &[CardId],
    flag: u32,
) -> Result<(), BrowserError> {
    collection
        .set_card_flag(cards, flag)
        .map(|_| ())
        .map_err(|source| BrowserError::Change { source })
}

pub fn set_deck(
    collection: &mut Collection,
    cards: &[CardId],
    deck: DeckId,
) -> Result<(), BrowserError> {
    collection
        .set_deck(cards, deck)
        .map(|_| ())
        .map_err(|source| BrowserError::Change { source })
}

pub fn add_tags(
    collection: &mut Collection,
    notes: &[NoteId],
    tags: &[String],
) -> Result<(), BrowserError> {
    collection
        .add_tags_to_notes(notes, &tags.join(" "))
        .map(|_| ())
        .map_err(|source| BrowserError::Change { source })
}

pub fn remove_tags(
    collection: &mut Collection,
    notes: &[NoteId],
    tags: &[String],
) -> Result<(), BrowserError> {
    collection
        .remove_tags_from_notes(notes, &tags.join(" "))
        .map(|_| ())
        .map_err(|source| BrowserError::Change { source })
}

/// Returns how many cards went with them.
pub fn delete_notes(collection: &mut Collection, notes: &[NoteId]) -> Result<usize, BrowserError> {
    let ids: Vec<i64> = notes.iter().map(|note| note.0).collect();
    let cards = cards(collection, true, &ids)?.len();

    collection
        .remove_notes(notes)
        .map(|_| cards)
        .map_err(|source| BrowserError::Change { source })
}

const fn keys(notes_mode: bool) -> (&'static str, &'static str, &'static str, [&'static str; 4]) {
    if notes_mode {
        (
            NOTE_COLUMNS_KEY,
            NOTE_SORT_KEY,
            NOTE_BACKWARDS_KEY,
            DEFAULT_NOTE_COLUMNS,
        )
    } else {
        (
            CARD_COLUMNS_KEY,
            CARD_SORT_KEY,
            CARD_BACKWARDS_KEY,
            DEFAULT_CARD_COLUMNS,
        )
    }
}

/// `None` when the key is missing or holds something else.
fn config<T: serde::de::DeserializeOwned>(collection: &mut Collection, key: &str) -> Option<T> {
    let json = collection
        .get_config_json(ProtoString {
            val: key.to_owned(),
        })
        .ok()?;

    serde_json::from_slice(&json.json).ok()
}

fn set_config(
    collection: &mut Collection,
    key: &str,
    value: &impl serde::Serialize,
) -> Result<(), BrowserError> {
    let value_json = serde_json::to_vec(value).map_err(|source| BrowserError::EncodeSetting {
        key: key.to_owned(),
        source,
    })?;

    collection
        .set_config_json_no_undo(SetConfigJsonRequest {
            key: key.to_owned(),
            value_json,
            undoable: false,
        })
        .map_err(|source| BrowserError::Change { source })
}

fn join(ids: &[i64]) -> String {
    ids.iter().map(i64::to_string).collect::<Vec<_>>().join(",")
}
