use anki::collection::Collection;
use anki::decks::{DeckId, DeckKind};
use anki::error::AnkiError;
use anki::prelude::I18n;
use anki::services::CardRenderingService;
use anki_proto::card_rendering::RenderMarkdownRequest;
use anki_proto::scheduler::CustomStudyRequest;
use anki_proto::scheduler::custom_study_request::cram::CramKind;
use anki_proto::scheduler::custom_study_request::{Cram, Value};
use anki_proto::scheduler::unbury_deck_request::Mode;

use crate::remote::english;
use crate::review::Counts;

#[derive(Debug, thiserror::Error)]
pub enum OverviewError {
    #[error("Failed to load the deck: {}", english(source))]
    Load {
        #[source]
        source: AnkiError,
    },
    #[error("The deck no longer exists")]
    Missing,
    #[error("Failed to unbury the cards: {}", english(source))]
    Unbury {
        #[source]
        source: AnkiError,
    },
    #[error("Failed to create the custom study session: {}", english(source))]
    CustomStudy {
        #[source]
        source: AnkiError,
    },
    #[error("Failed to rebuild the deck: {}", english(source))]
    Rebuild {
        #[source]
        source: AnkiError,
    },
    #[error("Failed to empty the deck: {}", english(source))]
    Empty {
        #[source]
        source: AnkiError,
    },
}

impl OverviewError {
    /// Anki's own error, which it can describe in the user's language.
    #[must_use]
    pub const fn anki_error(&self) -> Option<&AnkiError> {
        match self {
            Self::Load { source }
            | Self::Unbury { source }
            | Self::CustomStudy { source }
            | Self::Rebuild { source }
            | Self::Empty { source } => Some(source),
            Self::Missing => None,
        }
    }
}

/// What Anki's deck overview shows. Loading it makes the deck Anki's current deck.
pub struct Overview {
    pub deck: DeckId,
    pub name: String,
    /// HTML, with Markdown descriptions already converted.
    pub description: String,
    pub filtered: bool,
    /// What studying would show now, with burying applied.
    pub counts: Counts,
    pub have_user_buried: bool,
    pub have_sched_buried: bool,
}

