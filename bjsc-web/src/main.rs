mod api;
mod auth;

use auth::AuthState;
use bjsc::{Action, GameState, Stats, SupabaseConfig};
use leptos::prelude::*;
use spaced_rep::NUM_BOXES;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

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

struct DisplayData {
    dealer: String,
    player: String,
    score: String,
    hard: String,
    soft: String,
    split: String,
    double: String,
    box_counts: [u32; NUM_BOXES as usize],
    unseen: u32,
    new_count: u32,
    weak_count: u32,
    mastered_count: u32,
    due_count: u32,
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
            box_counts: gs.box_counts(),
            unseen: gs.unseen_count(),
            new_count: ds.unasked,
            weak_count: ds.weak,
            mastered_count: ds.mastered,
            due_count: ds.due,
        }
    })
}

#[allow(clippy::too_many_arguments)]
fn push_display(
    data: &DisplayData,
    dealer: RwSignal<String>,
    player: RwSignal<String>,
    score: RwSignal<String>,
    hard_stat: RwSignal<String>,
    soft_stat: RwSignal<String>,
    split_stat: RwSignal<String>,
    double_stat: RwSignal<String>,
) {
    dealer.set(data.dealer.clone());
    player.set(data.player.clone());
    score.set(data.score.clone());
    hard_stat.set(data.hard.clone());
    soft_stat.set(data.soft.clone());
    split_stat.set(data.split.clone());
    double_stat.set(data.double.clone());
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
        let _ = api::insert_answer_log(&config, &token, &row).await;
    });
}

