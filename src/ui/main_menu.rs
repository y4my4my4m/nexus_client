//! Main menu UI screen.

use ratatui::{Frame, layout::Rect, style::{Style, Color}, widgets::{Block, List, ListItem, Borders}, text::Span};
use crate::app::App;

pub fn draw_main_menu(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = ["Forums", "Chat", "Settings", "Logout"].iter().enumerate().map(|(i, &item)| {
        let style = if Some(i) == app.ui.main_menu_state.selected() {
            Style::default().bg(Color::LightCyan).fg(Color::Black)
        } else {
            Style::default()
        };
        ListItem::new(Span::styled(item, style))
    }).collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Main Menu"));
    f.render_stateful_widget(list, area, &mut app.ui.main_menu_state);
}
