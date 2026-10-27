use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher};

#[derive(Debug, Clone)]
pub struct Entry<T> {
    pub value: T,
    pub label: String,
}

impl<T> AsRef<str> for Entry<T> {
    fn as_ref(&self) -> &str {
        &self.label
    }
}

/// A list narrowed down by typing, like the command palette.
pub struct FuzzyList<T: Clone> {
    query: String,
    entries: Vec<Entry<T>>,
    matches: Vec<Entry<T>>,
    selected: usize,
    matcher: Matcher,
}

impl<T: Clone> FuzzyList<T> {
    pub fn new(entries: Vec<Entry<T>>) -> Self {
        Self {
            query: String::new(),
            matches: entries.clone(),
            entries,
            selected: 0,
            matcher: Matcher::new(Config::DEFAULT),
        }
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn matches(&self) -> &[Entry<T>] {
        &self.matches
    }

    pub const fn selected(&self) -> usize {
        self.selected
    }

    pub fn selected_value(&self) -> Option<&T> {
        self.matches.get(self.selected).map(|entry| &entry.value)
    }

    /// Starts on the first entry `is_current` accepts, like the current choice.
    pub fn select_where(&mut self, is_current: impl Fn(&T) -> bool) {
        if let Some(index) = self
            .matches
            .iter()
            .position(|entry| is_current(&entry.value))
        {
            self.selected = index;
        }
    }

    pub fn push(&mut self, character: char) {
        self.query.push(character);
        self.update_matches();
    }

    pub fn pop(&mut self) {
        self.query.pop();
        self.update_matches();
    }

    pub const fn select(&mut self, index: usize) {
        if index < self.matches.len() {
            self.selected = index;
        }
    }

    pub const fn select_previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub const fn select_next(&mut self) {
        self.select(self.selected.saturating_add(1));
    }

    fn update_matches(&mut self) {
        let pattern = Pattern::parse(&self.query, CaseMatching::Ignore, Normalization::Smart);

        self.matches = pattern
            .match_list(self.entries.iter(), &mut self.matcher)
            .into_iter()
            .map(|(entry, _)| entry.clone())
            .collect();
        self.selected = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list() -> FuzzyList<u8> {
        FuzzyList::new(
            [(1, "Help"), (2, "Quit")]
                .into_iter()
                .map(|(value, label)| Entry {
                    value,
                    label: label.to_owned(),
                })
                .collect(),
        )
    }

    #[test]
    fn empty_query_lists_everything() {
        assert_eq!(list().matches().len(), 2);
    }

    #[test]
    fn query_filters_fuzzily() {
        let mut list = list();
        list.push('q');
        list.push('t');

        assert_eq!(list.selected_value(), Some(&2));
        assert_eq!(list.matches().len(), 1);
    }

    #[test]
    fn selection_stays_in_bounds() {
        let mut list = list();
        list.select_previous();
        assert_eq!(list.selected(), 0);

        for _ in 0..10 {
            list.select_next();
        }
        assert_eq!(list.selected(), 1);
    }

    #[test]
    fn starts_on_the_current_value() {
        let mut list = list();
        list.select_where(|value| *value == 2);

        assert_eq!(list.selected_value(), Some(&2));
    }
}
