use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};

pub fn draw_screen_picker(f: &mut ratatui::Frame, area: Rect, selected: usize) {
    let width = 24u16;
    let height = 11u16;
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    let popup = Rect::new(x, y, width, height);

    f.render_widget(ratatui::widgets::Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Go to ");
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    // (index, before_key, key_char, after_key)
    let entries: [(usize, &str, &str, &str); 5] = [
        (0, "", "P", "lay"),
        (1, "", "S", "tats"),
        (2, "Pro", "g", "ress"),
        (3, "", "C", "oach"),
        (4, "S", "t", "rategy"),
    ];

    let mut items: Vec<ListItem> = entries
        .iter()
        .map(|(i, before, key, after)| {
            let is_sel = *i == selected;
            let bg = if is_sel { Color::Cyan } else { Color::Reset };
            let fg = if is_sel { Color::Black } else { Color::White };
            let key_fg = if is_sel {
                Color::Black
            } else {
                Color::Rgb(100, 180, 255)
            };
            ListItem::new(Line::from(vec![
                Span::styled("  ", Style::default().fg(fg).bg(bg)),
                Span::styled(*before, Style::default().fg(fg).bg(bg)),
                Span::styled(
                    *key,
                    Style::default()
                        .fg(key_fg)
                        .bg(bg)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                ),
                Span::styled(*after, Style::default().fg(fg).bg(bg)),
                Span::styled("  ", Style::default().fg(fg).bg(bg)),
            ]))
        })
        .collect();

    // Separator
    items.push(ListItem::new(Span::styled(
        "──────────────────────",
        Style::default().fg(Color::DarkGray),
    )));

    // Quit
    let is_sel = selected == 5;
    let bg = if is_sel { Color::Red } else { Color::Reset };
    let fg = if is_sel { Color::Black } else { Color::Red };
    let key_fg = if is_sel {
        Color::Black
    } else {
        Color::Rgb(100, 180, 255)
    };
    items.push(ListItem::new(Line::from(vec![
        Span::styled("  ", Style::default().fg(fg).bg(bg)),
        Span::styled(
            "Q",
            Style::default()
                .fg(key_fg)
                .bg(bg)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ),
        Span::styled("uit", Style::default().fg(fg).bg(bg)),
        Span::styled("  ", Style::default().fg(fg).bg(bg)),
    ])));

    f.render_widget(List::new(items), inner);
}
