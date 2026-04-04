use bjsc::{phrase_for_row, Action, GameState};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
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
        let mut game_state = GameState::default();
        game_state.deal_a_hand();
        App {
            game_state,
            status: StatusMessage::None,
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

        let action = match code {
            KeyCode::Char(ch) => Action::from_key(ch),
            _ => None,
        };

        let Some(action) = action else { return };

        if let Ok((chart_action, table_index)) = self.game_state.chart_action() {
            let action_from_rules = chart_action.apply_rules();
            if Some(action) == action_from_rules {
                self.game_state.answered_right();
                self.status = StatusMessage::Correct(format!("Correct: {}", action));
            } else {
                self.game_state.answered_wrong();
                self.status = StatusMessage::Wrong(format!(
                    "WRONG: {}",
                    action_from_rules.map(|r| r.to_string()).unwrap_or_default()
                ));

                if let Some(correct_action) = action_from_rules {
                    let log_entry = if let Some(table_index) = table_index {
                        format!(
                            "{} (P: {}, D: {})",
                            phrase_for_row(table_index.row),
                            self.game_state.player_hand(),
                            self.game_state.dealer_hand()
                        )
                    } else {
                        format!(
                            "Player: {}, Dealer: {}, Correct: {}, Guess: {}",
                            self.game_state.player_hand(),
                            self.game_state.dealer_hand(),
                            correct_action,
                            action
                        )
                    };
                    self.error_log.insert(0, log_entry);
                }
            }
            if !self.game_state.deal_a_hand() {
                self.show_shuffle_prompt = true;
            }
        }
    }
}

fn numbers_string(num: usize, wrong: usize) -> String {
    if num == 0 {
        return "0 / 0".to_string();
    }
    let pct = (num - wrong) as f64 / num as f64 * 100.0;
    format!("{} / {} ({:.1}%)", num - wrong, num, pct)
}

fn draw(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &App) -> io::Result<()> {
    terminal.draw(|f| {
        let area = f.area();

        // Main layout: stats, hands, status, log, keymap
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),  // stats
                Constraint::Length(2),  // spacing + dealer
                Constraint::Length(2),  // spacing + player
                Constraint::Length(2),  // spacing + status
                Constraint::Min(4),    // error log
                Constraint::Length(1), // keymap
            ])
            .split(area);

        // Stats block
        draw_stats(f, chunks[0], &app.game_state);

        // Dealer hand
        let dealer_line = Line::from(vec![
            Span::styled("Dealer: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(format!("{}", app.game_state.dealer_hand())),
        ]);
        f.render_widget(Paragraph::new(dealer_line), centered_line(chunks[1], 1));

        // Player hand
        let player_line = Line::from(vec![
            Span::styled("Player: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(format!("{}", app.game_state.player_hand())),
        ]);
        f.render_widget(Paragraph::new(player_line), centered_line(chunks[2], 1));

        // Status message
        let status_widget = match &app.status {
            StatusMessage::Correct(msg) => {
                Paragraph::new(msg.as_str()).style(Style::default().fg(Color::Green))
            }
            StatusMessage::Wrong(msg) => {
                Paragraph::new(format!(" {} ", msg))
                    .style(Style::default().fg(Color::White).bg(Color::Red))
            }
            StatusMessage::None => Paragraph::new(""),
        };
        f.render_widget(status_widget, centered_line(chunks[3], 1));

        // Error log
        let log_items: Vec<ListItem> = app
            .error_log
            .iter()
            .map(|entry| {
                ListItem::new(Line::from(vec![
                    Span::styled("Error: ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    Span::raw(entry.as_str()),
                ]))
            })
            .collect();
        let log_list = List::new(log_items)
            .block(Block::default().borders(Borders::ALL).title("Mistakes"));
        f.render_widget(log_list, chunks[4]);

        // Keymap
        let keymap = if app.show_shuffle_prompt {
            Paragraph::new("Shoe empty. Press ENTER or SPACE to shuffle.")
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        } else {
            Paragraph::new("(H)it | (S)tand | (D)ouble | S(P)lit | (Q)uit")
        };
        f.render_widget(keymap, chunks[5]);
    })?;
    Ok(())
}

fn centered_line(area: Rect, offset: u16) -> Rect {
    Rect::new(area.x, area.y + offset, area.width, 1)
}

fn draw_stats(f: &mut ratatui::Frame, area: Rect, gs: &GameState) {
    let block = Block::default().borders(Borders::ALL).title("Stats");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    // Top row: Hands score
    let hands_line = Line::from(vec![
        Span::styled("Hands: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(numbers_string(
            gs.num_questions_asked(),
            gs.num_questions_wrong(),
        )),
    ]);
    f.render_widget(Paragraph::new(hands_line), rows[0]);

    // Bottom row: Hard | Soft | Split | Double (placeholders matching original)
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(rows[1]);

    for (i, label) in ["Hard:", "Soft:", "Split:", "Double:"].iter().enumerate() {
        let line = Line::from(vec![
            Span::styled(*label, Style::default().add_modifier(Modifier::BOLD)),
        ]);
        f.render_widget(Paragraph::new(line), cols[i]);
    }
}

fn main() -> io::Result<()> {
    // Setup terminal
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

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
