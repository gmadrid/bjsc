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
use std::sync::mpsc;

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
    Progress,
    Coach,
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
    progress: bjsc::progress::ProgressStats,
    coaching_text: String,
    coach_scroll: u16,
    coaching_rx: Option<mpsc::Receiver<String>>,
}

impl App {
    fn new(mut auth: Option<AuthTokens>, rt: tokio::runtime::Runtime) -> Self {
        let saved = persistence::load_state();
        let mut game_state = GameState::default();
        game_state.set_deck(saved.deck);
        game_state.set_study_mode(saved.mode);

        // If authenticated, try to load from cloud (refresh token if needed)
        if let Some(ref mut auth) = auth {
            let config = supabase_config();
            let result = rt.block_on(api::fetch_user_deck(&config, &auth.access_token));
            let result = if result.is_err() {
                // Try refreshing the token
                if let Some(new_auth) = auth::refresh_tokens(&config, auth, &rt) {
                    let r = rt.block_on(api::fetch_user_deck(&config, &new_auth.access_token));
                    *auth = new_auth;
                    r
                } else {
                    result
                }
            } else {
                result
            };
            if let Ok(Some(row)) = result {
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
            progress: bjsc::progress::ProgressStats::default(),
            coaching_text: String::new(),
            coach_scroll: 0,
            coaching_rx: None,
        }
    }

    fn handle_key(&mut self, code: KeyCode) {
        if code == KeyCode::Tab {
            self.screen = match self.screen {
                Screen::Play => Screen::Histogram,
                Screen::Histogram => Screen::Progress,
                Screen::Progress => Screen::Coach,
                Screen::Coach => Screen::Play,
            };
            if self.screen == Screen::Progress {
                self.refresh_progress();
            }
            if self.screen == Screen::Coach {
                self.refresh_coaching();
            }
            return;
        }

        if self.screen == Screen::Coach {
            match code {
                KeyCode::Down | KeyCode::Char('j') => {
                    self.coach_scroll = self.coach_scroll.saturating_add(1)
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.coach_scroll = self.coach_scroll.saturating_sub(1)
                }
                _ => {}
            }
            return;
        }

        if self.screen != Screen::Play {
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
            // Capture log data before consuming fields
            let answer_log_data = result.table_index_key.clone().map(|key| {
                (
                    key,
                    result.correct,
                    result.player_action.to_string(),
                    result
                        .correct_action
                        .map(|a| a.to_string())
                        .unwrap_or_default(),
                )
            });

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
            if let Some((key, was_correct, player_act, correct_act)) = answer_log_data {
                self.log_answer(&key, was_correct, &player_act, &correct_act);
            }

            if !self.game_state.deal_a_hand() {
                self.show_shuffle_prompt = true;
            }
        }
    }

    fn save(&mut self) {
        // Save locally
        persistence::save_state(&bjsc::SavedState {
            mode: self.game_state.study_mode(),
            deck: self.game_state.deck().clone(),
        });

        // Sync to cloud if authenticated
        if let Some(ref auth) = self.auth {
            let config = supabase_config();
            let result = self.rt.block_on(api::upsert_user_deck(
                &config,
                &auth.access_token,
                &auth.user_id,
                self.game_state.study_mode(),
                self.game_state.deck(),
            ));

            // If save failed, try refreshing the token and retry
            if result.is_err() {
                if let Some(new_auth) = auth::refresh_tokens(&config, auth, &self.rt) {
                    let _ = self.rt.block_on(api::upsert_user_deck(
                        &config,
                        &new_auth.access_token,
                        &new_auth.user_id,
                        self.game_state.study_mode(),
                        self.game_state.deck(),
                    ));
                    self.auth = Some(new_auth);
                }
            }
        }
    }

    fn refresh_coaching(&mut self) {
        if let Some(ref auth) = self.auth {
            self.coaching_text = "Loading coaching advice...".to_string();
            self.coach_scroll = 0;

            let (tx, rx) = mpsc::channel();
            self.coaching_rx = Some(rx);

            let config = supabase_config();
            let auth_clone = auth.clone();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let result = rt.block_on(api::get_coaching(&config, &auth_clone.access_token));

                // If failed, try refreshing token and retry
                let result = if result.is_err() {
                    if let Some(new_auth) = auth::refresh_tokens(&config, &auth_clone, &rt) {
                        rt.block_on(api::get_coaching(&config, &new_auth.access_token))
                    } else {
                        result
                    }
                } else {
                    result
                };

                let text = match result {
                    Ok(text) => text,
                    Err(e) => format!("Error: {}", e),
                };
                let _ = tx.send(text);
            });
        } else {
            self.coaching_text = "Sign in to get coaching advice.".to_string();
        }
    }

    fn poll_coaching(&mut self) {
        if let Some(ref rx) = self.coaching_rx {
            if let Ok(text) = rx.try_recv() {
                self.coaching_text = text;
                self.coaching_rx = None;
            }
        }
    }

    fn refresh_progress(&mut self) {
        if let Some(ref auth) = self.auth {
            let config = supabase_config();
            if let Ok(logs) =
                self.rt
                    .block_on(api::fetch_answer_logs(&config, &auth.access_token, 1000))
            {
                self.progress = bjsc::progress::ProgressStats::from_logs(&logs);
            }
        }
    }

    fn log_answer(
        &self,
        table_index_key: &str,
        correct: bool,
        player_action: &str,
        correct_action: &str,
    ) {
        if let Some(ref auth) = self.auth {
            let config = supabase_config();
            let row = bjsc::supabase::AnswerLogRow {
                user_id: auth.user_id.clone(),
                table_index: table_index_key.to_string(),
                correct,
                player_action: player_action.to_string(),
                correct_action: correct_action.to_string(),
            };
            let _ = self
                .rt
                .block_on(api::insert_answer_log(&config, &auth.access_token, &row));
        }
    }
}

