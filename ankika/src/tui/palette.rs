use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher};

use crate::i18n::I18n;
use crate::tui::action::Action;

#[derive(Debug, Clone)]
pub struct Entry {
    action: Action,
    label: String,
}

impl Entry {
    pub fn label(&self) -> &str {
        &self.label
    }
}

impl AsRef<str> for Entry {
    fn as_ref(&self) -> &str {
        &self.label
    }
}

pub struct Palette {
    query: String,
    entries: Vec<Entry>,
    matches: Vec<Entry>,
    selected: usize,
    matcher: Matcher,
}

impl Palette {
    pub fn new(i18n: &I18n) -> Self {
        let entries: Vec<Entry> = Action::PALETTE
            .into_iter()
            .map(|action| Entry {
                action,
                label: action.label(i18n),
            })
            .collect();

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

    pub fn matches(&self) -> &[Entry] {
        &self.matches
    }

    pub const fn selected(&self) -> usize {
        self.selected
    }

    pub fn selected_action(&self) -> Option<Action> {
        self.matches.get(self.selected).map(|entry| entry.action)
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

    fn palette() -> Palette {
        Palette::new(&I18n::new().unwrap())
    }

    #[test]
    fn empty_query_lists_everything() {
        let palette = palette();

        assert_eq!(palette.matches().len(), Action::PALETTE.len());
    }

    #[test]
    fn query_filters_fuzzily() {
        let mut palette = palette();
        palette.push('q');
        palette.push('t');

        assert_eq!(palette.selected_action(), Some(Action::Quit));
        assert_eq!(palette.matches().len(), 1);
    }

    #[test]
    fn selection_stays_in_bounds() {
        let mut palette = palette();
        palette.select_previous();
        assert_eq!(palette.selected(), 0);

        for _ in 0..10 {
            palette.select_next();
        }
        assert_eq!(palette.selected(), Action::PALETTE.len() - 1);
    }
}
