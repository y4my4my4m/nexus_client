//! Main menu UI screen.

use ratatui::{Frame, layout::Rect, style::{Style, Color}, widgets::{Block, List, ListItem, Borders}};
use crate::app::App;

pub fn draw_main_menu(f: &mut Frame, app: &mut App, area: Rect) {
    let items = vec![
        ListItem::new("Forums"), ListItem::new("Chat"), ListItem::new("Settings"),
        ListItem::new(ratatui::text::Line::styled("Logout", Style::default().fg(Color::Red))),
    ];
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Main Menu"))
        .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black)).highlight_symbol(">> ");
    f.render_stateful_widget(list, area, &mut app.main_menu_state);
}
