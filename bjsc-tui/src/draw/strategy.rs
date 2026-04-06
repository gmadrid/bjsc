use crate::{App, StrategyTab};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn draw_strategy(f: &mut ratatui::Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title + tab bar
            Constraint::Min(5),    // content
            Constraint::Length(1), // hint
        ])
        .split(area);

    // Tab bar
    let desc_style = if app.strategy_tab == StrategyTab::Descriptive {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let table_style = if app.strategy_tab == StrategyTab::Tables {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let title = Line::from(vec![
        Span::styled(
            "Strategy",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled("Descriptive", desc_style),
        Span::raw("  |  "),
        Span::styled("Tables", table_style),
    ]);
    f.render_widget(Paragraph::new(title), chunks[0]);

    // Content
    let block = Block::default().borders(Borders::ALL);
    let inner = block.inner(chunks[1]);
    f.render_widget(block, chunks[1]);

    let lines: Vec<Line> = match app.strategy_tab {
        StrategyTab::Descriptive => {
            let mut lines = Vec::new();
            for (category, phrases) in bjsc::all_phrases() {
                lines.push(Line::from(Span::styled(
                    category,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )));
                for phrase in phrases {
                    lines.push(Line::from(Span::styled(
                        format!("  {}", phrase),
                        Style::default().fg(Color::White),
                    )));
                }
                lines.push(Line::default());
            }
            lines
        }
        StrategyTab::Tables => {
            let mut lines = Vec::new();
            for chart in bjsc::all_charts() {
                lines.push(Line::from(Span::styled(
                    chart.title,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )));
                // Header row
                let mut header_spans = vec![Span::styled(
                    format!("{:<6}", ""),
                    Style::default().fg(Color::DarkGray),
                )];
                for h in &chart.col_headers {
                    header_spans.push(Span::styled(
                        format!("{:>4}", h),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ));
                }
                lines.push(Line::from(header_spans));
                // Data rows
                for (label, cells) in &chart.rows {
                    let mut spans = vec![Span::styled(
                        format!("{:<6}", label),
                        Style::default().fg(Color::Gray),
                    )];
                    for cell in cells {
                        let color = match *cell {
                            "H" => Color::LightRed,
                            "S" => Color::LightGreen,
                            "Dh" | "Ds" => Color::Yellow,
                            "P" | "Pd" => Color::LightBlue,
                            _ => Color::DarkGray,
                        };
                        spans.push(Span::styled(
                            format!("{:>4}", cell),
                            Style::default().fg(color),
                        ));
                    }
                    lines.push(Line::from(spans));
                }
                lines.push(Line::default());
            }
            lines
        }
    };

    let text = Paragraph::new(lines)
        .scroll((app.strategy_scroll, 0))
        .wrap(ratatui::widgets::Wrap { trim: false });
    f.render_widget(text, inner);

    super::footer_with_hint(f, chunks[2], "↑/↓: Scroll | Tab: Switch | Esc: Menu");
}
