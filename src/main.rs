use crate::game_message::GameMessage;
use bjsc::{Action, GameState, Hand};
use cursive::align::{HAlign, VAlign};
use cursive::event::{Event, EventResult};
use cursive::style::BaseColor::{Blue, Red, White};
use cursive::style::PaletteColor::TitlePrimary;
use cursive::style::{ColorStyle, Style};
use cursive::theme::Color::Dark;
use cursive::theme::Theme;
use cursive::traits::{Nameable, Resizable};
use cursive::utils::markup::StyledString;
use cursive::utils::span::SpannedString;
use cursive::view::{Margins, Scrollable};
use cursive::views::{DummyView, LinearLayout, OnEventView, PaddedView, Panel, TextView};
use cursive::{Cursive, CursiveRunnable, View};
use std::sync::{Arc, RwLock};

mod game_message;

#[derive(Debug, Default)]
struct GameUserData {
    game_state: GameState,
    message: GameMessage,
}
type SharedUserData = Arc<RwLock<GameUserData>>;

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
        siv.with_user_data(|sgs: &mut SharedUserData| {
            let game_state = &sgs.read().unwrap().game_state;
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

fn numbers_string(num: usize, wrong: usize) -> String {
    let pct = (num - wrong) as f64 / num as f64 * 100.0;
    format!("{} / {} ({:.1}%)", num - wrong, num, pct)
}

fn make_score(gs: &GameState) -> impl View {
    let ss = numbers_string(gs.num_questions_asked(), gs.num_questions_wrong());

    let hands = labelled("Hands:", Some(ss), Some("score"));
    let hard = labelled("Hard:", None::<&str>, Some("hard"));
    let soft = labelled("Soft:", None::<&str>, Some("soft"));
    let split = labelled("Split:", None::<&str>, Some("split"));
    let double = labelled("Double:", None::<&str>, Some("double"));

    let top = hands.full_width();
    let left = LinearLayout::vertical()
        .child(hard)
        .child(soft)
        .full_width();
    let right = LinearLayout::vertical()
        .child(split)
        .child(double)
        .full_width();
    let bottom = LinearLayout::horizontal().child(left).child(right);

    let full = LinearLayout::vertical().child(top).child(bottom);

    Panel::new(PaddedView::new(Margins::lrtb(1, 1, 1, 1), full))
        .title("Stats")
        .title_position(HAlign::Left)
        .full_width()
}

fn labelled(
    label: impl Into<StyledString>,
    content: Option<impl Into<StyledString>>,
    name: Option<impl Into<String>>,
) -> impl View {
    let label = PaddedView::lrtb(0, 1, 0, 0, TextView::new(label));
    let content_inner = TextView::new(content.map(|c| c.into()).unwrap_or_default());

    let mut horiz = LinearLayout::horizontal().child(label);
    if let Some(name) = name {
        horiz.add_child(content_inner.with_name(name));
    } else {
        horiz.add_child(content_inner);
    }
    horiz
}

fn update_score(siv: &mut Cursive) {
    if let Some(score_string) = siv.with_user_data(|sgs: &mut SharedUserData| {
        let game_state = &sgs.read().unwrap().game_state;
        numbers_string(
            game_state.num_questions_asked(),
            game_state.num_questions_wrong(),
        )
    }) {
        siv.call_on_name("score", |view: &mut TextView| {
            view.set_content(score_string);
        });
    }
}

// fn score_string(gs: &GameState) -> SpannedString<Style> {
//     let mut ss = SpannedString::styled("Errors: ", Style::title_primary());
//     ss.append_plain(gs.num_questions_wrong().to_string());
//     ss.append_plain(" | ");
//     ss.append_styled("Hands seen: ", Style::title_primary());
//     ss.append_plain(gs.num_questions_asked().to_string());
//     ss
// }
//
fn check_event_input(e: &Event) -> bool {
    matches!(e, Event::Char('h' | 's' | 'p' | 'd'))
}

fn process_input(event: &Event) -> Option<EventResult> {
    if let Event::Char(ch) = event {
        Action::from_key(*ch).map(|action| {
            EventResult::with_cb(move |siv| {
                let mut log: Option<String> = None;
                process_input_inner(action, siv, &mut log);
                if let Some(log) = log {
                    add_log(siv, log);
                }
                update_status_message(siv);
                update_score(siv);
                update_hands(siv);
            })
        })
    } else {
        None
    }
}

fn process_input_inner(action: Action, siv: &mut Cursive, log: &mut Option<String>) {
    siv.with_user_data(|gs: &mut SharedUserData| {
        let mut user_data = gs.write().unwrap();
        if let Ok((chart_action, _)) = user_data.game_state.chart_action() {
            let action_from_rules = chart_action.apply_rules();
            if Some(action) == action_from_rules {
                user_data.game_state.answered_right();
                user_data.message = GameMessage::correct(format!("Correct: {}", action));
            } else {
                user_data.game_state.answered_wrong();
                user_data.message = GameMessage::wrong(format!(
                    "WRONG: {}",
                    action_from_rules.map(|r| r.to_string()).unwrap_or_default()
                ));

                if let Some(correct_action) = action_from_rules {
                    let _ = log.insert(format!(
                        "Player: {}, Dealer: {}, Correct: {}, Guess: {}",
                        user_data.game_state.player_hand(),
                        user_data.game_state.dealer_hand(),
                        correct_action,
                        action
                    ));
                }
            }
            if !user_data.game_state.deal_a_hand() {
                println!("COULDN'T DEAL A HAND");
            }
        } else {
            println!("NO CHART ACTION");
            println!("D: {:?}", user_data.game_state.dealer_hand());
            println!("P: {:?}", user_data.game_state.player_hand());
        }
    });
}

fn make_keymap() -> impl View {
    // TODO: need Insurance and Surrender
    LinearLayout::horizontal().child(
        TextView::new("(H)it | (S)tand | (D)ouble | S(P)lit")
            .h_align(HAlign::Center)
            .v_align(VAlign::Bottom)
            .full_width(), //            .full_screen(),
    )
}

fn make_log() -> impl View {
    Panel::new(
        LinearLayout::vertical()
            .with_name("log")
            .scrollable()
            .scroll_y(true)
            .full_screen(),
    )
    .title("Mistakes")
    .title_position(HAlign::Left)
}

fn add_log(siv: &mut Cursive, log: String) {
    let style = Style {
        effects: Default::default(),
        color: ColorStyle::new(White, Red.dark()),
    };
    let mut styled = SpannedString::styled("Error:", style);
    styled.append_plain(" ");
    styled.append_plain(log);
    siv.call_on_name("log", |view: &mut LinearLayout| {
        view.insert_child(0, TextView::new(styled));
    });
}

fn create_ui(siv: &mut CursiveRunnable) {
    if let Some(panel) = siv.with_user_data(|sgs: &mut SharedUserData| {
        let gs = &sgs.read().unwrap().game_state;
        OnEventView::new(Panel::new(
            LinearLayout::vertical()
                .child(make_score(gs))
                .child(DummyView)
                .child(DummyView)
                .child(DummyView)
                .child(make_dealer_hand(gs))
                .child(DummyView)
                .child(make_player_hand(gs))
                .child(DummyView)
                .child(make_status_message())
                .child(DummyView)
                .child(make_log())
                .child(DummyView)
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
        .h_align(HAlign::Left)
        .v_align(VAlign::Center)
        .with_name("status")
        .full_width()
}

fn update_status_message(siv: &mut Cursive) {
    let message = siv
        .with_user_data(|gs: &mut SharedUserData| gs.read().unwrap().message.clone())
        .unwrap_or_default();

    siv.call_on_name("status", |view: &mut TextView| view.set_content(message));
}

fn main() {
    let mut siv = cursive::default();
    siv.set_theme(create_theme());

    let mut user_data = GameUserData::default();
    user_data.game_state.deal_a_hand();

    siv.set_user_data(Arc::new(RwLock::new(user_data)));

    create_ui(&mut siv);

    siv.set_global_callback('q', |s| s.quit());

    siv.run();
}
