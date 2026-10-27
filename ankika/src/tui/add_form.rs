use anki::decks::DeckId;
use anki::notetype::NotetypeId;
use ankika_core::adding::AddChoices;

use crate::tui::fuzzy_list::{Entry, FuzzyList};

/// How many tag suggestions show under the input.
const MAX_COMPLETIONS: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Notetype,
    Deck,
    Tags,
    Add,
    Close,
}

const FIELDS: [Field; 5] = [
    Field::Notetype,
    Field::Deck,
    Field::Tags,
    Field::Add,
    Field::Close,
];

/// An entry of the list opened to pick a note type or deck.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pick {
    Notetype(NotetypeId),
    Deck(DeckId),
}

pub type Picker = FuzzyList<Pick>;

/// Anki's Add window, minus the fields, which the editor takes. It stays open between
/// notes and keeps its choices, like Anki's.
pub struct AddForm {
    choices: AddChoices,
    notetype: NotetypeId,
    deck: DeckId,
    tags: String,
    focus: Field,
    picker: Option<Picker>,
    completions: Vec<String>,
    completion: usize,
}

impl AddForm {
    pub const fn new(choices: AddChoices) -> Self {
        Self {
            notetype: choices.notetype,
            deck: choices.deck,
            choices,
            tags: String::new(),
            focus: Field::Notetype,
            picker: None,
            completions: Vec::new(),
            completion: 0,
        }
    }

    pub const fn notetype(&self) -> NotetypeId {
        self.notetype
    }

    pub const fn deck(&self) -> DeckId {
        self.deck
    }

    pub fn notetype_name(&self) -> &str {
        name(&self.choices.notetypes, self.notetype)
    }

    pub fn deck_name(&self) -> &str {
        name(&self.choices.decks, self.deck)
    }

    pub fn tags(&self) -> &str {
        &self.tags
    }

    pub fn tag_list(&self) -> Vec<String> {
        self.tags.split_whitespace().map(str::to_owned).collect()
    }

    pub const fn focus(&self) -> Field {
        self.focus
    }

    pub const fn set_focus(&mut self, field: Field) {
        self.focus = field;
    }

    pub const fn picker(&self) -> Option<&Picker> {
        self.picker.as_ref()
    }

    pub const fn picker_mut(&mut self) -> Option<&mut Picker> {
        self.picker.as_mut()
    }

    pub fn completions(&self) -> &[String] {
        &self.completions
    }

    pub const fn completion(&self) -> usize {
        self.completion
    }

    pub fn focus_next(&mut self) {
        self.focus = FIELDS[(self.index() + 1) % FIELDS.len()];
    }

    pub fn focus_previous(&mut self) {
        self.focus = FIELDS[(self.index() + FIELDS.len() - 1) % FIELDS.len()];
    }

    /// Opens the list for the focused picker, starting on the current choice.
    pub fn open_picker(&mut self) {
        let (entries, current) = match self.focus {
            Field::Notetype => (
                entries(&self.choices.notetypes, Pick::Notetype),
                Pick::Notetype(self.notetype),
            ),
            Field::Deck => (
                entries(&self.choices.decks, Pick::Deck),
                Pick::Deck(self.deck),
            ),
            _ => return,
        };
        let mut list = FuzzyList::new(entries);

        list.select_where(|pick| *pick == current);
        self.picker = Some(list);
    }

    pub fn close_picker(&mut self) {
        self.picker = None;
    }

    /// Takes the picker's selection, if there is one.
    pub fn confirm_picker(&mut self) {
        match self
            .picker
            .take()
            .and_then(|list| list.selected_value().copied())
        {
            Some(Pick::Notetype(id)) => self.notetype = id,
            Some(Pick::Deck(id)) => self.deck = id,
            None => {},
        }
    }

    pub fn push(&mut self, character: char) {
        if self.focus == Field::Tags {
            self.tags.push(character);
            self.update_completions();
        }
    }

    pub fn pop(&mut self) {
        if self.focus == Field::Tags {
            self.tags.pop();
            self.update_completions();
        }
    }

    pub const fn select_completion(&mut self, index: usize) {
        if index < self.completions.len() {
            self.completion = index;
        }
    }

    pub const fn previous_completion(&mut self) {
        self.completion = self.completion.saturating_sub(1);
    }

    pub const fn next_completion(&mut self) {
        self.select_completion(self.completion.saturating_add(1));
    }

    /// Replaces the word being typed with the selected suggestion. Returns whether there
    /// was one.
    pub fn complete(&mut self) -> bool {
        let Some(tag) = self.completions.get(self.completion).cloned() else {
            return false;
        };
        let start = self
            .tags
            .rfind(char::is_whitespace)
            .map_or(0, |index| index + 1);

        self.tags.truncate(start);
        self.tags.push_str(&tag);
        self.tags.push(' ');
        self.update_completions();

        true
    }

    /// Tags starting with the word being typed, ignoring case and the tags already there.
    fn update_completions(&mut self) {
        let typing = !self.tags.ends_with(char::is_whitespace);
        let mut words: Vec<&str> = self.tags.split_whitespace().collect();
        let word = if typing { words.pop() } else { None };

        self.completions = word.map_or_else(Vec::new, |word| {
            let word = word.to_lowercase();

            self.choices
                .tags
                .iter()
                .filter(|tag| tag.to_lowercase().starts_with(&word))
                .filter(|tag| !words.iter().any(|done| done.eq_ignore_ascii_case(tag)))
                .take(MAX_COMPLETIONS)
                .cloned()
                .collect()
        });
        self.completion = 0;
    }

    fn index(&self) -> usize {
        FIELDS
            .iter()
            .position(|field| *field == self.focus)
            .unwrap_or(0)
    }
}

fn name<T: Copy + PartialEq>(choices: &[(T, String)], id: T) -> &str {
    choices
        .iter()
        .find(|(choice, _)| *choice == id)
        .map_or("", |(_, name)| name.as_str())
}

fn entries<T: Copy>(choices: &[(T, String)], pick: impl Fn(T) -> Pick) -> Vec<Entry<Pick>> {
    choices
        .iter()
        .map(|(value, label)| Entry {
            value: pick(*value),
            label: label.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn form() -> AddForm {
        AddForm::new(AddChoices {
            notetypes: vec![
                (NotetypeId(1), "Basic".to_owned()),
                (NotetypeId(2), "Cloze".to_owned()),
            ],
            decks: vec![(DeckId(1), "Default".to_owned())],
            tags: vec!["verbs".to_owned(), "Vocab".to_owned(), "nouns".to_owned()],
            notetype: NotetypeId(2),
            deck: DeckId(1),
        })
    }

    fn type_text(form: &mut AddForm, text: &str) {
        form.set_focus(Field::Tags);
        text.chars().for_each(|character| form.push(character));
    }

    #[test]
    fn completes_the_word_being_typed() {
        let mut form = form();
        type_text(&mut form, "nouns v");

        assert_eq!(form.completions(), ["verbs", "Vocab"]);

        form.next_completion();
        assert!(form.complete());
        assert_eq!(form.tags(), "nouns Vocab ");
        assert_eq!(form.completions(), &[] as &[String]);
    }

    #[test]
    fn skips_tags_already_typed() {
        let mut form = form();
        type_text(&mut form, "verbs v");

        assert_eq!(form.completions(), ["Vocab"]);
    }

    #[test]
    fn picker_starts_on_the_current_choice() {
        let mut form = form();
        form.open_picker();

        let selected = form
            .picker()
            .and_then(|list| list.selected_value().copied());

        assert_eq!(selected, Some(Pick::Notetype(NotetypeId(2))));
    }
}
