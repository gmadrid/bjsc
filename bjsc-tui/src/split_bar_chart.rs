use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Span;
use ratatui::widgets::Paragraph;

/// A single row in a SplitBarChart.
pub struct SplitBar {
    pub label: String,
    /// Primary segment value (rendered first, in `color`).
    pub primary: u32,
    /// Secondary segment value (rendered after primary, in `secondary_color`).
    pub secondary: u32,
    pub color: Color,
    pub secondary_color: Color,
}

impl SplitBar {
    pub fn total(&self) -> u32 {
        self.primary + self.secondary
    }
}

/// A horizontal bar chart where each bar can have two color segments.
pub struct SplitBarChart<'a> {
    pub bars: &'a [SplitBar],
}

impl SplitBarChart<'_> {
    pub fn render(&self, f: &mut ratatui::Frame, area: Rect) {
        let max_val = self
            .bars
            .iter()
            .map(|b| b.total())
            .max()
            .unwrap_or(1)
            .max(1);

        let label_width = 12u16;
        let count_width = 10u16;
        let bar_area_width = area.width.saturating_sub(label_width + count_width);

        for (i, bar) in self.bars.iter().enumerate() {
            let y = area.y + i as u16;
            if y >= area.y + area.height {
                break;
            }

            // Label
            f.render_widget(
                Paragraph::new(Span::styled(&bar.label, Style::default().fg(Color::Gray))),
                Rect::new(area.x, y, label_width, 1),
            );

            // Bar segments
            let bar_x = area.x + label_width;
            let primary_width = if max_val > 0 {
                (bar.primary as f64 / max_val as f64 * bar_area_width as f64) as u16
            } else {
                0
            };
            let secondary_width = if max_val > 0 {
                (bar.secondary as f64 / max_val as f64 * bar_area_width as f64) as u16
            } else {
                0
            };

            if primary_width > 0 {
                let bar_str: String = "\u{2588}".repeat(primary_width as usize);
                f.render_widget(
                    Paragraph::new(Span::styled(bar_str, Style::default().fg(bar.color))),
                    Rect::new(bar_x, y, primary_width, 1),
                );
            }
            if secondary_width > 0 {
                let bar_str: String = "\u{2588}".repeat(secondary_width as usize);
                f.render_widget(
                    Paragraph::new(Span::styled(
                        bar_str,
                        Style::default().fg(bar.secondary_color),
                    )),
                    Rect::new(bar_x + primary_width, y, secondary_width, 1),
                );
            }

            // Count text
            let total = bar.total();
            let count_text = if bar.secondary > 0 {
                format!(" {} ({})", total, bar.secondary)
            } else {
                format!(" {}", total)
            };
            f.render_widget(
                Paragraph::new(Span::styled(count_text, Style::default().fg(Color::White))),
                Rect::new(bar_x + bar_area_width, y, count_width, 1),
            );
        }
    }
}
