use ankika_core::overview::StudyKind;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::i18n::I18n;
use crate::tui::custom_study_form::{Choice, CustomStudyForm, Field, KINDS};
use crate::tui::hit_map::{Hit, HitMap};
use crate::tui::state::State;
use crate::tui::theme::Theme;
use crate::tui::view::buttons::{self, Button};

const WIDTH: u16 = 72;
const AMOUNT_WIDTH: u16 = 8;
// Borders and margins
const FRAME_ROWS: u16 = 4;

/// One row of the form, top to bottom.
enum Row {
    Choice(Choice),
    Amount,
    Info(String),
    Kind(StudyKind),
    Label(String),
    Tags(Field),
    Error,
    Blank,
    Buttons,
}

pub fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    form: &CustomStudyForm,
    state: &State,
    i18n: &I18n,
    theme: Theme,
    hits: &mut HitMap,
) {
    let rows = rows(form, i18n);
    let height = u16::try_from(rows.len())
        .unwrap_or(u16::MAX)
        .saturating_add(FRAME_ROWS);
    let popup = area.centered(Constraint::Length(WIDTH), Constraint::Length(height));
    let title = i18n.anki().actions_custom_study();
    let inner = super::overlay(frame, popup, &title, theme, hits).inner(Margin::new(2, 1));
    let areas = Layout::vertical(vec![Constraint::Length(1); rows.len()]).split(inner);

    for (row, &row_area) in rows.iter().zip(areas.iter()) {
        match row {
            Row::Choice(choice) => {
                let line = radio(
                    form.choice() == *choice,
                    form.focus() == Field::Choice && form.choice() == *choice,
                    choice_label(*choice, i18n),
                    theme,
                );

                frame.render_widget(line, row_area);
                hits.register(row_area, Hit::CustomStudyChoice(*choice));
            },
            Row::Kind(kind) => {
                let line = radio(
                    form.kind() == *kind,
                    form.focus() == Field::Kind && form.kind() == *kind,
                    kind_label(*kind, i18n),
                    theme,
                );

                frame.render_widget(line, row_area);
                hits.register(row_area, Hit::CustomStudyKind(*kind));
            },
            Row::Amount => render_amount(frame, row_area, form, i18n, theme, hits),
            Row::Info(text) => {
                frame.render_widget(Line::styled(text.as_str(), theme.muted), row_area);
            },
            Row::Label(text) => frame.render_widget(Line::raw(text.as_str()), row_area),
            Row::Tags(field) => {
                let text = match field {
                    Field::IncludeTags => form.include_tags(),
                    _ => form.exclude_tags(),
                };

                render_input(frame, row_area, text, form.focus() == *field, theme);
                hits.register(row_area, Hit::CustomStudyField(*field));
            },
            Row::Error => frame.render_widget(
                Line::styled(i18n.text("custom-study-invalid-amount"), theme.error),
                row_area,
            ),
            Row::Blank => {},
            Row::Buttons => {
                let row: Vec<Button> = [
                    (Field::Ok, i18n.anki().custom_study_ok()),
                    (Field::Cancel, i18n.anki().actions_cancel()),
                ]
                .into_iter()
                .map(|(field, label)| Button {
                    label: label.into_owned(),
                    hit: Hit::CustomStudyField(field),
                    enabled: true,
                    focused: form.focus() == field,
                })
                .collect();

                buttons::render_right_aligned(frame, row_area, &row, state.mouse(), theme, hits);
            },
        }
    }
}

fn rows(form: &CustomStudyForm, i18n: &I18n) -> Vec<Row> {
    let tr = i18n.anki();
    let defaults = form.defaults();
    let mut rows: Vec<Row> = Choice::ALL.into_iter().map(Row::Choice).collect();

    rows.extend([Row::Blank, Row::Amount]);

    match form.choice() {
        Choice::NewLimit => rows.push(Row::Info(
            tr.custom_study_available_new_cards(defaults.available_new)
                .into_owned(),
        )),
        Choice::ReviewLimit => rows.push(Row::Info(
            tr.custom_study_available_review_cards(defaults.available_review)
                .into_owned(),
        )),
        Choice::StateOrTag => {
            rows.push(Row::Blank);
            rows.extend(KINDS.into_iter().map(Row::Kind));
            rows.extend([
                Row::Blank,
                Row::Label(tr.custom_study_require_one_or_more_of_these().into_owned()),
                Row::Tags(Field::IncludeTags),
                Row::Label(tr.custom_study_select_tags_to_exclude().into_owned()),
                Row::Tags(Field::ExcludeTags),
            ]);
        },
        Choice::Forgotten | Choice::Ahead | Choice::Preview => {},
    }

    if form.is_invalid() {
        rows.push(Row::Error);
    }

    rows.extend([Row::Blank, Row::Buttons]);
    rows
}

