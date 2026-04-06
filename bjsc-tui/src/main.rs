mod api;
mod auth;
mod draw;
mod split_bar_chart;

use auth::AuthTokens;
use bjsc::{persistence, Action, GameState, SupabaseConfig};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::sync::mpsc;

fn supabase_config() -> SupabaseConfig {
    SupabaseConfig::default()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Screen {
    Play,
    Histogram,
    Progress,
    Coach,
    Strategy,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum StrategyTab {
    Descriptive,
    Tables,
}

#[derive(Debug, Clone)]
pub(crate) enum StatusMessage {
    Correct(String),
    Wrong(String),
    SyncError(String),
    None,
}

pub(crate) struct App {
    pub(crate) game_state: GameState,
    pub(crate) status: StatusMessage,
    pub(crate) error_log: Vec<String>,
    pub(crate) show_shuffle_prompt: bool,
    pub(crate) screen: Screen,
    pub(crate) auth: Option<AuthTokens>,
    pub(crate) rt: tokio::runtime::Runtime,
    pub(crate) progress: bjsc::progress::ProgressStats,
    pub(crate) coaching_text: String,
    pub(crate) coach_scroll: u16,
    coaching_rx: Option<mpsc::Receiver<String>>,
    coaching_fetched_at_count: u32,
    coaching_fetched_date: String,
    sync_error_tx: mpsc::Sender<String>,
    sync_error_rx: mpsc::Receiver<String>,
    pub(crate) screen_picker: Option<usize>,
    pub(crate) confirm_quit: bool,
    pub(crate) strategy_tab: StrategyTab,
    pub(crate) strategy_scroll: u16,
}

impl App {
    fn new(mut auth: Option<AuthTokens>, rt: tokio::runtime::Runtime) -> Self {
        let saved = persistence::load_state();
        let mut game_state = GameState::default();
        game_state.set_deck(saved.deck);
        game_state.set_study_mode(saved.mode);

        // If authenticated, refresh token if expired, then load from cloud
        if let Some(ref mut auth) = auth {
            let config = supabase_config();

            // Proactively refresh if token is expired
            if bjsc::supabase::is_jwt_expired(&auth.access_token) {
                if let Some(new_auth) = auth::refresh_tokens(&config, auth, &rt) {
                    *auth = new_auth;
                }
            }

            let result = rt.block_on(bjsc::api::fetch_user_deck(
                &api::ReqwestClient,
                &config,
                &auth.access_token,
            ));
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

        let (sync_error_tx, sync_error_rx) = mpsc::channel();

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
            coaching_fetched_at_count: 0,
            coaching_fetched_date: String::new(),
            sync_error_tx,
            sync_error_rx,
            screen_picker: None,
            confirm_quit: false,
            strategy_tab: StrategyTab::Descriptive,
            strategy_scroll: 0,
        }
    }

    fn screen_index(&self) -> usize {
        match self.screen {
            Screen::Play => 0,
            Screen::Histogram => 1,
            Screen::Progress => 2,
            Screen::Coach => 3,
            Screen::Strategy => 4,
        }
    }

    fn go_to_screen(&mut self, idx: usize) {
        let screen = match idx {
            0 => Screen::Play,
            1 => Screen::Histogram,
            2 => Screen::Progress,
            3 => Screen::Coach,
            4 => Screen::Strategy,
            _ => return,
        };
        self.screen = screen;
        if screen == Screen::Progress {
            self.refresh_progress();
        }
        if screen == Screen::Coach && self.should_refresh_coaching() {
            self.refresh_coaching();
        }
    }

    /// Returns true if the app should quit.
    fn handle_key(&mut self, code: KeyCode) -> bool {
        // Confirm quit dialog is open
        if self.confirm_quit {
            match code {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => return true,
                _ => self.confirm_quit = false,
            }
            return false;
        }

        // Screen picker is open
        if let Some(ref mut sel) = self.screen_picker {
            match code {
                KeyCode::Up | KeyCode::Char('k') => *sel = sel.saturating_sub(1),
                KeyCode::Down | KeyCode::Char('j') => *sel = (*sel + 1).min(5),
                KeyCode::Enter => {
                    let idx = *sel;
                    self.screen_picker = None;
                    if idx == 5 {
                        self.confirm_quit = true;
                    } else {
                        self.go_to_screen(idx);
                    }
                }
                KeyCode::Char('p') => {
                    self.screen_picker = None;
                    self.go_to_screen(0);
                }
                KeyCode::Char('s') => {
                    self.screen_picker = None;
                    self.go_to_screen(1);
                }
                KeyCode::Char('g') => {
                    self.screen_picker = None;
                    self.go_to_screen(2);
                }
                KeyCode::Char('c') => {
                    self.screen_picker = None;
                    self.go_to_screen(3);
                }
                KeyCode::Char('t') => {
                    self.screen_picker = None;
                    self.go_to_screen(4);
                }
                KeyCode::Char('q') => {
                    self.screen_picker = None;
                    self.confirm_quit = true;
                }
                KeyCode::Esc => self.screen_picker = None,
                _ => {}
            }
            return false;
        }

        if code == KeyCode::Esc {
            self.screen_picker = Some(self.screen_index());
            return false;
        }

        if self.screen == Screen::Strategy {
            match code {
                KeyCode::Down | KeyCode::Char('j') => {
                    self.strategy_scroll = self.strategy_scroll.saturating_add(1)
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.strategy_scroll = self.strategy_scroll.saturating_sub(1)
                }
                KeyCode::Tab | KeyCode::Left | KeyCode::Right => {
                    self.strategy_tab = match self.strategy_tab {
                        StrategyTab::Descriptive => StrategyTab::Tables,
                        StrategyTab::Tables => StrategyTab::Descriptive,
                    };
                    self.strategy_scroll = 0;
                }
                _ => {}
            }
            return false;
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
            return false;
        }

        if self.screen != Screen::Play {
            return false;
        }

        if self.show_shuffle_prompt {
            if code == KeyCode::Enter || code == KeyCode::Char(' ') {
                self.game_state.shuffle();
                self.game_state.deal_a_hand();
                self.show_shuffle_prompt = false;
            }
            return false;
        }

        if code == KeyCode::Char('m') {
            let new_mode = self.game_state.study_mode().next();
            self.game_state.set_study_mode(new_mode);
            self.game_state.deal_a_hand();
            self.status = StatusMessage::None;
            self.save();
            return false;
        }

        let action = match code {
            KeyCode::Char(ch) => Action::from_key(ch),
            _ => None,
        };
        let Some(action) = action else {
            return false;
        };

        if let Some(result) = self.game_state.check_answer(action) {
            let log_data = result.log_data();

            if result.correct {
                self.status = StatusMessage::Correct(result.status_message());
            } else {
                self.status = StatusMessage::Wrong(result.status_message());
                if let Some(log_entry) = result.log_entry {
                    self.error_log.insert(0, log_entry);
                }
            }

            self.save();
            if let Some((key, was_correct, player_act, correct_act)) = log_data {
                self.log_answer(&key, was_correct, &player_act, &correct_act);
            }

            if !self.game_state.deal_a_hand() {
                self.show_shuffle_prompt = true;
            }
        }
        false
    }

    fn save(&mut self) {
        // Save locally
        persistence::save_state(&bjsc::SavedState {
            mode: self.game_state.study_mode(),
            deck: self.game_state.deck().clone(),
        });

        // Sync to cloud in background
        if let Some(ref auth) = self.auth {
            let config = supabase_config();
            let auth_clone = auth.clone();
            let mode = self.game_state.study_mode();
            let deck = self.game_state.deck().clone();

            let err_tx = self.sync_error_tx.clone();
            self.rt.spawn(async move {
                let result = bjsc::api::upsert_user_deck(
                    &api::ReqwestClient,
                    &config,
                    &auth_clone.access_token,
                    &auth_clone.user_id,
                    mode,
                    &deck,
                )
                .await;
                if let Err(e) = result {
                    if let Some(new_auth) =
                        bjsc::api::refresh_session(&api::ReqwestClient, &config, &auth_clone).await
                    {
                        if let Err(e) = bjsc::api::upsert_user_deck(
                            &api::ReqwestClient,
                            &config,
                            &new_auth.access_token,
                            &new_auth.user_id,
                            mode,
                            &deck,
                        )
                        .await
                        {
                            let _ = err_tx.send(format!("Cloud save failed: {}", e));
                        }
                    } else {
                        let _ = err_tx.send(format!("Cloud save failed: {}", e));
                    }
                }
            });
        }
    }

    fn should_refresh_coaching(&self) -> bool {
        // Never fetched
        if self.coaching_text.is_empty() {
            return true;
        }
        let current_count = self.game_state.stats().question_count;
        let questions_since = current_count.saturating_sub(self.coaching_fetched_at_count);

        // 50+ questions since last coaching
        if questions_since >= 50 {
            return true;
        }

        // Next day and at least 1 question since last coaching
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        if today != self.coaching_fetched_date && questions_since >= 1 {
            return true;
        }

        false
    }

    fn refresh_coaching(&mut self) {
        if let Some(ref auth) = self.auth {
            self.coaching_text = "Loading coaching advice...".to_string();
            self.coach_scroll = 0;
            self.coaching_fetched_at_count = self.game_state.stats().question_count;
            self.coaching_fetched_date = chrono::Local::now().format("%Y-%m-%d").to_string();

            let (tx, rx) = mpsc::channel();
            self.coaching_rx = Some(rx);

            let config = supabase_config();
            let auth_clone = auth.clone();

            self.rt.spawn(async move {
                let result =
                    bjsc::api::get_coaching(&api::ReqwestClient, &config, &auth_clone.access_token)
                        .await;

                // If failed, try refreshing token and retry
                let result = if result.is_err() {
                    if let Some(new_auth) =
                        bjsc::api::refresh_session(&api::ReqwestClient, &config, &auth_clone).await
                    {
                        bjsc::api::get_coaching(
                            &api::ReqwestClient,
                            &config,
                            &new_auth.access_token,
                        )
                        .await
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

    fn poll_sync_errors(&mut self) {
        if let Ok(msg) = self.sync_error_rx.try_recv() {
            self.status = StatusMessage::SyncError(msg);
        }
    }

    fn refresh_progress(&mut self) {
        if let Some(ref auth) = self.auth {
            let config = supabase_config();
            if let Ok(logs) = self.rt.block_on(bjsc::api::fetch_answer_logs(
                &api::ReqwestClient,
                &config,
                &auth.access_token,
                1000,
            )) {
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
            let token = auth.access_token.clone();
            let row = bjsc::supabase::AnswerLogRow {
                user_id: auth.user_id.clone(),
                table_index: table_index_key.to_string(),
                correct,
                player_action: player_action.to_string(),
                correct_action: correct_action.to_string(),
            };
            let err_tx = self.sync_error_tx.clone();
            self.rt.spawn(async move {
                if let Err(e) =
                    bjsc::api::insert_answer_log(&api::ReqwestClient, &config, &token, &row).await
                {
                    let _ = err_tx.send(format!("Log sync failed: {}", e));
                }
            });
        }
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
        app.poll_sync_errors();
        draw::draw(&mut terminal, &app)?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                if app.handle_key(key.code) {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
