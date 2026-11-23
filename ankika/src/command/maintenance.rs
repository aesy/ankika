use anki::collection::Collection;
use anki::prelude::I18n;
use ankika_core::collection::{self, CollectionError};
use ankika_core::lock::{CollectionLock, LockError};
use ankika_core::maintenance::{self, MaintenanceError};
use ankika_core::paths::Paths;
use serde::Serialize;

use crate::cli::{CheckMediaArgs, EmptyCardsArgs};

const REPORT_WIDTH: usize = 100;

#[derive(Debug, thiserror::Error)]
pub enum MaintenanceCommandError {
    #[error("ankika is open; use its palette instead")]
    AlreadyOpen,
    #[error(transparent)]
    Lock(LockError),
    #[error(transparent)]
    Collection(#[from] CollectionError),
    #[error(transparent)]
    Maintenance(#[from] MaintenanceError),
    #[error("Failed to turn Anki's report into text")]
    RenderReport(#[source] html2text::Error),
}

pub fn check_database(paths: &Paths, json: bool) -> Result<(), MaintenanceCommandError> {
    let (_lock, mut collection) = open(paths)?;
    let problems = maintenance::check_database(&mut collection);

    close(collection);

    let problems = problems?;

    if json {
        println!("{}", serde_json::json!({ "problems": problems }));
    } else if problems.is_empty() {
        println!("Database rebuilt and optimized");
    } else {
        for problem in problems {
            println!("{problem}");
        }
    }

    Ok(())
}

/// In this field order, which `json!` would sort.
#[derive(Serialize)]
struct MediaOutput {
    missing: Vec<String>,
    unused: Vec<String>,
    trashed: usize,
    trash_emptied: bool,
    trash_restored: bool,
}

pub fn check_media(
    paths: &Paths,
    args: &CheckMediaArgs,
    json: bool,
) -> Result<(), MaintenanceCommandError> {
    let (_lock, mut collection) = open(paths)?;
    let result = check_and_clean_media(&mut collection, args);

    close(collection);

    let (report, output) = result?;

    if json {
        // Strings, numbers and bools always serialize
        println!("{}", serde_json::to_string(&output).unwrap_or_default());
        return Ok(());
    }

    println!("{}", report.trim_end());

    if output.trashed > 0 {
        println!("Moved {} unused files to the media trash", output.trashed);
    }

    if output.trash_restored {
        println!("Restored the files in the media trash");
    }

    if output.trash_emptied {
        println!("Emptied the media trash");
    }

    Ok(())
}

/// Trashes before emptying, so `--delete-unused --empty-trash` deletes them for good.
fn check_and_clean_media(
    collection: &mut Collection,
    args: &CheckMediaArgs,
) -> Result<(String, MediaOutput), MaintenanceError> {
    let report = maintenance::check_media(collection)?;
    let trashed = if args.delete_unused {
        let count = report.unused.len();

        maintenance::trash_files(collection, report.unused.clone())?;
        count
    } else {
        0
    };

    if args.restore_trash {
        maintenance::restore_trash(collection)?;
    }

    if args.empty_trash {
        maintenance::empty_trash(collection)?;
    }

    let output = MediaOutput {
        missing: report.missing,
        unused: report.unused,
        trashed,
        trash_emptied: args.empty_trash,
        trash_restored: args.restore_trash,
    };

    Ok((report.report, output))
}

pub fn empty_cards(
    paths: &Paths,
    args: &EmptyCardsArgs,
    json: bool,
) -> Result<(), MaintenanceCommandError> {
    let (_lock, mut collection) = open(paths)?;
    let result = find_and_delete_empty(&mut collection, args);

    close(collection);

    let (report, cards) = result?;

    if json {
        println!(
            "{}",
            serde_json::json!({ "cards": cards, "deleted": args.delete && cards > 0 })
        );
        return Ok(());
    }

    let Some(report) = report else {
        println!("No empty cards");
        return Ok(());
    };

    println!("{}", report.trim_end());

    if args.delete {
        println!("Deleted {cards} cards");
    }

    Ok(())
}

/// The report as text, `None` when there are no empty cards, and how many cards there are
/// to delete.
fn find_and_delete_empty(
    collection: &mut Collection,
    args: &EmptyCardsArgs,
) -> Result<(Option<String>, usize), MaintenanceCommandError> {
    let empty = maintenance::empty_cards(collection)?;

    if empty.is_empty() {
        return Ok((None, 0));
    }

    let report = html2text::config::plain_no_decorate()
        .string_from_read(empty.report.as_bytes(), REPORT_WIDTH)
        .map_err(MaintenanceCommandError::RenderReport)?;
    let cards = empty.cards(!args.no_keep_notes);
    let count = cards.len();

    if args.delete {
        maintenance::delete_cards(collection, cards)?;
    }

    Ok((Some(report), count))
}

fn open(paths: &Paths) -> Result<(CollectionLock, Collection), MaintenanceCommandError> {
    let lock = CollectionLock::acquire(paths).map_err(|error| match error {
        LockError::AlreadyLocked => MaintenanceCommandError::AlreadyOpen,
        error => MaintenanceCommandError::Lock(error),
    })?;
    let collection = collection::open(paths, I18n::template_only())?;

    Ok((lock, collection))
}

fn close(collection: Collection) {
    if let Err(error) = collection.close(None) {
        tracing::warn!(?error, "Failed to close the collection");
    }
}
