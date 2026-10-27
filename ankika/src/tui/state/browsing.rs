use anki::card::CardId;
use anki::collection::Collection;
use anki::decks::DeckId;
use ankika_core::adding;
use ankika_core::browser::{self, BrowserError, Sort};
use ankika_core::card_actions::{self, CardAction};
use ankika_core::note_edit::NoteText;
use ankika_core::review;
use crossterm::event::KeyCode;

use super::{EditRequest, Overlay, Purpose, State};
use crate::i18n::I18n;
use crate::tui::action::Action;
use crate::tui::browser::{Browser, Focus};
use crate::tui::error_text;
use crate::tui::fuzzy_list::{Entry, FuzzyList};
use crate::tui::tag_input::TagInput;

impl State {
    /// Opens on Anki's default search, on the card being studied if there is one.
    pub(super) fn open_browser(&mut self, i18n: &I18n) {
        let Some(collection) = &mut self.collection else {
            return;
        };
        let notes_mode = browser::notes_mode(collection);
        let table = match browser::table(collection, notes_mode) {
            Ok(table) => table,
            Err(error) => return self.fail_browser(&error, i18n),
        };
        let query = browser::default_search(collection);
        let current = self
            .review
            .as_ref()
            .and_then(|review| review.card())
            .map(|card| {
                if notes_mode {
                    card.note_id.0
                } else {
                    card.id.0
                }
            });

        self.browser = Some(Browser::new(table, query.clone()));
        self.run_search(query, i18n);

        if let (Some(browser), Some(current)) = (&mut self.browser, current) {
            browser.select_id(current);
        }
    }

    pub(super) fn close_browser(&mut self, i18n: &I18n) {
        self.browser = None;
        self.reload_decks(i18n);
        // The card being studied may have changed
        self.refresh_deck_screen(i18n);
    }

    /// Shows the results, or keeps the old ones with Anki's message for an invalid search.
    pub(super) fn run_search(&mut self, query: String, i18n: &I18n) {
        let (Some(browser), Some(collection)) = (&mut self.browser, &mut self.collection) else {
            return;
        };

        match browser::search(collection, &query, browser.table()) {
            Ok(ids) => {
                browser.set_results(query, ids);
                browser.focus_table();
            },
            Err(error) => self.fail_browser(&error, i18n),
        }
    }

    /// Searches again, for after a change that may move or remove rows.
    pub(super) fn refresh_browser(&mut self, i18n: &I18n) {
        let Some(query) = self
            .browser
            .as_ref()
            .map(|browser| browser.query().to_owned())
        else {
            return;
        };

        self.run_search(query, i18n);
    }

    /// Loads the rows on screen and the current card's preview. Runs before every draw.
    pub fn load_browser_data(&mut self) {
        let (Some(browser), Some(collection)) = (&mut self.browser, &mut self.collection) else {
            return;
        };

        for id in browser.missing_rows() {
            match browser::row(collection, id) {
                Ok(row) => browser.insert_row(id, row),
                Err(error) => tracing::warn!(?error, id, "Failed to load a browser row"),
            }
        }

        if let Some(id) = browser.missing_preview() {
            let notes_mode = browser.table().notes_mode;
            let rendered = first_card(collection, notes_mode, id)
                .and_then(|card| review::render_card(collection, card).ok());

            if let Some(card) = rendered {
                browser.set_preview(id, card);
            }
        }
    }

