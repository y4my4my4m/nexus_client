use ratatui::{
    Frame,
    layout::Rect,
    style::{Style, Color},
    widgets::{Block, Borders, List, ListItem},
    text::{Line, Span}
};
use crate::app::App;

pub fn render_forum_list(f: &mut Frame, area: Rect, app: &mut App) {
    let items: Vec<ListItem> = app.forum.forums.iter().map(|forum| {
        ListItem::new(Line::from(vec![
            Span::styled(format!("{:<30}", forum.name), Style::default().fg(Color::Cyan)),
            Span::raw(forum.description.clone())
        ]))
    }).collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Forums"));
    f.render_stateful_widget(list, area, &mut app.forum.forum_list_state);
}