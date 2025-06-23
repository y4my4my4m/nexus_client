//! Forum, thread, and post list UI screens.

use ratatui::{Frame, layout::{Rect, Layout, Constraint, Direction}, style::{Style, Color, Modifier}, widgets::{Block, List, ListItem, Paragraph, Borders, Wrap}, text::{Line, Span}};
use ratatui::prelude::Stylize;
use crate::app::App;
use crate::ui::time_format::{format_message_timestamp, format_date_delimiter};
use chrono::Local;

pub fn draw_forum_list(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app.forum.forums.iter().map(|forum| {
        ListItem::new(Line::from(vec![
            Span::styled(format!("{:<30}", forum.name), Style::default().fg(Color::Cyan)),
            Span::raw(forum.description.clone())
        ]))
    }).collect();

    let title = if let Some(user) = &app.auth.current_user {
        if user.role == common::UserRole::Admin {
            "Forums | [N]ew Forum | [D]elete Forum"
        } else {
            "Forums"
        }
    } else {
        "Forums"
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, area, &mut app.forum.forum_list_state);
}

pub fn draw_thread_list(f: &mut Frame, app: &mut App, area: Rect) {
    let forum = match app.forum.current_forum_id.and_then(|id| app.forum.forums.iter().find(|f| f.id == id)) {
        Some(f) => f,
        None => {
            f.render_widget(Paragraph::new("Forum not found..."), area);
            return;
        }
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Threads in '{}' | [N]ew Thread{}", 
            forum.name,
            if let Some(user) = &app.auth.current_user {
                if user.role == common::UserRole::Admin {
                    " | [Alt+D]elete Thread"
                } else {
                    ""
                }
            } else {
                ""
            }
        ));
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
        let is_selected = app.forum.thread_list_state.selected() == Some(i);
        let bg_style = if is_selected {
            Style::default().bg(Color::Cyan)
        } else {
            Style::default()
        };
        let (title_fg, author_fg, date_fg) = if is_selected {
            (Color::Black, Color::Black, Color::Black)
        } else {
            (Color::Cyan, thread.author.color.clone().into(), Color::Gray)
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
    let thread = match (app.forum.current_forum_id, app.forum.current_thread_id) {
        (Some(fid), Some(tid)) => app.forum.forums.iter().find(|f| f.id == fid)
            .and_then(|f| f.threads.iter().find(|t| t.id == tid)),
        _ => None,
    };
    
    if let Some(thread) = thread {
        let reply_status = if app.forum.reply_to_post_id.is_some() {
            if let Some(post) = app.forum.get_selected_post() {
                format!(" | Replying to #{}", &post.id.to_string()[..8])
            } else {
                " | Replying".to_string()
            }
        } else {
            "".to_string()
        };
        
        let navigation_help = if app.forum.selected_reply_index.is_some() {
            " | ←→ Navigate Replies | Enter: Jump to Reply | Esc: Clear"
        } else if app.forum.show_reply_context {
            " | Enter: Jump to Original Post | Esc: Clear | C: Show Context"
        } else {
            " | ↑↓ Select Posts | →: View Replies | R: Reply To | Alt+R: Reply | C: Show Context"
        };
        
        let title = format!("Reading: {}{}{}", 
            thread.title,
            reply_status,
            navigation_help
        );
        
        let block = Block::default().borders(Borders::ALL).title(title);
        f.render_widget(&block, area);
        let inner_area = block.inner(area);
        
        if inner_area.width == 0 || inner_area.height == 0 {
            return;
        }
        
        // Calculate scrolling for post view
        let posts = &thread.posts;
        if posts.is_empty() {
            f.render_widget(Paragraph::new("No posts in this thread."), inner_area);
            return;
        }
        
        let selected_post_idx = app.forum.selected_post_index.unwrap_or(0);
        let scroll_offset = app.forum.scroll_offset;
        
        // Calculate how many posts can fit on screen (rough estimate)
        let estimated_lines_per_post = 6; // Header + author + content + separator
        let visible_posts = (inner_area.height as usize / estimated_lines_per_post).max(1);
        
        // Determine which posts to render based on scroll offset
        let start_post_idx = scroll_offset;
        let end_post_idx = (start_post_idx + visible_posts * 2).min(posts.len()); // Render extra for smoother scrolling
        let visible_posts_slice = &posts[start_post_idx..end_post_idx];
        
        let mut text_lines: Vec<Line> = Vec::new();
        
        for (post_offset, post) in visible_posts_slice.iter().enumerate() {
            let post_idx = start_post_idx + post_offset;
            let is_selected = selected_post_idx == post_idx;
            let post_id_short = &post.id.to_string()[..8];
            
            // Get replies to this post
            let replies = app.forum.get_replies_to_post(post.id);
            
            // Post header with ID and replies
            let mut header_spans = vec![
                Span::styled(
                    format!("Post #{}", post_id_short),
                    Style::default().fg(if is_selected { Color::Yellow } else { Color::Cyan }).add_modifier(Modifier::BOLD)
                )
            ];
            
            // Add OP indicator for the first post (original poster)
            if post_idx == start_post_idx && start_post_idx == 0 {
                header_spans.push(Span::raw(" "));
                header_spans.push(Span::styled(
                    "(OP)",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                ));
            }
            
            // Add reply indicators
            if !replies.is_empty() {
                header_spans.push(Span::raw(" "));
                let reply_ids: Vec<String> = replies.iter()
                    .map(|(_, p)| p.id.to_string()[..8].to_string())
                    .collect();
                header_spans.push(Span::styled(
                    format!(">>Replies: {}", reply_ids.join(", ")),
                    Style::default().fg(Color::Green).add_modifier(Modifier::ITALIC)
                ));
            }
            
            // Show if this post is a reply to another
            if let Some(reply_to_id) = post.reply_to {
                let reply_to_short = &reply_to_id.to_string()[..8];
                
                // Check if the reply is to the current user's post
                let is_reply_to_current_user = if let Some(current_user) = &app.auth.current_user {
                    if let Some(thread) = app.forum.get_current_thread() {
                        thread.posts.iter()
                            .find(|p| p.id == reply_to_id)
                            .map(|p| p.author.id == current_user.id)
                            .unwrap_or(false)
                    } else {
                        false
                    }
                } else {
                    false
                };
                
                header_spans.insert(1, Span::raw(" "));
                if is_reply_to_current_user {
                    header_spans.insert(2, Span::styled(
                        format!(">>{} (you)", reply_to_short),
                        Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD)
                    ));
                } else {
                    header_spans.insert(2, Span::styled(
                        format!(">>{}", reply_to_short),
                        Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
                    ));
                }
            }
            
            text_lines.push(Line::from(header_spans));
            
            // Author and timestamp
            let ts_str = format_message_timestamp(post.timestamp, Local::now());
            let author_line = Line::from(vec![
                Span::styled(
                    format!("From: {} ", post.author.username),
                    Style::default().fg(post.author.color.clone().into()).add_modifier(Modifier::BOLD)
                ),
                Span::styled(
                    format!("({})", ts_str),
                    Style::default().fg(Color::DarkGray)
                ),
            ]);
            text_lines.push(author_line);
            
            // Post content with word wrapping
            let content_style = if is_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };
            
            // Simple word wrapping for post content
            let content_words: Vec<&str> = post.content.split_whitespace().collect();
            let mut current_content_line = String::new();
            let line_width = (inner_area.width as usize).saturating_sub(4);
            
            for word in content_words {
                if current_content_line.len() + word.len() + 1 > line_width {
                    if !current_content_line.is_empty() {
                        text_lines.push(Line::from(Span::styled(current_content_line.clone(), content_style)));
                        current_content_line.clear();
                    }
                }
                if !current_content_line.is_empty() {
                    current_content_line.push(' ');
                }
                current_content_line.push_str(word);
            }
            if !current_content_line.is_empty() {
                text_lines.push(Line::from(Span::styled(current_content_line, content_style)));
            }
            
            // Show highlighted replies if this post is selected and has reply navigation active
            if is_selected && app.forum.selected_reply_index.is_some() {
                text_lines.push(Line::from(Span::styled(
                    "--- Replies to this post ---",
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                )));
                
                for (reply_idx, (_, reply_post)) in replies.iter().enumerate() {
                    let is_selected_reply = app.forum.selected_reply_index == Some(reply_idx);
                    let reply_style = if is_selected_reply {
                        Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::LightBlue)
                    };
                    
                    let reply_id_short = &reply_post.id.to_string()[..8];
                    let reply_preview = if reply_post.content.len() > 50 {
                        format!("#{}: {}...", reply_id_short, &reply_post.content[..47])
                    } else {
                        format!("#{}: {}", reply_id_short, reply_post.content)
                    };
                    
                    text_lines.push(Line::from(vec![
                        Span::styled("  → ", Style::default().fg(Color::Green)),
                        Span::styled(reply_preview, reply_style)
                    ]));
                }
            }
            
            // Show context information if this post is a reply and context mode is active
            if is_selected && app.forum.show_reply_context {
                if let Some((replied_to_post, _)) = app.forum.get_replied_to_post() {
                    text_lines.push(Line::from(Span::styled(
                        "--- Context: Original Post ---",
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    )));
                    
                    // Show original post info
                    let original_id_short = &replied_to_post.id.to_string()[..8];
                    let original_author = &replied_to_post.author.username;
                    let original_preview = if replied_to_post.content.len() > 80 {
                        format!("{}...", &replied_to_post.content[..77])
                    } else {
                        replied_to_post.content.clone()
                    };
                    
                    text_lines.push(Line::from(vec![
                        Span::styled("  Original: ", Style::default().fg(Color::Cyan)),
                        Span::styled(
                            format!("#{} by {}", original_id_short, original_author),
                            Style::default().fg(replied_to_post.author.color.clone().into()).add_modifier(Modifier::BOLD)
                        )
                    ]));
                    
                    text_lines.push(Line::from(vec![
                        Span::styled("  Content: ", Style::default().fg(Color::Cyan)),
                        Span::styled(original_preview, Style::default().fg(Color::White))
                    ]));
                    
                    text_lines.push(Line::from(vec![
                        Span::styled("  → Press Enter to jump to this post", 
                                    Style::default().fg(Color::Green).add_modifier(Modifier::ITALIC))
                    ]));
                }
            }
            
            // Separator between posts
            text_lines.push(Line::from(Span::styled(
                "─".repeat(inner_area.width as usize),
                Style::default().fg(Color::DarkGray)
            )));
            text_lines.push(Line::from(Span::raw(""))); // Empty line
        }
        
        // Add scroll indicator at the bottom
        if posts.len() > visible_posts {
            let scroll_info = format!("Posts {}-{} of {} | PgUp/PgDn: Scroll | Home/End: Jump", 
                start_post_idx + 1, 
                end_post_idx.min(posts.len()), 
                posts.len()
            );
            text_lines.push(Line::from(Span::styled(
                scroll_info,
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)
            )));
        }
        
        let paragraph = Paragraph::new(text_lines)
            .wrap(Wrap { trim: false })
            .scroll((0, 0));
        
        f.render_widget(paragraph, inner_area);
    } else {
        f.render_widget(Paragraph::new("Thread not found..."), area);
    }
}
