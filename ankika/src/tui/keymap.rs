use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::action::Action;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Context {
    Main,
    Help,
    Palette,
    Login,
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

    const fn ctrl(character: char, label: &'static str, action: Action) -> Self {
        Self::new(
            KeyCode::Char(character),
            KeyModifiers::CONTROL,
            label,
            action,
        )
    }

    fn matches(&self, key: &KeyEvent) -> bool {
        // Terminals disagree on whether shifted characters like `?` carry SHIFT
        let modifiers = match key.code {
            KeyCode::Char(_) => key.modifiers.difference(KeyModifiers::SHIFT),
            _ => key.modifiers,
        };

        key.code == self.code && modifiers == self.modifiers
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

const MAIN: &[Binding] = &[
    Binding::key('?', Action::ToggleHelp),
    Binding::key(':', Action::OpenPalette),
    Binding::ctrl('p', "Ctrl+P", Action::OpenPalette),
    Binding::key('q', Action::Quit),
    QUIT,
];

const HELP: &[Binding] = &[ESCAPE, Binding::key('?', Action::ToggleHelp), QUIT];

const OVERLAY: &[Binding] = &[ESCAPE, QUIT];

pub const fn bindings(context: Context) -> &'static [Binding] {
    match context {
        Context::Main => MAIN,
        Context::Help => HELP,
        Context::Palette | Context::Login => OVERLAY,
    }
}

pub fn action_for(context: Context, key: &KeyEvent) -> Option<Action> {
    bindings(context)
        .iter()
        .find(|binding| binding.matches(key))
        .map(|binding| binding.action)
}

/// Bindings grouped by action, in declaration order, for hints and help.
pub fn grouped(context: Context) -> Vec<(Action, Vec<String>)> {
    let mut groups: Vec<(Action, Vec<String>)> = Vec::new();

    for binding in bindings(context) {
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
                (Action::ToggleHelp, vec!["?".to_owned()]),
                (
                    Action::OpenPalette,
                    vec![":".to_owned(), "Ctrl+P".to_owned()]
                ),
                (Action::Quit, vec!["q".to_owned(), "Ctrl+C".to_owned()]),
            ]
        );
    }
}
