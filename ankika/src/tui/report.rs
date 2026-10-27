use std::cell::Cell;

use ankika_core::maintenance::{EmptyCards, MediaReport};
use crossterm::event::KeyCode;

use crate::i18n::I18n;
use crate::tui::card_text;
use crate::tui::choices::Choices;

const PAGE: i16 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportButton {
    DeleteUnused,
    EmptyTrash,
    RestoreTrash,
    KeepNotes,
    Delete,
    Close,
}

pub enum ReportKind {
    Database,
    Media { unused: Vec<String> },
    EmptyCards { empty: EmptyCards, keep_notes: bool },
}

/// What a Check Database, Check Media or Empty Cards found, with what can be done about it.
pub struct Report {
    kind: ReportKind,
    /// Wrapped to the overlay's width when drawn.
    html: String,
    choices: Choices<ReportButton>,
    scroll: u16,
    /// Set while drawing, which is when the wrapped height is known.
    max_scroll: Cell<u16>,
}

impl Report {
    pub fn database(problems: &[String], i18n: &I18n) -> Self {
        let html = if problems.is_empty() {
            card_text::escape(&i18n.anki().database_check_rebuilt())
        } else {
            plain_to_html(&problems.join("\n"))
        };

        Self::new(ReportKind::Database, html, &[ReportButton::Close])
    }

    pub fn media(report: MediaReport) -> Self {
        let buttons: &'static [ReportButton] = match (!report.unused.is_empty(), report.have_trash)
        {
            (true, true) => &[
                ReportButton::DeleteUnused,
                ReportButton::EmptyTrash,
                ReportButton::RestoreTrash,
                ReportButton::Close,
            ],
            (true, false) => &[ReportButton::DeleteUnused, ReportButton::Close],
            (false, true) => &[
                ReportButton::EmptyTrash,
                ReportButton::RestoreTrash,
                ReportButton::Close,
            ],
            (false, false) => &[ReportButton::Close],
        };
        let html = plain_to_html(&report.report);

        Self::new(
            ReportKind::Media {
                unused: report.unused,
            },
            html,
            buttons,
        )
    }

    pub fn empty_cards(empty: EmptyCards, i18n: &I18n) -> Self {
        if empty.is_empty() {
            let html = card_text::escape(&i18n.anki().empty_cards_not_found());
            let kind = ReportKind::EmptyCards {
                empty,
                keep_notes: true,
            };

            return Self::new(kind, html, &[ReportButton::Close]);
        }

        let html = empty.report.clone();
        let kind = ReportKind::EmptyCards {
            empty,
            keep_notes: true,
        };

        Self::new(
            kind,
            html,
            &[
                ReportButton::KeepNotes,
                ReportButton::Delete,
                ReportButton::Close,
            ],
        )
    }

    const fn new(kind: ReportKind, html: String, buttons: &'static [ReportButton]) -> Self {
        Self {
            kind,
            html,
            choices: Choices::new(buttons),
            scroll: 0,
            max_scroll: Cell::new(0),
        }
    }

    pub fn title(&self, i18n: &I18n) -> String {
        let tr = i18n.anki();

        match self.kind {
            ReportKind::Database => tr.database_check_title(),
            ReportKind::Media { .. } => tr.media_check_window_title(),
            ReportKind::EmptyCards { .. } => tr.empty_cards_window_title(),
        }
        .into_owned()
    }

    pub const fn kind(&self) -> &ReportKind {
        &self.kind
    }

    pub fn html(&self) -> &str {
        &self.html
    }

    pub const fn choices(&self) -> &Choices<ReportButton> {
        &self.choices
    }

    pub const fn keep_notes(&self) -> bool {
        matches!(
            self.kind,
            ReportKind::EmptyCards {
                keep_notes: true,
                ..
            }
        )
    }

    pub const fn toggle_keep_notes(&mut self) {
        if let ReportKind::EmptyCards { keep_notes, .. } = &mut self.kind {
            *keep_notes = !*keep_notes;
        }
    }

    pub fn scroll(&self) -> u16 {
        self.scroll.min(self.max_scroll.get())
    }

    pub fn set_max_scroll(&self, max: u16) {
        self.max_scroll.set(max);
    }

    pub fn scroll_by(&mut self, rows: i16) {
        self.scroll = self
            .scroll()
            .saturating_add_signed(rows)
            .min(self.max_scroll.get());
    }

    /// Returns the button pressed, if any.
    pub fn handle_key(&mut self, code: KeyCode) -> Option<ReportButton> {
        match code {
            KeyCode::Up => self.scroll_by(-1),
            KeyCode::Down => self.scroll_by(1),
            KeyCode::PageUp => self.scroll_by(-PAGE),
            KeyCode::PageDown => self.scroll_by(PAGE),
            KeyCode::Left | KeyCode::BackTab => self.choices.focus_previous(),
            KeyCode::Right | KeyCode::Tab => self.choices.focus_next(),
            KeyCode::Enter => return self.choices.focused(),
            _ => {},
        }

        None
    }
}

/// Anki's plain-text reports, one paragraph per line.
fn plain_to_html(text: &str) -> String {
    text.lines()
        .map(card_text::escape)
        .collect::<Vec<_>>()
        .join("<br>")
}
