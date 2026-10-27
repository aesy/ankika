use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use crate::i18n::I18n;
use crate::tui::hit_map::{Hit, HitMap};
use crate::tui::report::{Report, ReportButton};
use crate::tui::state::State;
use crate::tui::theme::Theme;
use crate::tui::view::buttons::{self, Button};

const WIDTH: u16 = 80;
// Borders, margins, a blank line and the buttons around the text
const FIXED_HEIGHT: u16 = 6;
// Borders and margins beside the text
const FIXED_WIDTH: u16 = 6;

pub fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    report: &Report,
    state: &State,
    i18n: &I18n,
    theme: Theme,
    hits: &mut HitMap,
) {
    let width = WIDTH.min(area.width);
    let lines = wrap(report.html(), width.saturating_sub(FIXED_WIDTH));
    let rows = u16::try_from(lines.len()).unwrap_or(u16::MAX).max(1);
    let popup = area.centered(
        Constraint::Length(width),
        Constraint::Length(rows.saturating_add(FIXED_HEIGHT)),
    );
    let inner =
        super::overlay(frame, popup, &report.title(i18n), theme, hits).inner(Margin::new(2, 1));
    let [text_area, _, button_area] = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(inner);

    report.set_max_scroll(rows.saturating_sub(text_area.height));
    frame.render_widget(
        Paragraph::new(lines).scroll((report.scroll(), 0)),
        text_area,
    );

    let choices = report.choices();
    let buttons: Vec<Button> = choices
        .items()
        .iter()
        .map(|&button| Button {
            label: label(button, report, i18n),
            hit: Hit::ReportButton(button),
            enabled: true,
            focused: choices.focused() == Some(button),
        })
        .collect();

    buttons::render_right_aligned(frame, button_area, &buttons, state.mouse(), theme, hits);
}

fn wrap(html: &str, width: u16) -> Vec<Line<'static>> {
    html2text::config::plain_no_decorate()
        .string_from_read(html.as_bytes(), usize::from(width.max(1)))
        // Only fails when too narrow for a single character
        .unwrap_or_default()
        .lines()
        .map(|line| Line::raw(line.to_owned()))
        .collect()
}

fn label(button: ReportButton, report: &Report, i18n: &I18n) -> String {
    let tr = i18n.anki();

    match button {
        ReportButton::DeleteUnused => tr.media_check_delete_unused().into_owned(),
        ReportButton::EmptyTrash => tr.media_check_empty_trash().into_owned(),
        ReportButton::RestoreTrash => i18n.text("maintenance-restore-trash"),
        ReportButton::KeepNotes => {
            let mark = if report.keep_notes() { "[x]" } else { "[ ]" };

            format!("{mark} {}", tr.empty_cards_preserve_notes_checkbox())
        },
        ReportButton::Delete => tr.empty_cards_delete_button().into_owned(),
        ReportButton::Close => tr.actions_close().into_owned(),
    }
}
