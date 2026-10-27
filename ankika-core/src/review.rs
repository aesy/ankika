use std::time::Duration;

use anki::card::{CardId, CardQueueNumber};
use anki::collection::Collection;
use anki::deckconfig::{DeckConfig, DeckConfigId};
use anki::decks::DeckId;
use anki::error::AnkiError;
use anki::notes::NoteId;
use anki::prelude::I18n;
use anki::scheduler::answering::{CardAnswer, Rating as AnkiRating};
use anki::scheduler::states::SchedulingStates;
use anki::services::CardRenderingService;
use anki::template::RenderedNode;
use anki::timestamp::TimestampMillis;
use anki_proto::card_rendering::av_tag::Value as AvValue;
use anki_proto::card_rendering::{
    CompareAnswerRequest, ExtractAvTagsRequest, ExtractClozeForTypingRequest,
};

use crate::card_actions::{self, CardActionError, Marks};
use crate::remote::english;

#[derive(Debug, thiserror::Error)]
pub enum ReviewError {
    #[error("Failed to select the deck: {}", english(source))]
    SelectDeck {
        #[source]
        source: AnkiError,
    },
    #[error("Failed to load the next card: {}", english(source))]
    LoadQueue {
        #[source]
        source: AnkiError,
    },
    #[error("Failed to render the card: {}", english(source))]
    Render {
        #[source]
        source: AnkiError,
    },
    #[error("Failed to load the deck options: {}", english(source))]
    LoadDeckConfig {
        #[source]
        source: AnkiError,
    },
    #[error("Failed to load the card's note: {}", english(source))]
    LoadNote {
        #[source]
        source: AnkiError,
    },
    #[error("Failed to compare the typed answer: {}", english(source))]
    CompareAnswer {
        #[source]
        source: AnkiError,
    },
    #[error("Failed to answer the card: {}", english(source))]
    Answer {
        #[source]
        source: AnkiError,
    },
    #[error(transparent)]
    LoadMarks(CardActionError),
    #[error("Failed to undo: {}", english(source))]
    Undo {
        #[source]
        source: AnkiError,
    },
    #[error("Failed to load today's study time: {}", english(source))]
    StudiedToday {
        #[source]
        source: AnkiError,
    },
}

