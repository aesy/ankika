use crossterm::event::KeyCode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Toggle(bool),
    Choice {
        options: Vec<String>,
        selected: usize,
    },
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub label: String,
    pub value: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Field(usize),
    Save,
    Cancel,
}

/// What a key did to the form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Nothing,
    Save,
    Cancel,
}

/// Labeled toggles, choices and text inputs above Save and Cancel, for the settings and
/// the deck options.
pub struct Form {
    pub title: String,
    /// Shown under the fields, like a note that some changes need a restart.
    pub note: Option<String>,
    fields: Vec<Field>,
    focus: Focus,
    error: Option<String>,
}

impl Form {
    pub const fn new(title: String, fields: Vec<Field>) -> Self {
        Self {
            title,
            note: None,
            fields,
            focus: Focus::Field(0),
            error: None,
        }
    }

    pub fn fields(&self) -> &[Field] {
        &self.fields
    }

    pub const fn focus(&self) -> Focus {
        self.focus
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    pub fn toggle(&self, index: usize) -> bool {
        matches!(
            self.fields.get(index).map(|field| &field.value),
            Some(Value::Toggle(true))
        )
    }

    pub fn choice(&self, index: usize) -> usize {
        match self.fields.get(index).map(|field| &field.value) {
            Some(Value::Choice { selected, .. }) => *selected,
            _ => 0,
        }
    }

    pub fn text(&self, index: usize) -> &str {
        match self.fields.get(index).map(|field| &field.value) {
            Some(Value::Text(text)) => text,
            _ => "",
        }
    }

    pub fn handle_key(&mut self, code: KeyCode, typed: Option<char>) -> Outcome {
        match code {
            KeyCode::Up | KeyCode::BackTab => self.move_focus(false),
            KeyCode::Down | KeyCode::Tab => self.move_focus(true),
            KeyCode::Left => self.change(false),
            KeyCode::Right => self.change(true),
            KeyCode::Backspace => {
                if let Some(Value::Text(text)) = self.focused_value() {
                    text.pop();
                    self.error = None;
                }
            },
            KeyCode::Enter => return self.press(),
            _ => {
                if let Some(character) = typed {
                    self.type_character(character);
                }
            },
        }

        Outcome::Nothing
    }

    /// A click focuses the field, and changes toggles and choices like Right does.
    pub fn click(&mut self, focus: Focus) -> Outcome {
        self.focus = focus;

        match focus {
            Focus::Field(_) => {
                self.change(true);
                Outcome::Nothing
            },
            Focus::Save | Focus::Cancel => self.press(),
        }
    }

    fn press(&mut self) -> Outcome {
        match self.focus {
            Focus::Field(_) => {
                self.move_focus(true);
                Outcome::Nothing
            },
            Focus::Save => Outcome::Save,
            Focus::Cancel => Outcome::Cancel,
        }
    }

    fn type_character(&mut self, character: char) {
        match self.focused_value() {
            Some(Value::Text(text)) => {
                text.push(character);
                self.error = None;
            },
            Some(Value::Toggle(_)) if character == ' ' => self.change(true),
            _ => {},
        }
    }

    /// Flips a toggle or moves a choice; text fields don't change.
    fn change(&mut self, forward: bool) {
        match self.focused_value() {
            Some(Value::Toggle(on)) => *on = !*on,
            Some(Value::Choice { options, selected }) => {
                let count = options.len().max(1);

                *selected = if forward {
                    (*selected + 1) % count
                } else {
                    (*selected + count - 1) % count
                };
            },
            _ => {},
        }
    }

    fn focused_value(&mut self) -> Option<&mut Value> {
        match self.focus {
            Focus::Field(index) => self.fields.get_mut(index).map(|field| &mut field.value),
            Focus::Save | Focus::Cancel => None,
        }
    }

    fn move_focus(&mut self, forward: bool) {
        let mut stops: Vec<Focus> = (0..self.fields.len()).map(Focus::Field).collect();
        stops.extend([Focus::Save, Focus::Cancel]);

        let index = stops
            .iter()
            .position(|stop| *stop == self.focus)
            .unwrap_or(0);
        let next = if forward {
            (index + 1) % stops.len()
        } else {
            (index + stops.len() - 1) % stops.len()
        };

        self.focus = stops[next];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn form() -> Form {
        Form::new(
            String::new(),
            vec![
                Field {
                    label: "On".to_owned(),
                    value: Value::Toggle(false),
                },
                Field {
                    label: "Pick".to_owned(),
                    value: Value::Choice {
                        options: vec!["a".to_owned(), "b".to_owned()],
                        selected: 0,
                    },
                },
                Field {
                    label: "Text".to_owned(),
                    value: Value::Text("1".to_owned()),
                },
            ],
        )
    }

    #[test]
    fn edits_each_kind_of_field() {
        let mut form = form();
        form.handle_key(KeyCode::Char(' '), Some(' '));
        assert!(form.toggle(0));

        form.handle_key(KeyCode::Down, None);
        form.handle_key(KeyCode::Left, None);
        assert_eq!(form.choice(1), 1);

        form.handle_key(KeyCode::Down, None);
        form.handle_key(KeyCode::Char('0'), Some('0'));
        assert_eq!(form.text(2), "10");
    }

    #[test]
    fn enter_saves_from_the_save_button() {
        let mut form = form();
        for _ in 0..3 {
            form.handle_key(KeyCode::Tab, None);
        }

        assert_eq!(form.focus(), Focus::Save);
        assert_eq!(form.handle_key(KeyCode::Enter, None), Outcome::Save);
    }
}
