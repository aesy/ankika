use ankika_core::overview::Overview;
use ankika_core::review::Sounds;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Paragraph};

use crate::i18n::I18n;
use crate::tui::action::Action;
use crate::tui::card_text::{self, CardBlock};
use crate::tui::hit_map::{Hit, HitMap};
use crate::tui::overview::DeckScreen;
use crate::tui::state::State;
use crate::tui::theme::Theme;
use crate::tui::view::buttons::{self, Button};

const COUNTS_WIDTH: u16 = 30;
const COUNT_ROWS: u16 = 3;
/// Leaves room for the counts and buttons below a long description.
const MAX_DESCRIPTION_ROWS: u16 = 12;

pub fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    screen: &DeckScreen,
    state: &State,
    i18n: &I18n,
    theme: Theme,
    hits: &mut HitMap,
) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme.border))
        .title(format!(" {} ", screen.name()));
    let inner = block.inner(area).inner(Margin::new(2, 1));

    frame.render_widget(block, area);

    let Some(overview) = screen.overview() else {
        let text = Line::styled(i18n.anki().sync_syncing().into_owned(), theme.muted);

        frame.render_widget(Paragraph::new(text).centered(), inner);
        return;
    };
    let description = description_lines(&overview.description, inner.width, theme);
    let description_rows = u16::try_from(description.len())
        .unwrap_or(u16::MAX)
        .min(MAX_DESCRIPTION_ROWS);
    let [description_area, _, counts_area, _, button_area] = Layout::vertical([
        Constraint::Length(description_rows),
        Constraint::Length(u16::from(description_rows > 0)),
        Constraint::Length(COUNT_ROWS),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(inner);

    frame.render_widget(Paragraph::new(description).centered(), description_area);
    render_counts(frame, counts_area, overview, state, i18n, theme);
    render_buttons(frame, button_area, overview, state, i18n, theme, hits);
}

fn description_lines(html: &str, width: u16, theme: Theme) -> Vec<Line<'static>> {
    let no_sounds = Sounds {
        question: Vec::new(),
        answer: Vec::new(),
        autoplay: false,
        replay_question: false,
    };

    card_text::render(html, width, theme, &no_sounds, None)
        .into_iter()
        .map(|block| match block {
            CardBlock::Line(line) => line.line,
            CardBlock::Image(src) => Line::styled(format!("[image: {src}]"), theme.muted),
        })
        .collect()
}

/// Anki's three counts, each with how many burying holds back, like Anki's "+N".
fn render_counts(
    frame: &mut Frame<'_>,
    area: Rect,
    overview: &Overview,
    state: &State,
    i18n: &I18n,
    theme: Theme,
) {
    let tr = i18n.anki();
    let node = state.decks().deck(overview.deck);
    let held_back = |shown: usize, total: Option<u32>| {
        total.map_or(0, |total| {
            usize::try_from(total).unwrap_or(0).saturating_sub(shown)
        })
    };
    let rows = [
        (
            tr.actions_new(),
            overview.counts.new,
            held_back(overview.counts.new, node.map(|node| node.new_count)),
            theme.new_count,
        ),
        (
            tr.scheduling_learning(),
            overview.counts.learning,
            held_back(overview.counts.learning, node.map(|node| node.learn_count)),
            theme.learn_count,
        ),
        (
            tr.studying_to_review(),
            overview.counts.review,
            held_back(overview.counts.review, node.map(|node| node.review_count)),
            theme.review_count,
        ),
    ];
    let column = area.centered_horizontally(Constraint::Length(COUNTS_WIDTH));
    let row_areas = Layout::vertical([Constraint::Length(1); 3]).split(column);

    for ((label, count, held, color), &row) in rows.into_iter().zip(row_areas.iter()) {
        let [label_area, count_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(10)]).areas(row);
        let mut count_spans = vec![Span::styled(
            count.to_string(),
            Style::new()
                .fg(if count == 0 { theme.muted } else { color })
                .add_modifier(Modifier::BOLD),
        )];

        if held > 0 {
            count_spans.push(Span::styled(format!(" +{held}"), theme.muted));
        }

        frame.render_widget(Line::raw(format!("{label}:")), label_area);
        frame.render_widget(Line::from(count_spans).right_aligned(), count_area);
    }
}

fn render_buttons(
    frame: &mut Frame<'_>,
    area: Rect,
    overview: &Overview,
    state: &State,
    i18n: &I18n,
    theme: Theme,
    hits: &mut HitMap,
) {
    let mut actions = vec![Action::Study];

    if overview.filtered {
        actions.extend([Action::Rebuild, Action::Empty]);
    } else {
        actions.push(Action::CustomStudy);
    }

    if overview.have_user_buried || overview.have_sched_buried {
        actions.push(Action::Unbury);
    }

    actions.push(Action::CloseOverview);

    let row: Vec<Button> = actions
        .into_iter()
        .map(|action| Button {
            label: action.label(i18n),
            hit: Hit::Action(action),
            enabled: true,
            focused: action == Action::Study,
        })
        .collect();
    let width = row
        .iter()
        .map(Button::width)
        .sum::<u16>()
        .saturating_add(u16::try_from(row.len().saturating_sub(1)).unwrap_or(0) * buttons::GAP);
    let centered = area.centered_horizontally(Constraint::Length(width));

    buttons::render_right_aligned(frame, centered, &row, state.mouse(), theme, hits);
}