    /// The browser's own actions, and the ones it handles differently from review. Returns
    /// whether the action was the browser's.
    pub(super) fn perform_in_browser(&mut self, action: Action, i18n: &I18n) -> bool {
        match action {
            Action::Undo => self.browser_undo(i18n),
            Action::Card(action) => self.browser_card_action(action, i18n),
            Action::EditNote => self.request_browser_edit(i18n),
            Action::CloseBrowser => self.close_browser(i18n),
            Action::FocusSearch => {
                if let Some(browser) = &mut self.browser {
                    browser.focus_search();
                }
            },
            Action::RunSearch => {
                if let Some(query) = self
                    .browser
                    .as_ref()
                    .map(|browser| browser.input().to_owned())
                {
                    self.run_search(query, i18n);
                }
            },
            Action::LeaveSearch => {
                if let Some(browser) = &mut self.browser {
                    browser.focus_table();
                }
            },
            Action::ToggleNotesMode => self.toggle_notes_mode(i18n),
            Action::TogglePreview => {
                if let Some(browser) = &mut self.browser {
                    browser.toggle_preview();
                }
            },
            Action::ToggleSelected => {
                if let Some(browser) = &mut self.browser {
                    browser.toggle_selected();
                    browser.move_cursor(1);
                }
            },
            Action::SelectAll => {
                if let Some(browser) = &mut self.browser {
                    browser.toggle_select_all();
                }
            },
            Action::ToggleSuspend => self.toggle_suspend(i18n),
            Action::ChangeDeck => self.open_deck_picker(i18n),
            Action::AddTags => self.open_tags(false, i18n),
            Action::RemoveTags => self.open_tags(true, i18n),
            Action::SortBy => self.open_sort_picker(),
            _ => return false,
        }

        true
    }

    pub(super) fn handle_browser_overlay_key(
        &mut self,
        code: KeyCode,
        typed: Option<char>,
        i18n: &I18n,
    ) {
        let key = code;

        match &mut self.overlay {
            Overlay::ChangeDeck(list) => {
                if list.handle_key(key, typed)
                    && let Some(&deck) = list.selected_value()
                {
                    self.move_to_deck(deck, i18n);
                }
            },
            Overlay::SortBy(list) => {
                if list.handle_key(key, typed) {
                    let column = list.selected_value().copied();
                    self.overlay = Overlay::None;

                    if let Some(column) = column {
                        self.sort_by(column, i18n);
                    }
                }
            },
            Overlay::Tags { input, .. } => match key {
                KeyCode::Tab => {
                    input.complete();
                },
                KeyCode::Up => input.previous_completion(),
                KeyCode::Down => input.next_completion(),
                KeyCode::Backspace => input.pop(),
                KeyCode::Enter => self.apply_tags(i18n),
                _ => {
                    if let Some(character) = typed {
                        input.push(character);
                    }
                },
            },
            _ => {},
        }
    }

    pub(super) fn handle_browser_key(&mut self, code: KeyCode, typed: Option<char>) {
        let Some(browser) = &mut self.browser else {
            return;
        };

        if browser.focus() == Focus::Search {
            match code {
                KeyCode::Backspace => browser.pop(),
                _ => {
                    if let Some(character) = typed {
                        browser.push(character);
                    }
                },
            }
            return;
        }

        match code {
            KeyCode::Up => browser.move_cursor(-1),
            KeyCode::Down => browser.move_cursor(1),
            KeyCode::PageUp => browser.page_up(),
            KeyCode::PageDown => browser.page_down(),
            KeyCode::Home => browser.first(),
            KeyCode::End => browser.last(),
            _ => {},
        }
    }

    pub(super) fn toggle_notes_mode(&mut self, i18n: &I18n) {
        let (Some(browser), Some(collection)) = (&mut self.browser, &mut self.collection) else {
            return;
        };
        let notes_mode = !browser.table().notes_mode;
        let table = browser::set_notes_mode(collection, notes_mode)
            .and_then(|()| browser::table(collection, notes_mode));

        match table {
            Ok(table) => {
                browser.set_table(table);
                self.refresh_browser(i18n);
            },
            Err(error) => self.fail_browser(&error, i18n),
        }
    }

