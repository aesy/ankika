use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;

use crate::tui::theme::Theme;

/// Partial blocks for the top of a bar, from one eighth to full.
const BAR_EIGHTHS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
/// Anki's calendar shades, from few reviews to many.
const HEAT: [Color; 4] = [
    Color::Rgb(0x0e, 0x44, 0x29),
    Color::Rgb(0x00, 0x6d, 0x32),
    Color::Rgb(0x26, 0xa6, 0x41),
    Color::Rgb(0x39, 0xd3, 0x53),
];
const CELL: &str = "■ ";

/// Sums neighbouring values so they fit `columns`.
pub fn buckets(values: &[u32], columns: usize) -> Vec<u32> {
    if columns == 0 || values.is_empty() {
        return Vec::new();
    }

    let size = values.len().div_ceil(columns);

    values
        .chunks(size)
        .map(|chunk| chunk.iter().sum())
        .collect()
}

/// Vertical bars across `area`, as wide as fits, scaled to the largest value, which is
/// labeled at the top.
pub fn render_bars(frame: &mut Frame<'_>, area: Rect, values: &[u32], color: Color, theme: Theme) {
    let max = values.iter().copied().max().unwrap_or(0);
    let count = u16::try_from(values.len()).unwrap_or(u16::MAX);

    if area.height == 0 || max == 0 || count == 0 {
        return;
    }

    let bar_width = (area.width / count).max(1);
    let height = u64::from(area.height);
    let buffer = frame.buffer_mut();

    for (column, &value) in (0..count).zip(values) {
        // In eighths of a row, rounding up so any value shows
        let eighths = (u64::from(value) * height * 8).div_ceil(u64::from(max));

        for row in 0..area.height {
            let filled = eighths.saturating_sub(u64::from(row) * 8).min(8);
            let Some(&block) = usize::try_from(filled)
                .ok()
                .and_then(|filled| filled.checked_sub(1))
                .and_then(|index| BAR_EIGHTHS.get(index))
            else {
                continue;
            };
            let y = area.bottom() - 1 - row;

            for dx in 0..bar_width {
                let x = area.x.saturating_add(column * bar_width + dx);

                if x < area.right()
                    && let Some(cell) = buffer.cell_mut((x, y))
                {
                    cell.set_char(block).set_fg(color);
                }
            }
        }
    }

    frame.render_widget(
        Line::styled(max.to_string(), theme.muted).right_aligned(),
        Rect { height: 1, ..area },
    );
}

/// One square per day, a column per week and a row per weekday, ending today.
/// `counts` has today first, then each day before it.
pub fn render_heatmap(
    frame: &mut Frame<'_>,
    area: Rect,
    counts: &[u32],
    weekday_today: u16,
    theme: Theme,
) {
    let max = counts.iter().copied().max().unwrap_or(0);
    let weeks = area.width / 2;

    for (days_ago, &count) in counts.iter().enumerate() {
        let Ok(days_ago) = u16::try_from(days_ago) else {
            break;
        };
        // Weeks start on Sunday, row 0
        let days_back = days_ago + (6 - weekday_today);
        let week = days_back / 7;
        let row = 6 - days_back % 7;

        if week >= weeks || row >= area.height {
            continue;
        }

        let x = area.x + (weeks - 1 - week) * 2;
        let color = if count == 0 || max == 0 {
            theme.border
        } else {
            let level = (u64::from(count) * 4).div_ceil(u64::from(max)).clamp(1, 4);
            HEAT[usize::try_from(level).unwrap_or(1) - 1]
        };

        frame.render_widget(
            Line::styled(CELL, Style::new().fg(color)),
            Rect::new(x, area.y + row, 2, 1),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buckets_sum_to_fit() {
        assert_eq!(buckets(&[1, 2, 3, 4, 5], 2), vec![6, 9]);
        assert_eq!(buckets(&[1, 2], 5), vec![1, 2]);
        assert_eq!(buckets(&[], 5), Vec::<u32>::new());
    }
}
