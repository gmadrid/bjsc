mod api;
mod auth;

use auth::AuthTokens;
use bjsc::card::Card;
use bjsc::hand::Hand;
use bjsc::supabase::SupabaseConfig;
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
use ratatui::widgets::{Bar, BarChart, BarGroup, Block, Borders, List, ListItem, Paragraph};
use ratatui::Terminal;
use std::io;

const SUPABASE_URL: &str = "https://pecwxusghnxlvzmfcqrj.supabase.co";
const SUPABASE_ANON_KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6InBlY3d4dXNnaG54bHZ6bWZjcXJqIiwicm9sZSI6ImFub24iLCJpYXQiOjE3NzUzNTY3MjUsImV4cCI6MjA5MDkzMjcyNX0.LwgaAHruQ8cA3mHrtCCB00WSqttpwRusAf0Y1WEFWuE";

fn supabase_config() -> SupabaseConfig {
    SupabaseConfig {
        base_url: SUPABASE_URL.to_string(),
        anon_key: SUPABASE_ANON_KEY.to_string(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Screen {
    Play,
    Histogram,
}

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
    screen: Screen,
    auth: Option<AuthTokens>,
    rt: tokio::runtime::Runtime,
}

impl App {
    fn new(auth: Option<AuthTokens>, rt: tokio::runtime::Runtime) -> Self {
        let saved = persistence::load_state();
        let mut game_state = GameState::default();
        game_state.set_deck(saved.deck);
        game_state.set_study_mode(saved.mode);

        // If authenticated, try to load from cloud
        if let Some(ref auth) = auth {
            let config = supabase_config();
            if let Ok(Some(row)) = rt.block_on(api::fetch_user_deck(&config, &auth.access_token)) {
                game_state.set_deck(row.deck);
                game_state.set_study_mode(row.study_mode);
            }
        }

        game_state.deal_a_hand();

        let status = if game_state.study_mode() != bjsc::StudyMode::All {
            StatusMessage::Correct(format!("Resumed: {}", game_state.study_mode()))
        } else {
            StatusMessage::None
        };

        App {
            game_state,
            status,
            error_log: Vec::new(),
            show_shuffle_prompt: false,
            screen: Screen::Play,
            auth,
            rt,
        }
    }

    fn handle_key(&mut self, code: KeyCode) {
        if code == KeyCode::Tab {
            self.screen = match self.screen {
                Screen::Play => Screen::Histogram,
                Screen::Histogram => Screen::Play,
            };
            return;
        }

        if self.screen == Screen::Histogram {
            return;
        }

        if self.show_shuffle_prompt {
            if code == KeyCode::Enter || code == KeyCode::Char(' ') {
                self.game_state.shuffle();
                self.game_state.deal_a_hand();
                self.show_shuffle_prompt = false;
            }
            return;
        }

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
        // Save locally
        persistence::save_state(&bjsc::SavedState {
            mode: self.game_state.study_mode(),
            deck: self.game_state.deck().clone(),
        });

        // Sync to cloud if authenticated
        if let Some(ref auth) = self.auth {
            let config = supabase_config();
            let _ = self.rt.block_on(api::upsert_user_deck(
                &config,
                &auth.access_token,
                &auth.user_id,
                self.game_state.study_mode(),
                self.game_state.deck(),
            ));
        }
    }
}

fn draw(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &App) -> io::Result<()> {
    terminal.draw(|f| {
        let area = f.area();
        match app.screen {
            Screen::Play => draw_play(f, area, app),
            Screen::Histogram => draw_histogram(f, area, app),
        }
    })?;
    Ok(())
}

fn draw_play(f: &mut ratatui::Frame, area: Rect, app: &App) {
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

    let mode_line = Line::from(vec![
        Span::styled("Mode: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(
            app.game_state.study_mode().to_string(),
            Style::default().fg(Color::Yellow),
        ),
    ]);
    f.render_widget(Paragraph::new(mode_line), chunks[0]);

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
        StatusMessage::None => Paragraph::new(""),
    };
    f.render_widget(status_widget, centered_line(chunks[4], 1));

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
        Paragraph::new("(H)it | (S)tand | (D)ouble | S(P)lit | (M)ode | Tab:Stats | (Q)uit")
    };
    f.render_widget(keymap, chunks[6]);
}

fn draw_histogram(f: &mut ratatui::Frame, area: Rect, app: &App) {
    let box_counts = app.game_state.box_counts();
    let unseen = app.game_state.unseen_count();

    let intervals = ["20s", "1m", "5m", "30m", "2h", "6h", "1d", "3d", "1w"];
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

    let bars: Vec<Bar> = box_counts
        .iter()
        .enumerate()
        .map(|(i, &count)| {
            Bar::default()
                .value(count as u64)
                .label(Line::from(format!("B{} ({})", i, intervals[i])))
                .text_value(format!("{}", count))
                .style(Style::default().fg(colors[i]))
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

    let chart = BarChart::default()
        .block(Block::default().borders(Borders::ALL))
        .bar_width(7)
        .bar_gap(2)
        .group_gap(0)
        .data(BarGroup::default().bars(&bars))
        .max(box_counts.iter().copied().max().unwrap_or(1).max(1) as u64);
    f.render_widget(chart, chunks[1]);

    let footer_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(chunks[2]);

    let unseen_line = Line::from(vec![
        Span::styled("Unseen: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(format!("{}", unseen), Style::default().fg(Color::DarkGray)),
    ]);
    f.render_widget(Paragraph::new(unseen_line), footer_rows[0]);

    let hint = Paragraph::new("Tab: Back | (Q)uit").style(Style::default().fg(Color::DarkGray));
    f.render_widget(hint, footer_rows[1]);
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
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    // Authenticate before entering TUI
    let config = supabase_config();
    let auth = match auth::load_stored_tokens() {
        Some(tokens) => {
            println!("Loaded saved auth session.");
            Some(tokens)
        }
        None => {
            println!("No saved session. Starting Google sign-in...");
            match auth::login(&config) {
                Ok(tokens) => {
                    println!("Signed in successfully!");
                    Some(tokens)
                }
                Err(e) => {
                    eprintln!("Auth failed: {}. Continuing offline.", e);
                    None
                }
            }
        }
    };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(auth, rt);

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
