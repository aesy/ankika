use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::i18n::I18n;
use crate::tui::action::Action;
use crate::tui::add_form::{AddForm, Field};
use crate::tui::hit_map::{Hit, HitMap};
use crate::tui::state::State;
use crate::tui::theme::Theme;
use crate::tui::view::buttons::{self, Button};
use crate::tui::view::fuzzy_list::{self, Labels};
use crate::tui::view::tag_input;

const WIDTH: u16 = 64;
const LABEL_WIDTH: u16 = 12;
// Borders, margins, the three rows, a blank line and the buttons
const FIXED_HEIGHT: u16 = 9;

pub fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    form: &AddForm,
    state: &State,
    i18n: &I18n,
    theme: Theme,
    hits: &mut HitMap,
) {
    let tr = i18n.anki();
    let completions = u16::try_from(form.tags().completions().len()).unwrap_or(0);
    let popup = area.centered(
        Constraint::Length(WIDTH),
        Constraint::Length(FIXED_HEIGHT.saturating_add(completions)),
    );
    let title = Action::AddNote.label(i18n);
    let inner = super::overlay(frame, popup, &title, theme, hits).inner(Margin::new(2, 1));
    let [
        notetype_area,
        deck_area,
        tags_area,
        completion_area,
        _,
        button_area,
    ] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(completions),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(inner);
    let focused = |field| form.focus() == field && form.picker().is_none();

    render_row(
        frame,
        notetype_area,
        &tr.notetypes_notetype(),
        &format!("{} ▾", form.notetype_name()),
        focused(Field::Notetype),
        theme,
    );
    hits.register(notetype_area, Hit::AddField(Field::Notetype));
    render_row(
        frame,
        deck_area,
        &tr.decks_deck(),
        &format!("{} ▾", form.deck_name()),
        focused(Field::Deck),
        theme,
    );
    hits.register(deck_area, Hit::AddField(Field::Deck));

    let cursor = if focused(Field::Tags) { "_" } else { "" };

    render_row(
        frame,
        tags_area,
        &tr.editing_tags(),
        &format!("{}{cursor}", form.tags().text()),
        focused(Field::Tags),
        theme,
    );
    hits.register(tags_area, Hit::AddField(Field::Tags));
    tag_input::render_completions(
        frame,
        completion_area,
        form.tags(),
        LABEL_WIDTH,
        theme,
        hits,
    );

    let row: Vec<Button> = [
        (Field::Add, tr.actions_add()),
        (Field::Close, tr.actions_close()),
    ]
    .into_iter()
    .map(|(field, label)| Button {
        label: label.into_owned(),
        hit: Hit::AddField(field),
        enabled: true,
        focused: focused(field),
    })
    .collect();

    buttons::render_right_aligned(frame, button_area, &row, state.mouse(), theme, hits);

    if let Some(picker) = form.picker() {
        let title = match form.focus() {
            Field::Deck => tr.decks_deck(),
            _ => tr.notetypes_notetype(),
        };
        let placeholder = i18n.text("picker-placeholder");
        let labels = Labels {
            title: &title,
            placeholder: &placeholder,
        };

        fuzzy_list::render(frame, area, picker, &labels, i18n, theme, hits);
    }
}

fn render_row(
    frame: &mut Frame<'_>,
    area: Rect,
    label: &str,
    value: &str,
    focused: bool,
    theme: Theme,
) {
    let [label_area, value_area] =
        Layout::horizontal([Constraint::Length(LABEL_WIDTH), Constraint::Fill(1)]).areas(area);
    let style = if focused {
        Style::new().bg(theme.border).add_modifier(Modifier::BOLD)
    } else {
        Style::new().bg(theme.background)
    };

    frame.render_widget(Line::styled(format!("{label}:"), theme.muted), label_area);
    frame.render_widget(
        Line::from(vec![Span::raw(format!(" {value}"))]).style(style),
        value_area,
    );
}
