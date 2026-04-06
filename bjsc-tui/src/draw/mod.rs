mod coach;
mod confirm_quit;
mod histogram;
mod play;
mod progress;
mod screen_picker;
mod strategy;

use crate::App;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::Terminal;
use std::io;

use crate::Screen;

pub fn draw(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &App) -> io::Result<()> {
    terminal.draw(|f| {
        let area = f.area();
        match app.screen {
            Screen::Play => play::draw_play(f, area, app),
            Screen::Histogram => histogram::draw_histogram(f, area, app),
            Screen::Progress => progress::draw_progress(f, area, app),
            Screen::Coach => coach::draw_coach(f, area, app),
            Screen::Strategy => strategy::draw_strategy(f, area, app),
        }
        if app.confirm_quit {
            confirm_quit::draw_confirm_quit(f, area);
        } else if let Some(sel) = app.screen_picker {
            screen_picker::draw_screen_picker(f, area, sel);
        }
    })?;
    Ok(())
}

// Common utilities used by multiple draw functions

pub fn footer_with_hint(f: &mut ratatui::Frame, area: Rect, hint_text: &str) {
    use ratatui::layout::{Constraint, Direction, Layout};
    use ratatui::style::{Color, Style};
    use ratatui::widgets::Paragraph;

    let hint_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Length(22)])
        .split(area);

    let hint = Paragraph::new(hint_text).style(Style::default().fg(Color::DarkGray));
    f.render_widget(hint, hint_cols[0]);

    let version = Paragraph::new(env!("BUILD_TIME")).style(Style::default().fg(Color::DarkGray));
    f.render_widget(version, hint_cols[1]);
}

pub fn centered_line(area: Rect, offset: u16) -> Rect {
    Rect::new(area.x, area.y + offset, area.width, 1)
}
