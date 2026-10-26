use std::fmt::Display;

use anki::prelude::I18n as AnkiI18n;
use fluent_bundle::FluentResource;
use fluent_bundle::concurrent::FluentBundle;
use unic_langid::langid;

const MESSAGES: &str = include_str!("../locales/en-US/ankika.ftl");

#[derive(Debug, thiserror::Error)]
pub enum I18nError {
    #[error("Failed to parse the built-in messages: {details}")]
    Parse { details: String },
    #[error("Failed to load the built-in messages: {details}")]
    AddResource { details: String },
}

/// Anki's own translations where a string exists, ankika's `.ftl` for the rest.
pub struct I18n {
    anki: AnkiI18n,
    bundle: FluentBundle<FluentResource>,
}

impl I18n {
    pub fn new() -> Result<Self, I18nError> {
        let resource = FluentResource::try_new(MESSAGES.to_owned()).map_err(|(_, errors)| {
            I18nError::Parse {
                details: join(&errors),
            }
        })?;

        let mut bundle = FluentBundle::new_concurrent(vec![langid!("en-US")]);
        // Isolation marks around placeables are invisible but break terminal width calculations
        bundle.set_use_isolating(false);
        bundle
            .add_resource(resource)
            .map_err(|errors| I18nError::AddResource {
                details: join(&errors),
            })?;

        Ok(Self {
            anki: AnkiI18n::new(&["en"]),
            bundle,
        })
    }

    pub const fn anki(&self) -> &AnkiI18n {
        &self.anki
    }

    /// Falls back to the id itself, so a missing message never breaks the UI.
    pub fn text(&self, id: &str) -> String {
        let Some(pattern) = self
            .bundle
            .get_message(id)
            .and_then(|message| message.value())
        else {
            tracing::warn!(id, "Missing message");
            return id.to_owned();
        };

        let mut errors = Vec::new();
        let text = self.bundle.format_pattern(pattern, None, &mut errors);

        if !errors.is_empty() {
            tracing::warn!(id, errors = join(&errors), "Failed to format message");
        }

        text.into_owned()
    }
}

fn join(errors: &[impl Display]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_messages_load() {
        assert!(I18n::new().is_ok());
    }

    #[test]
    fn text_resolves_message() {
        let i18n = I18n::new().unwrap();

        assert_eq!(i18n.text("action-quit"), "Quit");
    }

    #[test]
    fn text_falls_back_to_id() {
        let i18n = I18n::new().unwrap();

        assert_eq!(i18n.text("no-such-message"), "no-such-message");
    }
}
