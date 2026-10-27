use std::cell::Cell;
use std::collections::{HashMap, HashSet};

use ankika_core::browser::{Row, Sort, Table};
use ankika_core::review::RenderedCard;

/// Rows to load before the view has said how tall the table is.
const DEFAULT_PAGE: usize = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Table,
    Search,
}

/// Anki's browser: a search, a table of cards or notes, and a preview of the current one.
pub struct Browser {
    table: Table,
    /// What the table shows the results of.
    query: String,
    /// What is being typed into the search line.
    input: String,
    focus: Focus,
    ids: Vec<i64>,
    cursor: usize,
    offset: usize,
    /// Set by the view, which knows how many rows fit.
    page: Cell<usize>,
    selected: HashSet<i64>,
    /// Loaded lazily, only for the rows on screen.
    rows: HashMap<i64, Row>,
    show_preview: bool,
    preview: Option<(i64, RenderedCard)>,
}

impl Browser {
    pub fn new(table: Table, query: String) -> Self {
        Self {
            table,
            input: query.clone(),
            query,
            focus: Focus::Table,
            ids: Vec::new(),
            cursor: 0,
            offset: 0,
            page: Cell::new(DEFAULT_PAGE),
            selected: HashSet::new(),
            rows: HashMap::new(),
            show_preview: true,
            preview: None,
        }
    }

    pub const fn table(&self) -> &Table {
        &self.table
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub const fn focus(&self) -> Focus {
        self.focus
    }

    pub fn ids(&self) -> &[i64] {
        &self.ids
    }

    pub const fn cursor(&self) -> usize {
        self.cursor
    }

    pub const fn offset(&self) -> usize {
        self.offset
    }

    pub fn is_selected(&self, id: i64) -> bool {
        self.selected.contains(&id)
    }

    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }

    pub fn row(&self, id: i64) -> Option<&Row> {
        self.rows.get(&id)
    }

    pub const fn show_preview(&self) -> bool {
        self.show_preview
    }

    pub fn preview(&self) -> Option<&RenderedCard> {
        self.preview.as_ref().map(|(_, card)| card)
    }

    pub fn current(&self) -> Option<i64> {
        self.ids.get(self.cursor).copied()
    }

    /// The selection, or the current row when nothing is selected, in table order.
    pub fn targets(&self) -> Vec<i64> {
        if self.selected.is_empty() {
            return self.current().into_iter().collect();
        }

        self.ids
            .iter()
            .copied()
            .filter(|id| self.selected.contains(id))
            .collect()
    }

    pub fn set_page(&self, rows: usize) {
        self.page.set(rows.max(1));
    }

    /// The ids on screen, which need their rows loaded.
    pub fn visible(&self) -> &[i64] {
        let end = self
            .offset
            .saturating_add(self.page.get())
            .min(self.ids.len());

        &self.ids[self.offset.min(end)..end]
    }

    pub fn missing_rows(&self) -> Vec<i64> {
        self.visible()
            .iter()
            .copied()
            .filter(|id| !self.rows.contains_key(id))
            .collect()
    }

    pub fn insert_row(&mut self, id: i64, row: Row) {
        self.rows.insert(id, row);
    }

    /// The current row's id when its preview isn't loaded yet.
    pub fn missing_preview(&self) -> Option<i64> {
        let current = self.current()?;

        (self.show_preview && self.preview.as_ref().is_none_or(|(id, _)| *id != current))
            .then_some(current)
    }

    pub fn set_preview(&mut self, id: i64, card: RenderedCard) {
        self.preview = Some((id, card));
    }

    /// Shows new results, keeping the current row when it is still there.
    pub fn set_results(&mut self, query: String, ids: Vec<i64>) {
        let current = self.current();

        self.query = query;
        self.ids = ids;
        self.selected.retain(|id| self.ids.contains(id));
        self.invalidate();
        self.cursor = current
            .and_then(|current| self.ids.iter().position(|id| *id == current))
            .unwrap_or(0);
        self.scroll_to_cursor();
    }

