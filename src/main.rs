// use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
// use crossterm::execute;
// use crossterm::terminal::{
//     disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
// };
// use std::thread;
// use std::time::Duration;
// use tui::backend::CrosstermBackend;
// use tui::widgets::{Block, Borders};
// use tui::Terminal;

use cursive::direction::Orientation;
use cursive::style::BaseColor::Blue;
use cursive::style::PaletteColor::TitlePrimary;
use cursive::theme::Color::Dark;
use cursive::theme::{ColorStyle, Theme};
use cursive::traits::*;
use cursive::views::{
    Dialog, DummyView, EditView, LinearLayout, Panel, RadioGroup, SelectView, TextView,
};
use cursive::{Cursive, CursiveRunnable};

fn game_config() -> impl View {
    let mut game_radios = RadioGroup::new();

    LinearLayout::vertical()
        .child(TextView::new("Game Config").style(ColorStyle::title_primary()))
        .child(game_radios.button(false, "Full game").disabled())
        .child(game_radios.button(true, "3-cards only").selected())
}

fn questions_config() -> impl View {
    let mut questions_radios = RadioGroup::new();

    LinearLayout::vertical()
        .child(TextView::new("Question selection").style(ColorStyle::title_primary()))
        .child(questions_radios.button(true, "Random").selected())
        .child(questions_radios.button(false, "Targeted").disabled())
}

fn show_config_screen(siv: &mut CursiveRunnable) {
    let screen_size = siv.screen_size();
    println!("SS: {:?}", screen_size);

    siv.add_fullscreen_layer(
        Panel::new(
            LinearLayout::new(Orientation::Vertical)
                .child(game_config())
                .child(DummyView::new())
                .child(questions_config()),
        )
        .full_screen(),
    )
}

fn create_theme() -> Theme {
    let mut theme = Theme::retro();
    theme.palette[TitlePrimary] = Dark(Blue);
    theme
}

fn main() {
    let mut siv = cursive::default();
    siv.set_theme(create_theme());

    show_config_screen(&mut siv);

    // let select = SelectView::<String>::new()
    //     .on_submit(on_submit)
    //     .with_name("select")
    //     .fixed_size((10, 5));
    // let buttons = LinearLayout::vertical()
    //     .child(Button::new("Add new", add_name))
    //     .child(Button::new("Delete", delete_name))
    //     .child(DummyView)
    //     .child(Button::new("Quit", Cursive::quit));
    //
    // siv.add_layer(
    //     Dialog::around(
    //         LinearLayout::horizontal()
    //             .child(select)
    //             .child(DummyView)
    //             .child(buttons),
    //     )
    //     .title("Select a profile"),
    // );

    siv.run();

    // enable_raw_mode().unwrap();
    // let mut stdout = std::io::stdout();
    // execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
    // let backend = CrosstermBackend::new(stdout);
    // let mut terminal = Terminal::new(backend).unwrap();
    //
    // terminal
    //     .draw(|f| {
    //         let size = f.size();
    //         let block = Block::default().title("George block").borders(Borders::ALL);
    //         f.render_widget(block, size);
    //     })
    //     .unwrap();
    //
    // thread::sleep(Duration::from_millis(5000));
    //
    // disable_raw_mode().unwrap();
    // execute!(
    //     terminal.backend_mut(),
    //     LeaveAlternateScreen,
    //     DisableMouseCapture
    // )
    // .unwrap();
    // terminal.show_cursor().unwrap();
}

fn on_submit(s: &mut Cursive, name: &str) {
    s.pop_layer();
    s.add_layer(
        Dialog::text(format!("Name: {}\nAwesome: yes", name))
            .title(format!("{}'s info", name))
            .button("Quit", Cursive::quit),
    );
}

fn add_name(s: &mut Cursive) {
    fn ok(s: &mut Cursive, name: &str) {
        s.call_on_name("select", |view: &mut SelectView<String>| {
            view.add_item_str(name)
        });
        s.pop_layer();
    }

    s.add_layer(
        Dialog::around(
            EditView::new()
                .on_submit(ok)
                .with_name("name")
                .fixed_width(10),
        )
        .title("Enter a new name")
        .button("Ok", |s| {
            let name = s
                .call_on_name("name", |view: &mut EditView| view.get_content())
                .unwrap();
            ok(s, &name);
        })
        .button("Cancel", |s| {
            s.pop_layer();
        }),
    );
}

fn delete_name(siv: &mut Cursive) {}

fn show_next(s: &mut Cursive) {
    s.pop_layer();
    s.add_layer(
        Dialog::text("Did you do the thing?")
            .title("Question 1")
            .button("Yes!", |s| show_answer(s, "I knew it! Well done!"))
            .button("No!", |s| show_answer(s, "I knew you couldn't be trusted!"))
            .button("Uh?", |s| s.add_layer(Dialog::info("Try again."))),
    )
}

fn show_answer(s: &mut Cursive, msg: &str) {
    s.pop_layer();
    s.add_layer(
        Dialog::text(msg)
            .title("Results")
            .button("Finish", |s| s.quit()),
    );
}
