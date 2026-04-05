use bjsc::{Action, GameState, Stats};
use leptos::prelude::*;
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
}

/// Read all display data from game state in one borrow.
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
        }
    })
}

/// Push display data into signals (no game borrow held).
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

#[component]
fn App() -> impl IntoView {
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
    };
    sync_all();

    let do_action = move |action: Action| {
        if show_shuffle.get_untracked() {
            return;
        }

        // Do all game mutations in one borrow, extract results
        let outcome = GAME.with_borrow_mut(|gs| {
            let result = gs.check_answer(action);
            let shoe_done = if result.is_some() {
                !gs.deal_a_hand()
            } else {
                false
            };
            (result, shoe_done)
        });

        // Update signals with no borrow held
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

    view! {
        <div class="app">
            <div class="mode-bar">
                <span class="label">"Mode: "</span>
                <span class="mode-name">{move || mode_text.get()}</span>
                <button class="mode-btn" on:click=move |_| cycle_mode()>"(M) Next"</button>
            </div>

            <div class="stats-panel">
                <div class="panel-title">"Stats"</div>
                <div class="stats-content">
                    <div class="stats-row">
                        <span class="label">"Hands: "</span>
                        <span>{move || score_text.get()}</span>
                    </div>
                    <div class="stats-row sub-stats">
                        <span><span class="label">"Hard: "</span>{move || hard_stat.get()}</span>
                        <span><span class="label">"Soft: "</span>{move || soft_stat.get()}</span>
                        <span><span class="label">"Split: "</span>{move || split_stat.get()}</span>
                        <span><span class="label">"Dbl: "</span>{move || double_stat.get()}</span>
                    </div>
                </div>
            </div>

            <div class="hands-area">
                <div class="hand-row">
                    <span class="hand-label">"Dealer: "</span>
                    <span class="cards">{move || dealer_text.get()}</span>
                </div>
                <div class="hand-row">
                    <span class="hand-label">"Player: "</span>
                    <span class="cards">{move || player_text.get()}</span>
                </div>
            </div>

            <div
                class="status"
                class:error=move || status_is_error.get()
                class:correct=move || !status_is_error.get()
                class:hidden=move || !status_visible.get()
            >
                {move || status_text.get()}
            </div>

            <div class="actions">
                <button
                    class="action-btn shuffle-btn"
                    class:hidden=move || !show_shuffle.get()
                    on:click=move |_| do_shuffle()
                >
                    "Shuffle New Shoe"
                </button>
                <button
                    class="action-btn"
                    class:hidden=move || show_shuffle.get()
                    on:click=move |_| do_action(Action::Hit)
                >
                    "(H)it"
                </button>
                <button
                    class="action-btn"
                    class:hidden=move || show_shuffle.get()
                    on:click=move |_| do_action(Action::Stand)
                >
                    "(S)tand"
                </button>
                <button
                    class="action-btn"
                    class:hidden=move || show_shuffle.get()
                    on:click=move |_| do_action(Action::Double)
                >
                    "(D)ouble"
                </button>
                <button
                    class="action-btn"
                    class:hidden=move || show_shuffle.get()
                    on:click=move |_| do_action(Action::Split)
                >
                    "S(p)lit"
                </button>
            </div>

            <div class="error-log">
                <div class="panel-title">"Mistakes"</div>
                <div class="log-entries">
                    {move || {
                        errors
                            .get()
                            .into_iter()
                            .map(|e| {
                                view! {
                                    <div class="log-entry">
                                        <span class="error-marker">"Error: "</span>
                                        {e}
                                    </div>
                                }
                            })
                            .collect::<Vec<_>>()
                    }}
                </div>
            </div>

            <div class="keyboard-hint">"Keyboard: h / s / d / p / m (mode)"</div>
        </div>
    }
}
