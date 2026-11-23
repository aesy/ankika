#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Email,
    Password,
    Submit,
    Cancel,
}

impl Field {
    const fn next(self) -> Self {
        match self {
            Self::Email => Self::Password,
            Self::Password => Self::Submit,
            Self::Submit => Self::Cancel,
            Self::Cancel => Self::Email,
        }
    }

    const fn previous(self) -> Self {
        match self {
            Self::Email => Self::Cancel,
            Self::Password => Self::Email,
            Self::Submit => Self::Password,
            Self::Cancel => Self::Submit,
        }
    }
}

pub struct LoginForm {
    email: String,
    password: String,
    focus: Field,
    error: Option<String>,
    busy: bool,
}

impl LoginForm {
    pub const fn new() -> Self {
        Self {
            email: String::new(),
            password: String::new(),
            focus: Field::Email,
            error: None,
            busy: false,
        }
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub const fn focus(&self) -> Field {
        self.focus
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub const fn busy(&self) -> bool {
        self.busy
    }

    pub const fn can_submit(&self) -> bool {
        !self.busy && !self.email.is_empty() && !self.password.is_empty()
    }

    pub const fn set_focus(&mut self, field: Field) {
        self.focus = field;
    }

    pub const fn focus_next(&mut self) {
        self.focus = self.focus.next();
    }

    pub const fn focus_previous(&mut self) {
        self.focus = self.focus.previous();
    }

    pub fn push(&mut self, character: char) {
        if let Some(text) = self.focused_text() {
            text.push(character);
        }
    }

    pub fn pop(&mut self) {
        if let Some(text) = self.focused_text() {
            text.pop();
        }
    }

    pub fn start(&mut self) {
        self.busy = true;
        self.error = None;
    }

    pub fn fail(&mut self, error: String) {
        self.busy = false;
        self.error = Some(error);
    }

    const fn focused_text(&mut self) -> Option<&mut String> {
        match self.focus {
            Field::Email => Some(&mut self.email),
            Field::Password => Some(&mut self.password),
            Field::Submit | Field::Cancel => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typing_goes_to_focused_field() {
        let mut form = LoginForm::new();
        form.push('a');
        form.focus_next();
        form.push('b');
        form.focus_next();
        form.push('c');

        assert_eq!(form.email(), "a");
        assert_eq!(form.password(), "b");
    }

    #[test]
    fn focus_wraps_around() {
        let mut form = LoginForm::new();
        form.focus_previous();
        assert_eq!(form.focus(), Field::Cancel);

        form.focus_next();
        assert_eq!(form.focus(), Field::Email);
    }

    #[test]
    fn submit_needs_both_fields_and_no_login_in_progress() {
        let mut form = LoginForm::new();
        form.push('a');
        assert!(!form.can_submit());

        form.focus_next();
        form.push('b');
        assert!(form.can_submit());

        form.start();
        assert!(!form.can_submit());

        form.fail("Wrong password".to_owned());
        assert!(form.can_submit());
        assert_eq!(form.error(), Some("Wrong password"));
    }
}
