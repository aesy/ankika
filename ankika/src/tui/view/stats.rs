use std::time::{SystemTime, UNIX_EPOCH};

use ankika_core::stats::{self, Period, Retention, Scope, Stats};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Paragraph};

use crate::i18n::I18n;
use crate::tui::stats_screen::StatsScreen;
use crate::tui::theme::Theme;
use crate::tui::view::chart;

const MAX_WIDTH: u16 = 100;
const CHART_ROWS: u16 = 8;
const CALENDAR_DAYS: usize = 365;
const LABEL_WIDTH: u16 = 14;
const NUMBER_WIDTH: u16 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    Today,
    FutureDue,
    Calendar,
    Reviews,
    CardCounts,
    Retention,
}

const SECTIONS: [Section; 6] = [
    Section::Today,
    Section::FutureDue,
    Section::Calendar,
    Section::Reviews,
    Section::CardCounts,
    Section::Retention,
];

pub fn render(frame: &mut Frame<'_>, area: Rect, screen: &StatsScreen, i18n: &I18n, theme: Theme) {
    let tr = i18n.anki();
    let scope = match screen.scope() {
        Scope::Deck => tr.statistics_range_deck(),
        Scope::Collection => tr.statistics_range_collection(),
    };
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme.border))
        .title(format!(
            " {} ({scope}, {}) ",
            tr.statistics_title(),
            period_label(screen.period(), i18n)
        ));
    let inner = block
        .inner(area)
        .inner(Margin::new(2, 1))
        .centered_horizontally(Constraint::Max(MAX_WIDTH));

    frame.render_widget(block, area);

    let Some(stats) = screen.stats() else {
        let text = Line::styled(tr.sync_syncing().into_owned(), theme.muted);

        frame.render_widget(Paragraph::new(text).centered(), inner);
        return;
    };
    let heights: Vec<u16> = SECTIONS
        .iter()
        .map(|&section| height(section, stats))
        .collect();
    let total: u16 = heights.iter().sum();

    screen.set_max_scroll(total.saturating_sub(inner.height));

    let mut top = -i32::from(screen.scroll());

    for (&section, &section_height) in SECTIONS.iter().zip(&heights) {
        let bottom = top + i32::from(section_height);

        // Whole sections only, so charts never show cut off
        if let (Ok(y), true) = (u16::try_from(top), bottom <= i32::from(inner.height)) {
            let section_area = Rect::new(inner.x, inner.y + y, inner.width, section_height);

            render_section(
                frame,
                section_area,
                section,
                stats,
                screen.period(),
                i18n,
                theme,
            );
        }

        top = bottom;
    }
}

const fn height(section: Section, stats: &Stats) -> u16 {
    // Each section is a title, its content and a blank line
    match section {
        Section::Today => 2 + if stats.today.answers > 0 { 4 } else { 1 },
        Section::FutureDue | Section::Reviews => 2 + 1 + CHART_ROWS + 1 + 1,
        Section::Calendar | Section::CardCounts => 2 + 7,
        Section::Retention => 2 + 1 + 1 + 6,
    }
}

fn render_section(
    frame: &mut Frame<'_>,
    area: Rect,
    section: Section,
    stats: &Stats,
    period: Period,
    i18n: &I18n,
    theme: Theme,
) {
    let tr = i18n.anki();
    let title = match section {
        Section::Today => tr.statistics_today_title(),
        Section::FutureDue => tr.statistics_future_due_title(),
        Section::Calendar => tr.statistics_calendar_title(),
        Section::Reviews => tr.statistics_reviews_title(),
        Section::CardCounts => tr.statistics_counts_title(),
        Section::Retention => tr.statistics_true_retention_title(),
    };
    let [title_area, body_area, _] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(area);

    frame.render_widget(
        Line::styled(
            title.into_owned(),
            Style::new().fg(theme.accent).add_modifier(Modifier::BOLD),
        ),
        title_area,
    );

    match section {
        Section::Today => render_today(frame, body_area, stats, i18n, theme),
        Section::FutureDue => render_future_due(frame, body_area, stats, period, i18n, theme),
        Section::Calendar => render_calendar(frame, body_area, stats, theme),
        Section::Reviews => render_reviews(frame, body_area, stats, period, i18n, theme),
        Section::CardCounts => render_card_counts(frame, body_area, stats, i18n, theme),
        Section::Retention => render_retention(frame, body_area, stats, i18n, theme),
    }
}

