use ankika_core::card_actions::{CardAction, Flag};
use ankika_core::review::Rating;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::action::Action;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Context {
    Main,
    Help,
    Palette,
    Login,
    FullSync,
    Quit,
    Unbury,
    CustomStudy,
    Add,
    Browser,
    BrowserSearch,
    Stats,
    Form,
    Overview,
    FilteredOverview,
    ReviewQuestion,
    /// The question has an input line, so plain characters are typed instead.
    ReviewTyping,
    ReviewAnswer,
    /// Waiting for a sync, or finished.
    ReviewIdle,
}

#[derive(Debug, Clone, Copy)]
pub struct Binding {
    code: KeyCode,
    modifiers: KeyModifiers,
    label: &'static str,
    action: Action,
}

impl Binding {
    const fn new(
        code: KeyCode,
        modifiers: KeyModifiers,
        label: &'static str,
        action: Action,
    ) -> Self {
        Self {
            code,
            modifiers,
            label,
            action,
        }
    }

    const fn key(character: char, action: Action) -> Self {
        Self::new(KeyCode::Char(character), KeyModifiers::NONE, "", action)
    }

    const fn named(code: KeyCode, label: &'static str, action: Action) -> Self {
        Self::new(code, KeyModifiers::NONE, label, action)
    }

    const fn ctrl(character: char, label: &'static str, action: Action) -> Self {
        Self::new(
            KeyCode::Char(character),
            KeyModifiers::CONTROL,
            label,
            action,
        )
    }

    fn matches(&self, key: &KeyEvent) -> bool {
        // Terminals disagree on whether shifted characters like `?` carry SHIFT, and with
        // keyboard enhancement `Ctrl+Shift+A` may arrive as a lowercase `a` with SHIFT
        let (code, modifiers) = match key.code {
            KeyCode::Char(character) => (
                KeyCode::Char(if key.modifiers.contains(KeyModifiers::SHIFT) {
                    character.to_ascii_uppercase()
                } else {
                    character
                }),
                key.modifiers.difference(KeyModifiers::SHIFT),
            ),
            _ => (key.code, key.modifiers),
        };

        code == self.code && modifiers == self.modifiers
    }

    pub fn label(&self) -> String {
        match self.code {
            KeyCode::Char(character) if self.label.is_empty() => character.to_string(),
            _ => self.label.to_owned(),
        }
    }
}

const QUIT: Binding = Binding::ctrl('c', "Ctrl+C", Action::Quit);
const ESCAPE: Binding = Binding::new(
    KeyCode::Esc,
    KeyModifiers::NONE,
    "Esc",
    Action::CloseOverlay,
);

const UNDO: Binding = Binding::ctrl('z', "Ctrl+Z", Action::Undo);

/// Anki's card shortcuts. `Ctrl` with a digit needs a terminal that reports it, see
/// `terminal::init`; the palette has every action too.
const CARD_ACTIONS: &[Binding] = &[
    Binding::key('a', Action::AddNote),
    Binding::key('b', Action::OpenBrowser),
    Binding::key('e', Action::EditNote),
    Binding::key('*', Action::Card(CardAction::ToggleMark)),
    Binding::key('-', Action::Card(CardAction::BuryCard)),
    Binding::key('=', Action::Card(CardAction::BuryNote)),
    Binding::key('@', Action::Card(CardAction::SuspendCard)),
    Binding::key('!', Action::Card(CardAction::SuspendNote)),
    flag('1', Flag::Red),
    flag('2', Flag::Orange),
    flag('3', Flag::Green),
    flag('4', Flag::Blue),
    flag('5', Flag::Pink),
    flag('6', Flag::Turquoise),
    flag('7', Flag::Purple),
    Binding::new(
        KeyCode::Delete,
        KeyModifiers::CONTROL,
        "Ctrl+Delete",
        Action::Card(CardAction::DeleteNote),
    ),
];

/// The card actions that don't take a plain character, for while an answer is typed.
const CARD_ACTIONS_WHILE_TYPING: &[Binding] = CARD_ACTIONS.split_at(8).1;

