use anki::prelude::I18n;
use ankika_core::collection::{self, CollectionError};
use ankika_core::decks::{self, DeckError};
use ankika_core::lock::{CollectionLock, LockError};
use ankika_core::packages::{self, Contents, ExportScope, Format, PackageError, PackageKind};
use ankika_core::paths::Paths;

use crate::cli::ExportArgs;

#[derive(Debug, thiserror::Error)]
pub enum ExportCommandError {
    #[error("ankika is open; export from its palette instead")]
    AlreadyOpen,
    #[error(transparent)]
    Lock(LockError),
    #[error(transparent)]
    Collection(#[from] CollectionError),
    #[error(transparent)]
    Package(#[from] PackageError),
    #[error("There is no deck \"{name}\"")]
    UnknownDeck { name: String },
    #[error(transparent)]
    Deck(#[from] DeckError),
}

pub fn run(paths: &Paths, args: &ExportArgs, json: bool) -> Result<(), ExportCommandError> {
    let kind = packages::kind(&args.file)?;
    let _lock = CollectionLock::acquire(paths).map_err(|error| match error {
        LockError::AlreadyLocked => ExportCommandError::AlreadyOpen,
        error => ExportCommandError::Lock(error),
    })?;
    let mut collection = collection::open(paths, I18n::template_only())?;
    let format = if args.legacy {
        Format::Legacy
    } else {
        Format::Latest
    };

    match kind {
        PackageKind::Apkg => {
            let scope = match &args.deck {
                Some(name) => decks::find(&collection, name)?
                    .map(ExportScope::Deck)
                    .ok_or_else(|| ExportCommandError::UnknownDeck { name: name.clone() })?,
                None => ExportScope::Collection,
            };
            let contents = Contents {
                scheduling: !args.contents.no_scheduling,
                presets: args.contents.with_presets,
                media: !args.contents.no_media,
            };
            let notes = packages::export_apkg(&mut collection, &args.file, scope, contents, format);

            close(collection);

            let notes = notes?;

            if json {
                println!("{}", serde_json::json!({ "notes": notes }));
            } else {
                println!("Exported {notes} notes to {}", args.file.display());
            }
        },
        PackageKind::Colpkg => {
            let reopened =
                packages::export_colpkg(collection, &args.file, !args.contents.no_media, format);

            close(reopened.collection?);
            reopened.result?;

            if json {
                println!("{}", serde_json::json!({ "collection": true }));
            } else {
                println!("Exported the collection to {}", args.file.display());
            }
        },
    }

    Ok(())
}

fn close(collection: anki::collection::Collection) {
    if let Err(error) = collection.close(None) {
        tracing::warn!(?error, "Failed to close the collection");
    }
}
