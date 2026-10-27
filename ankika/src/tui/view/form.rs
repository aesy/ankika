use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Position, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::i18n::I18n;
use crate::tui::form::{Focus, Form, Value};
use crate::tui::hit_map::{Hit, HitMap};
use crate::tui::theme::Theme;
use crate::tui::view::buttons::{self, Button};

const WIDTH: u16 = 76;
const LABEL_WIDTH: u16 = 34;
// Borders, margins, a blank line and the buttons
const FIXED_HEIGHT: u16 = 6;

pub fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    form: &Form,
    mouse: Option<Position>,
    i18n: &I18n,
    theme: Theme,
    hits: &mut HitMap,
) {
    let extra_rows = u16::from(form.note.is_some()) + u16::from(form.error().is_some());
    let rows = u16::try_from(form.fields().len()).unwrap_or(u16::MAX);
    let popup = area.centered(
        Constraint::Length(WIDTH),
        Constraint::Length(FIXED_HEIGHT + rows + extra_rows),
    );
    let inner = super::overlay(frame, popup, &form.title, theme, hits).inner(Margin::new(2, 1));
    let [fields_area, note_area, error_area, _, button_area] = Layout::vertical([
        Constraint::Length(rows),
        Constraint::Length(u16::from(form.note.is_some())),
        Constraint::Length(u16::from(form.error().is_some())),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(inner);
    let row_areas =
        Layout::vertical(vec![Constraint::Length(1); form.fields().len()]).split(fields_area);

    for (index, (field, &row)) in form.fields().iter().zip(row_areas.iter()).enumerate() {
        let focused = form.focus() == Focus::Field(index);
        let [label_area, value_area] =
            Layout::horizontal([Constraint::Length(LABEL_WIDTH), Constraint::Fill(1)]).areas(row);
        let label_style = if focused {
            Style::new().fg(theme.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(theme.muted)
        };

        frame.render_widget(Line::styled(field.label.as_str(), label_style), label_area);
        frame.render_widget(value_line(&field.value, focused, theme), value_area);
        hits.register(row, Hit::Form(Focus::Field(index)));
    }

    if let Some(note) = &form.note {
        frame.render_widget(Line::styled(note.as_str(), theme.muted), note_area);
    }

    if let Some(error) = form.error() {
        frame.render_widget(Line::styled(error, theme.error), error_area);
    }

    let row: Vec<Button> = [
        (Focus::Save, i18n.anki().deck_config_save_button()),
        (Focus::Cancel, i18n.anki().actions_cancel()),
    ]
    .into_iter()
    .map(|(focus, label)| Button {
        label: label.into_owned(),
        hit: Hit::Form(focus),
        enabled: true,
        focused: form.focus() == focus,
    })
    .collect();

    buttons::render_right_aligned(frame, button_area, &row, mouse, theme, hits);
}

fn value_line(value: &Value, focused: bool, theme: Theme) -> Line<'static> {
    let accent = Style::new().fg(theme.accent);

    match value {
        Value::Toggle(on) => Line::from(vec![Span::styled(
            if *on { "[x] " } else { "[ ] " },
            accent,
        )]),
        Value::Choice { options, selected } => {
            let option = options.get(*selected).cloned().unwrap_or_default();

            if focused {
                Line::from(vec![
                    Span::styled("< ", accent),
                    Span::raw(option),
                    Span::styled(" >", accent),
                ])
            } else {
                Line::raw(format!("  {option}"))
            }
        },
        Value::Text(text) => {
            let style = Style::new().bg(if focused { theme.border } else { theme.surface });
            let cursor = if focused { "_" } else { "" };

            Line::styled(format!(" {text}{cursor}"), style)
        },
    }
}