const fn flag(digit: char, flag: Flag) -> Binding {
    let label = match flag {
        Flag::Red => "Ctrl+1",
        Flag::Orange => "Ctrl+2",
        Flag::Green => "Ctrl+3",
        Flag::Blue => "Ctrl+4",
        Flag::Pink => "Ctrl+5",
        Flag::Turquoise => "Ctrl+6",
        Flag::Purple => "Ctrl+7",
    };

    Binding::ctrl(digit, label, Action::Card(CardAction::ToggleFlag(flag)))
}
const CLOSE_REVIEW: Binding = Binding::named(KeyCode::Esc, "Esc", Action::CloseReview);

const MAIN: &[Binding] = &[
    Binding::named(KeyCode::Enter, "Enter", Action::OpenDeck),
    Binding::key('a', Action::AddNote),
    Binding::key('b', Action::OpenBrowser),
    Binding::key('t', Action::OpenStats),
    Binding::key(',', Action::OpenSettings),
    Binding::key('?', Action::ToggleHelp),
    Binding::key(':', Action::OpenPalette),
    Binding::ctrl('p', "Ctrl+P", Action::OpenPalette),
    Binding::key('s', Action::Sync),
    Binding::key('q', Action::Quit),
    QUIT,
];

/// Anki's browser shortcuts. Like the card actions, the `Ctrl` ones with digits, Shift
/// or Alt need keyboard enhancement; the palette has them all.
const BROWSER: &[Binding] = &[
    Binding::named(KeyCode::Char(' '), "Space", Action::ToggleSelected),
    Binding::key('/', Action::FocusSearch),
    Binding::ctrl('f', "Ctrl+F", Action::FocusSearch),
    Binding::key('t', Action::ToggleNotesMode),
    Binding::key('p', Action::TogglePreview),
    Binding::key('e', Action::EditNote),
    Binding::key('a', Action::AddNote),
    Binding::ctrl('a', "Ctrl+A", Action::SelectAll),
    Binding::ctrl('j', "Ctrl+J", Action::ToggleSuspend),
    Binding::ctrl('k', "Ctrl+K", Action::Card(CardAction::ToggleMark)),
    Binding::ctrl('d', "Ctrl+D", Action::ChangeDeck),
    Binding::ctrl('A', "Ctrl+Shift+A", Action::AddTags),
    Binding::new(
        KeyCode::Char('A'),
        KeyModifiers::CONTROL.union(KeyModifiers::ALT),
        "Ctrl+Alt+Shift+A",
        Action::RemoveTags,
    ),
    flag('1', Flag::Red),
    flag('2', Flag::Orange),
    flag('3', Flag::Green),
    flag('4', Flag::Blue),
    flag('5', Flag::Pink),
    flag('6', Flag::Turquoise),
    flag('7', Flag::Purple),
    Binding::new(
        KeyCode::Delete,
        KeyModifiers::CONTROL,
        "Ctrl+Delete",
        Action::Card(CardAction::DeleteNote),
    ),
    UNDO,
    Binding::named(KeyCode::Esc, "Esc", Action::CloseBrowser),
    Binding::key('q', Action::CloseBrowser),
    Binding::key('?', Action::ToggleHelp),
    Binding::key(':', Action::OpenPalette),
    Binding::ctrl('p', "Ctrl+P", Action::OpenPalette),
    QUIT,
];

const STATS: &[Binding] = &[
    Binding::named(KeyCode::Left, "Left", Action::PreviousPeriod),
    Binding::named(KeyCode::Right, "Right", Action::NextPeriod),
    Binding::key('c', Action::ToggleStatsScope),
    Binding::named(KeyCode::Esc, "Esc", Action::CloseStats),
    Binding::key('q', Action::CloseStats),
    Binding::key('s', Action::Sync),
    Binding::key('?', Action::ToggleHelp),
    Binding::key(':', Action::OpenPalette),
    Binding::ctrl('p', "Ctrl+P", Action::OpenPalette),
    QUIT,
];

