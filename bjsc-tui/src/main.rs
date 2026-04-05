use bjsc::card::Card;
use bjsc::hand::Hand;
use bjsc::{persistence, Action, GameState, Stats};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Terminal;
use std::io;

#[derive(Debug, Clone)]
enum StatusMessage {
    Correct(String),
    Wrong(String),
    None,
}

struct App {
    game_state: GameState,
    status: StatusMessage,
    error_log: Vec<String>,
    show_shuffle_prompt: bool,
}

impl App {
    fn new() -> Self {
        let saved = persistence::load_state();
        let mut game_state = GameState::default();
        game_state.set_deck(saved.deck);
        game_state.set_study_mode(saved.mode);
        game_state.deal_a_hand();
        let status = if saved.mode != bjsc::StudyMode::All {
            StatusMessage::Correct(format!("Resumed: {}", saved.mode))
        } else {
            StatusMessage::None
        };
        App {
            game_state,
            status,
            error_log: Vec::new(),
            show_shuffle_prompt: false,
        }
    }

    fn handle_key(&mut self, code: KeyCode) {
        if self.show_shuffle_prompt {
            if code == KeyCode::Enter || code == KeyCode::Char(' ') {
                self.game_state.shuffle();
                self.game_state.deal_a_hand();
                self.show_shuffle_prompt = false;
            }
            return;
        }

        // Mode cycling
        if code == KeyCode::Char('m') {
            let new_mode = self.game_state.study_mode().next();
            self.game_state.set_study_mode(new_mode);
            self.game_state.deal_a_hand();
            self.status = StatusMessage::None;
            self.save();
            return;
        }

        let action = match code {
            KeyCode::Char(ch) => Action::from_key(ch),
            _ => None,
        };
        let Some(action) = action else { return };

        if let Some(result) = self.game_state.check_answer(action) {
            if result.correct {
                self.status = StatusMessage::Correct(format!("Correct: {}", result.player_action));
            } else {
                self.status = StatusMessage::Wrong(format!(
                    "WRONG: {}",
                    result
                        .correct_action
                        .map(|a| a.to_string())
                        .unwrap_or_default()
                ));
                if let Some(log_entry) = result.log_entry {
                    self.error_log.insert(0, log_entry);
                }
            }

            self.save();

            if !self.game_state.deal_a_hand() {
                self.show_shuffle_prompt = true;
            }
        }
    }

    fn save(&self) {
        persistence::save_state(&bjsc::SavedState {
            mode: self.game_state.study_mode(),
            deck: self.game_state.deck().clone(),
        });
    }
}

fn draw(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &App) -> io::Result<()> {
    terminal.draw(|f| {
        let area = f.area();

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

        // Mode line
        let mode_line = Line::from(vec![
            Span::styled("Mode: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                app.game_state.study_mode().to_string(),
                Style::default().fg(Color::Yellow),
            ),
        ]);
        f.render_widget(Paragraph::new(mode_line), chunks[0]);

        // Stats block
        let summary = app.game_state.deck_summary();
        draw_stats(f, chunks[1], app.game_state.stats(), &summary);

        // Dealer hand
        draw_hand(f, chunks[2], "Dealer", app.game_state.dealer_hand());

        // Player hand
        draw_hand(f, chunks[3], "Player", app.game_state.player_hand());

        // Status message
        let status_widget = match &app.status {
            StatusMessage::Correct(msg) => {
                Paragraph::new(msg.as_str()).style(Style::default().fg(Color::Green))
            }
            StatusMessage::Wrong(msg) => Paragraph::new(format!(" {} ", msg))
                .style(Style::default().fg(Color::White).bg(Color::Red)),
            StatusMessage::None => Paragraph::new(""),
        };
        f.render_widget(status_widget, centered_line(chunks[4], 1));

        // Error log
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

        // Keymap
        let keymap = if app.show_shuffle_prompt {
            Paragraph::new("Shoe empty. Press ENTER or SPACE to shuffle.").style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Paragraph::new("(H)it | (S)tand | (D)ouble | S(P)lit | (M)ode | (Q)uit")
        };
        f.render_widget(keymap, chunks[6]);
    })?;
    Ok(())
}

fn centered_line(area: Rect, offset: u16) -> Rect {
    Rect::new(area.x, area.y + offset, area.width, 1)
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

    // Build 3 lines: tops, middles, bottoms
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
        let suit = format!("{}", card.suit);

        let pad = if pip.len() < 2 { " " } else { "" };
        let suit_color = card_color(card);
        let border = Style::default().fg(Color::DarkGray);
        let white = Style::default().fg(Color::White);

        top_spans.push(Span::styled("┌──────┐", border));
        mid_spans.push(Span::styled("│", border));
        mid_spans.push(Span::styled(format!(" {} ", pip), white));
        mid_spans.push(Span::styled(
            suit.to_string(),
            Style::default().fg(suit_color),
        ));
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

    // Row 1: total score
    let hands_line = Line::from(vec![
        Span::styled("Hands: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(Stats::numbers_string(
            stats.question_count,
            stats.questions_wrong,
        )),
    ]);
    f.render_widget(Paragraph::new(hands_line), rows[0]);

    // Row 2: per-category stats
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
        ("Split: ", stats.split_count, stats.splits_wrong),
        ("Dbl: ", stats.double_count, stats.doubles_wrong),
    ];

    for (i, (label, count, wrong)) in categories.iter().enumerate() {
        let line = Line::from(vec![
            Span::styled(*label, Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(Stats::numbers_string(*count, *wrong)),
        ]);
        f.render_widget(Paragraph::new(line), cols[i]);
    }

    // Row 3: Deck summary
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

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        draw(&mut terminal, &app)?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('q') if !app.show_shuffle_prompt => break,
                code => app.handle_key(code),
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
