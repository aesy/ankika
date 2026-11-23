use std::collections::BTreeMap;

use anki::collection::Collection;
use anki::error::AnkiError;
use anki::prelude::I18n;
use anki::services::StatsService;
use anki_proto::stats::GraphsRequest;
use anki_proto::stats::graphs_response::true_retention_stats::TrueRetention;

use crate::remote::english;

#[derive(Debug, thiserror::Error)]
#[error("Failed to load the statistics: {}", english(source))]
pub struct StatsError {
    #[source]
    source: AnkiError,
}

impl StatsError {
    /// Anki's own error, which it can describe in the user's language.
    #[must_use]
    pub const fn anki_error(&self) -> &AnkiError {
        &self.source
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Deck,
    Collection,
}

/// The ranges of Anki's statistics page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Period {
    Month,
    ThreeMonths,
    Year,
    AllTime,
}

impl Period {
    pub const ALL: [Self; 4] = [Self::Month, Self::ThreeMonths, Self::Year, Self::AllTime];

    /// How many days of reviews to load, and of future due to show; 0 loads all reviews.
    #[must_use]
    pub const fn days(self) -> u32 {
        match self {
            Self::Month => 31,
            Self::ThreeMonths => 92,
            Self::Year => 365,
            Self::AllTime => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Today {
    pub answers: u32,
    pub answer_millis: u32,
    pub correct: u32,
    pub mature_correct: u32,
    pub mature: u32,
    pub learn: u32,
    pub review: u32,
    pub relearn: u32,
    pub filtered: u32,
}

/// One day's reviews by kind.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Reviews {
    pub learn: u32,
    pub relearn: u32,
    pub young: u32,
    pub mature: u32,
    pub filtered: u32,
}

impl Reviews {
    #[must_use]
    pub const fn total(self) -> u32 {
        self.learn + self.relearn + self.young + self.mature + self.filtered
    }
}

/// With suspended and buried cards counted separately, like Anki's default.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CardCounts {
    pub new: u32,
    pub learn: u32,
    pub relearn: u32,
    pub young: u32,
    pub mature: u32,
    pub suspended: u32,
    pub buried: u32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Retention {
    pub young_passed: u32,
    pub young_failed: u32,
    pub mature_passed: u32,
    pub mature_failed: u32,
}

/// Anki's true retention ranges, in its order.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TrueRetentionStats {
    pub today: Retention,
    pub yesterday: Retention,
    pub week: Retention,
    pub month: Retention,
    pub year: Retention,
    pub all_time: Retention,
}

pub struct Stats {
    pub today: Today,
    /// Reviews due per day from today, which is day 0.
    pub future_due: BTreeMap<i32, u32>,
    /// Whether overdue reviews are included as negative days.
    pub has_backlog: bool,
    /// Reviews per day in the period, with today as day 0 and earlier days negative.
    pub reviews: BTreeMap<i32, Reviews>,
    pub card_counts: CardCounts,
    pub retention: TrueRetentionStats,
}

/// What Anki's statistics page computes for the scope and period.
pub fn load(
    collection: &mut Collection,
    scope: Scope,
    period: Period,
) -> Result<Stats, StatsError> {
    let search = match scope {
        Scope::Deck => "deck:current",
        Scope::Collection => "",
    };
    let graphs = collection
        .graphs(GraphsRequest {
            search: search.to_owned(),
            days: period.days(),
        })
        .map_err(|source| StatsError { source })?;
    let today = graphs.today.unwrap_or_default();
    let future_due = graphs.future_due.unwrap_or_default();
    let counts = graphs
        .card_counts
        .and_then(|counts| counts.excluding_inactive)
        .unwrap_or_default();
    let retention = graphs.true_retention.unwrap_or_default();
    let reviews = graphs.reviews.map_or_default(|reviews| {
        reviews
            .count
            .into_iter()
            .map(|(day, count)| {
                (
                    day,
                    Reviews {
                        learn: count.learn,
                        relearn: count.relearn,
                        young: count.young,
                        mature: count.mature,
                        filtered: count.filtered,
                    },
                )
            })
            .collect()
    });

    Ok(Stats {
        today: Today {
            answers: today.answer_count,
            answer_millis: today.answer_millis,
            correct: today.correct_count,
            mature_correct: today.mature_correct,
            mature: today.mature_count,
            learn: today.learn_count,
            review: today.review_count,
            relearn: today.relearn_count,
            filtered: today.early_review_count,
        },
        future_due: future_due.future_due.into_iter().collect(),
        has_backlog: future_due.have_backlog,
        reviews,
        card_counts: CardCounts {
            new: counts.new_cards,
            learn: counts.learn,
            relearn: counts.relearn,
            young: counts.young,
            mature: counts.mature,
            suspended: counts.suspended,
            buried: counts.buried,
        },
        retention: TrueRetentionStats {
            today: retention.today.map_or_default(Retention::from),
            yesterday: retention.yesterday.map_or_default(Retention::from),
            week: retention.week.map_or_default(Retention::from),
            month: retention.month.map_or_default(Retention::from),
            year: retention.year.map_or_default(Retention::from),
            all_time: retention.all_time.map_or_default(Retention::from),
        },
    })
}

/// Anki's "Studied N cards in M minutes today" line. rslib keeps its own version private:
/// seconds below a minute, minutes otherwise.
#[must_use]
#[expect(
    clippy::cast_precision_loss,
    reason = "Anki's strings take f32, and the rounding can't show in text"
)]
pub fn studied_text(tr: &I18n, cards: u32, millis: u32) -> String {
    let secs = millis as f32 / 1000.0;
    let (unit, amount) = if secs < 60.0 {
        ("seconds", secs)
    } else {
        ("minutes", secs / 60.0)
    };
    let per_card = if cards > 0 { secs / cards as f32 } else { 0.0 };
    let text = tr.statistics_studied_today(unit, per_card, amount, cards);

    // Anki's message puts its parts on separate lines and relies on HTML to join them
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

impl From<TrueRetention> for Retention {
    fn from(retention: TrueRetention) -> Self {
        Self {
            young_passed: retention.young_passed,
            young_failed: retention.young_failed,
            mature_passed: retention.mature_passed,
            mature_failed: retention.mature_failed,
        }
    }
}