const BROWSER_SEARCH: &[Binding] = &[
    Binding::named(KeyCode::Enter, "Enter", Action::RunSearch),
    Binding::named(KeyCode::Esc, "Esc", Action::LeaveSearch),
    QUIT,
];

const CLOSE_OVERVIEW: Binding = Binding::named(KeyCode::Esc, "Esc", Action::CloseOverview);

const OVERVIEW: &[Binding] = &[
    Binding::named(KeyCode::Enter, "Enter", Action::Study),
    Binding::key('a', Action::AddNote),
    Binding::key('b', Action::OpenBrowser),
    Binding::key('t', Action::OpenStats),
    Binding::key(',', Action::OpenSettings),
    Binding::key('o', Action::DeckOptions),
    Binding::key('c', Action::CustomStudy),
    Binding::key('u', Action::Unbury),
    CLOSE_OVERVIEW,
    Binding::key('q', Action::CloseOverview),
    Binding::key('s', Action::Sync),
    Binding::key('?', Action::ToggleHelp),
    Binding::key(':', Action::OpenPalette),
    Binding::ctrl('p', "Ctrl+P", Action::OpenPalette),
    QUIT,
];

/// Filtered decks are rebuilt and emptied instead of offering custom study, like in Anki.
const FILTERED_OVERVIEW: &[Binding] = &[
    Binding::named(KeyCode::Enter, "Enter", Action::Study),
    Binding::key('a', Action::AddNote),
    Binding::key('b', Action::OpenBrowser),
    Binding::key('t', Action::OpenStats),
    Binding::key(',', Action::OpenSettings),
    Binding::key('r', Action::Rebuild),
    Binding::key('e', Action::Empty),
    Binding::key('u', Action::Unbury),
    CLOSE_OVERVIEW,
    Binding::key('q', Action::CloseOverview),
    Binding::key('s', Action::Sync),
    Binding::key('?', Action::ToggleHelp),
    Binding::key(':', Action::OpenPalette),
    Binding::ctrl('p', "Ctrl+P", Action::OpenPalette),
    QUIT,
];

const REVIEW_QUESTION: &[Binding] = &[
    Binding::named(KeyCode::Char(' '), "Space", Action::ShowAnswer),
    Binding::named(KeyCode::Enter, "Enter", Action::ShowAnswer),
    Binding::key('r', Action::ReplayAudio),
    UNDO,
    CLOSE_REVIEW,
    Binding::key('q', Action::CloseReview),
    Binding::key('?', Action::ToggleHelp),
    Binding::key(':', Action::OpenPalette),
    Binding::ctrl('p', "Ctrl+P", Action::OpenPalette),
    QUIT,
];

const REVIEW_TYPING: &[Binding] = &[
    Binding::named(KeyCode::Enter, "Enter", Action::ShowAnswer),
    UNDO,
    CLOSE_REVIEW,
    Binding::ctrl('p', "Ctrl+P", Action::OpenPalette),
    QUIT,
];

const REVIEW_ANSWER: &[Binding] = &[
    Binding::key('1', Action::Answer(Rating::Again)),
    Binding::key('2', Action::Answer(Rating::Hard)),
    Binding::key('3', Action::Answer(Rating::Good)),
    Binding::named(KeyCode::Char(' '), "Space", Action::Answer(Rating::Good)),
    Binding::named(KeyCode::Enter, "Enter", Action::Answer(Rating::Good)),
    Binding::key('4', Action::Answer(Rating::Easy)),
    Binding::key('r', Action::ReplayAudio),
    UNDO,
    CLOSE_REVIEW,
    Binding::key('q', Action::CloseReview),
    Binding::key('?', Action::ToggleHelp),
    Binding::key(':', Action::OpenPalette),
    Binding::ctrl('p', "Ctrl+P", Action::OpenPalette),
    QUIT,
];

const REVIEW_IDLE: &[Binding] = &[
    CLOSE_REVIEW,
    Binding::named(KeyCode::Enter, "Enter", Action::CloseReview),
    Binding::key('q', Action::CloseReview),
    UNDO,
    Binding::key('?', Action::ToggleHelp),
    Binding::key(':', Action::OpenPalette),
    Binding::ctrl('p', "Ctrl+P", Action::OpenPalette),
    QUIT,
];

