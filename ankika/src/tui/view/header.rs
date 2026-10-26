use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;

use crate::i18n::I18n;
use crate::tui::action::Action;
use crate::tui::hit_map::{Hit, HitMap};
use crate::tui::state::{State, SyncStatus};
use crate::tui::theme::Theme;

const GAP: u16 = 1;

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

    let Some(session) = state.session() else {
        // Logged out: the status doubles as a login button
        let login = HeaderButton {
            text: i18n.text("header-not-logged-in"),
            color: theme.muted,
            action: Action::LogIn,
        };
        render_button(frame, area, area.right(), &login, state, theme, hits);
        return;
    };

    let username = Line::styled(format!("{} ", session.account.username), theme.muted);
    let username_width = u16::try_from(username.width()).unwrap_or(u16::MAX);
    let (text, color) = match state.sync_status() {
        SyncStatus::Syncing => (i18n.anki().sync_syncing().into_owned(), theme.muted),
        _ if state.media_syncing() => (i18n.text("sync-status-media"), theme.muted),
        SyncStatus::Idle => (i18n.text("action-sync"), theme.accent),
        SyncStatus::Synced => (i18n.text("sync-status-synced"), theme.muted),
        SyncStatus::Failed => (i18n.text("sync-status-failed"), theme.error),
    };
    let sync = HeaderButton {
        text,
        color,
        action: Action::Sync,
    };
    let sync_right = area
        .right()
        .saturating_sub(username_width)
        .saturating_sub(GAP);

    frame.render_widget(username.right_aligned(), area);
    render_button(frame, area, sync_right, &sync, state, theme, hits);
}

struct HeaderButton {
    text: String,
    color: Color,
    action: Action,
}

/// A clickable label ending at `right`, highlighted on hover.
fn render_button(
    frame: &mut Frame<'_>,
    area: Rect,
    right: u16,
    button: &HeaderButton,
    state: &State,
    theme: Theme,
    hits: &mut HitMap,
) {
    let line = Line::raw(format!(" {} ", button.text));
    let width = u16::try_from(line.width()).unwrap_or(u16::MAX);
    let button_area = Rect {
        x: right.saturating_sub(width),
        width,
        ..area
    }
    .intersection(area);
    let hovered = state
        .mouse()
        .is_some_and(|position| button_area.contains(position));
    let style = if hovered {
        Style::new().bg(theme.surface).fg(theme.accent)
    } else {
        Style::new().fg(button.color)
    };

    frame.render_widget(line.style(style), button_area);
    hits.register(button_area, Hit::Action(button.action));
}