/// Anki's lines for today, built the way its statistics page builds them.
fn render_today(frame: &mut Frame<'_>, area: Rect, stats: &Stats, i18n: &I18n, theme: Theme) {
    let tr = i18n.anki();
    let today = &stats.today;

    if today.answers == 0 {
        frame.render_widget(
            Line::styled(tr.statistics_today_no_cards().into_owned(), theme.muted),
            area,
        );
        return;
    }

    let again = today.answers.saturating_sub(today.correct);
    let mature = if today.mature > 0 {
        // Rounded like the other percentages; Anki passes the raw value
        let rate = (percent(today.mature_correct, today.mature) * 10.0).round() / 10.0;

        tr.statistics_today_correct_mature(today.mature_correct, today.mature, rate)
            .into_owned()
    } else {
        tr.statistics_today_no_mature_cards().into_owned()
    };
    let lines = vec![
        Line::raw(stats::studied_text(tr, today.answers, today.answer_millis)),
        Line::raw(format!(
            "{} {again} ({:.0}%)",
            tr.statistics_today_again_count(),
            percent(again, today.answers)
        )),
        Line::raw(
            tr.statistics_today_type_counts(
                today.learn,
                today.review,
                today.relearn,
                today.filtered,
            )
            .into_owned(),
        ),
        Line::raw(mature),
    ];

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_future_due(
    frame: &mut Frame<'_>,
    area: Rect,
    stats: &Stats,
    period: Period,
    i18n: &I18n,
    theme: Theme,
) {
    let tr = i18n.anki();
    let last_day = match period {
        Period::AllTime => stats.future_due.keys().copied().max().unwrap_or(0),
        _ => i32::try_from(period.days()).unwrap_or(i32::MAX),
    };
    let days: Vec<u32> = (0..=last_day)
        .map(|day| stats.future_due.get(&day).copied().unwrap_or(0))
        .collect();
    let total: u32 = days.iter().sum();
    let tomorrow = stats.future_due.get(&1).copied().unwrap_or(0);
    let summary = format!(
        "{}: {total}   {}: {tomorrow}",
        tr.statistics_total(),
        tr.statistics_due_tomorrow()
    );

    render_chart(
        frame,
        area,
        &ChartText {
            subtitle: tr.statistics_future_due_subtitle().into_owned(),
            start: tr.statistics_true_retention_today().into_owned(),
            end: format!("+{last_day}d"),
            summary,
        },
        &days,
        theme.review_count,
        theme,
    );
}

fn render_reviews(
    frame: &mut Frame<'_>,
    area: Rect,
    stats: &Stats,
    period: Period,
    i18n: &I18n,
    theme: Theme,
) {
    let tr = i18n.anki();
    let first_day = match period {
        Period::AllTime => stats.reviews.keys().copied().min().unwrap_or(0),
        _ => 1 - i32::try_from(period.days()).unwrap_or(i32::MAX),
    };
    let days: Vec<u32> = (first_day..=0)
        .map(|day| stats.reviews.get(&day).map_or(0, |reviews| reviews.total()))
        .collect();
    let total: u32 = days.iter().sum();
    let studied = days.iter().filter(|count| **count > 0).count();
    let summary = format!(
        "{}: {studied}   {}: {total}",
        tr.statistics_days_studied(),
        tr.statistics_total()
    );

    render_chart(
        frame,
        area,
        &ChartText {
            subtitle: tr.statistics_reviews_count_subtitle().into_owned(),
            start: format!("{first_day}d"),
            end: tr.statistics_true_retention_today().into_owned(),
            summary,
        },
        &days,
        theme.accent,
        theme,
    );
}

struct ChartText {
    subtitle: String,
    start: String,
    end: String,
    summary: String,
}

/// A subtitle, the bars, the first and last day under them, and a summary line.
fn render_chart(
    frame: &mut Frame<'_>,
    area: Rect,
    text: &ChartText,
    days: &[u32],
    color: ratatui::style::Color,
    theme: Theme,
) {
    let [subtitle_area, bars_area, axis_area, summary_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(CHART_ROWS),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(area);
    let values = chart::buckets(days, usize::from(bars_area.width));
    // Bars widen to fill the area, up to a whole number of columns each
    let count = u16::try_from(values.len())
        .unwrap_or(bars_area.width)
        .max(1);
    let used = ((bars_area.width / count).max(1) * count).min(bars_area.width);
    let axis = Rect {
        width: used,
        ..axis_area
    };

    frame.render_widget(
        Line::styled(text.subtitle.as_str(), theme.muted),
        subtitle_area,
    );
    chart::render_bars(
        frame,
        Rect {
            width: used,
            ..bars_area
        },
        &values,
        color,
        theme,
    );
    frame.render_widget(Line::styled(text.start.as_str(), theme.muted), axis);
    frame.render_widget(
        Line::styled(text.end.as_str(), theme.muted).right_aligned(),
        axis,
    );
    frame.render_widget(Line::raw(text.summary.as_str()), summary_area);
}

fn render_calendar(frame: &mut Frame<'_>, area: Rect, stats: &Stats, theme: Theme) {
    let counts: Vec<u32> = (0..CALENDAR_DAYS)
        .map(|days_ago| {
            let day = -i32::try_from(days_ago).unwrap_or(i32::MAX);
            stats.reviews.get(&day).map_or(0, |reviews| reviews.total())
        })
        .collect();

    chart::render_heatmap(frame, area, &counts, weekday_today(), theme);
}

fn render_card_counts(frame: &mut Frame<'_>, area: Rect, stats: &Stats, i18n: &I18n, theme: Theme) {
    let tr = i18n.anki();
    let counts = &stats.card_counts;
    let rows = [
        (
            tr.statistics_counts_new_cards(),
            counts.new,
            theme.new_count,
        ),
        (
            tr.statistics_counts_learning_cards(),
            counts.learn,
            theme.learn_count,
        ),
        (
            tr.statistics_counts_relearning_cards(),
            counts.relearn,
            theme.learn_count,
        ),
        (
            tr.statistics_counts_young_cards(),
            counts.young,
            theme.review_count,
        ),
        (
            tr.statistics_counts_mature_cards(),
            counts.mature,
            theme.review_count,
        ),
        (
            tr.statistics_counts_suspended_cards(),
            counts.suspended,
            theme.muted,
        ),
        (
            tr.statistics_counts_buried_cards(),
            counts.buried,
            theme.muted,
        ),
    ];
    let total: u32 = rows.iter().map(|(_, count, _)| count).sum();
    let row_areas = Layout::vertical([Constraint::Length(1); 7]).split(area);

    for ((label, count, color), &row) in rows.into_iter().zip(row_areas.iter()) {
        let [label_area, count_area, percent_area] = Layout::horizontal([
            Constraint::Length(LABEL_WIDTH),
            Constraint::Length(NUMBER_WIDTH),
            Constraint::Length(NUMBER_WIDTH),
        ])
        .areas(row);

        frame.render_widget(
            Line::styled(label.into_owned(), Style::new().fg(color)),
            label_area,
        );
        frame.render_widget(Line::raw(count.to_string()).right_aligned(), count_area);
        frame.render_widget(
            Line::styled(format!("{:.1}%", percent(count, total)), theme.muted).right_aligned(),
            percent_area,
        );
    }
}

/// Anki's table: pass rates of young, mature and all cards for each range.
fn render_retention(frame: &mut Frame<'_>, area: Rect, stats: &Stats, i18n: &I18n, theme: Theme) {
    let tr = i18n.anki();
    let retention = &stats.retention;
    let ranges = [
        (tr.statistics_true_retention_today(), retention.today),
        (
            tr.statistics_true_retention_yesterday(),
            retention.yesterday,
        ),
        (tr.statistics_true_retention_week(), retention.week),
        (tr.statistics_true_retention_month(), retention.month),
        (tr.statistics_true_retention_year(), retention.year),
        (tr.statistics_true_retention_all_time(), retention.all_time),
    ];
    let [subtitle_area, header_area, rows_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(area);
    let columns = |row: Rect| {
        Layout::horizontal([
            Constraint::Length(LABEL_WIDTH),
            Constraint::Length(NUMBER_WIDTH),
            Constraint::Length(NUMBER_WIDTH),
            Constraint::Length(NUMBER_WIDTH),
        ])
        .split(row)
    };

    frame.render_widget(
        Line::styled(
            tr.statistics_true_retention_subtitle().into_owned(),
            theme.muted,
        ),
        subtitle_area,
    );

    let headers = [
        tr.statistics_true_retention_range(),
        tr.statistics_true_retention_young(),
        tr.statistics_true_retention_mature(),
        tr.statistics_true_retention_all(),
    ];

    for (index, (header, &cell)) in headers
        .into_iter()
        .zip(columns(header_area).iter())
        .enumerate()
    {
        let line = Line::styled(
            header.into_owned(),
            Style::new().fg(theme.muted).add_modifier(Modifier::BOLD),
        );

        frame.render_widget(
            if index == 0 {
                line
            } else {
                line.right_aligned()
            },
            cell,
        );
    }

    let row_areas = Layout::vertical([Constraint::Length(1); 6]).split(rows_area);

    for ((label, range), &row) in ranges.into_iter().zip(row_areas.iter()) {
        let cells = columns(row);
        let all = Retention {
            young_passed: range.young_passed + range.mature_passed,
            young_failed: range.young_failed + range.mature_failed,
            ..Retention::default()
        };
        let rates = [
            rate(range.young_passed, range.young_failed, i18n),
            rate(range.mature_passed, range.mature_failed, i18n),
            rate(all.young_passed, all.young_failed, i18n),
        ];

        frame.render_widget(Line::raw(label.into_owned()), cells[0]);

        for (text, &cell) in rates.into_iter().zip(cells.iter().skip(1)) {
            frame.render_widget(Line::from(Span::raw(text)).right_aligned(), cell);
        }
    }
}

fn rate(passed: u32, failed: u32, i18n: &I18n) -> String {
    let total = passed + failed;

    if total == 0 {
        i18n.anki()
            .statistics_true_retention_not_applicable()
            .into_owned()
    } else {
        format!("{:.1}%", percent(passed, total))
    }
}

/// In `f32`, which Anki's strings take.
#[expect(
    clippy::cast_precision_loss,
    reason = "Anki's strings take f32, and the rounding can't show in a percentage"
)]
fn percent(part: u32, total: u32) -> f32 {
    if total == 0 {
        0.0
    } else {
        part as f32 * 100.0 / total as f32
    }
}

fn period_label(period: Period, i18n: &I18n) -> String {
    match period {
        Period::Month => i18n.text("stats-period-month"),
        Period::ThreeMonths => i18n.text("stats-period-three-months"),
        Period::Year => i18n.text("stats-period-year"),
        Period::AllTime => i18n.anki().statistics_range_all_time().into_owned(),
    }
}

/// 0 for Sunday. In UTC, so it can be a day off near midnight; only the calendar's rows
/// depend on it.
fn weekday_today() -> u16 {
    let days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs() / 86_400);

    // 1970-01-01 was a Thursday
    u16::try_from((days + 4) % 7).unwrap_or(0)
}
