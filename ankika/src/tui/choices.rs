/// A row of buttons, one of them focused.
pub struct Choices<T: Copy + 'static> {
    items: &'static [T],
    focused: usize,
}

impl<T: Copy + 'static> Choices<T> {
    /// Focuses the last item, which the prompts use for Cancel.
    pub const fn new(items: &'static [T]) -> Self {
        Self {
            items,
            focused: items.len().saturating_sub(1),
        }
    }

    pub const fn items(&self) -> &'static [T] {
        self.items
    }

    pub fn focused(&self) -> Option<T> {
        self.items.get(self.focused).copied()
    }

    pub const fn focus_next(&mut self) {
        self.focused = (self.focused + 1) % self.items.len();
    }

    pub const fn focus_previous(&mut self) {
        let count = self.items.len();
        self.focused = (self.focused + count - 1) % count;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_on_last() {
        let choices = Choices::new(&[1, 2, 3]);

        assert_eq!(choices.focused(), Some(3));
    }

    #[test]
    fn focus_wraps_around() {
        let mut choices = Choices::new(&[1, 2]);
        choices.focus_next();
        assert_eq!(choices.focused(), Some(1));

        choices.focus_previous();
        assert_eq!(choices.focused(), Some(2));
    }
}