    /// Rows and the preview are loaded again, for after a change.
    pub fn invalidate(&mut self) {
        self.rows.clear();
        self.preview = None;
    }

    pub fn set_table(&mut self, table: Table) {
        self.table = table;
        self.selected.clear();
        self.invalidate();
    }

    pub fn set_sort(&mut self, sort: Sort) {
        self.table.sort = Some(sort);
    }

    pub fn select_id(&mut self, id: i64) {
        if let Some(index) = self.ids.iter().position(|candidate| *candidate == id) {
            self.cursor = index;
            self.scroll_to_cursor();
        }
    }

    pub fn move_cursor(&mut self, by: isize) {
        let last = self.ids.len().saturating_sub(1);

        self.cursor = self.cursor.saturating_add_signed(by).min(last);
        self.scroll_to_cursor();
    }

    pub fn page_down(&mut self) {
        self.move_cursor(self.page.get().cast_signed());
    }

    pub fn page_up(&mut self) {
        self.move_cursor(-self.page.get().cast_signed());
    }

    pub const fn first(&mut self) {
        self.cursor = 0;
        self.scroll_to_cursor();
    }

    pub const fn last(&mut self) {
        self.cursor = self.ids.len().saturating_sub(1);
        self.scroll_to_cursor();
    }

    pub const fn click(&mut self, index: usize) {
        if index < self.ids.len() {
            self.cursor = index;
        }
    }

    pub fn toggle_selected(&mut self) {
        if let Some(id) = self.current()
            && !self.selected.remove(&id)
        {
            self.selected.insert(id);
        }
    }

    /// Selects everything, or nothing when everything already is.
    pub fn toggle_select_all(&mut self) {
        if self.selected.len() == self.ids.len() {
            self.selected.clear();
        } else {
            self.selected = self.ids.iter().copied().collect();
        }
    }

    pub const fn toggle_preview(&mut self) {
        self.show_preview = !self.show_preview;
    }

    pub const fn focus_search(&mut self) {
        self.focus = Focus::Search;
    }

    /// Leaves the search line, dropping what was typed but not searched.
    pub fn focus_table(&mut self) {
        self.focus = Focus::Table;
        self.input.clone_from(&self.query);
    }

    pub fn push(&mut self, character: char) {
        self.input.push(character);
    }

    pub fn pop(&mut self) {
        self.input.pop();
    }

    const fn scroll_to_cursor(&mut self) {
        let page = self.page.get();

        if self.cursor < self.offset {
            self.offset = self.cursor;
        } else if self.cursor >= self.offset + page {
            self.offset = self.cursor + 1 - page;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn browser(ids: Vec<i64>) -> Browser {
        let mut browser = Browser::new(
            Table {
                notes_mode: false,
                columns: Vec::new(),
                sort: None,
            },
            String::new(),
        );
        browser.set_page(3);
        browser.set_results(String::new(), ids);
        browser
    }

    #[test]
    fn targets_the_current_row_without_a_selection() {
        let mut browser = browser(vec![1, 2, 3]);
        browser.move_cursor(1);
        assert_eq!(browser.targets(), vec![2]);

        browser.toggle_selected();
        browser.move_cursor(1);
        browser.toggle_selected();
        assert_eq!(browser.targets(), vec![2, 3]);
    }

    #[test]
    fn scrolls_to_keep_the_cursor_visible() {
        let mut browser = browser((1..=10).collect());
        browser.move_cursor(4);
        assert_eq!(browser.offset(), 2);
        assert_eq!(browser.visible(), &[3, 4, 5]);

        browser.first();
        assert_eq!(browser.offset(), 0);
    }

    #[test]
    fn keeps_the_current_row_across_searches() {
        let mut browser = browser(vec![1, 2, 3]);
        browser.move_cursor(2);
        browser.set_results(String::new(), vec![3, 4]);

        assert_eq!(browser.current(), Some(3));
    }
}
