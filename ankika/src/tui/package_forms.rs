use std::path::PathBuf;

use anki::decks::DeckId;
use ankika_core::packages::{Contents, ExportScope, Format, ImportOptions, UpdateCondition};

use crate::i18n::I18n;
use crate::tui::form::{Field, Form, Value};

const CONDITIONS: [UpdateCondition; 3] = [
    UpdateCondition::IfNewer,
    UpdateCondition::Always,
    UpdateCondition::Never,
];

// Both forms start with the file
const FILE: usize = 0;

// The other fields' positions in the import form
const WITH_SCHEDULING: usize = 1;
const WITH_PRESETS: usize = 2;
const MERGE_NOTETYPES: usize = 3;
const UPDATE_NOTES: usize = 4;
const UPDATE_NOTETYPES: usize = 5;

// The other fields' positions in the export form
const DECK: usize = 1;
const SCHEDULING: usize = 2;
const MEDIA: usize = 3;
const PRESETS: usize = 4;
const LEGACY: usize = 5;

/// Anki's `.apkg` import options with Anki's defaults; `.colpkg` files ignore them.
pub fn import(i18n: &I18n) -> Form {
    let tr = i18n.anki();
    let defaults = ImportOptions::default();
    let conditions = || Value::Choice {
        options: vec![
            tr.importing_update_if_newer().into_owned(),
            tr.importing_update_always().into_owned(),
            tr.importing_update_never().into_owned(),
        ],
        selected: 0,
    };
    let fields = vec![
        file_field(i18n),
        toggle(
            tr.importing_include_reviews().into_owned(),
            defaults.with_scheduling,
        ),
        toggle(
            tr.importing_with_deck_configs().into_owned(),
            defaults.with_presets,
        ),
        toggle(
            tr.importing_merge_notetypes().into_owned(),
            defaults.merge_notetypes,
        ),
        Field {
            label: tr.importing_update_notes().into_owned(),
            value: conditions(),
        },
        Field {
            label: tr.importing_update_notetypes().into_owned(),
            value: conditions(),
        },
    ];
    let mut form = Form::new(tr.actions_import().into_owned(), fields);

    form.note = Some(i18n.text("package-import-note"));
    form.submit = Some(tr.actions_import().into_owned());
    form
}

/// The decks to offer, with the one to start on; `None` means the whole collection.
pub fn export(decks: &[(DeckId, String)], current: Option<DeckId>, i18n: &I18n) -> Form {
    let tr = i18n.anki();
    let defaults = Contents::default();
    let mut deck_names = vec![tr.exporting_all_decks().into_owned()];
    deck_names.extend(decks.iter().map(|(_, name)| name.clone()));
    let selected = current
        .and_then(|current| decks.iter().position(|(id, _)| *id == current))
        .map_or(0, |index| index + 1);
    let fields = vec![
        file_field(i18n),
        Field {
            label: tr.decks_deck().into_owned(),
            value: Value::Choice {
                options: deck_names,
                selected,
            },
        },
        toggle(
            tr.exporting_include_scheduling_information().into_owned(),
            defaults.scheduling,
        ),
        toggle(tr.exporting_include_media().into_owned(), defaults.media),
        toggle(
            tr.exporting_include_deck_configs().into_owned(),
            defaults.presets,
        ),
        toggle(
            tr.exporting_support_older_anki_versions().into_owned(),
            false,
        ),
    ];
    let mut form = Form::new(tr.actions_export().into_owned(), fields);

    form.note = Some(i18n.text("package-export-note"));
    form.submit = Some(tr.actions_export().into_owned());
    form
}

/// The file typed into either form, with a leading `~` meaning the home directory.
pub fn file(form: &Form) -> Option<PathBuf> {
    let text = form.text(FILE).trim();

    if text.is_empty() {
        return None;
    }

    let home = || std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"));

    let expanded = text.strip_prefix("~/").map_or_else(
        || PathBuf::from(text),
        |rest| {
            home().map_or_else(
                || PathBuf::from(text),
                |home| PathBuf::from(home).join(rest),
            )
        },
    );

    Some(expanded)
}

pub fn import_options(form: &Form) -> ImportOptions {
    ImportOptions {
        with_scheduling: form.toggle(WITH_SCHEDULING),
        with_presets: form.toggle(WITH_PRESETS),
        merge_notetypes: form.toggle(MERGE_NOTETYPES),
        update_notes: CONDITIONS[form.choice(UPDATE_NOTES).min(CONDITIONS.len() - 1)],
        update_notetypes: CONDITIONS[form.choice(UPDATE_NOTETYPES).min(CONDITIONS.len() - 1)],
    }
}

pub fn export_scope(form: &Form, decks: &[(DeckId, String)]) -> ExportScope {
    form.choice(DECK)
        .checked_sub(1)
        .and_then(|index| decks.get(index))
        .map_or(ExportScope::Collection, |(id, _)| ExportScope::Deck(*id))
}

pub fn export_contents(form: &Form) -> (Contents, Format) {
    let contents = Contents {
        scheduling: form.toggle(SCHEDULING),
        presets: form.toggle(PRESETS),
        media: form.toggle(MEDIA),
    };
    let format = if form.toggle(LEGACY) {
        Format::Legacy
    } else {
        Format::Latest
    };

    (contents, format)
}

fn file_field(i18n: &I18n) -> Field {
    Field {
        label: i18n.text("package-file"),
        value: Value::Text(String::new()),
    }
}

const fn toggle(label: String, on: bool) -> Field {
    Field {
        label,
        value: Value::Toggle(on),
    }
}
