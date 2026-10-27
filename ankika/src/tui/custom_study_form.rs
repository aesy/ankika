use anki::decks::DeckId;
use ankika_core::overview::{CustomStudy, CustomStudyDefaults, StudyKind};

/// The choices of Anki's Custom Study dialog, in its order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    NewLimit,
    ReviewLimit,
    Forgotten,
    Ahead,
    Preview,
    StateOrTag,
}

impl Choice {
    pub const ALL: [Self; 6] = [
        Self::NewLimit,
        Self::ReviewLimit,
        Self::Forgotten,
        Self::Ahead,
        Self::Preview,
        Self::StateOrTag,
    ];
}

pub const KINDS: [StudyKind; 4] = [
    StudyKind::NewOnly,
    StudyKind::DueOnly,
    StudyKind::AllReviewRandom,
    StudyKind::AllRandom,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Choice,
    Amount,
    Kind,
    IncludeTags,
    ExcludeTags,
    Ok,
    Cancel,
}

/// Anki's suggestions for the choices without one from the deck.
const DEFAULT_DAYS: u32 = 1;
const DEFAULT_CARD_LIMIT: u32 = 100;

pub struct CustomStudyForm {
    deck: DeckId,
    defaults: CustomStudyDefaults,
    choice: Choice,
    amount: String,
    kind: StudyKind,
    include_tags: String,
    exclude_tags: String,
    focus: Field,
    invalid: bool,
}

/// Why the form can't be submitted yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidAmount;

impl CustomStudyForm {
    pub fn new(deck: DeckId, defaults: CustomStudyDefaults) -> Self {
        let mut form = Self {
            deck,
            defaults,
            choice: Choice::NewLimit,
            amount: String::new(),
            kind: StudyKind::NewOnly,
            include_tags: String::new(),
            exclude_tags: String::new(),
            focus: Field::Choice,
            invalid: false,
        };
        form.select(Choice::NewLimit);
        form
    }

    pub const fn deck(&self) -> DeckId {
        self.deck
    }

    pub const fn defaults(&self) -> &CustomStudyDefaults {
        &self.defaults
    }

    pub const fn choice(&self) -> Choice {
        self.choice
    }

    pub fn amount(&self) -> &str {
        &self.amount
    }

    pub const fn kind(&self) -> StudyKind {
        self.kind
    }

    pub fn include_tags(&self) -> &str {
        &self.include_tags
    }

    pub fn exclude_tags(&self) -> &str {
        &self.exclude_tags
    }

    pub const fn focus(&self) -> Field {
        self.focus
    }

    pub const fn is_invalid(&self) -> bool {
        self.invalid
    }

    /// Switches the choice and puts in its suggested amount, like Anki's dialog.
    pub fn select(&mut self, choice: Choice) {
        let amount = match choice {
            Choice::NewLimit => self.defaults.extend_new,
            Choice::ReviewLimit => self.defaults.extend_review,
            Choice::Forgotten | Choice::Ahead | Choice::Preview => DEFAULT_DAYS,
            Choice::StateOrTag => DEFAULT_CARD_LIMIT,
        };

        self.choice = choice;
        self.amount = amount.to_string();
        self.invalid = false;
    }

    pub const fn select_kind(&mut self, kind: StudyKind) {
        self.kind = kind;
    }

    pub const fn set_focus(&mut self, field: Field) {
        self.focus = field;
    }

    /// The fields in Tab order; the tag fields only exist for the last choice.
    pub fn fields(&self) -> Vec<Field> {
        let mut fields = vec![Field::Choice, Field::Amount];

        if self.choice == Choice::StateOrTag {
            fields.extend([Field::Kind, Field::IncludeTags, Field::ExcludeTags]);
        }

        fields.extend([Field::Ok, Field::Cancel]);
        fields
    }

    pub fn focus_next(&mut self) {
        self.move_focus(1);
    }

    pub fn focus_previous(&mut self) {
        self.move_focus(-1);
    }

    /// Moves within the focused list: the choices or the card kinds.
    pub fn step(&mut self, forward: bool) {
        match self.focus {
            Field::Choice => {
                let next = cycle(&Choice::ALL, self.choice, forward);
                self.select(next);
            },
            Field::Kind => self.kind = cycle(&KINDS, self.kind, forward),
            _ => {},
        }
    }

