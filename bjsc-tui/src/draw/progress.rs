use crate::App;
use bjsc::progress::ProgressStats;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

pub fn draw_progress(f: &mut ratatui::Frame, area: Rect, app: &App) {
    let p = &app.progress;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title
            Constraint::Length(4), // overall + category breakdown
            Constraint::Length(1), // separator
            Constraint::Min(8),    // trouble spots
            Constraint::Length(1), // separator
            Constraint::Length(8), // sessions
            Constraint::Length(1), // hint
        ])
        .split(area);

    // Title
    let title = Span::styled(
        "Progress Dashboard",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(Paragraph::new(Line::from(title)), chunks[0]);

    // Overall + category stats
    let overall_block = Block::default().borders(Borders::ALL).title("Accuracy");
    let overall_inner = overall_block.inner(chunks[1]);
    f.render_widget(overall_block, chunks[1]);

    let overall_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(overall_inner);

    let overall_line = Line::from(vec![
        Span::styled("Overall: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(
            format!("{:.1}%", p.accuracy_pct),
            Style::default().fg(if p.accuracy_pct >= 80.0 {
                Color::Green
            } else if p.accuracy_pct >= 60.0 {
                Color::Yellow
            } else {
                Color::Red
            }),
        ),
        Span::raw(format!("  ({}/{})", p.total_correct, p.total_answers)),
    ]);
    f.render_widget(Paragraph::new(overall_line), overall_rows[0]);

    let cat_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(overall_rows[1]);

    let cats = [
        ("Hard: ", p.hard_correct, p.hard_total),
        ("Soft: ", p.soft_correct, p.soft_total),
        ("Split: ", p.split_correct, p.split_total),
        ("Dbl: ", p.double_correct, p.double_total),
    ];
    for (i, (label, correct, total)) in cats.iter().enumerate() {
        let line = Line::from(vec![
            Span::styled(*label, Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(ProgressStats::category_pct(*correct, *total)),
        ]);
        f.render_widget(Paragraph::new(line), cat_cols[i]);
    }

    // Trouble spots
    let trouble_block = Block::default()
        .borders(Borders::ALL)
        .title("Trouble Spots");
    let trouble_inner = trouble_block.inner(chunks[3]);
    f.render_widget(trouble_block, chunks[3]);

    let trouble_items: Vec<ListItem> = p
        .trouble_spots
        .iter()
        .map(|(idx, wrong, seen)| {
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:<20}", idx), Style::default().fg(Color::LightRed)),
                Span::raw(format!(
                    "{}/{} wrong ({:.0}%)",
                    wrong,
                    seen,
                    *wrong as f64 / *seen as f64 * 100.0
                )),
            ]))
        })
        .collect();
    if trouble_items.is_empty() {
        f.render_widget(
            Paragraph::new("No mistakes recorded yet.").style(Style::default().fg(Color::DarkGray)),
            trouble_inner,
        );
    } else {
        f.render_widget(List::new(trouble_items), trouble_inner);
    }

    // Recent sessions
    let session_block = Block::default()
        .borders(Borders::ALL)
        .title("Recent Sessions");
    let session_inner = session_block.inner(chunks[5]);
    f.render_widget(session_block, chunks[5]);

    let session_items: Vec<ListItem> = p
        .sessions
        .iter()
        .map(|(day, total, correct)| {
            let pct = if *total > 0 {
                *correct as f64 / *total as f64 * 100.0
            } else {
                0.0
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:<14}", day), Style::default().fg(Color::Cyan)),
                Span::raw(format!("{:>4} answered  ", total)),
                Span::styled(
                    format!("{:.0}%", pct),
                    Style::default().fg(if pct >= 80.0 {
                        Color::Green
                    } else if pct >= 60.0 {
                        Color::Yellow
                    } else {
                        Color::Red
                    }),
                ),
            ]))
        })
        .collect();
    if session_items.is_empty() {
        f.render_widget(
            Paragraph::new("No sessions recorded yet.").style(Style::default().fg(Color::DarkGray)),
            session_inner,
        );
    } else {
        f.render_widget(List::new(session_items), session_inner);
    }

    super::footer_with_hint(f, chunks[6], "Esc: Menu");
}
