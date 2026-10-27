use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::i18n::I18n;
use crate::tui::hit_map::{Hit, HitMap};
use crate::tui::keymap;
use crate::tui::state::State;
use crate::tui::theme::Theme;

const GAP: u16 = 1;

/// One clickable hint per action, showing its first key.
pub fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &State,
    i18n: &I18n,
    theme: Theme,
    hits: &mut HitMap,
) {
    let notice = state
        .notice()
        .map(|notice| Line::styled(format!("{notice} "), Style::new().fg(theme.error)));
    // The notice keeps its place on the right; hints stop before it
    let hints_right = notice.as_ref().map_or_else(
        || area.right(),
        |notice| {
            let width = u16::try_from(notice.width()).unwrap_or(u16::MAX);

            area.right().saturating_sub(width).saturating_sub(GAP)
        },
    );
    let mut x = area.x.saturating_add(GAP);

    for (action, labels) in keymap::grouped(state.context()) {
        let Some(key) = labels.first() else {
            continue;
        };
        let line = Line::from(vec![
            Span::styled(
                format!(" {key} "),
                Style::new().fg(theme.accent).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{} ", action.label(i18n)),
                Style::new().fg(theme.muted),
            ),
        ]);
        let width = u16::try_from(line.width()).unwrap_or(u16::MAX);

        // Whole hints only; the rest stay listed in the help overlay
        if x.saturating_add(width) > hints_right {
            break;
        }

        let hint = Rect::new(x, area.y, width, 1).intersection(area);
        let hovered = state
            .mouse()
            .is_some_and(|position| hint.contains(position));
        let line = if hovered {
            line.style(Style::new().bg(theme.surface))
        } else {
            line
        };

        frame.render_widget(line, hint);
        hits.register(hint, Hit::Action(action));
        x = x.saturating_add(width).saturating_add(GAP);
    }

    if let Some(notice) = notice {
        frame.render_widget(notice.right_aligned(), area);
    }
}
