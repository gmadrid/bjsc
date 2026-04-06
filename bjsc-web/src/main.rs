mod api;
mod auth;

use auth::AuthState;
use bjsc::{Action, GameState, Stats, SupabaseConfig};
use leptos::prelude::*;
use spaced_rep::NUM_BOXES;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Play,
    Stats,
    Progress,
    Coach,
    Strategy,
}

impl Screen {
    fn title(self) -> &'static str {
        match self {
            Screen::Play => "Play",
            Screen::Stats => "Stats",
            Screen::Progress => "Progress",
            Screen::Coach => "Coach",
            Screen::Strategy => "Strategy",
        }
    }
}

fn local_date_string() -> String {
    let d = js_sys::Date::new_0();
    format!(
        "{}-{:02}-{:02}",
        d.get_full_year(),
        d.get_month() + 1,
        d.get_date()
    )
}

thread_local! {
    static GAME: RefCell<GameState> = RefCell::new({
        let mut gs = GameState::default();
        gs.deal_a_hand();
        gs
    });
}

fn supabase_config() -> SupabaseConfig {
    SupabaseConfig::default()
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

/// Start a 1-second interval that ticks the countdown display while drill-waiting.
/// When a card becomes due, deals it and clears the interval.
/// If not in drill-waiting state, clears any existing interval.
fn schedule_drill_timer(timer_id: RwSignal<Option<i32>>, game_display: RwSignal<DisplayData>) {
    // Clear existing interval
    if let Some(id) = timer_id.get_untracked() {
        if let Some(w) = web_sys::window() {
            w.clear_interval_with_handle(id);
        }
        timer_id.set(None);
    }

    let wait = GAME.with_borrow(|gs| gs.drill_wait_secs());
    if wait.is_some() {
        let cb = Closure::<dyn FnMut()>::new(move || {
            // Check if a card is now due
            let ready = GAME.with_borrow(|gs| gs.drill_wait_secs().is_none());
            if ready {
                GAME.with_borrow_mut(|gs| {
                    gs.deal_a_hand();
                });
                // Clear the interval
                if let Some(id) = timer_id.get_untracked() {
                    if let Some(w) = web_sys::window() {
                        w.clear_interval_with_handle(id);
                    }
                    timer_id.set(None);
                }
            }
            // Refresh display (updates countdown or shows new card)
            game_display.set(read_display());
        });
        if let Some(w) = web_sys::window() {
            if let Ok(id) = w.set_interval_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                1000,
            ) {
                timer_id.set(Some(id));
            }
        }
        cb.forget();
    }
}

#[derive(Clone, Default)]
struct DisplayData {
    dealer: String,
    player: String,
    score: String,
    hard: String,
    soft: String,
    split: String,
    double: String,
    box_counts: [u32; NUM_BOXES as usize],
    box_due: [u32; NUM_BOXES as usize],
    unseen: u32,
    new_count: u32,
    weak_count: u32,
    mastered_count: u32,
    due_count: u32,
    mode_key: String,
    drill_wait_secs: Option<u64>,
}

fn read_display() -> DisplayData {
    GAME.with_borrow(|gs| {
        let s = gs.stats();
        let ds = gs.deck_summary();
        DisplayData {
            dealer: gs.dealer_hand().to_string(),
            player: gs.player_hand().to_string(),
            score: Stats::numbers_string(s.question_count, s.questions_wrong),
            hard: Stats::numbers_string(s.hard_count, s.hard_wrong),
            soft: Stats::numbers_string(s.soft_count, s.soft_wrong),
            split: Stats::numbers_string(s.split_count, s.split_wrong),
            double: Stats::numbers_string(s.double_count, s.double_wrong),
            mode_key: gs.study_mode().key().to_string(),
            box_counts: gs.box_counts(),
            box_due: gs.box_due_counts(),
            unseen: gs.unseen_count(),
            new_count: ds.unasked,
            weak_count: ds.weak,
            mastered_count: ds.mastered,
            due_count: ds.due,
            drill_wait_secs: gs.drill_wait_secs(),
        }
    })
}

/// Log an answer to Supabase (fire-and-forget).
fn log_answer_to_cloud(
    auth: &AuthState,
    table_index_key: &str,
    correct: bool,
    player_action: &str,
    correct_action: &str,
) {
    let config = supabase_config();
    let token = auth.access_token.clone();
    let row = bjsc::supabase::AnswerLogRow {
        user_id: auth.user_id.clone(),
        table_index: table_index_key.to_string(),
        correct,
        player_action: player_action.to_string(),
        correct_action: correct_action.to_string(),
    };

    leptos::task::spawn_local(async move {
        if let Err(e) = bjsc::api::insert_answer_log(&api::GlooClient, &config, &token, &row).await
        {
            web_sys::console::warn_1(&format!("Log sync failed: {}", e).into());
        }
    });
}

