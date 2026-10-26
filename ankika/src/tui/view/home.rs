use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, BorderType};

use crate::i18n::I18n;
use crate::tui::theme::Theme;

pub fn render(frame: &mut Frame<'_>, area: Rect, i18n: &I18n, theme: Theme) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme.border))
        .title(format!(" {} ", i18n.anki().actions_decks()));

    frame.render_widget(block, area);
}
