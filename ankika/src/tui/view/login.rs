use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Position, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};

use crate::i18n::I18n;
use crate::tui::hit_map::{Hit, HitMap};
use crate::tui::login_form::{Field, LoginForm};
use crate::tui::state::State;
use crate::tui::theme::Theme;

const WIDTH: u16 = 52;
// Borders, one blank line above and below, and the rows other than the error
const FIXED_HEIGHT: u16 = 10;
// Anki's network errors are a summary, a blank line and wrapped details
const ERROR_ROWS: u16 = 5;
const BUTTON_GAP: u16 = 1;
const PASSWORD_MASK: char = '*';

pub fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    form: &LoginForm,
    state: &State,
    i18n: &I18n,
    theme: Theme,
    hits: &mut HitMap,
) {
    let error_rows = if form.error().is_some() {
        ERROR_ROWS
    } else {
        1
    };
    let height = FIXED_HEIGHT.saturating_add(error_rows);
    let popup = area.centered(Constraint::Length(WIDTH), Constraint::Length(height));
    let inner = super::overlay(frame, popup, &i18n.anki().sync_sign_in(), theme, hits)
        .inner(Margin::new(2, 1));
    let [
        email_label,
        email_input,
        password_label,
        password_input,
        _,
        error_area,
        button_area,
    ] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(error_rows),
        Constraint::Length(1),
    ])
    .areas(inner);
    let masked: String = form.password().chars().map(|_| PASSWORD_MASK).collect();

    let email = TextField {
        field: Field::Email,
        label: i18n.anki().sync_ankiweb_id_label().into_owned(),
        value: form.email(),
    };
    let password = TextField {
        field: Field::Password,
        label: i18n.anki().sync_password_label().into_owned(),
        value: &masked,
    };

    email.render(frame, email_label, email_input, form, theme, hits);
    password.render(frame, password_label, password_input, form, theme, hits);

    if let Some(error) = form.error() {
        frame.render_widget(
            Paragraph::new(error)
                .style(Style::new().fg(theme.error))
                .wrap(Wrap { trim: true }),
            error_area,
        );
    }

    render_buttons(frame, button_area, form, state, i18n, theme, hits);
}

struct TextField<'a> {
    field: Field,
    label: String,
    value: &'a str,
}

impl TextField<'_> {
    fn render(
        &self,
        frame: &mut Frame<'_>,
        label_area: Rect,
        input_area: Rect,
        form: &LoginForm,
        theme: Theme,
        hits: &mut HitMap,
    ) {
        let focused = form.focus() == self.field;
        let label_style = if focused {
            Style::new().fg(theme.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(theme.muted)
        };
        let value = Line::raw(format!(" {}", self.value));
        let cursor_offset = u16::try_from(value.width()).unwrap_or(u16::MAX);

        frame.render_widget(Line::styled(self.label.as_str(), label_style), label_area);
        frame.render_widget(
            value.style(Style::new().bg(theme.background).fg(theme.text)),
            input_area,
        );
        hits.register(label_area.union(input_area), Hit::LoginField(self.field));

        if focused {
            frame.set_cursor_position(Position::new(
                input_area
                    .x
                    .saturating_add(cursor_offset)
                    .min(input_area.right().saturating_sub(1)),
                input_area.y,
            ));
        }
    }
}

/// Right-aligned submit and cancel buttons.
fn render_buttons(
    frame: &mut Frame<'_>,
    area: Rect,
    form: &LoginForm,
    state: &State,
    i18n: &I18n,
    theme: Theme,
    hits: &mut HitMap,
) {
    let submit_label = if form.busy() {
        i18n.anki().sync_connecting()
    } else {
        i18n.anki().sync_log_in_button()
    };
    let buttons = [
        (Field::Submit, submit_label, form.can_submit()),
        (Field::Cancel, i18n.anki().sync_cancel_button(), true),
    ];
    let mut right = area.right();

    for (field, label, enabled) in buttons.into_iter().rev() {
        let line = Line::raw(format!(" {label} "));
        let width = u16::try_from(line.width()).unwrap_or(u16::MAX);
        let button = Rect {
            x: right.saturating_sub(width),
            width,
            ..area
        }
        .intersection(area);
        let hovered = state
            .mouse()
            .is_some_and(|position| button.contains(position));
        let style = if !enabled {
            Style::new().bg(theme.border).fg(theme.muted)
        } else if form.focus() == field || hovered {
            Style::new()
                .bg(theme.accent)
                .fg(theme.background)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::new().bg(theme.border).fg(theme.text)
        };

        frame.render_widget(line.style(style), button);
        hits.register(button, Hit::LoginField(field));
        right = button.x.saturating_sub(BUTTON_GAP);
    }
}