impl ReviewError {
    /// Anki's own error, which it can describe in the user's language.
    #[must_use]
    pub const fn anki_error(&self) -> &AnkiError {
        match self {
            Self::SelectDeck { source }
            | Self::LoadQueue { source }
            | Self::Render { source }
            | Self::LoadDeckConfig { source }
            | Self::LoadNote { source }
            | Self::CompareAnswer { source }
            | Self::Answer { source }
            | Self::Undo { source }
            | Self::StudiedToday { source } => source,
            Self::LoadMarks(error) => error.anki_error(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rating {
    Again,
    Hard,
    Good,
    Easy,
}

impl Rating {
    pub const ALL: [Self; 4] = [Self::Again, Self::Hard, Self::Good, Self::Easy];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Queue {
    New,
    Learning,
    Review,
}

/// Cards left today in the current deck, as Anki counts them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Counts {
    pub new: usize,
    pub learning: usize,
    pub review: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Question,
    Answer,
}

/// One of a card's sounds, numbered per side in the order they appear.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoundRef {
    pub side: Side,
    pub index: usize,
}

impl SoundRef {
    /// The reference Anki leaves in the card HTML where the sound was.
    #[must_use]
    pub fn tag(self) -> String {
        let side = match self.side {
            Side::Question => 'q',
            Side::Answer => 'a',
        };

        format!("[anki:play:{side}:{}]", self.index)
    }
}

pub enum Sound {
    /// A file in the media folder.
    File(String),
    /// Text to speak; not supported yet.
    Tts(String),
}

pub struct Sounds {
    pub question: Vec<Sound>,
    pub answer: Vec<Sound>,
    /// The deck options' inverse of "Don't play audio automatically".
    pub autoplay: bool,
    /// The deck options' inverse of "Skip question when replaying answer".
    pub replay_question: bool,
}

impl Sounds {
    #[must_use]
    pub fn side(&self, side: Side) -> &[Sound] {
        match side {
            Side::Question => &self.question,
            Side::Answer => &self.answer,
        }
    }

    #[must_use]
    pub fn get(&self, sound: SoundRef) -> Option<&Sound> {
        self.side(sound.side).get(sound.index)
    }

    /// Every sound on `side`, which is what Anki plays automatically.
    #[must_use]
    pub fn all(&self, side: Side) -> Vec<SoundRef> {
        (0..self.side(side).len())
            .map(|index| SoundRef { side, index })
            .collect()
    }

    /// What Anki's Replay Audio plays on `side`.
    #[must_use]
    pub fn replay(&self, side: Side) -> Vec<SoundRef> {
        match side {
            Side::Answer if self.replay_question => {
                let mut sounds = self.all(Side::Question);
                sounds.extend(self.all(Side::Answer));
                sounds
            },
            _ => self.all(side),
        }
    }
}

/// A card ready to show, with its question and answer as HTML.
pub struct StudyCard {
    pub id: CardId,
    pub note_id: NoteId,
    /// Kept up to date by the caller when it changes them.
    pub marks: Marks,
    pub queue: Queue,
    /// The question, with any type-in field removed; the caller shows its own input.
    /// Sounds are left as their `SoundRef::tag`.
    pub question: String,
    answer: String,
    pub sounds: Sounds,
    pub type_answer: Option<TypeAnswer>,
    /// The next interval for each rating, in `Rating::ALL` order.
    pub intervals: [String; 4],
    states: SchedulingStates,
}

/// The field a `{{type:...}}` replacement asks for.
pub struct TypeAnswer {
    /// `None` when the field does not exist; the question then shows Anki's warning.
    expected: Option<String>,
    combining: bool,
}

impl TypeAnswer {
    #[must_use]
    pub const fn wants_input(&self) -> bool {
        self.expected.is_some()
    }
}

pub struct NextCard {
    pub card: StudyCard,
    pub counts: Counts,
}

/// Anki writes `{{type:Field}}` as this placeholder and leaves the rest to the reviewer.
const TYPE_START: &str = "[[type:";
const TYPE_END: &str = "]]";
const ANSWER_SEPARATOR: &str = "<hr id=answer>";

pub fn select_deck(collection: &mut Collection, deck: DeckId) -> Result<(), ReviewError> {
    collection
        .set_current_deck(deck)
        .map(|_| ())
        .map_err(|source| ReviewError::SelectDeck { source })
}

/// The next card of the current deck, or `None` when it is finished for today.
pub fn next_card(collection: &mut Collection, tr: &I18n) -> Result<Option<NextCard>, ReviewError> {
    let queued = collection
        .get_queued_cards(1, false)
        .map_err(|source| ReviewError::LoadQueue { source })?;
    let counts = Counts {
        new: queued.new_count,
        learning: queued.learning_count,
        review: queued.review_count,
    };
    let Some(queued_card) = queued.cards.into_iter().next() else {
        return Ok(None);
    };
    let card = &queued_card.card;
    // Partial, so that FrontSide gets the question after its sounds became references,
    // which keeps them out of the answer's own sounds like in Anki
    let rendered = collection
        .render_existing_card(card.id(), false, true)
        .map_err(|source| ReviewError::Render { source })?;
    let question = extract_sounds(collection, &join_nodes(&rendered.qnodes, None), true)?;
    let answer = extract_sounds(
        collection,
        &join_nodes(&rendered.anodes, Some(&question.text)),
        false,
    )?;
    let config = deck_config(collection, card.deck_id())?;
    let sounds = Sounds {
        question: question.sounds,
        answer: answer.sounds,
        autoplay: !config.inner.disable_autoplay,
        replay_question: !config.inner.skip_question_when_replaying_answer,
    };
    let (question, answer) = (question.text, answer.text);
    let marks = card_actions::marks(collection, card.id()).map_err(ReviewError::LoadMarks)?;
    let type_answer = type_field(&question)
        .map(|field| load_type_answer(collection, card.note_id(), card.template_idx(), field))
        .transpose()?;
    let question = match &type_answer {
        Some(TypeAnswer { expected: None, .. }) => {
            let field = type_field(&question).unwrap_or_default();
            let warning = if field.starts_with("cloze:") {
                tr.studying_please_run_toolsempty_cards()
            } else {
                tr.studying_type_answer_unknown_field(field)
            };

            replace_type_pattern(&question, &warning)
        },
        _ => replace_type_pattern(&question, ""),
    };
    let intervals = collection
        .describe_next_states(&queued_card.states)
        .map_err(|source| ReviewError::Render { source })?;
    let queue = match card.queue_number() {
        CardQueueNumber::New => Queue::New,
        CardQueueNumber::Review => Queue::Review,
        CardQueueNumber::Learning | CardQueueNumber::Invalid => Queue::Learning,
    };

    Ok(Some(NextCard {
        card: StudyCard {
            id: card.id(),
            note_id: card.note_id(),
            marks,
            queue,
            question,
            answer,
            sounds,
            type_answer,
            intervals: intervals.try_into().unwrap_or_default(),
            states: queued_card.states,
        },
        counts,
    }))
}

impl StudyCard {
    /// The answer side, with the typed answer compared against the expected one the way
    /// Anki's reviewer does it.
    pub fn answer(&self, collection: &mut Collection, typed: &str) -> Result<String, ReviewError> {
        let Some(TypeAnswer {
            expected: Some(expected),
            combining,
        }) = &self.type_answer
        else {
            return Ok(replace_type_pattern(&self.answer, ""));
        };
        let comparison = collection
            .compare_answer(CompareAnswerRequest {
                expected: expected.clone(),
                provided: typed.to_owned(),
                combining: *combining,
            })
            .map_err(|source| ReviewError::CompareAnswer { source })?
            .val;
        let without_separator = self.answer.replace(ANSWER_SEPARATOR, "");
        let had_separator = without_separator.len() != self.answer.len();

        // Keeps the separator before the comparison when the answer repeats the question
        if had_separator && type_field(&without_separator).is_none() {
            return Ok(self.answer.clone());
        }

        let replacement = if had_separator {
            format!("{ANSWER_SEPARATOR}<div>{comparison}</div>")
        } else {
            format!("<div>{comparison}</div>")
        };

        Ok(replace_type_pattern(&without_separator, &replacement))
    }
}

pub fn answer(
    collection: &mut Collection,
    card: &StudyCard,
    rating: Rating,
    taken: Duration,
) -> Result<(), ReviewError> {
    let (new_state, anki_rating) = match rating {
        Rating::Again => (card.states.again, AnkiRating::Again),
        Rating::Hard => (card.states.hard, AnkiRating::Hard),
        Rating::Good => (card.states.good, AnkiRating::Good),
        Rating::Easy => (card.states.easy, AnkiRating::Easy),
    };
    let mut answer = CardAnswer {
        card_id: card.id,
        current_state: card.states.current,
        new_state,
        rating: anki_rating,
        answered_at: TimestampMillis::now(),
        milliseconds_taken: u32::try_from(taken.as_millis()).unwrap_or(u32::MAX),
        custom_data: None,
        from_queue: true,
    };

    collection
        .answer_card(&mut answer)
        .map(|_| ())
        .map_err(|source| ReviewError::Answer { source })
}

/// Undoes the last change, if there is one. Returns whether anything was undone.
pub fn undo(collection: &mut Collection) -> Result<bool, ReviewError> {
    if collection.can_undo().is_none() {
        return Ok(false);
    }

    collection
        .undo()
        .map(|_| true)
        .map_err(|source| ReviewError::Undo { source })
}

/// Anki's "Studied N cards in M minutes today" line.
pub fn studied_today(collection: &mut Collection) -> Result<String, ReviewError> {
    let text = collection
        .studied_today()
        .map_err(|source| ReviewError::StudiedToday { source })?;

    // Anki's message puts its parts on separate lines and relies on HTML to join them
    Ok(text.split_whitespace().collect::<Vec<_>>().join(" "))
}

struct Extracted {
    text: String,
    sounds: Vec<Sound>,
}

/// Fills in `FrontSide`, and keeps other fields as rendered, since only Anki desktop's
/// add-ons define the filters left for it.
fn join_nodes(nodes: &[RenderedNode], front_side: Option<&str>) -> String {
    nodes
        .iter()
        .map(|node| match node {
            RenderedNode::Text { text } => text.as_str(),
            RenderedNode::Replacement { field_name, .. } if field_name == "FrontSide" => {
                front_side.unwrap_or_default()
            },
            RenderedNode::Replacement { current_text, .. } => current_text.as_str(),
        })
        .collect()
}

/// Replaces sound and TTS tags with references and lists what they were.
fn extract_sounds(
    collection: &mut Collection,
    html: &str,
    question_side: bool,
) -> Result<Extracted, ReviewError> {
    let extracted = collection
        .extract_av_tags(ExtractAvTagsRequest {
            text: html.to_owned(),
            question_side,
        })
        .map_err(|source| ReviewError::Render { source })?;
    let sounds = extracted
        .av_tags
        .into_iter()
        .map(|tag| match tag.value {
            Some(AvValue::SoundOrVideo(file)) => Sound::File(file),
            Some(AvValue::Tts(tts)) => Sound::Tts(tts.field_text),
            None => Sound::Tts(String::new()),
        })
        .collect();

    Ok(Extracted {
        text: extracted.text,
        sounds,
    })
}

/// Filtered decks have no options of their own, so they use the default preset.
fn deck_config(collection: &mut Collection, deck: DeckId) -> Result<DeckConfig, ReviewError> {
    let config_id = collection
        .get_deck(deck)
        .map_err(|source| ReviewError::LoadDeckConfig { source })?
        .and_then(|deck| deck.config_id())
        .unwrap_or(DeckConfigId(1));

    // With the fallback, rslib always returns a config, at worst its defaults
    collection
        .get_deck_config(config_id, true)
        .map(Option::unwrap_or_default)
        .map_err(|source| ReviewError::LoadDeckConfig { source })
}

fn load_type_answer(
    collection: &mut Collection,
    note_id: NoteId,
    template_idx: u16,
    field: &str,
) -> Result<TypeAnswer, ReviewError> {
    let (cloze, field) = field
        .strip_prefix("cloze:")
        .map_or((false, field), |rest| (true, rest));
    let (combining, field) = field
        .strip_prefix("nc:")
        .map_or((true, field), |rest| (false, rest));
    let note = collection
        .storage
        .get_note(note_id)
        .map_err(|source| ReviewError::LoadNote { source })?;
    let notetype = match &note {
        Some(note) => collection
            .get_notetype(note.notetype_id)
            .map_err(|source| ReviewError::LoadNote { source })?,
        None => None,
    };
    let value = note.as_ref().zip(notetype).and_then(|(note, notetype)| {
        let index = notetype.fields.iter().position(|f| f.name == field)?;
        note.fields().get(index).cloned()
    });
    let expected = match value {
        Some(text) if cloze => {
            let text = collection
                .extract_cloze_for_typing(ExtractClozeForTypingRequest {
                    text,
                    ordinal: u32::from(template_idx) + 1,
                })
                .map_err(|source| ReviewError::Render { source })?
                .val;

            (!text.is_empty()).then_some(text)
        },
        other => other,
    };

    Ok(TypeAnswer {
        expected,
        combining,
    })
}

fn type_field(html: &str) -> Option<&str> {
    let start = html.find(TYPE_START)? + TYPE_START.len();
    let length = html[start..].find(TYPE_END)?;

    Some(&html[start..start + length])
}

fn replace_type_pattern(html: &str, replacement: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut rest = html;

    while let Some(start) = rest.find(TYPE_START) {
        let Some(length) = rest[start..].find(TYPE_END) else {
            break;
        };

        result.push_str(&rest[..start]);
        result.push_str(replacement);
        rest = &rest[start + length + TYPE_END.len()..];
    }

    result.push_str(rest);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sounds(replay_question: bool) -> Sounds {
        Sounds {
            question: vec![Sound::File("q.mp3".to_owned())],
            answer: vec![Sound::File("a.mp3".to_owned())],
            autoplay: true,
            replay_question,
        }
    }

    #[test]
    fn replays_question_before_answer_unless_skipped() {
        let question = SoundRef {
            side: Side::Question,
            index: 0,
        };
        let answer = SoundRef {
            side: Side::Answer,
            index: 0,
        };

        assert_eq!(sounds(true).replay(Side::Answer), vec![question, answer]);
        assert_eq!(sounds(false).replay(Side::Answer), vec![answer]);
        assert_eq!(sounds(false).replay(Side::Question), vec![question]);
    }

    #[test]
    fn finds_type_field() {
        assert_eq!(type_field("a [[type:cloze:Text]] b"), Some("cloze:Text"));
        assert_eq!(type_field("no field"), None);
    }

    #[test]
    fn replaces_every_type_pattern() {
        assert_eq!(
            replace_type_pattern("[[type:A]] x [[type:B]]", "-"),
            "- x -"
        );
    }
}
