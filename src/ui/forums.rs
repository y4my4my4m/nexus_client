//! Forum, thread, and post list UI screens.

use ratatui::{Frame, layout::{Rect, Layout, Constraint, Direction}, style::{Style, Color, Modifier}, widgets::{Block, List, ListItem, Paragraph, Borders, Wrap}, text::{Line, Span}};
use ratatui::prelude::Stylize;
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
        None => {
            f.render_widget(Paragraph::new("Forum not found..."), area);
            return;
        }
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Threads in '{}' | [N]ew Thread", forum.name));
    f.render_widget(&block, area);
    let inner_area = block.inner(area);

    // Column constraints for dynamic width
    let constraints = [
        Constraint::Percentage(60), // Title
        Constraint::Percentage(25), // Author
        Constraint::Percentage(15), // Date
    ];
    let row_height = 1;

    // Header row
    let header_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(Rect {
            x: inner_area.x,
            y: inner_area.y,
            width: inner_area.width,
            height: row_height,
        });
    f.render_widget(
        Paragraph::new(Span::styled("Title", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)))
            .alignment(ratatui::layout::Alignment::Left),
        header_layout[0],
    );
    f.render_widget(
        Paragraph::new(Span::styled("Author", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)))
            .alignment(ratatui::layout::Alignment::Left),
        header_layout[1],
    );
    f.render_widget(
        Paragraph::new(Span::styled("Date", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)))
            .alignment(ratatui::layout::Alignment::Left),
        header_layout[2],
    );

    // Thread rows
    let mut y = inner_area.y + row_height;
    for (i, thread) in forum.threads.iter().enumerate() {
        if y + row_height > inner_area.y + inner_area.height {
            break;
        }
        let row_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(Rect {
                x: inner_area.x,
                y,
                width: inner_area.width,
                height: row_height,
            });
        let is_selected = app.thread_list_state.selected() == Some(i);
        let bg_style = if is_selected {
            Style::default().bg(Color::Cyan)
        } else {
            Style::default()
        };
        let (title_fg, author_fg, date_fg) = if is_selected {
            (Color::Black, Color::Black, Color::Black)
        } else {
            (Color::Cyan, thread.author.color, Color::Gray)
        };
        // Title
        let title = thread.title.clone();
        f.render_widget(
            Paragraph::new(Span::styled(title, Style::default().fg(title_fg)).bg(bg_style.bg.unwrap_or(Color::Reset)))
                .alignment(ratatui::layout::Alignment::Left),
            row_layout[0],
        );
        // Author
        let author = thread.author.username.clone();
        f.render_widget(
            Paragraph::new(Span::styled(author, Style::default().fg(author_fg)).bg(bg_style.bg.unwrap_or(Color::Reset)))
                .alignment(ratatui::layout::Alignment::Left),
            row_layout[1],
        );
        // Date
        let date_str = format_date_delimiter(thread.timestamp);
        f.render_widget(
            Paragraph::new(Span::styled(date_str, Style::default().fg(date_fg)).bg(bg_style.bg.unwrap_or(Color::Reset)))
                .alignment(ratatui::layout::Alignment::Left),
            row_layout[2],
        );
        y += row_height;
    }
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
