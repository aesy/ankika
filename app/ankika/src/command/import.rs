use std::io::{self, BufRead, Write};
use std::path::Path;

use anki::prelude::I18n;
use ankika_core::collection::{self, CollectionError};
use ankika_core::lock::{CollectionLock, LockError};
use ankika_core::packages::{self, ImportOptions, ImportSummary, PackageError, PackageKind};
use ankika_core::paths::Paths;

use crate::cli::ImportArgs;

#[derive(Debug, thiserror::Error)]
pub enum ImportCommandError {
    #[error("ankika is open; import from its palette instead")]
    AlreadyOpen,
    #[error(transparent)]
    Lock(LockError),
    #[error(transparent)]
    Collection(#[from] CollectionError),
    #[error(transparent)]
    Package(#[from] PackageError),
    #[error("Failed to read the answer")]
    ReadAnswer(#[source] io::Error),
}

pub fn run(paths: &Paths, args: &ImportArgs, json: bool) -> Result<(), ImportCommandError> {
    let kind = packages::kind(&args.file)?;
    let _lock = CollectionLock::acquire(paths).map_err(|error| match error {
        LockError::AlreadyLocked => ImportCommandError::AlreadyOpen,
        error => ImportCommandError::Lock(error),
    })?;
    let collection = collection::open(paths, I18n::template_only())?;

    match kind {
        PackageKind::Apkg => {
            let mut collection = collection;
            let flags = &args.options;
            let options = ImportOptions {
                with_scheduling: flags.with_scheduling,
                with_presets: flags.with_presets,
                merge_notetypes: flags.merge_notetypes,
                update_notes: flags.update_notes.into(),
                update_notetypes: flags.update_notetypes.into(),
            };
            let summary = packages::import_apkg(&mut collection, &args.file, options);

            close(collection);
            print_summary(&summary?, json);
        },
        PackageKind::Colpkg => {
            if !args.yes && !confirm(&args.file).map_err(ImportCommandError::ReadAnswer)? {
                close(collection);
                println!("Kept the collection");
                return Ok(());
            }

            let reopened = packages::import_colpkg(collection, paths, &args.file);

            close(reopened.collection?);
            reopened.result?;

            if json {
                println!("{}", serde_json::json!({ "replaced": true }));
            } else {
                println!("Replaced the collection; the next sync will be a full sync");
            }
        },
    }

    Ok(())
}

fn print_summary(summary: &ImportSummary, json: bool) {
    if json {
        let output = serde_json::json!({
            "found": summary.found,
            "new": summary.new,
            "updated": summary.updated,
            "duplicate": summary.duplicate,
            "skipped": summary.skipped,
        });

        println!("{output}");
    } else {
        println!(
            "Imported {} notes: {} new, {} updated, {} duplicate, {} skipped",
            summary.found, summary.new, summary.updated, summary.duplicate, summary.skipped
        );
    }
}

fn confirm(file: &Path) -> io::Result<bool> {
    print!(
        "Replace this profile's collection and media with {}? [y/N] ",
        file.display()
    );
    io::stdout().flush()?;

    let mut answer = String::new();
    io::stdin().lock().read_line(&mut answer)?;

    Ok(matches!(answer.trim(), "y" | "Y" | "yes"))
}

fn close(collection: anki::collection::Collection) {
    if let Err(error) = collection.close(None) {
        tracing::warn!(?error, "Failed to close the collection");
    }
}