fn draw(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &App) -> io::Result<()> {
    terminal.draw(|f| {
        let area = f.area();
        match app.screen {
            Screen::Play => draw_play(f, area, app),
            Screen::Histogram => draw_histogram(f, area, app),
            Screen::Progress => draw_progress(f, area, app),
            Screen::Coach => draw_coach(f, area, app),
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

    let footer_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Length(22)])
        .split(chunks[6]);

    let keymap = if app.show_shuffle_prompt {
        Paragraph::new("Shoe empty. Press ENTER or SPACE to shuffle.").style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Paragraph::new("(H)it | (S)tand | (D)ouble | S(P)lit | (M)ode | Tab:Stats | (Q)uit")
    };
    f.render_widget(keymap, footer_cols[0]);

    let version = Paragraph::new(env!("BUILD_TIME")).style(Style::default().fg(Color::DarkGray));
    f.render_widget(version, footer_cols[1]);
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

    let hint_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Length(22)])
        .split(footer_rows[1]);

    let hint = Paragraph::new("Tab: Back | (Q)uit").style(Style::default().fg(Color::DarkGray));
    f.render_widget(hint, hint_cols[0]);

    let version = Paragraph::new(env!("BUILD_TIME")).style(Style::default().fg(Color::DarkGray));
    f.render_widget(version, hint_cols[1]);
}

fn draw_progress(f: &mut ratatui::Frame, area: Rect, app: &App) {
    use bjsc::progress::ProgressStats;

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

    // Hint
    let hint_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Length(22)])
        .split(chunks[6]);

    let hint = Paragraph::new("Tab: Next | (Q)uit").style(Style::default().fg(Color::DarkGray));
    f.render_widget(hint, hint_cols[0]);

    let version = Paragraph::new(env!("BUILD_TIME")).style(Style::default().fg(Color::DarkGray));
    f.render_widget(version, hint_cols[1]);
}

fn draw_coach(f: &mut ratatui::Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title
            Constraint::Min(5),    // coaching text
            Constraint::Length(1), // hint
        ])
        .split(area);

    let title = Span::styled(
        "Coach (powered by Claude)",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(Paragraph::new(Line::from(title)), chunks[0]);

    let md_lines = markdown_to_text(&app.coaching_text);
    let text = Paragraph::new(md_lines)
        .wrap(ratatui::widgets::Wrap { trim: false })
        .block(Block::default().borders(Borders::ALL))
        .scroll((app.coach_scroll, 0));
    f.render_widget(text, chunks[1]);

    let hint_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Length(22)])
        .split(chunks[2]);

    let hint = Paragraph::new("↑/↓: Scroll | Tab: Next | (Q)uit")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(hint, hint_cols[0]);

    let version = Paragraph::new(env!("BUILD_TIME")).style(Style::default().fg(Color::DarkGray));
    f.render_widget(version, hint_cols[1]);
}

/// Convert markdown text to styled ratatui Lines.
fn markdown_to_text(md: &str) -> Vec<Line<'_>> {
    use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

    let parser = Parser::new_ext(md, Options::empty());
    let mut lines: Vec<Line> = Vec::new();
    let mut spans: Vec<Span> = Vec::new();
    let mut bold = false;
    let mut in_heading = false;
    let mut list_bullet = false;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                in_heading = true;
                if level == HeadingLevel::H1 || level == HeadingLevel::H2 {
                    bold = true;
                }
            }
            Event::End(TagEnd::Heading(_)) => {
                if !spans.is_empty() {
                    lines.push(Line::from(std::mem::take(&mut spans)));
                }
                lines.push(Line::default());
                in_heading = false;
                bold = false;
            }
            Event::Start(Tag::Paragraph) => {}
            Event::End(TagEnd::Paragraph) => {
                if !spans.is_empty() {
                    lines.push(Line::from(std::mem::take(&mut spans)));
                }
                lines.push(Line::default());
            }
            Event::Start(Tag::Strong) => bold = true,
            Event::End(TagEnd::Strong) => bold = false,
            Event::Start(Tag::Emphasis) => {}
            Event::End(TagEnd::Emphasis) => {}
            Event::Start(Tag::List(_)) => {}
            Event::End(TagEnd::List(_)) => {}
            Event::Start(Tag::Item) => {
                list_bullet = true;
            }
            Event::End(TagEnd::Item) => {
                if !spans.is_empty() {
                    lines.push(Line::from(std::mem::take(&mut spans)));
                }
            }
            Event::Text(text) => {
                if list_bullet {
                    spans.push(Span::raw("• "));
                    list_bullet = false;
                }
                let style = if in_heading {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else if bold {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                spans.push(Span::styled(text.into_string(), style));
            }
            Event::SoftBreak => {
                spans.push(Span::raw(" "));
            }
            Event::HardBreak => {
                if !spans.is_empty() {
                    lines.push(Line::from(std::mem::take(&mut spans)));
                }
            }
            Event::Code(code) => {
                spans.push(Span::styled(
                    code.into_string(),
                    Style::default().fg(Color::Yellow),
                ));
            }
            _ => {}
        }
    }
    if !spans.is_empty() {
        lines.push(Line::from(spans));
    }
    lines
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
        app.poll_coaching();
        draw(&mut terminal, &app)?;

        if event::poll(std::time::Duration::from_millis(100))? {
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
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
