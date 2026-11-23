use anki::collection::Collection;
use anki::decks::DeckId;
use anki::error::AnkiError;
use anki::services::DeckConfigService;
use anki_proto::deck_config::deck_configs_for_update::current_deck::Limits;
use anki_proto::deck_config::{DeckConfig, UpdateDeckConfigsMode, UpdateDeckConfigsRequest};

use crate::remote::english;

#[derive(Debug, thiserror::Error)]
pub enum DeckOptionsError {
    #[error("Failed to load the deck options: {}", english(source))]
    Load {
        #[source]
        source: AnkiError,
    },
    #[error("The deck has no options preset")]
    MissingPreset,
    #[error("Failed to save the deck options: {}", english(source))]
    Save {
        #[source]
        source: AnkiError,
    },
}

impl DeckOptionsError {
    /// Anki's own error, which it can describe in the user's language.
    #[must_use]
    pub const fn anki_error(&self) -> Option<&AnkiError> {
        match self {
            Self::Load { source } | Self::Save { source } => Some(source),
            Self::MissingPreset => None,
        }
    }
}

/// The options of a deck's preset that ankika edits, and what Anki's options screen sends
/// back unchanged.
pub struct DeckOptions {
    pub preset_name: String,
    /// How many decks share the preset, and so the changes.
    pub preset_use_count: u32,
    pub new_per_day: u32,
    pub reviews_per_day: u32,
    /// In minutes, like Anki stores them.
    pub learn_steps: Vec<f32>,
    pub relearn_steps: Vec<f32>,
    pub graduating_interval: u32,
    pub easy_interval: u32,
    pub burying: Burying,
    /// Collection-wide in Anki, unlike the rest.
    pub fsrs: bool,
    pub desired_retention: f32,
    preset: DeckConfig,
    unchanged: Unchanged,
}

/// Which siblings of a studied card wait until tomorrow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Burying {
    pub new: bool,
    pub reviews: bool,
    pub interday_learning: bool,
}

/// What Anki's options screen sends back with a save, which ankika doesn't edit.
struct Unchanged {
    limits: Limits,
    card_state_customizer: String,
    new_cards_ignore_review_limit: bool,
    apply_all_parent_limits: bool,
    fsrs_health_check: bool,
}

pub fn load(collection: &mut Collection, deck: DeckId) -> Result<DeckOptions, DeckOptionsError> {
    // The service takes and returns the protobuf types the options screen works with
    let update = DeckConfigService::get_deck_configs_for_update(
        collection,
        anki_proto::decks::DeckId { did: deck.0 },
    )
    .map_err(|source| DeckOptionsError::Load { source })?;
    let current = update.current_deck.unwrap_or_default();
    let with_extra = update
        .all_config
        .into_iter()
        .find(|config| {
            config
                .config
                .as_ref()
                .is_some_and(|config| config.id == current.config_id)
        })
        .ok_or(DeckOptionsError::MissingPreset)?;
    let preset = with_extra.config.ok_or(DeckOptionsError::MissingPreset)?;
    let inner = preset.config.clone().unwrap_or_default();

    Ok(DeckOptions {
        preset_name: preset.name.clone(),
        preset_use_count: with_extra.use_count,
        new_per_day: inner.new_per_day,
        reviews_per_day: inner.reviews_per_day,
        learn_steps: inner.learn_steps,
        relearn_steps: inner.relearn_steps,
        graduating_interval: inner.graduating_interval_good,
        easy_interval: inner.graduating_interval_easy,
        burying: Burying {
            new: inner.bury_new,
            reviews: inner.bury_reviews,
            interday_learning: inner.bury_interday_learning,
        },
        fsrs: update.fsrs,
        desired_retention: inner.desired_retention,
        preset,
        unchanged: Unchanged {
            limits: current.limits.unwrap_or_default(),
            card_state_customizer: update.card_state_customizer,
            new_cards_ignore_review_limit: update.new_cards_ignore_review_limit,
            apply_all_parent_limits: update.apply_all_parent_limits,
            fsrs_health_check: update.fsrs_health_check,
        },
    })
}

