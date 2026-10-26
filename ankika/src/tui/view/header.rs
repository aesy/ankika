use ankika_core::review::Queue;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use crate::i18n::I18n;
use crate::tui::action::Action;
use crate::tui::hit_map::{Hit, HitMap};
use crate::tui::review::{Review, Stage};
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

    let title_width = u16::try_from(title.width()).unwrap_or(u16::MAX);
    let status = state.review().map_or_else(
        || {
            Line::styled(
                i18n.anki()
                    .statistics_cards_due(state.decks().due_count())
                    .into_owned(),
                theme.muted,
            )
        },
        |review| review_counts(review, theme),
    );
    let status_area = Rect {
        x: area.x.saturating_add(title_width).saturating_add(GAP * 2),
        ..area
    }
    .intersection(area);

    frame.render_widget(title, area);
    frame.render_widget(status, status_area);

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

/// The remaining counts, with the current card's queue underlined like in Anki.
fn review_counts(review: &Review, theme: Theme) -> Line<'static> {
    let Some(counts) = review.counts() else {
        return Line::default();
    };
    let current = match review.stage() {
        Stage::Question { card, .. } | Stage::Answer { card, .. } => Some(card.queue),
        Stage::Waiting | Stage::Finished { .. } => None,
    };
    let count = |value: usize, queue: Queue, color| {
        let style = Style::new().fg(color);
        let style = if current == Some(queue) {
            style.add_modifier(Modifier::UNDERLINED | Modifier::BOLD)
        } else {
            style
        };

        Span::styled(value.to_string(), style)
    };

    Line::from(vec![
        count(counts.new, Queue::New, theme.new_count),
        Span::raw(" "),
        count(counts.learning, Queue::Learning, theme.learn_count),
        Span::raw(" "),
        count(counts.review, Queue::Review, theme.review_count),
    ])
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
