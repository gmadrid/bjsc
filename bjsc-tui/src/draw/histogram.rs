use crate::split_bar_chart::{SplitBar, SplitBarChart};
use crate::App;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn draw_histogram(f: &mut ratatui::Frame, area: Rect, app: &App) {
    let box_counts = app.game_state.box_counts();
    let box_due = app.game_state.box_due_counts();
    let unseen = app.game_state.unseen_count();

    let intervals = bjsc::BOX_LABELS;
    let colors = [
        Color::LightRed,
        Color::Red,
        Color::Yellow,
        Color::Yellow,
        Color::Cyan,
        Color::Cyan,
        Color::Blue,
        Color::Green,
        Color::LightGreen,
    ];
    let due_colors = [
        Color::Rgb(255, 200, 200),
        Color::Rgb(255, 140, 140),
        Color::Rgb(255, 243, 180),
        Color::Rgb(255, 224, 100),
        Color::Rgb(200, 240, 247),
        Color::Rgb(150, 230, 240),
        Color::Rgb(180, 218, 255),
        Color::Rgb(170, 238, 176),
        Color::Rgb(212, 247, 214),
    ];

    let bars: Vec<SplitBar> = (0..spaced_rep::NUM_BOXES as usize)
        .map(|i| {
            let total = box_counts[i];
            let due = box_due[i];
            SplitBar {
                label: format!(" B{} ({:<3})", i, intervals[i]),
                primary: total - due,
                secondary: due,
                color: colors[i],
                secondary_color: due_colors[i],
            }
        })
        .collect();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(10),
            Constraint::Length(2),
        ])
        .split(area);

    let title = Line::from(vec![
        Span::styled(
            "Spaced Repetition Buckets",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("  (Mode: {})", app.game_state.study_mode())),
    ]);
    f.render_widget(Paragraph::new(title), chunks[0]);

    let block = Block::default().borders(Borders::ALL);
    let inner = block.inner(chunks[1]);
    f.render_widget(block, chunks[1]);

    SplitBarChart { bars: &bars }.render(f, inner);

    let footer_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(chunks[2]);

    let unseen_line = Line::from(vec![
        Span::styled("Unseen: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(format!("{}", unseen), Style::default().fg(Color::DarkGray)),
    ]);
    f.render_widget(Paragraph::new(unseen_line), footer_rows[0]);

    super::footer_with_hint(f, footer_rows[1], "Esc: Menu");
}