/// Saves into the preset, so every deck using it changes, like in Anki. Undoable.
pub fn save(
    collection: &mut Collection,
    deck: DeckId,
    options: DeckOptions,
) -> Result<(), DeckOptionsError> {
    let unchanged = options.unchanged;
    let mut preset = options.preset;
    let mut inner = preset.config.unwrap_or_default();

    inner.new_per_day = options.new_per_day;
    inner.reviews_per_day = options.reviews_per_day;
    inner.learn_steps = options.learn_steps;
    inner.relearn_steps = options.relearn_steps;
    inner.graduating_interval_good = options.graduating_interval;
    inner.graduating_interval_easy = options.easy_interval;
    inner.bury_new = options.burying.new;
    inner.bury_reviews = options.burying.reviews;
    inner.bury_interday_learning = options.burying.interday_learning;
    inner.desired_retention = options.desired_retention;
    preset.config = Some(inner);

    DeckConfigService::update_deck_configs(
        collection,
        UpdateDeckConfigsRequest {
            target_deck_id: deck.0,
            configs: vec![preset],
            removed_config_ids: Vec::new(),
            mode: UpdateDeckConfigsMode::Normal.into(),
            card_state_customizer: unchanged.card_state_customizer,
            limits: Some(unchanged.limits),
            new_cards_ignore_review_limit: unchanged.new_cards_ignore_review_limit,
            fsrs: options.fsrs,
            apply_all_parent_limits: unchanged.apply_all_parent_limits,
            // Only Anki's optimizer reschedules; ankika doesn't run it
            fsrs_reschedule: false,
            fsrs_health_check: unchanged.fsrs_health_check,
        },
    )
    .map(|_| ())
    .map_err(|source| DeckOptionsError::Save { source })
}

const MINUTES_PER_HOUR: f32 = 60.0;
const MINUTES_PER_DAY: f32 = 1440.0;

/// Steps in Anki's notation, like `1m 10m 1h 2d`; a bare number means minutes. `None`
/// when a step isn't a positive duration.
#[must_use]
pub fn parse_steps(text: &str) -> Option<Vec<f32>> {
    text.split_whitespace()
        .map(|step| {
            let (number, unit) = step
                .find(|character: char| character.is_ascii_alphabetic())
                .map_or((step, "m"), |index| step.split_at(index));
            let value: f32 = number.parse().ok()?;
            let minutes = match unit {
                "s" => value / 60.0,
                "m" => value,
                "h" => value * MINUTES_PER_HOUR,
                "d" => value * MINUTES_PER_DAY,
                _ => return None,
            };

            (minutes > 0.0 && minutes.is_finite()).then_some(minutes)
        })
        .collect()
}

/// The largest unit that keeps a step whole, like Anki's options screen shows them.
#[must_use]
pub fn format_steps(steps: &[f32]) -> String {
    steps
        .iter()
        .map(|&minutes| {
            if minutes < 1.0 {
                format!("{}s", trim(minutes * 60.0))
            } else if minutes % MINUTES_PER_DAY == 0.0 {
                format!("{}d", trim(minutes / MINUTES_PER_DAY))
            } else if minutes % MINUTES_PER_HOUR == 0.0 {
                format!("{}h", trim(minutes / MINUTES_PER_HOUR))
            } else {
                format!("{}m", trim(minutes))
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Whole numbers without a trailing `.0`.
fn trim(value: f32) -> String {
    let text = format!("{value:.2}");
    let text = text.trim_end_matches('0').trim_end_matches('.');

    text.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_anki_steps() {
        assert_eq!(
            parse_steps("1m 10 1h 2d 30s"),
            Some(vec![1.0, 10.0, 60.0, 2880.0, 0.5])
        );
        assert_eq!(parse_steps(""), Some(vec![]));
        assert_eq!(parse_steps("1x"), None);
        assert_eq!(parse_steps("0m"), None);
    }

    #[test]
    fn formats_with_the_largest_whole_unit() {
        assert_eq!(
            format_steps(&[1.0, 10.0, 60.0, 2880.0, 0.5, 90.0]),
            "1m 10m 1h 2d 30s 90m"
        );
    }
}
