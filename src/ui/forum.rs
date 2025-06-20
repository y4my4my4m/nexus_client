```rust
use tui::widgets::{Block, Borders, List, ListItem};
use tui::Frame;

pub fn render_forum_list(f: &mut Frame, area: Rect, app: &mut App) {
    let items: Vec<ListItem> = app.forum.forums.iter().map(|forum| {
        // ...existing code...
    }).collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Forums"));
    f.render_stateful_widget(list, area, &mut app.forum.forum_list_state);
}
```