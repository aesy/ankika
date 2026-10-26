use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;

use crate::i18n::I18n;
use crate::tui::theme::Theme;

pub fn render(frame: &mut Frame<'_>, area: Rect, i18n: &I18n, theme: Theme) {
    let title = Line::styled(
        " ankika",
        Style::new().fg(theme.accent).add_modifier(Modifier::BOLD),
    );
    let status = Line::styled(
        format!("{} ", i18n.text("header-not-logged-in")),
        Style::new().fg(theme.muted),
    )
    .right_aligned();

    frame.render_widget(title, area);
    frame.render_widget(status, area);
}
