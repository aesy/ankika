use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;

use crate::i18n::I18n;
use crate::tui::action::Action;
use crate::tui::hit_map::{Hit, HitMap};
use crate::tui::state::State;
use crate::tui::theme::Theme;

pub fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &State,
    i18n: &I18n,
    theme: Theme,
    hits: &mut HitMap,
) {
    let title = Line::styled(
        " ankika",
        Style::new().fg(theme.accent).add_modifier(Modifier::BOLD),
    );

    frame.render_widget(title, area);

    if let Some(session) = state.session() {
        let status = Line::styled(
            format!("{} ", session.account.username),
            Style::new().fg(theme.muted),
        );
        frame.render_widget(status.right_aligned(), area);
        return;
    }

    // Logged out: the status doubles as a login button
    let status = Line::raw(format!(" {} ", i18n.text("header-not-logged-in")));
    let width = u16::try_from(status.width()).unwrap_or(u16::MAX);
    let button = Rect {
        x: area.right().saturating_sub(width),
        width,
        ..area
    }
    .intersection(area);
    let hovered = state
        .mouse()
        .is_some_and(|position| button.contains(position));
    let style = if hovered {
        Style::new().bg(theme.surface).fg(theme.accent)
    } else {
        Style::new().fg(theme.muted)
    };

    frame.render_widget(status.style(style), button);
    hits.register(button, Hit::Action(Action::LogIn));
}
