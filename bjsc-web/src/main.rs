use bjsc::{phrase_for_row, Action, GameState};
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

fn numbers_string(num: usize, wrong: usize) -> String {
    if num == 0 {
        return "0 / 0".to_string();
    }
    let pct = (num - wrong) as f64 / num as f64 * 100.0;
    format!("{} / {} ({:.1}%)", num - wrong, num, pct)
}

fn sync_display(dealer: RwSignal<String>, player: RwSignal<String>, score: RwSignal<String>) {
    GAME.with_borrow(|gs| {
        dealer.set(gs.dealer_hand().to_string());
        player.set(gs.player_hand().to_string());
        score.set(numbers_string(
            gs.num_questions_asked(),
            gs.num_questions_wrong(),
        ));
    });
}

#[component]
fn App() -> impl IntoView {
    let dealer_text = RwSignal::new(String::new());
    let player_text = RwSignal::new(String::new());
    let score_text = RwSignal::new(String::new());
    let status_text = RwSignal::new(String::new());
    let status_is_error = RwSignal::new(false);
    let status_visible = RwSignal::new(false);
    let errors: RwSignal<Vec<String>> = RwSignal::new(vec![]);
    let show_shuffle = RwSignal::new(false);

    sync_display(dealer_text, player_text, score_text);

    let do_action = move |action: Action| {
        if show_shuffle.get_untracked() {
            return;
        }
        GAME.with_borrow_mut(|gs| {
            if let Ok((chart_action, table_index)) = gs.chart_action() {
                let action_from_rules = chart_action.apply_rules();
                if Some(action) == action_from_rules {
                    gs.answered_right();
                    status_text.set(format!("Correct: {}", action));
                    status_is_error.set(false);
                } else {
                    gs.answered_wrong();
                    status_text.set(format!(
                        "WRONG: {}",
                        action_from_rules.map(|r| r.to_string()).unwrap_or_default()
                    ));
                    status_is_error.set(true);
                    if let Some(correct_action) = action_from_rules {
                        let entry = if let Some(ti) = table_index {
                            format!(
                                "{} (P: {}, D: {})",
                                phrase_for_row(ti.row),
                                gs.player_hand(),
                                gs.dealer_hand()
                            )
                        } else {
                            format!(
                                "P: {}, D: {}, Correct: {}, Guess: {}",
                                gs.player_hand(),
                                gs.dealer_hand(),
                                correct_action,
                                action
                            )
                        };
                        errors.update(|e| e.insert(0, entry));
                    }
                }
                status_visible.set(true);
                if !gs.deal_a_hand() {
                    show_shuffle.set(true);
                }
            }
        });
        sync_display(dealer_text, player_text, score_text);
    };

    let do_shuffle = move || {
        GAME.with_borrow_mut(|gs| {
            gs.shuffle();
            gs.deal_a_hand();
        });
        show_shuffle.set(false);
        sync_display(dealer_text, player_text, score_text);
    };

    // Global keyboard listener
    let closure =
        Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |e: web_sys::KeyboardEvent| {
            // Don't capture keys when typing in form fields
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
            <div class="stats-panel">
                <div class="panel-title">"Stats"</div>
                <div class="stats-content">
                    <div class="stats-row">
                        <span class="label">"Hands: "</span>
                        <span>{move || score_text.get()}</span>
                    </div>
                    <div class="stats-row sub-stats">
                        <span class="label">"Hard: "</span>
                        <span class="label">"Soft: "</span>
                        <span class="label">"Split: "</span>
                        <span class="label">"Double: "</span>
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

            <div class="keyboard-hint">"Keyboard: h / s / d / p"</div>
        </div>
    }
}
