use ankika_core::sync::{FullSyncDirection, FullSyncRequired};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    Sync(FullSyncDirection),
    Cancel,
}

/// Asks which side a full sync should overwrite. Focus starts on Cancel, since both
/// other choices throw data away.
pub struct FullSyncPrompt {
    required: FullSyncRequired,
    focused: usize,
}

impl FullSyncPrompt {
    pub const fn new(required: FullSyncRequired) -> Self {
        let choices = Self::choices_for(required);

        Self {
            required,
            focused: choices.len().saturating_sub(1),
        }
    }

    pub const fn required(&self) -> FullSyncRequired {
        self.required
    }

    pub const fn choices(&self) -> &'static [Choice] {
        Self::choices_for(self.required)
    }

    pub fn focused(&self) -> Option<Choice> {
        self.choices().get(self.focused).copied()
    }

    pub const fn focus_next(&mut self) {
        self.focused = (self.focused + 1) % self.choices().len();
    }

    pub const fn focus_previous(&mut self) {
        let count = self.choices().len();
        self.focused = (self.focused + count - 1) % count;
    }

    const fn choices_for(required: FullSyncRequired) -> &'static [Choice] {
        match required {
            FullSyncRequired::Conflict => &[
                Choice::Sync(FullSyncDirection::Download),
                Choice::Sync(FullSyncDirection::Upload),
                Choice::Cancel,
            ],
            FullSyncRequired::UploadOnly => {
                &[Choice::Sync(FullSyncDirection::Upload), Choice::Cancel]
            },
            FullSyncRequired::DownloadOnly => {
                &[Choice::Sync(FullSyncDirection::Download), Choice::Cancel]
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_on_cancel() {
        let prompt = FullSyncPrompt::new(FullSyncRequired::Conflict);

        assert_eq!(prompt.focused(), Some(Choice::Cancel));
    }

    #[test]
    fn focus_wraps_around() {
        let mut prompt = FullSyncPrompt::new(FullSyncRequired::UploadOnly);
        prompt.focus_next();
        assert_eq!(
            prompt.focused(),
            Some(Choice::Sync(FullSyncDirection::Upload))
        );

        prompt.focus_previous();
        assert_eq!(prompt.focused(), Some(Choice::Cancel));
    }
}
