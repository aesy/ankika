use std::path::PathBuf;

use anki::collection::Collection;
use ankika_core::adding;
use ankika_core::collection::CollectionError;
use ankika_core::packages::{self, PackageError, PackageKind};

use super::{Busy, Overlay, State};
use crate::i18n::I18n;
use crate::tui::error_text;
use crate::tui::form::Form;
use crate::tui::overview::DeckScreen;
use crate::tui::package_forms;
use crate::tui::replace_prompt::{ReplaceChoice, ReplacePrompt};
use crate::tui::tasks::{PackageDone, PackageJob};

impl State {
    pub(super) fn open_import(&mut self, i18n: &I18n) {
        self.overlay = Overlay::Import(package_forms::import(i18n));
    }

    pub(super) fn open_export(&mut self, i18n: &I18n) {
        let Some(collection) = &mut self.collection else {
            return;
        };

        match adding::normal_decks(collection) {
            Ok(decks) => {
                let current = self.deck_screen.as_ref().map(DeckScreen::deck);
                let form = package_forms::export(&decks, current, i18n);

                self.overlay = Overlay::Export { form, decks };
            },
            Err(error) => self.notice = Some(error.anki_error().message(i18n.anki())),
        }
    }

    /// An `.apkg` imports straight away; a `.colpkg` asks first, since it replaces
    /// everything.
    pub(super) fn submit_import(&mut self, mut form: Form, i18n: &I18n) {
        let Some((file, kind)) = file_and_kind(&form) else {
            form.set_error(i18n.text("package-invalid-file"));
            self.overlay = Overlay::Import(form);
            return;
        };

        match kind {
            PackageKind::Apkg => {
                let options = package_forms::import_options(&form);

                self.start_package(PackageJob::ImportApkg { file, options }, Busy::Importing);
            },
            PackageKind::Colpkg => {
                self.overlay = Overlay::ConfirmReplace(ReplacePrompt::new(file));
            },
        }
    }

    pub(super) fn choose_replace(&mut self, choice: ReplaceChoice) {
        let Overlay::ConfirmReplace(prompt) = std::mem::replace(&mut self.overlay, Overlay::None)
        else {
            return;
        };

        if choice == ReplaceChoice::Replace {
            let job = PackageJob::ImportColpkg { file: prompt.file };

            self.start_package(job, Busy::Importing);
        }
    }

    pub(super) fn submit_export(
        &mut self,
        mut form: Form,
        decks: Vec<(anki::decks::DeckId, String)>,
        i18n: &I18n,
    ) {
        let Some((file, kind)) = file_and_kind(&form) else {
            form.set_error(i18n.text("package-invalid-file"));
            self.overlay = Overlay::Export { form, decks };
            return;
        };
        let (contents, format) = package_forms::export_contents(&form);
        let job = match kind {
            PackageKind::Apkg => PackageJob::ExportApkg {
                file,
                scope: package_forms::export_scope(&form, &decks),
                contents,
                format,
            },
            PackageKind::Colpkg => PackageJob::ExportColpkg {
                file,
                media: contents.media,
                format,
            },
        };

        self.start_package(job, Busy::Exporting);
    }

    /// Hands the collection to the job; studying and syncing wait meanwhile, like during
    /// a sync.
    fn start_package(&mut self, job: PackageJob, busy: Busy) {
        let Some(collection) = self.collection.take() else {
            return;
        };

        self.busy = Some(busy);
        self.tasks.package(collection, self.paths.clone(), job);
    }

    pub(super) fn finish_package(
        &mut self,
        collection: Result<Collection, CollectionError>,
        result: Result<PackageDone, PackageError>,
        i18n: &I18n,
    ) {
        self.busy = None;

        match collection {
            Ok(collection) => self.collection = Some(collection),
            Err(error) => {
                tracing::error!(
                    ?error,
                    "Failed to reopen the collection after a package job"
                );
                self.lost_collection = Some(error);
                self.quit_now();
                return;
            },
        }

        let tr = i18n.anki();

        self.notice = Some(match result {
            Ok(PackageDone::Imported(summary)) => i18n.text_with(
                "package-import-summary",
                &[
                    ("found", &summary.found.to_string()),
                    ("new", &summary.new.to_string()),
                    ("updated", &summary.updated.to_string()),
                    ("duplicate", &summary.duplicate.to_string()),
                    ("skipped", &summary.skipped.to_string()),
                ],
            ),
            Ok(PackageDone::Replaced) => i18n.text("package-replaced"),
            Ok(PackageDone::ExportedNotes(notes)) => tr.exporting_note_exported(notes).into_owned(),
            Ok(PackageDone::ExportedCollection) => tr.exporting_collection_exported().into_owned(),
            Err(error) => {
                tracing::warn!(?error, "Import or export failed");
                error_text::package(&error, i18n)
            },
        });

        self.reload_decks(i18n);
        self.refresh_deck_screen(i18n);
    }
}

/// The typed file and what it is.
fn file_and_kind(form: &Form) -> Option<(PathBuf, PackageKind)> {
    let file = package_forms::file(form)?;
    let kind = packages::kind(&file).ok()?;

    Some((file, kind))
}