/// Save the current game state to Supabase (fire-and-forget).
fn save_to_cloud(auth: &AuthState) {
    let config = supabase_config();
    let token = auth.access_token.clone();
    let user_id = auth.user_id.clone();
    let (mode, deck) = GAME.with_borrow(|gs| (gs.study_mode(), gs.deck().clone()));

    leptos::task::spawn_local(async move {
        if let Err(e) =
            bjsc::api::upsert_user_deck(&api::GlooClient, &config, &token, &user_id, mode, &deck)
                .await
        {
            web_sys::console::warn_1(&format!("Cloud save failed: {}", e).into());
        }
    });
}

#[component]
fn App() -> impl IntoView {
    // Check for OAuth redirect tokens, then try localStorage
    let initial_auth = auth::check_url_for_tokens().or_else(auth::load_from_storage);
    let auth_state: RwSignal<Option<AuthState>> = RwSignal::new(initial_auth);

    view! {
        <div class="w-full max-w-xl px-4">
            {move || {
                if auth_state.get().is_some() {
                    view! { <GameView auth_state=auth_state /> }.into_any()
                } else {
                    view! { <LoginView /> }.into_any()
                }
            }}
        </div>
    }
}

#[component]
fn LoginView() -> impl IntoView {
    let login_url = auth::google_login_url(bjsc::supabase::SUPABASE_URL);

    view! {
        <div class="flex flex-col items-center justify-center pt-24 gap-8">
            <h1 class="text-3xl font-bold text-cyan-400">"BJSC"</h1>
            <p class="text-gray-400">"Blackjack Strategy Card Trainer"</p>
            <a
                href=login_url
                class="px-6 py-3 bg-white text-gray-900 font-bold rounded-lg hover:bg-gray-200 transition-colors"
            >
                "Sign in with Google"
            </a>
        </div>
    }
}