    /// Sorts by the column, the other way round when it already is, like clicking a header
    /// in Anki.
    pub(super) fn sort_by(&mut self, column: usize, i18n: &I18n) {
        let (Some(browser), Some(collection)) = (&mut self.browser, &mut self.collection) else {
            return;
        };
        let Some(info) = browser.table().columns.get(column) else {
            return;
        };

        if !info.sortable {
            self.notice = Some(
                i18n.anki()
                    .browsing_sorting_on_this_column_is_not()
                    .into_owned(),
            );
            return;
        }

        let reverse = browser
            .table()
            .sort
            .as_ref()
            .is_some_and(|sort| sort.column == info.key && !sort.reverse);
        let sort = Sort {
            column: info.key.clone(),
            reverse,
        };

        if let Err(error) = browser::set_sort(collection, browser.table().notes_mode, &sort) {
            return self.fail_browser(&error, i18n);
        }

        browser.set_sort(sort);
        self.refresh_browser(i18n);
    }

    pub(super) fn open_sort_picker(&mut self) {
        let Some(browser) = &self.browser else {
            return;
        };
        let entries = browser
            .table()
            .columns
            .iter()
            .enumerate()
            .filter(|(_, column)| column.sortable)
            .map(|(index, column)| Entry {
                value: index,
                label: column.label.clone(),
            })
            .collect();

        self.overlay = Overlay::SortBy(FuzzyList::new(entries));
    }

    pub(super) fn open_deck_picker(&mut self, i18n: &I18n) {
        let Some(collection) = &mut self.collection else {
            return;
        };

        match adding::normal_decks(collection) {
            Ok(decks) => {
                let entries = decks
                    .into_iter()
                    .map(|(value, label)| Entry { value, label })
                    .collect();

                self.overlay = Overlay::ChangeDeck(FuzzyList::new(entries));
            },
            Err(error) => self.notice = Some(error.anki_error().message(i18n.anki())),
        }
    }

    pub(super) fn open_tags(&mut self, remove: bool, i18n: &I18n) {
        let Some(collection) = &self.collection else {
            return;
        };

        match adding::all_tags(collection) {
            Ok(tags) => {
                self.overlay = Overlay::Tags {
                    remove,
                    input: TagInput::new(tags),
                };
            },
            Err(error) => self.notice = Some(error.anki_error().message(i18n.anki())),
        }
    }

    pub(super) fn move_to_deck(&mut self, deck: DeckId, i18n: &I18n) {
        self.overlay = Overlay::None;
        self.change_cards(i18n, |collection, cards, _| {
            browser::set_deck(collection, cards, deck)
        });
    }

    pub(super) fn apply_tags(&mut self, i18n: &I18n) {
        let Overlay::Tags { remove, input } = std::mem::replace(&mut self.overlay, Overlay::None)
        else {
            return;
        };
        let tags = input.tags();

        if tags.is_empty() {
            return;
        }

        self.change_notes(i18n, |collection, notes| {
            if remove {
                browser::remove_tags(collection, notes, &tags)
            } else {
                browser::add_tags(collection, notes, &tags)
            }
        });
    }

    /// Like Anki, the current card decides which way a toggle goes for the whole selection.
    pub(super) fn browser_card_action(&mut self, action: CardAction, i18n: &I18n) {
        match action {
            CardAction::ToggleFlag(flag) => {
                self.change_cards(i18n, |collection, cards, current| {
                    let marks = card_actions::marks(collection, current).unwrap_or_default();
                    let number = if marks.flag == Some(flag) {
                        0
                    } else {
                        flag as u32
                    };

                    browser::set_flag(collection, cards, number)
                });
            },
            CardAction::ToggleMark => {
                let marked = self.current_card_marks().is_some_and(|marks| marks.marked);

                self.change_notes(i18n, |collection, notes| {
                    let tags = [String::from("marked")];

                    if marked {
                        browser::remove_tags(collection, notes, &tags)
                    } else {
                        browser::add_tags(collection, notes, &tags)
                    }
                });
            },
            CardAction::DeleteNote => {
                let mut deleted = 0;

                self.change_notes(i18n, |collection, notes| {
                    deleted = browser::delete_notes(collection, notes)?;
                    Ok(())
                });

                if deleted > 0 {
                    self.notice = Some(i18n.anki().browsing_cards_deleted(deleted).into_owned());
                }
            },
            // The reviewer's bury and suspend actions; the browser toggles suspend instead
            CardAction::BuryCard
            | CardAction::BuryNote
            | CardAction::SuspendCard
            | CardAction::SuspendNote => {},
        }
    }