    pub fn push(&mut self, character: char) {
        match self.focus {
            Field::Amount if character.is_ascii_digit() || character == '-' => {
                self.amount.push(character);
                self.invalid = false;
            },
            Field::IncludeTags => self.include_tags.push(character),
            Field::ExcludeTags => self.exclude_tags.push(character),
            _ => {},
        }
    }

    pub fn pop(&mut self) {
        match self.focus {
            Field::Amount => {
                self.amount.pop();
                self.invalid = false;
            },
            Field::IncludeTags => {
                self.include_tags.pop();
            },
            Field::ExcludeTags => {
                self.exclude_tags.pop();
            },
            _ => {},
        }
    }

    /// The request for Anki, or marks the amount as invalid. Only the limit increases may
    /// be negative, and the day counts start at one.
    pub fn submit(&mut self) -> Result<CustomStudy, InvalidAmount> {
        let study = self.study();

        self.invalid = study.is_none();
        study.ok_or(InvalidAmount)
    }

    fn study(&self) -> Option<CustomStudy> {
        let days = || self.amount.parse::<u32>().ok().filter(|days| *days > 0);

        Some(match self.choice {
            Choice::NewLimit => CustomStudy::IncreaseNewLimit(self.amount.parse().ok()?),
            Choice::ReviewLimit => CustomStudy::IncreaseReviewLimit(self.amount.parse().ok()?),
            Choice::Forgotten => CustomStudy::ForgottenInLastDays(days()?),
            Choice::Ahead => CustomStudy::AheadDays(days()?),
            Choice::Preview => CustomStudy::PreviewAddedInLastDays(days()?),
            Choice::StateOrTag => CustomStudy::ByStateOrTag {
                kind: self.kind,
                limit: self.amount.parse().ok().filter(|limit| *limit > 0)?,
                include_tags: tags(&self.include_tags),
                exclude_tags: tags(&self.exclude_tags),
            },
        })
    }

    fn move_focus(&mut self, by: isize) {
        let fields = self.fields();
        let count = fields.len().cast_signed();
        let index = fields
            .iter()
            .position(|field| *field == self.focus)
            .map_or(0, usize::cast_signed);

        self.focus = fields[(index + by).rem_euclid(count).cast_unsigned()];
    }
}

fn cycle<T: Copy + PartialEq>(items: &[T], current: T, forward: bool) -> T {
    let index = items.iter().position(|item| *item == current).unwrap_or(0);
    let next = if forward {
        (index + 1) % items.len()
    } else {
        (index + items.len() - 1) % items.len()
    };

    items[next]
}

fn tags(text: &str) -> Vec<String> {
    text.split_whitespace().map(str::to_owned).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn form() -> CustomStudyForm {
        CustomStudyForm::new(
            DeckId(1),
            CustomStudyDefaults {
                extend_new: 20,
                extend_review: 50,
                available_new: 30,
                available_review: 0,
            },
        )
    }

    #[test]
    fn suggests_anki_defaults() {
        let mut form = form();
        assert_eq!(form.amount(), "20");

        form.select(Choice::Ahead);
        assert_eq!(form.amount(), "1");
    }

    #[test]
    fn builds_tag_requests() {
        let mut form = form();
        form.select(Choice::StateOrTag);
        form.set_focus(Field::IncludeTags);
        "verbs  nouns"
            .chars()
            .for_each(|character| form.push(character));

        assert_eq!(
            form.submit(),
            Ok(CustomStudy::ByStateOrTag {
                kind: StudyKind::NewOnly,
                limit: 100,
                include_tags: vec!["verbs".to_owned(), "nouns".to_owned()],
                exclude_tags: Vec::new(),
            })
        );
    }

    #[test]
    fn rejects_zero_days_but_allows_lower_limits() {
        let mut form = form();
        form.select(Choice::Forgotten);
        form.set_focus(Field::Amount);
        form.pop();
        form.push('0');
        assert_eq!(form.submit(), Err(InvalidAmount));
        assert!(form.is_invalid());

        form.select(Choice::NewLimit);
        form.pop();
        form.pop();
        form.push('-');
        form.push('5');
        assert_eq!(form.submit(), Ok(CustomStudy::IncreaseNewLimit(-5)));
    }

    #[test]
    fn tag_fields_only_for_state_or_tag() {
        let mut form = form();
        assert!(!form.fields().contains(&Field::IncludeTags));

        form.select(Choice::StateOrTag);
        assert!(form.fields().contains(&Field::IncludeTags));
    }
}
