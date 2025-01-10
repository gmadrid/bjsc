use bjsc::{GameState, Hand};
use cursive::style::BaseColor::Blue;
use cursive::style::PaletteColor::TitlePrimary;
use cursive::style::Style;
use cursive::theme::Color::Dark;
use cursive::theme::Theme;
use cursive::traits::Resizable;
use cursive::utils::span::SpannedString;
use cursive::views::{DummyView, LinearLayout, Panel, TextView};
use cursive::{CursiveRunnable, View};

fn create_theme() -> Theme {
    let mut theme = Theme::retro();
    theme.palette[TitlePrimary] = Dark(Blue);
    theme
}

fn make_dealer_hand(gs: &GameState) -> impl View {
    let hand = gs.dealer_hand();
    make_hand_view("Dealer: ", hand)
}

fn make_player_hand(gs: &GameState) -> impl View {
    let hand = gs.player_hand();
    make_hand_view("Player: ", hand)
}

fn make_hand_view(title: &str, hand: &Hand) -> impl View {
    let mut ss = SpannedString::styled(title, Style::title_primary());
    for card in hand.cards() {
        ss.append_plain(format!("{} ", card));
    }

    TextView::new(ss)
}

fn make_score(gs: &GameState) -> impl View {
    // let ss = SpannedString::styled("Errors:", Style::title_primary())
    //     .plain("0")
    //     .plain("  ")
    //     .styled("Hands seen: ", Style::title_primary())
    //     .plain("32");

    let mut ss = SpannedString::styled("Errors: ", Style::title_primary());
    ss.append_plain("0");
    ss.append_plain("  ");
    ss.append_styled("Hands seen: ", Style::title_primary());
    ss.append_plain("1");

    TextView::new(ss)
}

fn create_ui(siv: &mut CursiveRunnable) {
    // unwrap: it's always there.
    let gs: &GameState = siv.user_data().unwrap();
    let panel = Panel::new(
        LinearLayout::vertical()
            .child(make_score(gs))
            .child(DummyView)
            .child(make_dealer_hand(gs))
            .child(DummyView)
            .child(make_player_hand(gs)),
    )
    .full_screen();

    siv.add_fullscreen_layer(panel);
}

fn main() {
    let mut siv = cursive::default();
    siv.set_theme(create_theme());

    let mut game_state = GameState::new();
    game_state.deal_a_hand();
    siv.set_user_data(game_state);

    create_ui(&mut siv);

    siv.set_global_callback('q', |s| s.quit());

    siv.run();
}