    pub(super) fn toggle_suspend(&mut self, i18n: &I18n) {
        self.change_cards(i18n, |collection, cards, current| {
            let suspended = browser::is_suspended(collection, current)?;

            browser::set_suspended(collection, cards, !suspended)
        });
    }

    pub(super) fn request_browser_edit(&mut self, i18n: &I18n) {
        let (Some(browser), Some(collection)) = (&self.browser, &mut self.collection) else {
            return;
        };
        let Some(current) = browser.current() else {
            return;
        };
        let note = browser::notes(collection, browser.table().notes_mode, &[current])
            .ok()
            .and_then(|notes| notes.first().copied());
        let Some(note) = note else {
            return;
        };

        match NoteText::load(collection, note) {
            Ok(note) => {
                self.edit = Some(EditRequest {
                    note,
                    purpose: Purpose::Edit,
                });
            },
            Err(error) => self.notice = Some(error_text::note_edit(&error, i18n)),
        }
    }

    pub(super) fn browser_undo(&mut self, i18n: &I18n) {
        let Some(collection) = &mut self.collection else {
            return;
        };

        if let Err(error) = review::undo(collection) {
            self.notice = Some(error_text::review(&error, i18n));
        }

        self.refresh_browser(i18n);
    }

    fn current_card_marks(&mut self) -> Option<card_actions::Marks> {
        let (Some(browser), Some(collection)) = (&self.browser, &mut self.collection) else {
            return None;
        };
        let card = first_card(collection, browser.table().notes_mode, browser.current()?)?;

        card_actions::marks(collection, card).ok()
    }

    /// Runs a change on the targeted cards, then shows the results again.
    fn change_cards(
        &mut self,
        i18n: &I18n,
        change: impl FnOnce(&mut Collection, &[CardId], CardId) -> Result<(), BrowserError>,
    ) {
        let (Some(browser), Some(collection)) = (&self.browser, &mut self.collection) else {
            return;
        };
        let notes_mode = browser.table().notes_mode;
        let targets = browser.targets();
        let current = browser
            .current()
            .and_then(|id| first_card(collection, notes_mode, id));
        let result =
            browser::cards(collection, notes_mode, &targets).and_then(|cards| match current {
                Some(current) if !cards.is_empty() => change(collection, &cards, current),
                _ => Ok(()),
            });

        self.finish_change(result, i18n);
    }

    fn change_notes(
        &mut self,
        i18n: &I18n,
        change: impl FnOnce(&mut Collection, &[anki::notes::NoteId]) -> Result<(), BrowserError>,
    ) {
        let (Some(browser), Some(collection)) = (&self.browser, &mut self.collection) else {
            return;
        };
        let targets = browser.targets();
        let result =
            browser::notes(collection, browser.table().notes_mode, &targets).and_then(|notes| {
                if notes.is_empty() {
                    Ok(())
                } else {
                    change(collection, &notes)
                }
            });

        self.finish_change(result, i18n);
    }

    fn finish_change(&mut self, result: Result<(), BrowserError>, i18n: &I18n) {
        if let Err(error) = result {
            self.fail_browser(&error, i18n);
        }

        self.refresh_browser(i18n);
    }

    fn fail_browser(&mut self, error: &BrowserError, i18n: &I18n) {
        tracing::warn!(?error, "Browser action failed");
        self.notice = Some(error_text::browser(error, i18n));
    }
}

/// The card a table row stands for; in notes mode, the note's first card.
fn first_card(collection: &mut Collection, notes_mode: bool, id: i64) -> Option<CardId> {
    browser::cards(collection, notes_mode, &[id])
        .ok()
        .and_then(|cards| cards.first().copied())
}
