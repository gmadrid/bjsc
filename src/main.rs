use bjsc::{Action, GameState, Hand, Message};
use cursive::align::{HAlign, VAlign};
use cursive::event::{Event, EventResult};
use cursive::style::BaseColor::Blue;
use cursive::style::PaletteColor::TitlePrimary;
use cursive::style::Style;
use cursive::theme::Color::Dark;
use cursive::theme::Theme;
use cursive::traits::{Nameable, Resizable};
use cursive::utils::span::SpannedString;
use cursive::views::{DummyView, LinearLayout, OnEventView, Panel, TextView};
use cursive::{Cursive, CursiveRunnable, View};
use std::sync::{Arc, RwLock};

type SharedGameState = Arc<RwLock<GameState>>;

fn create_theme() -> Theme {
    let mut theme = Theme::retro();
    theme.palette[TitlePrimary] = Dark(Blue);
    theme
}

fn make_dealer_hand(gs: &GameState) -> impl View {
    let hand = gs.dealer_hand();
    make_hand_view("Dealer", "dealer_hand", hand)
}

fn make_player_hand(gs: &GameState) -> impl View {
    let hand = gs.player_hand();
    make_hand_view("Player", "player_hand", hand)
}

fn make_hand_view(title: &str, view_name: &str, hand: &Hand) -> impl View {
    TextView::new(make_hand_string(hand, title)).with_name(view_name)
}

fn update_hands(siv: &mut Cursive) {
    if let Some((dealer_hand_string, player_hand_string)) =
        siv.with_user_data(|sgs: &mut SharedGameState| {
            let game_state = sgs.read().unwrap();
            let dealer_hand_string = make_hand_string(game_state.dealer_hand(), "Dealer");
            let player_hand_string = make_hand_string(game_state.player_hand(), "Player");
            (dealer_hand_string, player_hand_string)
        })
    {
        siv.call_on_name("dealer_hand", |view: &mut TextView| {
            view.set_content(dealer_hand_string)
        });
        siv.call_on_name("player_hand", |view: &mut TextView| {
            view.set_content(player_hand_string)
        });
    }
}

fn make_hand_string(hand: &Hand, title: &str) -> SpannedString<Style> {
    let mut ss = SpannedString::styled(format!("{}: ", title), Style::title_primary());
    for card in hand.cards() {
        ss.append_plain(format!("{} ", card));
    }
    ss
}

fn make_score(gs: &GameState) -> impl View {
    let ss = score_string(gs);

    TextView::new(ss).with_name("score")
}

fn update_score(siv: &mut Cursive) {
    let ss = {
        // unwrap: it's always there.
        let shared_game_state: &SharedGameState = siv.user_data().unwrap();
        // unwrap: poisoned means nothing will work.
        let game_state = shared_game_state.read().unwrap();
        score_string(&game_state)
    };

    siv.call_on_name("score", |view: &mut TextView| {
        view.set_content(ss);
    });
}

fn score_string(gs: &GameState) -> SpannedString<Style> {
    let mut ss = SpannedString::styled("Errors: ", Style::title_primary());
    ss.append_plain(gs.num_questions_wrong().to_string());
    ss.append_plain(" | ");
    ss.append_styled("Hands seen: ", Style::title_primary());
    ss.append_plain(gs.num_questions_asked().to_string());
    ss
}

fn check_event_input(e: &Event) -> bool {
    if let Event::Char(ch) = e {
        match ch {
            'h' | 's' | 'p' | 'd' => {
                // we have to store this away, since the event is inexplicably not passed to handler
                //                game_state.write().unwrap().set_last_input(*ch);
                true
            }
            _ => false,
        }
    } else {
        false
    }
}

