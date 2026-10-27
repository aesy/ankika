use ankika_core::deck_options::{self, DeckOptions};

use crate::i18n::I18n;
use crate::tui::form::{Field, Form, Value};

// The fields' positions in the form, in the order of Anki's options screen
const NEW_PER_DAY: usize = 0;
const REVIEWS_PER_DAY: usize = 1;
const LEARN_STEPS: usize = 2;
const GRADUATING_INTERVAL: usize = 3;
const EASY_INTERVAL: usize = 4;
const RELEARN_STEPS: usize = 5;
const BURY_NEW: usize = 6;
const BURY_REVIEWS: usize = 7;
const BURY_INTERDAY: usize = 8;
const FSRS: usize = 9;
const DESIRED_RETENTION: usize = 10;

/// Anki's limits for desired retention.
const RETENTION_RANGE: std::ops::RangeInclusive<f32> = 0.7..=0.99;

pub fn new(deck_name: &str, options: &DeckOptions, i18n: &I18n) -> Form {
    let tr = i18n.anki();
    let text = |label: std::borrow::Cow<'_, str>, value: String| Field {
        label: label.into_owned(),
        value: Value::Text(value),
    };
    let toggle = |label: std::borrow::Cow<'_, str>, on: bool| Field {
        label: label.into_owned(),
        value: Value::Toggle(on),
    };
    let fields = vec![
        text(
            tr.scheduling_new_cardsday(),
            options.new_per_day.to_string(),
        ),
        text(
            tr.scheduling_maximum_reviewsday(),
            options.reviews_per_day.to_string(),
        ),
        text(
            tr.deck_config_learning_steps(),
            deck_options::format_steps(&options.learn_steps),
        ),
        text(
            tr.scheduling_graduating_interval(),
            options.graduating_interval.to_string(),
        ),
        text(
            tr.scheduling_easy_interval(),
            options.easy_interval.to_string(),
        ),
        text(
            tr.deck_config_relearning_steps(),
            deck_options::format_steps(&options.relearn_steps),
        ),
        toggle(tr.deck_config_bury_new_siblings(), options.burying.new),
        toggle(
            tr.deck_config_bury_review_siblings(),
            options.burying.reviews,
        ),
        toggle(
            tr.deck_config_bury_interday_learning_siblings(),
            options.burying.interday_learning,
        ),
        toggle(i18n.text("deck-options-fsrs").into(), options.fsrs),
        text(
            tr.deck_config_desired_retention(),
            format!("{:.2}", options.desired_retention),
        ),
    ];
    let title = format!(
        "{}: {} ({})",
        tr.actions_options_for(deck_name),
        options.preset_name,
        tr.deck_config_used_by_decks(options.preset_use_count)
    );
    let mut form = Form::new(title, fields);

    form.note = Some(i18n.text("deck-options-preset-note"));
    form
}

/// Takes the form's values, or names the first field that isn't valid.
pub fn apply(form: &Form, options: &mut DeckOptions) -> Result<(), String> {
    let invalid = |index: usize| form.fields()[index].label.clone();
    let number = |index: usize| {
        form.text(index)
            .trim()
            .parse::<u32>()
            .map_err(|_| invalid(index))
    };
    let steps =
        |index: usize| deck_options::parse_steps(form.text(index)).ok_or_else(|| invalid(index));
    let retention = form
        .text(DESIRED_RETENTION)
        .trim()
        .parse::<f32>()
        .ok()
        .filter(|value| RETENTION_RANGE.contains(value))
        .ok_or_else(|| invalid(DESIRED_RETENTION))?;

    options.new_per_day = number(NEW_PER_DAY)?;
    options.reviews_per_day = number(REVIEWS_PER_DAY)?;
    options.learn_steps = steps(LEARN_STEPS)?;
    options.graduating_interval = number(GRADUATING_INTERVAL)?;
    options.easy_interval = number(EASY_INTERVAL)?;
    options.relearn_steps = steps(RELEARN_STEPS)?;
    options.burying.new = form.toggle(BURY_NEW);
    options.burying.reviews = form.toggle(BURY_REVIEWS);
    options.burying.interday_learning = form.toggle(BURY_INTERDAY);
    options.fsrs = form.toggle(FSRS);
    options.desired_retention = retention;

    Ok(())
}