/// Save the current game state to Supabase (fire-and-forget).
fn save_to_cloud(auth: &AuthState) {
    let config = supabase_config();
    let token = auth.access_token.clone();
    let user_id = auth.user_id.clone();
    let (mode, deck) = GAME.with_borrow(|gs| (gs.study_mode(), gs.deck().clone()));

    leptos::task::spawn_local(async move {
        let _ = api::upsert_user_deck(&config, &token, &user_id, mode, &deck).await;
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
    let dealer_text = RwSignal::new(String::new());
    let player_text = RwSignal::new(String::new());
    let score_text = RwSignal::new(String::new());
    let hard_stat = RwSignal::new(String::new());
    let soft_stat = RwSignal::new(String::new());
    let split_stat = RwSignal::new(String::new());
    let double_stat = RwSignal::new(String::new());
    let status_text = RwSignal::new(String::new());
    let status_is_error = RwSignal::new(false);
    let status_visible = RwSignal::new(false);
    let errors: RwSignal<Vec<String>> = RwSignal::new(vec![]);
    let show_shuffle = RwSignal::new(false);
    // Screen: 0=play, 1=histogram, 2=progress, 3=coach
    let screen = RwSignal::new(0u8);
    let coaching_text = RwSignal::new(String::new());
    let box_counts: RwSignal<[u32; NUM_BOXES as usize]> = RwSignal::new([0; NUM_BOXES as usize]);
    let unseen_count = RwSignal::new(0u32);
    let new_count = RwSignal::new(0u32);
    let weak_count = RwSignal::new(0u32);
    let mastered_count = RwSignal::new(0u32);
    let due_count = RwSignal::new(0u32);
    let progress_stats: RwSignal<bjsc::progress::ProgressStats> =
        RwSignal::new(bjsc::progress::ProgressStats::default());
    let loading = RwSignal::new(true);

    let sync_all = move || {
        let data = read_display();
        push_display(
            &data,
            dealer_text,
            player_text,
            score_text,
            hard_stat,
            soft_stat,
            split_stat,
            double_stat,
        );
        box_counts.set(data.box_counts);
        unseen_count.set(data.unseen);
        new_count.set(data.new_count);
        weak_count.set(data.weak_count);
        mastered_count.set(data.mastered_count);
        due_count.set(data.due_count);
    };

    // Load deck from Supabase on mount, refreshing token if needed
    {
        let auth = auth_state.get_untracked().unwrap();
        let config = supabase_config();
        leptos::task::spawn_local(async move {
            let mut token = auth.access_token.clone();

            // Try to fetch; if it fails, attempt token refresh
            let result = api::fetch_user_deck(&config, &token).await;
            let result = if result.is_err() {
                if let Some(new_auth) = auth::refresh_session(&config, &auth).await {
                    token = new_auth.access_token.clone();
                    auth_state.set(Some(new_auth));
                    api::fetch_user_deck(&config, &token).await
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

            if let Ok(Some(row)) = result {
                GAME.with_borrow_mut(|gs| {
                    gs.set_deck(row.deck);
                    gs.set_study_mode(row.study_mode);
                    gs.deal_a_hand();
                });
            }
            loading.set(false);
            sync_all();
        });
    }

    let do_action = move |action: Action| {
        if show_shuffle.get_untracked() || loading.get_untracked() {
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

    let go_to_screen = move |next: u8| {
        if next == 2 {
            if let Some(auth) = auth_state.get_untracked() {
                let config = supabase_config();
                let token = auth.access_token.clone();
                leptos::task::spawn_local(async move {
                    if let Ok(logs) = api::fetch_answer_logs(&config, &token, 1000).await {
                        progress_stats.set(bjsc::progress::ProgressStats::from_logs(&logs));
                    }
                });
            }
        }
        if next == 3 {
            coaching_text.set("Loading coaching advice...".to_string());
            if let Some(auth) = auth_state.get_untracked() {
                let config = supabase_config();
                let token = auth.access_token.clone();
                leptos::task::spawn_local(async move {
                    match api::get_coaching(&config, &token).await {
                        Ok(text) => coaching_text.set(text),
                        Err(e) => coaching_text.set(format!("Error: {}", e)),
                    }
                });
            }
        }
        screen.set(next);
        menu_open.set(false);
    };

    // Global keyboard listener
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
            if screen.get_untracked() != 0 {
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
    let _ = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());
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
                <span class="font-bold text-cyan-400">
                    {move || match screen.get() { 0 => "Play", 1 => "Stats", 2 => "Progress", 3 => "Coach", _ => "" }}
                </span>
                // Mode selector (play screen only)
                <select
                    id="mode-select"
                    class="text-sm px-2 py-1 border border-gray-600 rounded bg-slate-800 text-amber-300 font-bold cursor-pointer hover:border-cyan-400 focus:border-cyan-400 focus:outline-none"
                    class:hidden=move || screen.get() != 0
                    on:change=move |ev| {
                        let val = leptos::prelude::event_target_value(&ev);
                        if let Some(mode) = bjsc::StudyMode::from_key(&val) {
                            set_mode(mode);
                        }
                    }
                    prop:value=move || {
                        GAME.with_borrow(|gs| gs.study_mode().key().to_string())
                    }
                >
                    {bjsc::StudyMode::ALL.iter().map(|m| {
                        let key = m.key();
                        let label = m.to_string();
                        view! { <option value={key}>{label}</option> }
                    }).collect::<Vec<_>>()}
                </select>
                // Hamburger menu button (right side)
                <button
                    class="ml-auto text-sm px-3 py-1 border border-gray-600 rounded bg-slate-800 text-gray-400 cursor-pointer hover:bg-slate-700 hover:border-cyan-400"
                    on:click=move |_| menu_open.set(!menu_open.get_untracked())
                >
                    {move || auth_state.get().map(|a| a.email).unwrap_or_else(|| "\u{2630}".to_string())}
                </button>
            </div>

            // Slide-out menu overlay
            <div
                class="fixed inset-0 bg-black/50 z-40"
                class:hidden=move || !menu_open.get()
                on:click=move |_| menu_open.set(false)
            />
            // Slide-out menu panel
            <div
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
                        class:text-cyan-400=move || screen.get() == 0
                        class:font-bold=move || screen.get() == 0
                        class:text-gray-300=move || screen.get() != 0
                        on:click=move |_| go_to_screen(0)
                    >"Play"</button>
                    <button
                        class="text-left px-3 py-2 rounded text-sm hover:bg-slate-800"
                        class:text-cyan-400=move || screen.get() == 1
                        class:font-bold=move || screen.get() == 1
                        class:text-gray-300=move || screen.get() != 1
                        on:click=move |_| go_to_screen(1)
                    >"Stats"</button>
                    <button
                        class="text-left px-3 py-2 rounded text-sm hover:bg-slate-800"
                        class:text-cyan-400=move || screen.get() == 2
                        class:font-bold=move || screen.get() == 2
                        class:text-gray-300=move || screen.get() != 2
                        on:click=move |_| go_to_screen(2)
                    >"Progress"</button>
                    <button
                        class="text-left px-3 py-2 rounded text-sm hover:bg-slate-800"
                        class:text-cyan-400=move || screen.get() == 3
                        class:font-bold=move || screen.get() == 3
                        class:text-gray-300=move || screen.get() != 3
                        on:click=move |_| go_to_screen(3)
                    >"Coach"</button>
                </nav>
                <div class="border-t border-gray-700 mx-2" />
                <div class="px-2 py-2">
                    <button
                        class="text-left w-full px-3 py-2 rounded text-sm text-red-400 hover:bg-red-950"
                        on:click=move |_| { menu_open.set(false); sign_out(); }
                    >"Sign out"</button>
                </div>
            </div>

            // Histogram screen
            <div class:hidden=move || screen.get() != 1>
                <h2 class="font-bold text-cyan-400 text-lg mb-4">"Spaced Repetition Buckets"</h2>
                <div class="space-y-1.5">
                    {move || {
                        let counts = box_counts.get();
                        let max_val = counts.iter().copied().max().unwrap_or(1).max(1);
                        let labels = bjsc::BOX_LABELS;
                        let colors = ["#ff6b6b", "#e03131", "#ffd43b", "#fab005", "#66d9e8", "#22b8cf", "#4dabf7", "#51cf66", "#8ce99a"];

                        counts.iter().enumerate().map(|(i, &count)| {
                            let pct = if max_val > 0 { (count as f64 / max_val as f64) * 100.0 } else { 0.0 };
                            let color = colors[i];
                            let label = labels[i];
                            view! {
                                <div class="flex items-center gap-3">
                                    <span class="w-22 text-right text-sm text-gray-400 shrink-0">{format!("B{} ({})", i, label)}</span>
                                    <div class="flex-1 h-5 bg-slate-800 rounded overflow-hidden">
                                        <div class="histogram-bar-fill"
                                             style:width=format!("{}%", pct)
                                             style:background-color=color>
                                        </div>
                                    </div>
                                    <span class="w-10 text-right text-sm shrink-0">{count}</span>
                                </div>
                            }
                        }).collect::<Vec<_>>()
                    }}
                </div>
                <div class="mt-4 text-sm text-gray-500">
                    <span class="font-bold">"Unseen: "</span>
                    <span>{move || unseen_count.get()}</span>
                </div>
            </div>

            // Progress screen
            <div class:hidden=move || screen.get() != 2>
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

            // Coach screen
            <div class:hidden=move || screen.get() != 3>
                <h2 class="font-bold text-cyan-400 text-lg mb-4">"Coach (powered by Claude)"</h2>
                <div
                    class="border border-gray-700 rounded-md px-4 py-4 text-sm leading-relaxed prose prose-invert prose-sm max-w-none"
                    inner_html=move || {
                        let md = coaching_text.get();
                        let parser = pulldown_cmark::Parser::new(&md);
                        let mut html = String::new();
                        pulldown_cmark::html::push_html(&mut html, parser);
                        html
                    }
                />
            </div>

            // Play screen
            <div class:hidden=move || screen.get() != 0>
                // Stats panel
                <div class="border border-gray-700 rounded-md px-4 py-3 mb-6">
                    <div class="font-bold text-cyan-400 text-sm uppercase tracking-wider mb-2">"Stats"</div>
                    <div>
                        <span class="font-bold text-gray-400">"Hands: "</span>
                        <span>{move || score_text.get()}</span>
                    </div>
                    <div class="flex gap-6 mt-1">
                        <span><span class="font-bold text-gray-400">"Hard: "</span>{move || hard_stat.get()}</span>
                        <span><span class="font-bold text-gray-400">"Soft: "</span>{move || soft_stat.get()}</span>
                        <span><span class="font-bold text-gray-400">"Split: "</span>{move || split_stat.get()}</span>
                        <span><span class="font-bold text-gray-400">"Dbl: "</span>{move || double_stat.get()}</span>
                    </div>
                    <div class="flex gap-6 mt-1">
                        <span><span class="font-bold text-gray-500">"New: "</span><span class="text-gray-500">{move || new_count.get()}</span></span>
                        <span><span class="font-bold text-gray-400">"Weak: "</span><span class="text-red-400">{move || weak_count.get()}</span></span>
                        <span><span class="font-bold text-gray-400">"Mastered: "</span><span class="text-green-400">{move || mastered_count.get()}</span></span>
                        <span><span class="font-bold text-gray-400">"Due: "</span><span class="text-yellow-400">{move || due_count.get()}</span></span>
                    </div>
                </div>

                // Hands
                <div class="mb-6">
                    <div class="text-xl py-1">
                        <span class="font-bold text-cyan-400">"Dealer: "</span>
                        <span class="text-2xl tracking-wide">{move || dealer_text.get()}</span>
                    </div>
                    <div class="text-xl py-1">
                        <span class="font-bold text-cyan-400">"Player: "</span>
                        <span class="text-2xl tracking-wide">{move || player_text.get()}</span>
                    </div>
                </div>

                // Status message
                <div
                    class="text-center px-4 py-2 rounded font-bold text-lg mb-6"
                    class:hidden=move || !status_visible.get()
                    class:bg-green-900=move || !status_is_error.get()
                    class:text-green-300=move || !status_is_error.get()
                    class:bg-red-900=move || status_is_error.get()
                    class:text-red-200=move || status_is_error.get()
                >
                    {move || status_text.get()}
                </div>

                // Action buttons
                <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 justify-center mb-6">
                    <button
                        class="col-span-2 sm:col-span-4 px-5 py-2.5 border border-green-700 rounded-md bg-green-950 text-gray-200 text-base font-mono cursor-pointer transition-colors hover:bg-green-900 hover:border-green-500"
                        class:hidden=move || !show_shuffle.get()
                        on:click=move |_| do_shuffle()
                    >
                        "Shuffle New Shoe"
                    </button>
                    <button
                        class="px-5 py-2.5 border border-gray-600 rounded-md bg-slate-800 text-gray-200 text-base font-mono cursor-pointer transition-colors hover:bg-slate-700 hover:border-cyan-400 active:bg-slate-600"
                        class:hidden=move || show_shuffle.get()
                        on:click=move |_| do_action(Action::Hit)
                    >"(H)it"</button>
                    <button
                        class="px-5 py-2.5 border border-gray-600 rounded-md bg-slate-800 text-gray-200 text-base font-mono cursor-pointer transition-colors hover:bg-slate-700 hover:border-cyan-400 active:bg-slate-600"
                        class:hidden=move || show_shuffle.get()
                        on:click=move |_| do_action(Action::Stand)
                    >"(S)tand"</button>
                    <button
                        class="px-5 py-2.5 border border-gray-600 rounded-md bg-slate-800 text-gray-200 text-base font-mono cursor-pointer transition-colors hover:bg-slate-700 hover:border-cyan-400 active:bg-slate-600"
                        class:hidden=move || show_shuffle.get()
                        on:click=move |_| do_action(Action::Double)
                    >"(D)ouble"</button>
                    <button
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

            // Keyboard hint
            <div class="flex justify-between text-xs py-4">
                <span class="text-gray-400">"Keyboard: h / s / d / p"</span>
                <span class="text-gray-600">{env!("BUILD_TIME")}</span>
            </div>
        </div>
    }
}
