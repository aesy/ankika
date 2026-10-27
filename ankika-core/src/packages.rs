use std::path::Path;

use anki::collection::Collection;
use anki::decks::DeckId;
use anki::error::AnkiError;
use anki::import_export::package::{
    ExportAnkiPackageOptions, ImportAnkiPackageOptions, UpdateCondition as AnkiUpdateCondition,
};
use anki::search::SearchNode;

use crate::collection::{self, CollectionError};
use crate::paths::Paths;
use crate::remote::english;

#[derive(Debug, thiserror::Error)]
pub enum PackageError {
    #[error("Failed to import {}: {}", path, english(source))]
    Import {
        path: String,
        #[source]
        source: AnkiError,
    },
    #[error("Failed to export to {}: {}", path, english(source))]
    Export {
        path: String,
        #[source]
        source: AnkiError,
    },
    #[error("Failed to close the collection: {}", english(source))]
    Close {
        #[source]
        source: AnkiError,
    },
    #[error("The path {path} isn't valid UTF-8, which Anki needs")]
    Path { path: String },
    #[error("{path} isn't an .apkg or .colpkg file")]
    UnknownKind { path: String },
}

impl PackageError {
    /// Anki's own error, which it can describe in the user's language.
    #[must_use]
    pub const fn anki_error(&self) -> Option<&AnkiError> {
        match self {
            Self::Import { source, .. } | Self::Export { source, .. } | Self::Close { source } => {
                Some(source)
            },
            Self::Path { .. } | Self::UnknownKind { .. } => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageKind {
    /// Decks, merged into the collection.
    Apkg,
    /// A whole collection, replacing this one.
    Colpkg,
}

/// By extension, like Anki.
pub fn kind(path: &Path) -> Result<PackageKind, PackageError> {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase);

    match extension.as_deref() {
        Some("apkg") => Ok(PackageKind::Apkg),
        Some("colpkg") => Ok(PackageKind::Colpkg),
        _ => Err(PackageError::UnknownKind {
            path: path.display().to_string(),
        }),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UpdateCondition {
    #[default]
    IfNewer,
    Always,
    Never,
}

/// Anki's `.apkg` import options, with Anki's defaults.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ImportOptions {
    pub with_scheduling: bool,
    pub with_presets: bool,
    pub merge_notetypes: bool,
    pub update_notes: UpdateCondition,
    pub update_notetypes: UpdateCondition,
}

/// Anki's import log, as counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ImportSummary {
    pub found: u32,
    pub new: usize,
    pub updated: usize,
    pub duplicate: usize,
    /// Conflicting, first field matches, missing note types or decks, empty first fields.
    pub skipped: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportScope {
    Collection,
    /// With its subdecks.
    Deck(DeckId),
}

/// What an `.apkg` export includes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Contents {
    pub scheduling: bool,
    pub presets: bool,
    pub media: bool,
}

impl Default for Contents {
    /// Anki's defaults.
    fn default() -> Self {
        Self {
            scheduling: true,
            presets: false,
            media: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Format {
    #[default]
    Latest,
    /// For Anki versions before 2.1.50.
    Legacy,
}

/// The collection reopened after an operation that closes it, and how the operation went.
pub struct Reopened<T> {
    pub collection: Result<Collection, CollectionError>,
    pub result: Result<T, PackageError>,
}

/// Adds the package's notes; undoable like in Anki.
pub fn import_apkg(
    collection: &mut Collection,
    path: &Path,
    options: ImportOptions,
) -> Result<ImportSummary, PackageError> {
    let anki_options = ImportAnkiPackageOptions {
        merge_notetypes: options.merge_notetypes,
        update_notes: update_condition(options.update_notes).into(),
        update_notetypes: update_condition(options.update_notetypes).into(),
        with_scheduling: options.with_scheduling,
        with_deck_configs: options.with_presets,
    };
    let log = collection
        .import_apkg(path, anki_options)
        .map_err(|source| PackageError::Import {
            path: path.display().to_string(),
            source,
        })?
        .output;

    Ok(ImportSummary {
        found: log.found_notes,
        new: log.new.len(),
        updated: log.updated.len(),
        duplicate: log.duplicate.len(),
        skipped: log.conflicting.len()
            + log.first_field_match.len()
            + log.missing_notetype.len()
            + log.missing_deck.len()
            + log.empty_first_field.len(),
    })
}

/// Returns how many notes were exported.
pub fn export_apkg(
    collection: &mut Collection,
    path: &Path,
    scope: ExportScope,
    contents: Contents,
    format: Format,
) -> Result<usize, PackageError> {
    let options = ExportAnkiPackageOptions {
        with_scheduling: contents.scheduling,
        with_deck_configs: contents.presets,
        with_media: contents.media,
        legacy: format == Format::Legacy,
    };
    let error = |source| PackageError::Export {
        path: path.display().to_string(),
        source,
    };

    match scope {
        ExportScope::Collection => collection.export_apkg(path, options, "", None),
        ExportScope::Deck(deck) => {
            collection.export_apkg(path, options, SearchNode::from_deck_id(deck, true), None)
        },
    }
    .map_err(error)
}

/// Writes the whole collection. rslib closes it to do that, so it comes back reopened.
pub fn export_colpkg(
    collection: Collection,
    path: &Path,
    media: bool,
    format: Format,
) -> Reopened<()> {
    let builder = collection.as_builder();
    let result = collection
        .export_colpkg(path, media, format == Format::Legacy)
        .map_err(|source| PackageError::Export {
            path: path.display().to_string(),
            source,
        });

    Reopened {
        collection: collection::reopen(builder),
        result,
    }
}

/// Replaces the collection and media with the package's. The next sync must be a full
/// sync, as with Anki.
pub fn import_colpkg(collection: Collection, paths: &Paths, path: &Path) -> Reopened<()> {
    let builder = collection.as_builder();
    // rslib keeps the handler's type private, so it's only created and passed on here
    let progress = collection.new_progress_handler();
    let import_error = |source| PackageError::Import {
        path: path.display().to_string(),
        source,
    };
    let result = collection
        .close(None)
        .map_err(|source| PackageError::Close { source })
        .and_then(|()| {
            let package = utf8(path)?;
            let target = utf8(&paths.collection_file())?;

            anki::import_export::package::import_colpkg(
                &package,
                &target,
                &paths.media_dir(),
                &paths.media_db_file(),
                progress,
            )
            .map_err(import_error)
        });

    Reopened {
        collection: collection::reopen(builder),
        result,
    }
}

fn utf8(path: &Path) -> Result<String, PackageError> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| PackageError::Path {
            path: path.display().to_string(),
        })
}

const fn update_condition(condition: UpdateCondition) -> AnkiUpdateCondition {
    match condition {
        UpdateCondition::IfNewer => AnkiUpdateCondition::IfNewer,
        UpdateCondition::Always => AnkiUpdateCondition::Always,
        UpdateCondition::Never => AnkiUpdateCondition::Never,
    }
}
