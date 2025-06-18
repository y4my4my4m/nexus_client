//! Forum, thread, and post list UI screens.

use ratatui::{Frame, layout::Rect, style::{Style, Color, Modifier}, widgets::{Block, List, ListItem, Paragraph, Borders, Wrap}, text::{Line, Span}};
use crate::app::App;
use crate::ui::time_format::{format_message_timestamp, format_date_delimiter};
use chrono::Local;

pub fn draw_forum_list(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app.forums.iter().map(|forum| {
        ListItem::new(Line::from(vec![
            Span::styled(format!("{:<30}", forum.name), Style::default().fg(Color::Cyan)),
            Span::raw(forum.description.clone())
        ]))
    }).collect();
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Forums"))
        .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, area, &mut app.forum_list_state);
}

pub fn draw_thread_list(f: &mut Frame, app: &mut App, area: Rect) {
    let forum = match app.current_forum_id.and_then(|id| app.forums.iter().find(|f| f.id == id)) {
        Some(f) => f,
        None => { f.render_widget(Paragraph::new("Forum not found..."), area); return; }
    };
    let items: Vec<ListItem> = forum.threads.iter().map(|thread| {
        let date_str = format_date_delimiter(thread.timestamp);
        ListItem::new(Line::from(vec![
            Span::styled(format!("{:<40}", thread.title), Style::default().fg(Color::Cyan)),
            Span::raw(" by "),
            Span::styled(&thread.author.username, Style::default().fg(thread.author.color)),
            Span::raw(" | "),
            Span::styled(date_str, Style::default().fg(Color::Gray)),
        ]))
    }).collect();
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(format!("Threads in '{}' | [N]ew Thread", forum.name)))
        .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, area, &mut app.thread_list_state);
}

pub fn draw_post_view(f: &mut Frame, app: &mut App, area: Rect) {
    let thread = match (app.current_forum_id, app.current_thread_id) {
        (Some(fid), Some(tid)) => app.forums.iter().find(|f| f.id == fid)
            .and_then(|f| f.threads.iter().find(|t| t.id == tid)),
        _ => None,
    };
    if let Some(thread) = thread {
        let title = format!("Reading: {} | [R]eply", thread.title);
        let mut text: Vec<Line> = Vec::new();
        for post in &thread.posts {
            let ts_str = format_message_timestamp(post.timestamp, Local::now());
            text.push(Line::from(vec![
                Span::styled(format!("From: {} ", post.author.username), Style::default().fg(post.author.color).add_modifier(Modifier::BOLD)),
                Span::raw(format!("({})", ts_str)),
            ]));
            text.push(Line::from(Span::raw("---------------------------------")));
            text.push(Line::from(Span::raw(&post.content)));
            text.push(Line::from(Span::raw("")));
        }
        let paragraph = Paragraph::new(text).block(Block::default().borders(Borders::ALL).title(title)).wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
    } else {
        f.render_widget(Paragraph::new("Thread not found..."), area);
    }
}