#[component]
fn GameView(auth_state: RwSignal<Option<AuthState>>) -> impl IntoView {
    let game_display = RwSignal::new(DisplayData::default());
    let status_text = RwSignal::new(String::new());
    let status_is_error = RwSignal::new(false);
    let status_visible = RwSignal::new(false);
    let errors: RwSignal<Vec<String>> = RwSignal::new(vec![]);
    let show_shuffle = RwSignal::new(false);
    let screen = RwSignal::new(Screen::Play);
    let coaching_text = RwSignal::new(String::new());
    let coaching_fetched_at_count = RwSignal::new(0u32);
    let coaching_fetched_date = RwSignal::new(String::new());
    let progress_stats: RwSignal<bjsc::progress::ProgressStats> =
        RwSignal::new(bjsc::progress::ProgressStats::default());
    let loading = RwSignal::new(true);
    let drill_timer_id: RwSignal<Option<i32>> = RwSignal::new(None);

    let sync_all = move || {
        game_display.set(read_display());
    };

    // Load deck from Supabase on mount, refreshing token if needed
    if let Some(auth) = auth_state.get_untracked() {
        let config = supabase_config();
        leptos::task::spawn_local(async move {
            let mut token = auth.access_token.clone();

            // Try to fetch; if it fails, attempt token refresh
            let result = bjsc::api::fetch_user_deck(&api::GlooClient, &config, &token).await;
            let result = if result.is_err() {
                if let Some(new_auth) =
                    bjsc::api::refresh_session(&api::GlooClient, &config, &auth).await
                {
                    token = new_auth.access_token.clone();
                    auth_state.set(Some(new_auth));
                    bjsc::api::fetch_user_deck(&api::GlooClient, &config, &token).await
                } else {
                    // Refresh failed — clear auth and force re-login
                    auth::clear_storage();
                    auth_state.set(None);
                    loading.set(false);
                    return;
                }
            } else {
                result
            };

            match result {
                Ok(Some(row)) => {
                    GAME.with_borrow_mut(|gs| {
                        gs.set_deck(row.deck);
                        gs.set_study_mode(row.study_mode);
                        gs.deal_a_hand();
                    });
                }
                Ok(None) => {
                    // No saved deck — start fresh
                    GAME.with_borrow_mut(|gs| gs.deal_a_hand());
                }
                Err(e) => {
                    web_sys::console::warn_1(
                        &format!("Failed to load deck from cloud: {}", e).into(),
                    );
                    GAME.with_borrow_mut(|gs| gs.deal_a_hand());
                }
            }
            loading.set(false);
            sync_all();
            schedule_drill_timer(drill_timer_id, game_display);
        });
    } else {
        web_sys::console::warn_1(&"Auth state missing on mount, using local data.".into());
        loading.set(false);
        sync_all();
        schedule_drill_timer(drill_timer_id, game_display);
    }

    let do_action = move |action: Action| {
        if show_shuffle.get_untracked() || loading.get_untracked() {
            return;
        }
        // Block actions when waiting for next drill card
        if game_display.get_untracked().drill_wait_secs.is_some() {
            return;
        }
        let outcome = GAME.with_borrow_mut(|gs| {
            let result = gs.check_answer(action);
            let shoe_done = if result.is_some() {
                !gs.deal_a_hand()
            } else {
                false
            };
            (result, shoe_done)
        });
        if let (Some(result), shoe_done) = outcome {
            let log_data = result.log_data();
            status_text.set(result.status_message());
            status_is_error.set(!result.correct);
            if !result.correct {
                if let Some(entry) = result.log_entry {
                    errors.update(|e| e.insert(0, entry));
                }
            }
            status_visible.set(true);
            if shoe_done {
                show_shuffle.set(true);
            }

            // Save to cloud and log answer
            if let Some(auth) = auth_state.get_untracked() {
                save_to_cloud(&auth);
                if let Some((key, was_correct, player_act, correct_act)) = log_data {
                    log_answer_to_cloud(&auth, &key, was_correct, &player_act, &correct_act);
                }
            }
        }
        sync_all();
        schedule_drill_timer(drill_timer_id, game_display);
    };

    let do_shuffle = move || {
        GAME.with_borrow_mut(|gs| {
            gs.shuffle();
            gs.deal_a_hand();
        });
        show_shuffle.set(false);
        sync_all();
    };

    let set_mode = move |mode: bjsc::StudyMode| {
        GAME.with_borrow_mut(|gs| {
            gs.set_study_mode(mode);
            gs.deal_a_hand();
        });
        status_visible.set(false);
        show_shuffle.set(false);
        sync_all();
        schedule_drill_timer(drill_timer_id, game_display);

        // Save mode change to cloud immediately
        if let Some(auth) = auth_state.get_untracked() {
            save_to_cloud(&auth);
        }
    };

    let sign_out = move || {
        auth::clear_storage();
        auth_state.set(None);
    };

    let menu_open = RwSignal::new(false);

    let go_to_screen = move |next: Screen| {
        if next == Screen::Progress {
            if let Some(auth) = auth_state.get_untracked() {
                let config = supabase_config();
                let token = auth.access_token.clone();
                leptos::task::spawn_local(async move {
                    if let Ok(logs) =
                        bjsc::api::fetch_answer_logs(&api::GlooClient, &config, &token, 1000).await
                    {
                        progress_stats.set(bjsc::progress::ProgressStats::from_logs(&logs));
                    }
                });
            }
        }
        if next == Screen::Coach {
            let current_count = GAME.with_borrow(|gs| gs.stats().question_count);
            let questions_since =
                current_count.saturating_sub(coaching_fetched_at_count.get_untracked());
            let today = local_date_string();
            let is_new_day = coaching_fetched_date.get_untracked() != today;
            let never_fetched = coaching_text.get_untracked().is_empty();

            let should_refresh =
                never_fetched || questions_since >= 50 || (is_new_day && questions_since >= 1);

            if should_refresh {
                coaching_fetched_at_count.set(current_count);
                coaching_fetched_date.set(today);
                coaching_text.set("Loading coaching advice...".to_string());
                if let Some(auth) = auth_state.get_untracked() {
                    let config = supabase_config();
                    leptos::task::spawn_local(async move {
                        let mut token = auth.access_token.clone();
                        let result =
                            bjsc::api::get_coaching(&api::GlooClient, &config, &token).await;
                        let result = if result.is_err() {
                            if let Some(new_auth) =
                                bjsc::api::refresh_session(&api::GlooClient, &config, &auth).await
                            {
                                token = new_auth.access_token.clone();
                                auth_state.set(Some(new_auth));
                                bjsc::api::get_coaching(&api::GlooClient, &config, &token).await
                            } else {
                                result
                            }
                        } else {
                            result
                        };
                        match result {
                            Ok(text) => coaching_text.set(text),
                            Err(e) => coaching_text.set(format!("Error: {}", e)),
                        }
                    });
                }
            }
        }
        screen.set(next);
        menu_open.set(false);
    };

    // Global keyboard listener (with cleanup to prevent leaks on unmount)
    let closure =
        Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |e: web_sys::KeyboardEvent| {
            let tag = e
                .target()
                .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                .map(|el| el.tag_name());
            if matches!(tag.as_deref(), Some("INPUT" | "TEXTAREA" | "SELECT")) {
                return;
            }
            let key = e.key();

            // Escape toggles the slide-out menu from any screen
            if key == "Escape" {
                menu_open.set(!menu_open.get_untracked());
                return;
            }

            // Screen shortcuts when menu is open
            if menu_open.get_untracked() {
                match key.as_str() {
                    "p" => {
                        go_to_screen(Screen::Play);
                        return;
                    }
                    "s" => {
                        go_to_screen(Screen::Stats);
                        return;
                    }
                    "g" => {
                        go_to_screen(Screen::Progress);
                        return;
                    }
                    "c" => {
                        go_to_screen(Screen::Coach);
                        return;
                    }
                    "t" => {
                        go_to_screen(Screen::Strategy);
                        return;
                    }
                    _ => {}
                }
                return;
            }

            // Play screen actions
            if screen.get_untracked() != Screen::Play {
                return;
            }
            if show_shuffle.get_untracked() {
                if key == "Enter" || key == " " {
                    do_shuffle();
                }
                return;
            }
            if let Some(ch) = key.chars().next() {
                if let Some(action) = Action::from_key(ch) {
                    do_action(action);
                }
            }
        });
    // Clone the JS function reference for cleanup, then register the listener.
    // closure.forget() intentionally leaks the closure to keep the callback valid.
    // on_cleanup removes the listener from the DOM to prevent duplicate handlers.
    let js_ref = closure.as_ref().clone();
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        let _ = doc.add_event_listener_with_callback("keydown", js_ref.unchecked_ref());
        on_cleanup(move || {
            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                let _ = doc.remove_event_listener_with_callback("keydown", js_ref.unchecked_ref());
            }
        });
    }
    closure.forget();

    view! {
        // Loading state
        <div class="text-center py-8 text-gray-400" class:hidden=move || !loading.get()>
            "Loading..."
        </div>

        <div class:hidden=move || loading.get()>
            // Top bar
            <div class="flex items-center gap-4 mb-4 py-2 border-b border-gray-700">
                // Screen title
                <h1 class="font-bold text-cyan-400 text-base m-0">
                    {move || screen.get().title()}
                </h1>
                // Mode selector (play screen only)
                {
                    let mode_dropdown_open = RwSignal::new(false);
                    view! {
                        <div class="relative" class:hidden=move || screen.get() != Screen::Play>
                            <button
                                aria-label="Study mode"
                                class="text-sm px-2 py-1 border border-gray-600 rounded bg-slate-800 cursor-pointer hover:border-cyan-400 text-amber-300"
                                on:click=move |_| mode_dropdown_open.set(!mode_dropdown_open.get_untracked())
                            >
                                {move || {
                                    let key = game_display.get().mode_key;
                                    bjsc::StudyMode::from_key(&key).map(|m| m.icon()).unwrap_or("\u{1F0CF}")
                                }}
                                <span class="hidden sm:inline ml-2 text-gray-200">
                                    {move || {
                                        let key = game_display.get().mode_key;
                                        bjsc::StudyMode::from_key(&key).map(|m| m.to_string()).unwrap_or_default()
                                    }}
                                </span>
                            </button>
                            <div
                                class="absolute top-full left-0 mt-1 bg-slate-800 border border-gray-600 rounded shadow-lg z-30 w-48"
                                class:hidden=move || !mode_dropdown_open.get()
                            >
                                {bjsc::StudyMode::ALL.iter().map(|m| {
                                    let mode = *m;
                                    let icon = m.icon();
                                    let label = m.to_string();
                                    view! {
                                        <button
                                            class="w-full text-left px-3 py-1.5 text-sm hover:bg-slate-700 cursor-pointer"
                                            class:text-amber-300=move || game_display.get().mode_key == mode.key()
                                            class:text-gray-300=move || game_display.get().mode_key != mode.key()
                                            on:click=move |_| {
                                                set_mode(mode);
                                                mode_dropdown_open.set(false);
                                            }
                                        >
                                            {format!("{} {}", icon, label)}
                                        </button>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    }
                }
                // Username + hamburger menu (right side)
                <span class="ml-auto text-xs text-gray-500">{move || auth_state.get().map(|a| a.email).unwrap_or_default()}</span>
                <button
                    aria-label="Menu"
                    class="text-2xl leading-none w-10 h-10 pb-1 flex items-center justify-center border border-gray-600 rounded text-gray-400 cursor-pointer hover:text-cyan-400 hover:border-cyan-400"
                    on:click=move |_| menu_open.set(!menu_open.get_untracked())
                >
                    "\u{2630}"
                </button>
            </div>

            // Slide-out menu overlay
            <div
                role="presentation"
                class="fixed inset-0 bg-black/50 z-40"
                class:hidden=move || !menu_open.get()
                on:click=move |_| menu_open.set(false)
            />
            // Slide-out menu panel
            <div
                role="navigation"
                aria-label="Main menu"
                class="fixed top-0 right-0 h-full w-64 bg-slate-900 border-l border-gray-700 z-50 transform transition-transform duration-200 ease-in-out flex flex-col"
                class:translate-x-0=move || menu_open.get()
                class:translate-x-full=move || !menu_open.get()
            >
                <div class="px-4 py-4 border-b border-gray-700">
                    <span class="text-xs text-gray-500">{move || auth_state.get().map(|a| a.email).unwrap_or_default()}</span>
                </div>
                <nav class="flex flex-col px-2 py-2 gap-1">
                    <button
                        class="text-left px-3 py-2 rounded text-sm hover:bg-slate-800"
                        class:text-cyan-400=move || screen.get() == Screen::Play
                        class:font-bold=move || screen.get() == Screen::Play
                        class:text-gray-300=move || screen.get() != Screen::Play
                        on:click=move |_| go_to_screen(Screen::Play)
                    ><span class="text-amber-400 font-bold">"P"</span>"lay"</button>
                    <button
                        class="text-left px-3 py-2 rounded text-sm hover:bg-slate-800"
                        class:text-cyan-400=move || screen.get() == Screen::Stats
                        class:font-bold=move || screen.get() == Screen::Stats
                        class:text-gray-300=move || screen.get() != Screen::Stats
                        on:click=move |_| go_to_screen(Screen::Stats)
                    ><span class="text-amber-400 font-bold">"S"</span>"tats"</button>
                    <button
                        class="text-left px-3 py-2 rounded text-sm hover:bg-slate-800"
                        class:text-cyan-400=move || screen.get() == Screen::Progress
                        class:font-bold=move || screen.get() == Screen::Progress
                        class:text-gray-300=move || screen.get() != Screen::Progress
                        on:click=move |_| go_to_screen(Screen::Progress)
                    >"Pro"<span class="text-amber-400 font-bold">"g"</span>"ress"</button>
                    <button
                        class="text-left px-3 py-2 rounded text-sm hover:bg-slate-800"
                        class:text-cyan-400=move || screen.get() == Screen::Coach
                        class:font-bold=move || screen.get() == Screen::Coach
                        class:text-gray-300=move || screen.get() != Screen::Coach
                        on:click=move |_| go_to_screen(Screen::Coach)
                    ><span class="text-amber-400 font-bold">"C"</span>"oach"</button>
                    <button
                        class="text-left px-3 py-2 rounded text-sm hover:bg-slate-800"
                        class:text-cyan-400=move || screen.get() == Screen::Strategy
                        class:font-bold=move || screen.get() == Screen::Strategy
                        class:text-gray-300=move || screen.get() != Screen::Strategy
                        on:click=move |_| go_to_screen(Screen::Strategy)
                    >"S"<span class="text-amber-400 font-bold">"t"</span>"rategy"</button>
                </nav>
                <div class="border-t border-gray-700 mx-2" />
                <div class="px-2 py-2">
                    <button
                        class="text-left w-full px-3 py-2 rounded text-sm text-red-400 hover:bg-red-950"
                        on:click=move |_| { menu_open.set(false); sign_out(); }
                    >"Sign out"</button>
                </div>
            </div>

            <HistogramScreen screen=screen game_data=game_display />
            <ProgressScreen screen=screen progress_stats=progress_stats />
            <CoachScreen screen=screen coaching_text=coaching_text />
            <StrategyScreen screen=screen />
            <PlayScreen
                screen=screen game_data=game_display
                status_text=status_text status_is_error=status_is_error status_visible=status_visible
                show_shuffle=show_shuffle errors=errors
                do_action=do_action do_shuffle=do_shuffle
            />

            // Keyboard hint
            <div class="flex justify-between text-xs py-4">
                <span class="text-gray-400">"Keyboard: h / s / d / p"</span>
                <span class="text-gray-600">{env!("BUILD_TIME")}</span>
            </div>
        </div>
    }
}

#[component]
fn HistogramScreen(screen: RwSignal<Screen>, game_data: RwSignal<DisplayData>) -> impl IntoView {
    view! {
        <div class:hidden=move || screen.get() != Screen::Stats>
            <h2 class="font-bold text-cyan-400 text-lg mb-4">"Spaced Repetition Buckets"</h2>
            <div class="space-y-1.5">
                {move || {
                    let d = game_data.get();
                    let counts = d.box_counts;
                    let dues = d.box_due;
                    let max_val = counts.iter().copied().max().unwrap_or(1).max(1);
                    let labels = bjsc::BOX_LABELS;
                    let colors = ["#ff6b6b", "#e03131", "#ffd43b", "#fab005", "#66d9e8", "#22b8cf", "#4dabf7", "#51cf66", "#8ce99a"];
                    let due_colors = ["#ffcccc", "#ff8888", "#fff3b0", "#ffe066", "#ccf2f7", "#99e5f0", "#b3d9ff", "#aaeeb0", "#d4f7d6"];

                    counts.iter().enumerate().map(|(i, &count)| {
                        let due = dues[i];
                        let not_due = count - due;
                        let not_due_pct = if max_val > 0 { (not_due as f64 / max_val as f64) * 100.0 } else { 0.0 };
                        let due_pct = if max_val > 0 { (due as f64 / max_val as f64) * 100.0 } else { 0.0 };
                        let color = colors[i];
                        let due_color = due_colors[i];
                        let label = labels[i];
                        let count_text = if due > 0 {
                            format!("{} ({})", count, due)
                        } else {
                            format!("{}", count)
                        };
                        view! {
                            <div class="flex items-center gap-3">
                                <span class="w-22 text-right text-sm text-gray-400 shrink-0">{format!("B{} ({})", i, label)}</span>
                                <div class="flex-1 h-5 bg-slate-800 rounded overflow-hidden flex">
                                    <div class="h-full rounded-l transition-all duration-300"
                                         style:width=format!("{}%", not_due_pct)
                                         style:background-color=color>
                                    </div>
                                    <div class="h-full transition-all duration-300"
                                         style:width=format!("{}%", due_pct)
                                         style:background-color=due_color>
                                    </div>
                                </div>
                                <span class="w-16 text-right text-sm shrink-0">{count_text}</span>
                            </div>
                        }
                    }).collect::<Vec<_>>()
                }}
            </div>
            <div class="mt-4 text-sm text-gray-500">
                <span class="font-bold">"Unseen: "</span>
                <span>{move || game_data.get().unseen}</span>
            </div>
        </div>
    }
}

#[component]
fn ProgressScreen(
    screen: RwSignal<Screen>,
    progress_stats: RwSignal<bjsc::progress::ProgressStats>,
) -> impl IntoView {
    view! {
        <div class:hidden=move || screen.get() != Screen::Progress>
            <h2 class="font-bold text-cyan-400 text-lg mb-4">"Progress Dashboard"</h2>

            // Overall accuracy
            <div class="border border-gray-700 rounded-md px-4 py-3 mb-4">
                <div class="font-bold text-cyan-400 text-sm uppercase tracking-wider mb-2">"Accuracy"</div>
                <div class="mb-1">
                    <span class="font-bold text-gray-400">"Overall: "</span>
                    <span class="font-bold">{move || format!("{:.1}%", progress_stats.get().accuracy_pct)}</span>
                    <span class="text-gray-500">{move || format!("  ({}/{})", progress_stats.get().total_correct, progress_stats.get().total_answers)}</span>
                </div>
                <div class="flex gap-6">
                    <span><span class="font-bold text-gray-400">"Hard: "</span>{move || bjsc::progress::ProgressStats::category_pct(progress_stats.get().hard_correct, progress_stats.get().hard_total)}</span>
                    <span><span class="font-bold text-gray-400">"Soft: "</span>{move || bjsc::progress::ProgressStats::category_pct(progress_stats.get().soft_correct, progress_stats.get().soft_total)}</span>
                    <span><span class="font-bold text-gray-400">"Split: "</span>{move || bjsc::progress::ProgressStats::category_pct(progress_stats.get().split_correct, progress_stats.get().split_total)}</span>
                    <span><span class="font-bold text-gray-400">"Dbl: "</span>{move || bjsc::progress::ProgressStats::category_pct(progress_stats.get().double_correct, progress_stats.get().double_total)}</span>
                </div>
            </div>

            // Trouble spots
            <div class="border border-gray-700 rounded-md px-4 py-3 mb-4">
                <div class="font-bold text-cyan-400 text-sm uppercase tracking-wider mb-2">"Trouble Spots"</div>
                {move || {
                    let stats = progress_stats.get();
                    if stats.trouble_spots.is_empty() {
                        view! { <div class="text-gray-500 text-sm">"No mistakes recorded yet."</div> }.into_any()
                    } else {
                        view! {
                            <div>
                                {stats.trouble_spots.iter().map(|(idx, wrong, seen)| {
                                    let pct = *wrong as f64 / *seen as f64 * 100.0;
                                    view! {
                                        <div class="flex justify-between py-0.5 text-sm border-b border-gray-800">
                                            <span class="text-red-400">{idx.clone()}</span>
                                            <span>{format!("{}/{} wrong ({:.0}%)", wrong, seen, pct)}</span>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }
                }}
            </div>

            // Recent sessions
            <div class="border border-gray-700 rounded-md px-4 py-3 mb-4">
                <div class="font-bold text-cyan-400 text-sm uppercase tracking-wider mb-2">"Recent Sessions"</div>
                {move || {
                    let stats = progress_stats.get();
                    if stats.sessions.is_empty() {
                        view! { <div class="text-gray-500 text-sm">"No sessions recorded yet."</div> }.into_any()
                    } else {
                        view! {
                            <div>
                                {stats.sessions.iter().map(|(day, total, correct)| {
                                    let pct = if *total > 0 { *correct as f64 / *total as f64 * 100.0 } else { 0.0 };
                                    let color_class = if pct >= 80.0 { "text-green-400" } else if pct >= 60.0 { "text-yellow-400" } else { "text-red-400" };
                                    view! {
                                        <div class="flex justify-between py-0.5 text-sm border-b border-gray-800">
                                            <span class="text-cyan-400">{day.clone()}</span>
                                            <span>{format!("{} answered", total)}</span>
                                            <span class=color_class>{format!("{:.0}%", pct)}</span>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}

#[component]
fn CoachScreen(screen: RwSignal<Screen>, coaching_text: RwSignal<String>) -> impl IntoView {
    let coaching_html = Memo::new(move |_| {
        let md = coaching_text.get();
        let parser = pulldown_cmark::Parser::new(&md);
        let mut html = String::new();
        pulldown_cmark::html::push_html(&mut html, parser);
        html
    });

    view! {
        <div class:hidden=move || screen.get() != Screen::Coach>
            <h2 class="font-bold text-cyan-400 text-lg mb-4">"Coach (powered by Claude)"</h2>
            <div
                class="border border-gray-700 rounded-md px-4 py-4 text-sm leading-relaxed prose prose-invert prose-sm max-w-none"
                inner_html=move || coaching_html.get()
            />
        </div>
    }
}

#[component]
fn PlayScreen(
    screen: RwSignal<Screen>,
    game_data: RwSignal<DisplayData>,
    status_text: RwSignal<String>,
    status_is_error: RwSignal<bool>,
    status_visible: RwSignal<bool>,
    show_shuffle: RwSignal<bool>,
    errors: RwSignal<Vec<String>>,
    do_action: impl Fn(Action) + Copy + 'static,
    do_shuffle: impl Fn() + Copy + 'static,
) -> impl IntoView {
    view! {
        <div class:hidden=move || screen.get() != Screen::Play>
            // Stats panel
            <div class="border border-gray-700 rounded-md px-4 py-3 mb-6">
                <div class="font-bold text-cyan-400 text-sm uppercase tracking-wider mb-2">"Stats"</div>
                <div>
                    <span class="font-bold text-gray-400">"Hands: "</span>
                    <span>{move || game_data.get().score.clone()}</span>
                </div>
                <div class="flex gap-6 mt-1">
                    <span><span class="font-bold text-gray-400">"Hard: "</span>{move || game_data.get().hard.clone()}</span>
                    <span><span class="font-bold text-gray-400">"Soft: "</span>{move || game_data.get().soft.clone()}</span>
                    <span><span class="font-bold text-gray-400">"Split: "</span>{move || game_data.get().split.clone()}</span>
                    <span><span class="font-bold text-gray-400">"Dbl: "</span>{move || game_data.get().double.clone()}</span>
                </div>
                <div class="flex gap-6 mt-1">
                    <span><span class="font-bold text-gray-500">"New: "</span><span class="text-gray-500">{move || game_data.get().new_count}</span></span>
                    <span><span class="font-bold text-gray-400">"Weak: "</span><span class="text-red-400">{move || game_data.get().weak_count}</span></span>
                    <span><span class="font-bold text-gray-400">"Mastered: "</span><span class="text-green-400">{move || game_data.get().mastered_count}</span></span>
                    <span><span class="font-bold text-gray-400">"Due: "</span><span class="text-yellow-400">{move || game_data.get().due_count}</span></span>
                </div>
            </div>

            // Drill waiting message
            <div
                class="text-center py-8 mb-6 border border-yellow-800 rounded-md bg-yellow-950"
                class:hidden=move || game_data.get().drill_wait_secs.is_none()
            >
                <div class="text-yellow-400 text-lg font-bold mb-2">
                    "\u{23F3} All cards reviewed!"
                </div>
                <div class="text-yellow-300 text-sm">
                    {move || {
                        game_data.get().drill_wait_secs.map(|secs| {
                            format!("Next card due in {}", bjsc::format_wait_time(secs))
                        }).unwrap_or_default()
                    }}
                </div>
            </div>

            // Hands (hidden when drill waiting)
            <div class="mb-6" class:hidden=move || game_data.get().drill_wait_secs.is_some()>
                <div class="text-xl py-1">
                    <span class="font-bold text-cyan-400">"Dealer: "</span>
                    <span class="text-2xl tracking-wide">{move || game_data.get().dealer.clone()}</span>
                </div>
                <div class="text-xl py-1">
                    <span class="font-bold text-cyan-400">"Player: "</span>
                    <span class="text-2xl tracking-wide">{move || game_data.get().player.clone()}</span>
                </div>
            </div>

            // Status message (hidden when drill waiting)
            <div
                class="text-center px-4 py-2 rounded font-bold text-lg mb-6"
                class:hidden=move || !status_visible.get() || game_data.get().drill_wait_secs.is_some()
                class:bg-green-900=move || !status_is_error.get()
                class:text-green-300=move || !status_is_error.get()
                class:bg-red-900=move || status_is_error.get()
                class:text-red-200=move || status_is_error.get()
            >
                {move || status_text.get()}
            </div>

            // Action buttons (hidden when drill waiting)
            <div
                class="grid grid-cols-2 sm:grid-cols-4 gap-3 justify-center mb-6"
                class:hidden=move || game_data.get().drill_wait_secs.is_some()
            >
                <button
                    class="col-span-2 sm:col-span-4 px-5 py-2.5 border border-green-700 rounded-md bg-green-950 text-gray-200 text-base font-mono cursor-pointer transition-colors hover:bg-green-900 hover:border-green-500"
                    class:hidden=move || !show_shuffle.get()
                    on:click=move |_| do_shuffle()
                >
                    "Shuffle New Shoe"
                </button>
                <button
                    aria-label="Hit (H key)"
                    class="px-5 py-2.5 border border-gray-600 rounded-md bg-slate-800 text-gray-200 text-base font-mono cursor-pointer transition-colors hover:bg-slate-700 hover:border-cyan-400 active:bg-slate-600"
                    class:hidden=move || show_shuffle.get()
                    on:click=move |_| do_action(Action::Hit)
                >"(H)it"</button>
                <button
                    aria-label="Stand (S key)"
                    class="px-5 py-2.5 border border-gray-600 rounded-md bg-slate-800 text-gray-200 text-base font-mono cursor-pointer transition-colors hover:bg-slate-700 hover:border-cyan-400 active:bg-slate-600"
                    class:hidden=move || show_shuffle.get()
                    on:click=move |_| do_action(Action::Stand)
                >"(S)tand"</button>
                <button
                    aria-label="Double (D key)"
                    class="px-5 py-2.5 border border-gray-600 rounded-md bg-slate-800 text-gray-200 text-base font-mono cursor-pointer transition-colors hover:bg-slate-700 hover:border-cyan-400 active:bg-slate-600"
                    class:hidden=move || show_shuffle.get()
                    on:click=move |_| do_action(Action::Double)
                >"(D)ouble"</button>
                <button
                    aria-label="Split (P key)"
                    class="px-5 py-2.5 border border-gray-600 rounded-md bg-slate-800 text-gray-200 text-base font-mono cursor-pointer transition-colors hover:bg-slate-700 hover:border-cyan-400 active:bg-slate-600"
                    class:hidden=move || show_shuffle.get()
                    on:click=move |_| do_action(Action::Split)
                >"S(p)lit"</button>
            </div>

            // Error log
            <div class="border border-gray-700 rounded-md px-4 py-3 mb-6 max-h-72 overflow-y-auto">
                <div class="font-bold text-cyan-400 text-sm uppercase tracking-wider mb-2">"Mistakes"</div>
                {move || {
                    errors
                        .get()
                        .into_iter()
                        .map(|e| {
                            view! {
                                <div class="py-1 border-b border-gray-800 text-sm">
                                    <span class="font-bold text-red-400">"Error: "</span>
                                    {e}
                                </div>
                            }
                        })
                        .collect::<Vec<_>>()
                }}
            </div>
        </div>
    }
}

#[component]
fn StrategyScreen(screen: RwSignal<Screen>) -> impl IntoView {
    let tab = RwSignal::new(0u8); // 0 = Descriptive, 1 = Tables

    view! {
        <div class:hidden=move || screen.get() != Screen::Strategy>
            // Tab bar
            <div class="flex gap-4 mb-4 border-b border-gray-700 pb-2">
                <button
                    class="text-sm font-bold cursor-pointer"
                    class:text-cyan-400=move || tab.get() == 0
                    class:underline=move || tab.get() == 0
                    class:text-gray-500=move || tab.get() != 0
                    on:click=move |_| tab.set(0)
                >"Descriptive"</button>
                <button
                    class="text-sm font-bold cursor-pointer"
                    class:text-cyan-400=move || tab.get() == 1
                    class:underline=move || tab.get() == 1
                    class:text-gray-500=move || tab.get() != 1
                    on:click=move |_| tab.set(1)
                >"Tables"</button>
            </div>

            // Descriptive tab
            <div class:hidden=move || tab.get() != 0>
                {bjsc::all_phrases().into_iter().map(|(category, phrases)| {
                    view! {
                        <div class="mb-4">
                            <h3 class="font-bold text-cyan-400 mb-1">{category}</h3>
                            <ul class="space-y-0.5">
                                {phrases.into_iter().map(|p| {
                                    view! { <li class="text-sm text-gray-300 ml-4">{p}</li> }
                                }).collect::<Vec<_>>()}
                            </ul>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>

            // Tables tab
            <div class:hidden=move || tab.get() != 1>
                {bjsc::all_charts().into_iter().map(|chart| {
                    view! {
                        <div class="mb-6">
                            <h3 class="font-bold text-cyan-400 mb-2">{chart.title}</h3>
                            <div class="overflow-x-auto">
                                <table class="text-xs font-mono border-collapse">
                                    <thead>
                                        <tr>
                                            <th class="px-2 py-1 text-gray-500"></th>
                                            {chart.col_headers.iter().map(|h| {
                                                view! { <th class="px-2 py-1 text-yellow-400 font-bold">{*h}</th> }
                                            }).collect::<Vec<_>>()}
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {chart.rows.iter().map(|(label, cells)| {
                                            view! {
                                                <tr>
                                                    <td class="px-2 py-0.5 text-gray-400 font-bold">{*label}</td>
                                                    {cells.iter().map(|cell| {
                                                        let color = match *cell {
                                                            "H" => "text-red-400",
                                                            "S" => "text-green-400",
                                                            "Dh" | "Ds" => "text-yellow-300",
                                                            "P" | "Pd" => "text-blue-400",
                                                            _ => "text-gray-600",
                                                        };
                                                        view! { <td class=format!("px-2 py-0.5 text-center {}", color)>{*cell}</td> }
                                                    }).collect::<Vec<_>>()}
                                                </tr>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
                <div class="text-xs text-gray-500 mt-2">
                    <span class="font-bold">"Legend: "</span>
                    <span class="text-red-400">"H"</span>" = Hit, "
                    <span class="text-green-400">"S"</span>" = Stand, "
                    <span class="text-yellow-300">"Dh"</span>" = Double (hit), "
                    <span class="text-yellow-300">"Ds"</span>" = Double (stand), "
                    <span class="text-blue-400">"P"</span>" = Split, "
                    <span class="text-blue-400">"Pd"</span>" = Split (DAS)"
                </div>
            </div>
        </div>
    }
}