fn process_input(event: &Event) -> Option<EventResult> {
    if let Event::Char(ch) = event {
        if let Some(action) = Action::from_key(*ch) {
            Some(EventResult::with_cb(move |siv| {
                siv.with_user_data(|gs: &mut SharedGameState| {
                    let mut game_state = gs.write().unwrap();
                    if let Ok(chart_action) = game_state.chart_action() {
                        if Some(action) == chart_action.apply_rules() {
                            game_state.answered_right();
                            game_state
                                .set_message(Message::correct(format!("Correct: {}", action)));
                        } else {
                            game_state.answered_wrong();
                            game_state.set_message(Message::wrong(format!(
                                "WRONG: {}",
                                chart_action
                                    .apply_rules()
                                    .map(|r| r.to_string())
                                    .unwrap_or_default()
                            )));
                        }
                        if !game_state.deal_a_hand() {
                            println!("COULDN'T DEAL A HAND");
                        }
                    } else {
                        println!("NO CHART ACTION");
                        println!("D: {:?}", game_state.dealer_hand());
                        println!("P: {:?}", game_state.player_hand());
                    }
                });
                update_status_message(siv);
                update_score(siv);
                update_hands(siv);
            }))
        } else {
            None
        }
    } else {
        None
    }

    // siv.with_user_data(|sgs: &mut SharedGameState| {
    //     // unwrap: poisoned means nothing will work.
    //     let mut game_state = sgs.write().unwrap();
    //     if let Ok(chart_action) = game_state.chart_action() {
    //         if let Some(last_input) = game_state.last_input() {
    //             if let Some(action) = Action::from_key(last_input) {
    //                 if Some(action) == chart_action.apply_rules() {
    //                     game_state.answered_right();
    //                     game_state.set_message(Message::correct(format!("Correct: {}", action)));
    //                 } else {
    //                     game_state.answered_wrong();
    //                     game_state
    //                         .set_message(Message::wrong(format!("WRONG: {}", action.to_string())));
    //                 }
    //             } else {
    //                 dbg!("3");
    //             }
    //         } else {
    //             dbg!("2");
    //         }
    //     } else {
    //         dbg!("1");
    //     }
    //
    //     game_state.deal_a_hand();
    // });

    // siv.cb_sink().send(Box::new(move |s| {
    //     update_hands(s);
    //     update_score(s);
    //     update_status_message(s);
    // }));
}

fn make_keymap() -> impl View {
    // TODO: need Insurance and Surrender
    LinearLayout::horizontal().child(
        TextView::new("(H)it | (S)tand | (D)ouble | S(P)lit")
            .h_align(HAlign::Center)
            .v_align(VAlign::Bottom)
            .full_screen(),
    )
}

fn create_ui(siv: &mut CursiveRunnable) {
    if let Some(panel) = siv.with_user_data(|sgs: &mut SharedGameState| {
        let gs = sgs.read().unwrap();
        OnEventView::new(Panel::new(
            LinearLayout::vertical()
                .child(make_score(&gs))
                .child(DummyView)
                .child(DummyView)
                .child(DummyView)
                .child(make_dealer_hand(&gs))
                .child(DummyView)
                .child(make_player_hand(&gs))
                .child(make_status_message())
                .child(make_keymap()),
        ))
        .on_event_inner(check_event_input, |_, e| process_input(e))
        .full_screen()
    }) {
        siv.add_fullscreen_layer(panel);
    }
}

fn make_status_message() -> impl View {
    TextView::new("")
        .h_align(HAlign::Center)
        .v_align(VAlign::Center)
        .with_name("status")
        .full_screen()
}

fn update_status_message(siv: &mut Cursive) {
    let message = siv
        .with_user_data(|gs: &mut SharedGameState| {
            let game_state = gs.write().unwrap();
            game_state.message().clone()
        })
        .unwrap_or_default();

    siv.call_on_name("status", |view: &mut TextView| view.set_content(message));
}

fn main() {
    let mut siv = cursive::default();
    siv.set_theme(create_theme());

    let mut game_state = GameState::new();
    game_state.deal_a_hand();

    let shared_game_state = Arc::new(RwLock::new(game_state));
    siv.set_user_data(shared_game_state);

    create_ui(&mut siv);

    siv.set_global_callback('q', |s| s.quit());

    siv.run();
}