const HELP: &[Binding] = &[ESCAPE, Binding::key('?', Action::ToggleHelp), QUIT];

const OVERLAY: &[Binding] = &[ESCAPE, QUIT];

/// The context's own bindings, then any shared ones.
const fn tables(context: Context) -> [&'static [Binding]; 2] {
    let shared: &[Binding] = match context {
        Context::ReviewQuestion | Context::ReviewAnswer => CARD_ACTIONS,
        Context::ReviewTyping => CARD_ACTIONS_WHILE_TYPING,
        _ => &[],
    };

    [bindings(context), shared]
}

const fn bindings(context: Context) -> &'static [Binding] {
    match context {
        Context::Main => MAIN,
        Context::Help => HELP,
        Context::Palette
        | Context::Login
        | Context::FullSync
        | Context::Quit
        | Context::Unbury
        | Context::CustomStudy
        | Context::Add
        | Context::Form => OVERLAY,
        Context::Browser => BROWSER,
        Context::BrowserSearch => BROWSER_SEARCH,
        Context::Stats => STATS,
        Context::Overview => OVERVIEW,
        Context::FilteredOverview => FILTERED_OVERVIEW,
        Context::ReviewQuestion => REVIEW_QUESTION,
        Context::ReviewTyping => REVIEW_TYPING,
        Context::ReviewAnswer => REVIEW_ANSWER,
        Context::ReviewIdle => REVIEW_IDLE,
    }
}

pub fn action_for(context: Context, key: &KeyEvent) -> Option<Action> {
    tables(context)
        .into_iter()
        .flatten()
        .find(|binding| binding.matches(key))
        .map(|binding| binding.action)
}

/// Bindings grouped by action, in declaration order, for hints and help.
pub fn grouped(context: Context) -> Vec<(Action, Vec<String>)> {
    let mut groups: Vec<(Action, Vec<String>)> = Vec::new();

    for binding in tables(context).into_iter().flatten() {
        match groups
            .iter_mut()
            .find(|(action, _)| *action == binding.action)
        {
            Some((_, labels)) => labels.push(binding.label()),
            None => groups.push((binding.action, vec![binding.label()])),
        }
    }

    groups
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn shifted_character_matches_plain_binding() {
        let key = press(KeyCode::Char('?'), KeyModifiers::SHIFT);

        assert_eq!(action_for(Context::Main, &key), Some(Action::ToggleHelp));
    }

    #[test]
    fn control_binding_requires_control() {
        let plain = press(KeyCode::Char('p'), KeyModifiers::NONE);
        let control = press(KeyCode::Char('p'), KeyModifiers::CONTROL);

        assert_eq!(action_for(Context::Main, &plain), None);
        assert_eq!(
            action_for(Context::Main, &control),
            Some(Action::OpenPalette)
        );
    }

    #[test]
    fn palette_leaves_characters_for_typing() {
        let key = press(KeyCode::Char('q'), KeyModifiers::NONE);

        assert_eq!(action_for(Context::Palette, &key), None);
    }

    #[test]
    fn grouped_merges_labels_per_action() {
        let groups = grouped(Context::Main);

        assert_eq!(
            groups,
            vec![
                (Action::OpenDeck, vec!["Enter".to_owned()]),
                (Action::AddNote, vec!["a".to_owned()]),
                (Action::OpenBrowser, vec!["b".to_owned()]),
                (Action::OpenStats, vec!["t".to_owned()]),
                (Action::OpenSettings, vec![",".to_owned()]),
                (Action::ToggleHelp, vec!["?".to_owned()]),
                (
                    Action::OpenPalette,
                    vec![":".to_owned(), "Ctrl+P".to_owned()]
                ),
                (Action::Sync, vec!["s".to_owned()]),
                (Action::Quit, vec!["q".to_owned(), "Ctrl+C".to_owned()]),
            ]
        );
    }
}