/// The sentence around the number, like "Review ahead by [ 1 ] days".
fn render_amount(
    frame: &mut Frame<'_>,
    area: Rect,
    form: &CustomStudyForm,
    i18n: &I18n,
    theme: Theme,
    hits: &mut HitMap,
) {
    let tr = i18n.anki();
    let count = form.amount().parse::<i64>().unwrap_or(0);
    let (before, after) = match form.choice() {
        Choice::NewLimit => (
            tr.custom_study_increase_todays_new_card_limit_by(),
            tr.custom_study_cards(count),
        ),
        Choice::ReviewLimit => (
            tr.custom_study_increase_todays_review_limit_by(),
            tr.custom_study_cards(count),
        ),
        Choice::Forgotten => (
            tr.custom_study_review_cards_forgotten_in_last(),
            tr.custom_study_days(count),
        ),
        Choice::Ahead => (
            tr.custom_study_review_ahead_by(),
            tr.custom_study_days(count),
        ),
        Choice::Preview => (
            tr.custom_study_preview_new_cards_added_in_the(),
            tr.custom_study_days(count),
        ),
        Choice::StateOrTag => (
            tr.custom_study_select(),
            tr.custom_study_cards_from_the_deck(count),
        ),
    };
    let before_width = u16::try_from(Line::raw(before.as_ref()).width()).unwrap_or(u16::MAX);
    let [before_area, _, input_area, _, after_area] = Layout::horizontal([
        Constraint::Length(before_width),
        Constraint::Length(1),
        Constraint::Length(AMOUNT_WIDTH),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(area);

    frame.render_widget(Line::raw(before.into_owned()), before_area);
    render_input(
        frame,
        input_area,
        form.amount(),
        form.focus() == Field::Amount,
        theme,
    );
    frame.render_widget(Line::raw(after.into_owned()), after_area);
    hits.register(input_area, Hit::CustomStudyField(Field::Amount));
}

fn render_input(frame: &mut Frame<'_>, area: Rect, text: &str, focused: bool, theme: Theme) {
    let mut spans = vec![Span::raw(format!(" {text}"))];

    if focused {
        spans.push(Span::styled("_", Style::new().fg(theme.accent)));
    }

    let style = Style::new().bg(if focused {
        theme.border
    } else {
        theme.background
    });

    frame.render_widget(Line::from(spans).style(style), area);
}

fn radio(selected: bool, focused: bool, label: String, theme: Theme) -> Line<'static> {
    let marker = if selected { "(•) " } else { "( ) " };
    let style = if focused {
        Style::new().fg(theme.accent).add_modifier(Modifier::BOLD)
    } else {
        Style::new()
    };

    Line::from(vec![
        Span::styled(marker, Style::new().fg(theme.accent)),
        Span::styled(label, style),
    ])
}

fn choice_label(choice: Choice, i18n: &I18n) -> String {
    let tr = i18n.anki();

    match choice {
        Choice::NewLimit => tr.custom_study_increase_todays_new_card_limit(),
        Choice::ReviewLimit => tr.custom_study_increase_todays_review_card_limit(),
        Choice::Forgotten => tr.custom_study_review_forgotten_cards(),
        Choice::Ahead => tr.custom_study_review_ahead(),
        Choice::Preview => tr.custom_study_preview_new_cards(),
        Choice::StateOrTag => tr.custom_study_study_by_card_state_or_tag(),
    }
    .into_owned()
}

fn kind_label(kind: StudyKind, i18n: &I18n) -> String {
    let tr = i18n.anki();

    match kind {
        StudyKind::NewOnly => tr.custom_study_new_cards_only(),
        StudyKind::DueOnly => tr.custom_study_due_cards_only(),
        StudyKind::AllReviewRandom => tr.custom_study_all_review_cards_in_random_order(),
        StudyKind::AllRandom => tr.custom_study_all_cards_in_random_order_dont(),
    }
    .into_owned()
}
