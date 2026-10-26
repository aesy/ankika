use ankika_core::sync::{FullSyncDirection, FullSyncRequired};

use crate::tui::choices::Choices;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    Sync(FullSyncDirection),
    Cancel,
}

/// Asks which side a full sync should overwrite. Focus starts on Cancel, since both
/// other choices throw data away.
pub struct FullSyncPrompt {
    required: FullSyncRequired,
    choices: Choices<Choice>,
}

impl FullSyncPrompt {
    pub const fn new(required: FullSyncRequired) -> Self {
        let items: &[Choice] = match required {
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
        };

        Self {
            required,
            choices: Choices::new(items),
        }
    }

    pub const fn required(&self) -> FullSyncRequired {
        self.required
    }

    pub const fn choices(&self) -> &Choices<Choice> {
        &self.choices
    }

    pub const fn choices_mut(&mut self) -> &mut Choices<Choice> {
        &mut self.choices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_on_cancel() {
        let prompt = FullSyncPrompt::new(FullSyncRequired::Conflict);

        assert_eq!(prompt.choices().focused(), Some(Choice::Cancel));
    }
}
