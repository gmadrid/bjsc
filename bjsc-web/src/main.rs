mod api;
mod auth;

use auth::AuthState;
use bjsc::supabase::SupabaseConfig;
use bjsc::{Action, GameState, Stats};
use leptos::prelude::*;
use spaced_rep::NUM_BOXES;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

const SUPABASE_URL: &str = "https://pecwxusghnxlvzmfcqrj.supabase.co";
const SUPABASE_ANON_KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6InBlY3d4dXNnaG54bHZ6bWZjcXJqIiwicm9sZSI6ImFub24iLCJpYXQiOjE3NzUzNTY3MjUsImV4cCI6MjA5MDkzMjcyNX0.LwgaAHruQ8cA3mHrtCCB00WSqttpwRusAf0Y1WEFWuE";

thread_local! {
    static GAME: RefCell<GameState> = RefCell::new({
        let mut gs = GameState::default();
        gs.deal_a_hand();
        gs
    });
}

fn supabase_config() -> SupabaseConfig {
    SupabaseConfig {
        base_url: SUPABASE_URL.to_string(),
        anon_key: SUPABASE_ANON_KEY.to_string(),
    }
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
    mode: String,
    box_counts: [u32; NUM_BOXES as usize],
    unseen: u32,
}

fn read_display() -> DisplayData {
    GAME.with_borrow(|gs| {
        let s = gs.stats();
        DisplayData {
            dealer: gs.dealer_hand().to_string(),
            player: gs.player_hand().to_string(),
            score: Stats::numbers_string(s.question_count, s.questions_wrong),
            hard: Stats::numbers_string(s.hard_count, s.hard_wrong),
            soft: Stats::numbers_string(s.soft_count, s.soft_wrong),
            split: Stats::numbers_string(s.split_count, s.splits_wrong),
            double: Stats::numbers_string(s.double_count, s.doubles_wrong),
            mode: gs.study_mode().to_string(),
            box_counts: gs.box_counts(),
            unseen: gs.unseen_count(),
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
    mode_text: RwSignal<String>,
) {
    dealer.set(data.dealer.clone());
    player.set(data.player.clone());
    score.set(data.score.clone());
    hard_stat.set(data.hard.clone());
    soft_stat.set(data.soft.clone());
    split_stat.set(data.split.clone());
    double_stat.set(data.double.clone());
    mode_text.set(data.mode.clone());
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
    let login_url = auth::google_login_url(SUPABASE_URL);

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
    let mode_text = RwSignal::new(String::new());
    let status_text = RwSignal::new(String::new());
    let status_is_error = RwSignal::new(false);
    let status_visible = RwSignal::new(false);
    let errors: RwSignal<Vec<String>> = RwSignal::new(vec![]);
    let show_shuffle = RwSignal::new(false);
    let show_histogram = RwSignal::new(false);
    let box_counts: RwSignal<[u32; NUM_BOXES as usize]> = RwSignal::new([0; NUM_BOXES as usize]);
    let unseen_count = RwSignal::new(0u32);
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
            mode_text,
        );
        box_counts.set(data.box_counts);
        unseen_count.set(data.unseen);
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
            if result.correct {
                status_text.set(format!("Correct: {}", result.player_action));
                status_is_error.set(false);
            } else {
                status_text.set(format!(
                    "WRONG: {}",
                    result
                        .correct_action
                        .map(|a| a.to_string())
                        .unwrap_or_default()
                ));
                status_is_error.set(true);
                if let Some(entry) = result.log_entry {
                    errors.update(|e| e.insert(0, entry));
                }
            }
            status_visible.set(true);
            if shoe_done {
                show_shuffle.set(true);
            }
        }
        sync_all();

        // Save to cloud
        if let Some(auth) = auth_state.get_untracked() {
            save_to_cloud(&auth);
        }
    };

    let do_shuffle = move || {
        GAME.with_borrow_mut(|gs| {
            gs.shuffle();
            gs.deal_a_hand();
        });
        show_shuffle.set(false);
        sync_all();
    };

    let cycle_mode = move || {
        GAME.with_borrow_mut(|gs| {
            let new_mode = gs.study_mode().next();
            gs.set_study_mode(new_mode);
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

    let sign_out = move |_| {
        auth::clear_storage();
        auth_state.set(None);
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
            if key == "Tab" {
                e.prevent_default();
                show_histogram.update(|v| *v = !*v);
                return;
            }
            if show_histogram.get_untracked() {
                return;
            }
            if show_shuffle.get_untracked() {
                if key == "Enter" || key == " " {
                    do_shuffle();
                }
                return;
            }
            if key == "m" {
                cycle_mode();
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

    let toggle_histogram = move |_| show_histogram.update(|v| *v = !*v);

    view! {
        // Loading state
        <div class="text-center py-8 text-gray-400" class:hidden=move || !loading.get()>
            "Loading..."
        </div>

        <div class:hidden=move || loading.get()>
            // Mode bar
            <div class="flex items-center gap-4 mb-4 py-2 border-b border-gray-700">
                <span class="font-bold text-gray-400">"Mode: "</span>
                <span class="font-bold text-amber-300">{move || mode_text.get()}</span>
                <button
                    class="text-sm px-3 py-1 border border-gray-600 rounded bg-slate-800 text-gray-400 cursor-pointer hover:bg-slate-700 hover:border-cyan-400"
                    class:hidden=move || show_histogram.get()
                    on:click=move |_| cycle_mode()
                >"(M) Next"</button>
                <button
                    class="ml-auto text-sm px-3 py-1 border border-gray-600 rounded bg-slate-800 text-gray-400 cursor-pointer hover:bg-slate-700 hover:border-cyan-400"
                    on:click=toggle_histogram
                >
                    {move || if show_histogram.get() { "Back (Tab)" } else { "Stats (Tab)" }}
                </button>
                <button
                    class="text-sm px-3 py-1 border border-red-900 rounded bg-slate-800 text-red-400 cursor-pointer hover:bg-red-950 hover:border-red-700"
                    on:click=sign_out
                >"Sign out"</button>
            </div>

            // Histogram screen
            <div class:hidden=move || !show_histogram.get()>
                <h2 class="font-bold text-cyan-400 text-lg mb-4">"Spaced Repetition Buckets"</h2>
                <div class="space-y-1.5">
                    {move || {
                        let counts = box_counts.get();
                        let max_val = counts.iter().copied().max().unwrap_or(1).max(1);
                        let labels = ["20s", "1m", "5m", "30m", "2h", "6h", "1d", "3d", "1w"];
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

            // Play screen
            <div class:hidden=move || show_histogram.get()>
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
                <div class="flex flex-wrap gap-3 justify-center mb-6">
                    <button
                        class="px-5 py-2.5 border border-green-700 rounded-md bg-green-950 text-gray-200 text-base font-mono cursor-pointer transition-colors hover:bg-green-900 hover:border-green-500"
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
            <div class="text-center text-gray-600 text-xs py-4">"Keyboard: h / s / d / p / m (mode) / Tab (stats)"</div>
        </div>
    }
}