pub fn load(
    collection: &mut Collection,
    deck: DeckId,
    tr: &I18n,
) -> Result<Overview, OverviewError> {
    let load_error = |source| OverviewError::Load { source };

    collection.set_current_deck(deck).map_err(load_error)?;

    let current = collection
        .get_deck(deck)
        .map_err(load_error)?
        .ok_or(OverviewError::Missing)?;
    let filtered = matches!(current.kind, DeckKind::Filtered(_));
    let description = match &current.kind {
        DeckKind::Filtered(_) => [
            tr.studying_this_is_a_special_deck_for(),
            tr.studying_cards_will_be_automatically_returned_to(),
            tr.studying_deleting_this_deck_from_the_deck(),
        ]
        .join(" "),
        DeckKind::Normal(normal) if normal.markdown_description => {
            collection
                .render_markdown(RenderMarkdownRequest {
                    markdown: normal.description.clone(),
                    sanitize: true,
                })
                .map_err(load_error)?
                .val
        },
        DeckKind::Normal(normal) => normal.description.clone(),
    };
    let queued = collection.get_queued_cards(0, false).map_err(load_error)?;
    let congrats = collection.congrats_info().map_err(load_error)?;

    Ok(Overview {
        deck,
        name: current.human_name(),
        description,
        filtered,
        counts: Counts {
            new: queued.new_count,
            learning: queued.learning_count,
            review: queued.review_count,
        },
        have_user_buried: congrats.have_user_buried,
        have_sched_buried: congrats.have_sched_buried,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unbury {
    /// Buried with the reviewer's bury actions.
    Manual,
    /// Buried automatically because a sibling was studied.
    Siblings,
    All,
}

pub fn unbury(
    collection: &mut Collection,
    deck: DeckId,
    which: Unbury,
) -> Result<(), OverviewError> {
    let mode = match which {
        Unbury::Manual => Mode::UserOnly,
        Unbury::Siblings => Mode::SchedOnly,
        Unbury::All => Mode::All,
    };

    collection
        .unbury_deck(deck, mode)
        .map(|_| ())
        .map_err(|source| OverviewError::Unbury { source })
}

pub fn rebuild(collection: &mut Collection, deck: DeckId) -> Result<(), OverviewError> {
    collection
        .rebuild_filtered_deck(deck)
        .map(|_| ())
        .map_err(|source| OverviewError::Rebuild { source })
}

/// Sends the cards back to their home decks.
pub fn empty(collection: &mut Collection, deck: DeckId) -> Result<(), OverviewError> {
    collection
        .empty_filtered_deck(deck)
        .map(|_| ())
        .map_err(|source| OverviewError::Empty { source })
}

/// The choices of Anki's Custom Study dialog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomStudy {
    IncreaseNewLimit(i32),
    IncreaseReviewLimit(i32),
    ForgottenInLastDays(u32),
    AheadDays(u32),
    PreviewAddedInLastDays(u32),
    ByStateOrTag {
        kind: StudyKind,
        limit: u32,
        include_tags: Vec<String>,
        exclude_tags: Vec<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudyKind {
    NewOnly,
    DueOnly,
    AllReviewRandom,
    /// Without rescheduling.
    AllRandom,
}

/// What Anki suggests and has available for the dialog.
pub struct CustomStudyDefaults {
    pub extend_new: u32,
    pub extend_review: u32,
    pub available_new: u32,
    pub available_review: u32,
}

pub fn custom_study_defaults(
    collection: &mut Collection,
    deck: DeckId,
) -> Result<CustomStudyDefaults, OverviewError> {
    let defaults = collection
        .custom_study_defaults(deck)
        .map_err(|source| OverviewError::Load { source })?;

    Ok(CustomStudyDefaults {
        extend_new: defaults.extend_new,
        extend_review: defaults.extend_review,
        available_new: defaults.available_new,
        available_review: defaults.available_review,
    })
}

/// Returns the deck to show next: the same one for limit changes, otherwise Anki's
/// "Custom Study Session" filtered deck.
pub fn custom_study(
    collection: &mut Collection,
    deck: DeckId,
    study: CustomStudy,
    tr: &I18n,
) -> Result<DeckId, OverviewError> {
    let error = |source| OverviewError::CustomStudy { source };
    let creates_deck = !matches!(
        study,
        CustomStudy::IncreaseNewLimit(_) | CustomStudy::IncreaseReviewLimit(_)
    );
    let value = match study {
        CustomStudy::IncreaseNewLimit(delta) => Value::NewLimitDelta(delta),
        CustomStudy::IncreaseReviewLimit(delta) => Value::ReviewLimitDelta(delta),
        CustomStudy::ForgottenInLastDays(days) => Value::ForgotDays(days),
        CustomStudy::AheadDays(days) => Value::ReviewAheadDays(days),
        CustomStudy::PreviewAddedInLastDays(days) => Value::PreviewDays(days),
        CustomStudy::ByStateOrTag {
            kind,
            limit,
            include_tags,
            exclude_tags,
        } => Value::Cram(Cram {
            kind: match kind {
                StudyKind::NewOnly => CramKind::New,
                StudyKind::DueOnly => CramKind::Due,
                StudyKind::AllReviewRandom => CramKind::Review,
                StudyKind::AllRandom => CramKind::All,
            }
            .into(),
            card_limit: limit,
            tags_to_include: include_tags,
            tags_to_exclude: exclude_tags,
        }),
    };

    collection
        .custom_study(CustomStudyRequest {
            deck_id: deck.0,
            value: Some(value),
        })
        .map_err(error)?;

    if !creates_deck {
        return Ok(deck);
    }

    collection
        .get_deck_id(&tr.custom_study_custom_study_session())
        .map_err(error)?
        .ok_or(OverviewError::Missing)
}
