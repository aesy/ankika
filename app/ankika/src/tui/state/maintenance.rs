use anki::collection::Collection;
use ankika_core::maintenance::MaintenanceError;

use super::{Busy, Overlay, State};
use crate::i18n::I18n;
use crate::tui::report::{Report, ReportButton, ReportKind};
use crate::tui::tasks::{MaintenanceDone, MaintenanceJob, MediaChange, MediaChanged};

impl State {
    /// Hands the collection to the job, like an import; the report opens when it's done.
    pub(super) fn start_maintenance(&mut self, job: MaintenanceJob, busy: Busy) {
        let Some(collection) = self.collection.take() else {
            return;
        };

        self.overlay = Overlay::None;
        self.busy = Some(busy);
        self.tasks.maintenance(collection, job);
    }

    pub(super) fn press_report_button(&mut self, button: ReportButton) {
        let Overlay::Report(report) = &mut self.overlay else {
            return;
        };
        let job = match (button, report.kind()) {
            (ReportButton::KeepNotes, _) => {
                report.toggle_keep_notes();
                return;
            },
            (ReportButton::DeleteUnused, ReportKind::Media { unused }) => {
                MaintenanceJob::ChangeMedia(MediaChange::Trash(unused.clone()))
            },
            (ReportButton::EmptyTrash, _) => MaintenanceJob::ChangeMedia(MediaChange::EmptyTrash),
            (ReportButton::RestoreTrash, _) => {
                MaintenanceJob::ChangeMedia(MediaChange::RestoreTrash)
            },
            (ReportButton::Delete, ReportKind::EmptyCards { empty, keep_notes }) => {
                let job = MaintenanceJob::DeleteCards(empty.cards(*keep_notes));

                self.start_maintenance(job, Busy::Deleting);
                return;
            },
            (ReportButton::Close | ReportButton::DeleteUnused | ReportButton::Delete, _) => {
                self.overlay = Overlay::None;
                return;
            },
        };

        self.start_maintenance(job, Busy::Checking);
    }

    pub(super) fn finish_maintenance(
        &mut self,
        collection: Collection,
        result: Result<MaintenanceDone, MaintenanceError>,
        i18n: &I18n,
    ) {
        let tr = i18n.anki();

        self.busy = None;
        self.collection = Some(collection);

        match result {
            Ok(MaintenanceDone::Database(problems)) => {
                self.overlay = Overlay::Report(Report::database(&problems, i18n));
            },
            Ok(MaintenanceDone::Media { report, changed }) => {
                self.overlay = Overlay::Report(Report::media(report));
                self.notice = changed.map(|changed| match changed {
                    MediaChanged::Trashed(count) => tr
                        .media_check_delete_unused_complete(
                            u32::try_from(count).unwrap_or(u32::MAX),
                        )
                        .into_owned(),
                    MediaChanged::TrashEmptied => tr.media_check_trash_emptied().into_owned(),
                    MediaChanged::TrashRestored => tr.media_check_trash_restored().into_owned(),
                });
            },
            Ok(MaintenanceDone::EmptyCards(empty)) => {
                self.overlay = Overlay::Report(Report::empty_cards(empty, i18n));
            },
            Ok(MaintenanceDone::DeletedCards(count)) => {
                self.notice = Some(
                    tr.empty_cards_deleted_count(u32::try_from(count).unwrap_or(u32::MAX))
                        .into_owned(),
                );
            },
            Err(error) => {
                tracing::warn!(?error, "Maintenance failed");
                self.notice = Some(error.anki_error().message(tr));
            },
        }

        self.reload_decks(i18n);
        self.refresh_deck_screen(i18n);
    }
}
