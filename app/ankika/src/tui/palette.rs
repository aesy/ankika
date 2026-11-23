use crate::i18n::I18n;
use crate::tui::action::Action;
use crate::tui::fuzzy_list::{Entry, FuzzyList};

pub type Palette = FuzzyList<Action>;

pub fn new(actions: &[Action], i18n: &I18n) -> Palette {
    FuzzyList::new(
        actions
            .iter()
            .map(|&action| Entry {
                value: action,
                label: action.label(i18n),
            })
            .collect(),
    )
}
