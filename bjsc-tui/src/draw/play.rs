use crate::{App, StatusMessage};
use bjsc::card::Card;
use bjsc::hand::Hand;
use bjsc::Stats;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

pub fn draw_play(f: &mut ratatui::Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // mode
            Constraint::Length(6), // stats
            Constraint::Length(3), // dealer
            Constraint::Length(3), // player
            Constraint::Length(2), // status
            Constraint::Min(4),    // error log
            Constraint::Length(1), // keymap
        ])
        .split(area);

    let mode_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Length(30)])
        .split(chunks[0]);

    let mode_line = Line::from(vec![
        Span::styled("Mode: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(
            app.game_state.study_mode().to_string(),
            Style::default().fg(Color::Yellow),
        ),
    ]);
    f.render_widget(Paragraph::new(mode_line), mode_cols[0]);

    if let Some(ref auth) = app.auth {
        let email = Paragraph::new(auth.email.as_str()).style(Style::default().fg(Color::DarkGray));
        f.render_widget(email, mode_cols[1]);
    }

    let summary = app.game_state.deck_summary();
    draw_stats(f, chunks[1], app.game_state.stats(), &summary);
    draw_hand(f, chunks[2], "Dealer", app.game_state.dealer_hand());
    draw_hand(f, chunks[3], "Player", app.game_state.player_hand());

    let status_widget = match &app.status {
        StatusMessage::Correct(msg) => {
            Paragraph::new(msg.as_str()).style(Style::default().fg(Color::Green))
        }
        StatusMessage::Wrong(msg) => Paragraph::new(format!(" {} ", msg))
            .style(Style::default().fg(Color::White).bg(Color::Red)),
        StatusMessage::SyncError(msg) => {
            Paragraph::new(msg.as_str()).style(Style::default().fg(Color::Yellow))
        }
        StatusMessage::None => Paragraph::new(""),
    };
    f.render_widget(status_widget, super::centered_line(chunks[4], 1));

    let log_width = chunks[5].width.saturating_sub(2) as usize;
    let log_items: Vec<ListItem> = app
        .error_log
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            if i == 0 {
                ListItem::new(Line::from(vec![
                    Span::styled(
                        "Error: ",
                        Style::default()
                            .fg(Color::LightRed)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(entry.as_str(), Style::default().fg(Color::White)),
                ]))
            } else if i == 1 {
                ListItem::new(vec![
                    Line::from(vec![Span::styled(
                        "─".repeat(log_width),
                        Style::default().fg(Color::DarkGray),
                    )]),
                    Line::from(vec![
                        Span::styled(
                            "Error: ",
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(entry.as_str()),
                    ]),
                ])
            } else {
                ListItem::new(Line::from(vec![
                    Span::styled(
                        "Error: ",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(entry.as_str()),
                ]))
            }
        })
        .collect();
    let log_list =
        List::new(log_items).block(Block::default().borders(Borders::ALL).title("Mistakes"));
    f.render_widget(log_list, chunks[5]);

    let keymap = if app.show_shuffle_prompt {
        Paragraph::new("Shoe empty. Press ENTER or SPACE to shuffle.").style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Paragraph::new("(H)it | (S)tand | (D)ouble | S(P)lit | (M)ode | Esc:Menu")
    };

    super::footer_with_hint(f, chunks[6], "");
    // Override the hint with the keymap which is special for play
    f.render_widget(keymap, chunks[6]);
}

fn card_color(card: &Card) -> Color {
    if card.suit.is_red() {
        Color::LightRed
    } else {
        Color::White
    }
}

fn draw_hand(f: &mut ratatui::Frame, area: Rect, label: &str, hand: &Hand) {
    let cards = hand.cards();

    let mut top_spans: Vec<Span> = vec![Span::styled(format!("{:>8}", ""), Style::default())];
    let mut mid_spans: Vec<Span> = vec![Span::styled(
        format!("{:>7} ", label),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )];
    let mut bot_spans: Vec<Span> = vec![Span::styled(format!("{:>8}", ""), Style::default())];

    for card in cards {
        let pip = format!("{}", card.pip);
        let suit = card.suit.to_string();

        let pad = if pip.len() < 2 { " " } else { "" };
        let suit_color = card_color(card);
        let border = Style::default().fg(Color::DarkGray);
        let white = Style::default().fg(Color::White);

        top_spans.push(Span::styled("┌──────┐", border));
        mid_spans.push(Span::styled("│", border));
        mid_spans.push(Span::styled(format!(" {} ", pip), white));
        mid_spans.push(Span::styled(suit, Style::default().fg(suit_color)));
        mid_spans.push(Span::styled(format!(" {}", pad), white));
        mid_spans.push(Span::styled("│", border));
        bot_spans.push(Span::styled("└──────┘", border));
    }

    let lines = vec![
        Line::from(top_spans),
        Line::from(mid_spans),
        Line::from(bot_spans),
    ];

    f.render_widget(Paragraph::new(lines), area);
}

fn draw_stats(f: &mut ratatui::Frame, area: Rect, stats: &Stats, summary: &bjsc::DeckSummary) {
    let block = Block::default().borders(Borders::ALL).title("Stats");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    let hands_line = Line::from(vec![
        Span::styled("Hands: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(Stats::numbers_string(
            stats.question_count,
            stats.questions_wrong,
        )),
    ]);
    f.render_widget(Paragraph::new(hands_line), rows[0]);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(rows[1]);

    let categories = [
        ("Hard: ", stats.hard_count, stats.hard_wrong),
        ("Soft: ", stats.soft_count, stats.soft_wrong),
        ("Split: ", stats.split_count, stats.split_wrong),
        ("Dbl: ", stats.double_count, stats.double_wrong),
    ];

    for (i, (label, count, wrong)) in categories.iter().enumerate() {
        let line = Line::from(vec![
            Span::styled(*label, Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(Stats::numbers_string(*count, *wrong)),
        ]);
        f.render_widget(Paragraph::new(line), cols[i]);
    }

    let deck_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(rows[2]);

    let items = [
        ("New: ", summary.unasked, Color::DarkGray),
        ("Weak: ", summary.weak, Color::LightRed),
        ("Mastered: ", summary.mastered, Color::Green),
        ("Due: ", summary.due, Color::Yellow),
    ];

    for (i, (label, count, color)) in items.iter().enumerate() {
        let line = Line::from(vec![
            Span::styled(*label, Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(format!("{}", count), Style::default().fg(*color)),
        ]);
        f.render_widget(Paragraph::new(line), deck_cols[i]);
    }
}
