use std::fs;
use std::io;
use std::path::PathBuf;

use anki::collection::{Collection, CollectionBuilder};
use anki::error::AnkiError;
use anki::prelude::I18n;

use crate::paths::Paths;
use crate::remote::english;

#[derive(Debug, thiserror::Error)]
pub enum CollectionError {
    #[error("Failed to create directory {}", path.display())]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("Failed to open the collection at {}: {}", path.display(), english(source))]
    Open {
        path: PathBuf,
        #[source]
        source: AnkiError,
    },
    #[error(
        "Failed to reopen the collection after a full sync: {}",
        english(source)
    )]
    Reopen {
        #[source]
        source: AnkiError,
    },
}

/// Opens the collection, creating an empty one on first start.
pub fn open(paths: &Paths, tr: I18n) -> Result<Collection, CollectionError> {
    let media_dir = paths.media_dir();

    // Anki expects the media folder to exist
    fs::create_dir_all(&media_dir).map_err(|source| CollectionError::CreateDirectory {
        path: media_dir.clone(),
        source,
    })?;

    let path = paths.collection_file();

    CollectionBuilder::new(&path)
        .set_media_paths(media_dir, paths.media_db_file())
        .set_tr(tr)
        .build()
        .map_err(|source| CollectionError::Open { path, source })
}

/// Reopens a collection that a full sync consumed, using the builder taken before it.
pub fn reopen(mut builder: CollectionBuilder) -> Result<Collection, CollectionError> {
    builder
        .build()
        .map_err(|source| CollectionError::Reopen { source })
}
