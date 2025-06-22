//! Chat and user list UI screens.

use ratatui::{Frame, layout::{Rect, Layout, Constraint, Direction}, style::{Style, Color, Modifier}, widgets::{Block, Paragraph, Borders, List, ListItem, Wrap}, text::{Line, Span}};
use crate::app::{App, ChatFocus};
use crate::ui::avatar::get_avatar_protocol;
use ratatui_image::StatefulImage;
use ratatui::widgets::ListState;
use ratatui::widgets::{Tabs};
use crate::ui::time_format::{format_date_delimiter, format_message_timestamp};
use chrono::TimeZone;

pub fn draw_chat(f: &mut Frame, app: &mut App, area: Rect) {
    // Sidebar with Tabs: [ Servers ] [ DMs ]
    let sidebar_width = 28;
    let show_users = app.chat.show_user_list;
    let focus = app.chat.chat_focus;
    let chunks = if show_users {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(sidebar_width),
                Constraint::Percentage(55),
                Constraint::Percentage(25),
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(sidebar_width),
                Constraint::Min(0),
            ])
            .split(area)
    };
    // Tabs at the top of the sidebar
    let tab_titles = vec![
        if app.chat.unread_channels.is_empty() {
            Line::from("Servers")
        } else {
            Line::from(vec![Span::raw("Servers "), Span::styled("○", Style::default().fg(Color::Red))])
        },
        if app.chat.unread_dm_conversations.is_empty() {
            Line::from("DMs")
        } else {
            Line::from(vec![Span::raw("DMs "), Span::styled("○", Style::default().fg(Color::Red))])
        },
    ];
    let tab_idx = match app.chat.sidebar_tab {
        crate::state::SidebarTab::Servers => 0,
        crate::state::SidebarTab::DMs => 1,
    };
    let tabs_border_style = if focus == ChatFocus::Sidebar {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let tabs = Tabs::new(tab_titles)
        .select(tab_idx)
        .block(Block::default().borders(Borders::ALL).border_style(tabs_border_style))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .style(Style::default());
    // Layout: Tabs (1 row), then content
    let sidebar_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(chunks[0]);
    f.render_widget(tabs, sidebar_chunks[0]);
    match app.chat.sidebar_tab {
        crate::state::SidebarTab::Servers => {
            draw_sidebar_servers(f, app, sidebar_chunks[1], focus == ChatFocus::Sidebar);
            draw_chat_main(f, app, chunks[1], focus == ChatFocus::Messages);
        }
        crate::state::SidebarTab::DMs => {
            draw_sidebar_dms(f, app, sidebar_chunks[1], focus == ChatFocus::Sidebar);
            draw_chat_main(f, app, chunks[1], focus == ChatFocus::Messages);
        }
    }
    if show_users && chunks.len() > 2 {
        draw_user_list(f, app, chunks[2], focus == ChatFocus::Users);
    }
    if app.chat.chat_focus == ChatFocus::DMInput {
        crate::ui::popups::draw_dm_input_popup(f, app);
    }
}

// Draw server/channel list with unread indicators
pub fn draw_sidebar_servers(f: &mut Frame, app: &mut App, area: Rect, focused: bool) {
    let border_style = if focused {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let block = Block::default().borders(Borders::ALL).title("Servers").border_style(border_style);
    f.render_widget(block.clone(), area);
    let inner = block.inner(area);
    if inner.width == 0 || inner.height == 0 { return; }
    let mut items = Vec::new();
    for (si, server) in app.chat.servers.iter().enumerate() {
        let selected_server = app.chat.selected_server == Some(si);
        // Unread indicator for server: any channel in this server is unread
        let has_unread = server.channels.iter().any(|c| app.chat.unread_channels.contains(&c.id));
        let mut server_spans = vec![Span::styled(format!("● {}", server.name), if selected_server {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else { Style::default().fg(Color::Gray) })];
        if has_unread {
            server_spans.push(Span::raw(" "));
            server_spans.push(Span::styled("○", Style::default().fg(Color::Red)));
        }
        items.push(ListItem::new(Line::from(server_spans)));
        if selected_server {
            for (ci, channel) in server.channels.iter().enumerate() {
                let selected_channel = app.chat.selected_channel == Some(ci);
                let channel_name = format!("  #{}", channel.name);
                if app.chat.unread_channels.contains(&channel.id) {
                    items.push(ListItem::new(Line::from(vec![
                        Span::styled(channel_name, if selected_channel {
                            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                        } else { Style::default() }),
                        Span::styled(" ○", Style::default().fg(Color::Red)),
                    ])));
                } else {
                    items.push(ListItem::new(Line::from(vec![
                        Span::styled(channel_name, if selected_channel {
                            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                        } else { Style::default() }),
                    ])));
                }
            }
        }
    }
    let mut list_state = ListState::default();
    // Highlight selected server/channel
    let mut idx = 0;
    for (si, _server) in app.chat.servers.iter().enumerate() {
        if app.chat.selected_server == Some(si) {
            idx += 1 + app.chat.selected_channel.unwrap_or(0);
            break;
        }
        idx += 1;
    }
    list_state.select(Some(idx));
    let list = List::new(items)
        .block(Block::default())
        .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, inner, &mut list_state);
}

// Draw DM conversation list, ordered by most recent, with unread indicators
pub fn draw_sidebar_dms(f: &mut Frame, app: &mut App, area: Rect, focused: bool) {
    let border_style = if focused {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let block = Block::default().borders(Borders::ALL).title("Direct Messages").border_style(border_style);
    f.render_widget(block.clone(), area);
    let inner = block.inner(area);
    if inner.width == 0 || inner.height == 0 { return; }
    
    // Create a list of (original_index, user) pairs to track original indices
    let mut indexed_users: Vec<(usize, &common::User)> = app.chat.dm_user_list.iter().enumerate().collect();
    // Sort by unread first, then by username
    indexed_users.sort_by_key(|(_, u)| (!app.chat.unread_dm_conversations.contains(&u.id), u.username.clone()));
    
    let items: Vec<ListItem> = indexed_users.iter().map(|(_original_idx, u)| {
            let mut spans = vec![Span::styled(format!("{} {}", if u.status == common::UserStatus::Connected { "●" } else { "○" }, u.username), Style::default().fg(u.color.clone().into()))];
            if app.chat.unread_dm_conversations.contains(&u.id) {
                spans.push(Span::raw(" "));
                spans.push(Span::styled("○", Style::default().fg(Color::Red)));
            }
            ListItem::new(Line::from(spans))
        }).collect();
    
    // Find the display index for the selected DM user
    let display_selection = if let Some(selected_original_idx) = app.chat.selected_dm_user {
        indexed_users.iter().position(|(original_idx, _)| *original_idx == selected_original_idx)
    } else {
        None
    };
    
    let mut list_state = ListState::default();
    list_state.select(display_selection);
    let list = List::new(items)
        .block(Block::default())
        .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, inner, &mut list_state);
}

fn draw_message_list(f: &mut Frame, app: &mut App, area: Rect, focused: bool, title: &str) {
    use ratatui::widgets::{Block, Borders, Paragraph};
    use ratatui::text::{Span, Line};
    use crate::ui::avatar::get_avatar_protocol;
    use ratatui_image::StatefulImage;

    let border_style = if focused {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let block = Block::default().borders(Borders::ALL).title(title).border_style(border_style);
    f.render_widget(block.clone(), area);
    let inner_area = block.inner(area);
    if inner_area.width == 0 || inner_area.height == 0 { return; }

    const AVATAR_PIXEL_SIZE: u32 = 32;
    let (font_w, font_h) = app.profile.picker.font_size();
    let (font_w, font_h) = if font_w == 0 || font_h == 0 { (8, 16) } else { (font_w, font_h) };
    let avatar_cell_width = (AVATAR_PIXEL_SIZE as f32 / font_w as f32).ceil() as u16;
    let avatar_cell_height = (AVATAR_PIXEL_SIZE as f32 / font_h as f32).ceil() as u16;
    let min_row_height = avatar_cell_height.max(2);

    let messages = app.get_current_message_list();
    
    // Calculate how many messages we can fit by working backwards from the bottom
    // For scrolling calculation, use average row height estimation
    let estimated_avg_row_height = min_row_height + 2; // +2 for spacing and potential wrapping
    let max_rows_estimate = (inner_area.height as usize) / (estimated_avg_row_height as usize + 1);
    app.chat.last_chat_rows = Some(max_rows_estimate); // Store for scroll calculations
    
    let total_msgs = messages.len();
    let max_scroll = total_msgs.saturating_sub(max_rows_estimate);
    let scroll_offset = app.chat.chat_scroll_offset.min(max_scroll);
    let end_idx = total_msgs.saturating_sub(scroll_offset);
    let start_idx = end_idx.saturating_sub(max_rows_estimate * 2); // Get more messages than estimated to account for varying heights
    let display_items = &messages[start_idx.max(0)..end_idx];

    let now = chrono::Local::now();
    let mut last_date: Option<chrono::NaiveDate> = None;
    let text_area_width = inner_area.width.saturating_sub(avatar_cell_width + 1);
    
    // Render messages from bottom up to handle dynamic heights properly
    let mut message_heights = Vec::new();
    
    // First pass: calculate heights for all messages
    for msg in display_items.iter() {
        // Calculate content height more accurately
        let content_str = &msg.content;
        let lines_needed = if text_area_width > 0 {
            // Split content by explicit newlines first
            let content_lines: Vec<&str> = content_str.split('\n').collect();
            let mut total_lines = 0;
            
            for line in content_lines {
                if line.is_empty() {
                    total_lines += 1; // Empty lines still take space
                } else {
                    // Calculate how many wrapped lines this content line will take
                    let line_len = line.chars().count();
                    let wrapped_lines = if line_len == 0 {
                        1
                    } else {
                        (line_len + text_area_width as usize - 1) / text_area_width as usize
                    };
                    total_lines += wrapped_lines;
                }
            }
            total_lines
        } else {
            1
        };
        
        // Message height = max(avatar_height, content_lines + header_line)
        let content_height = (lines_needed + 1) as u16; // +1 for author/timestamp line
        let message_height = content_height.max(min_row_height);
        message_heights.push(message_height);
    }
    
    // Find how many messages actually fit, working backwards
    let mut total_height = 0u16;
    let mut visible_count = 0;
    for &height in message_heights.iter().rev() {
        if total_height + height + 1 <= inner_area.height { // +1 for spacing
            total_height += height + 1;
            visible_count += 1;
        } else {
            break;
        }
    }
    
    // Render the visible messages
    let visible_start = display_items.len().saturating_sub(visible_count);
    let visible_messages = &display_items[visible_start..];
    let visible_heights = &message_heights[visible_start..];
    
    // Pre-calculate date delimiter positions to avoid interrupting message rendering
    let mut date_delimiters = Vec::new();
    let mut last_date: Option<chrono::NaiveDate> = None;
    
    // First pass: identify where date delimiters should go (in forward order)
    // Only show delimiters if there are actually multiple dates in the visible messages
    for (i, msg) in visible_messages.iter().enumerate() {
        if let Some(ts) = msg.timestamp {
            if let Some(dt) = chrono::Local.timestamp_opt(ts, 0).single() {
                let msg_date = dt.date_naive();
                if let Some(last) = last_date {
                    if last != msg_date {
                        // Mark this position for a date delimiter before this message
                        date_delimiters.push((i, ts));
                    }
                }
                last_date = Some(msg_date);
            }
        }
    }
    
    // Start from bottom and work up
    let mut current_y = inner_area.y + inner_area.height;
    
    for (msg_idx, (msg, &msg_height)) in visible_messages.iter().zip(visible_heights.iter()).enumerate().rev() {
        current_y = current_y.saturating_sub(msg_height + 1);
        
        if current_y < inner_area.y { break; }
        
        // Check if we need to render a date delimiter before this message
        if let Some(&(delimiter_idx, delimiter_ts)) = date_delimiters.iter().find(|&&(idx, _)| idx == msg_idx) {
            let header = Block::default()
                .borders(Borders::TOP)
                .title_alignment(ratatui::layout::Alignment::Center)
                .title(format_date_delimiter(delimiter_ts))
                .border_style(Style::default().fg(Color::DarkGray))
                .style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC));
            f.render_widget(header, Rect::new(inner_area.x, current_y, inner_area.width, min_row_height));
            current_y = current_y.saturating_sub(min_row_height + 1);
        }
        
        let row_area = Rect::new(inner_area.x, current_y, inner_area.width, msg_height);
        let avatar_area = Rect::new(row_area.x, row_area.y, avatar_cell_width, avatar_cell_height);
        let text_area = Rect::new(row_area.x + avatar_cell_width + 1, row_area.y, text_area_width, msg_height);
        
        // Avatar/profile pic rendering
        let user_for_avatar = match &app.chat.current_chat_target {
            Some(crate::state::ChatTarget::Channel { channel_id: _, server_id: _ }) => {
                // Clone the user to avoid borrowing issues
                app.chat.channel_userlist.iter().find(|u| u.username == msg.author).cloned()
            }
            Some(crate::state::ChatTarget::DM { user_id: _ }) => {
                if let Some(dm_user) = app.chat.dm_user_list.iter().find(|u| u.username == msg.author) {
                    Some(dm_user.clone())
                } else if let Some(current) = &app.auth.current_user {
                    if &current.username == &msg.author {
                        Some(current.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None
        };
        if let Some(user) = user_for_avatar {
            if let Some(state) = get_avatar_protocol(app, &user, AVATAR_PIXEL_SIZE) {
                let image_widget = StatefulImage::default();
                f.render_stateful_widget(image_widget, avatar_area, state);
            }
        } else if let Some(ref pic) = msg.profile_pic {
            // fallback: build a User with just the info from the message
            let fallback_user = common::User {
                id: uuid::Uuid::nil(),
                username: msg.author.clone(),
                color: msg.color.clone().into(),
                role: common::UserRole::User,
                profile_pic: Some(pic.clone()),
                cover_banner: None,
                status: common::UserStatus::Offline,
            };
            if let Some(state) = get_avatar_protocol(app, &fallback_user, AVATAR_PIXEL_SIZE) {
                let image_widget = StatefulImage::default();
                f.render_stateful_widget(image_widget, avatar_area, state);
            }
        } else {
            let fallback = Line::from(Span::styled("○", Style::default().fg(Color::Gray)));
            f.render_widget(Paragraph::new(fallback), avatar_area);
        }
        
        // Mention parsing and coloring
        let mut spans = Vec::new();
        let mut last = 0;
        let content_str = &msg.content;
        let mention_re = regex::Regex::new(r"@([a-zA-Z0-9_]+)").unwrap();
        for m in mention_re.find_iter(content_str) {
            let start = m.start();
            let end = m.end();
            if start > last {
                spans.push(Span::raw(&content_str[last..start]));
            }
            let mention = &content_str[start+1..end];
            let mention_color = app.chat.channel_userlist.iter().find(|u| u.username == mention).map(|u| u.color.clone().into());
            if let Some(mcolor) = mention_color {
                spans.push(Span::styled(format!("@{}", mention), Style::default().fg(Color::Black).bg(mcolor).add_modifier(Modifier::BOLD)));
            } else {
                spans.push(Span::styled(format!("@{}", mention), Style::default().add_modifier(Modifier::BOLD)));
            }
            last = end;
        }
        if last < content_str.len() {
            spans.push(Span::raw(&content_str[last..]));
        }
        
        let author = &msg.author;
        let timestamp_str = msg.timestamp.map(|ts| format_message_timestamp(ts, now.clone())).unwrap_or_default();
        let text = if !timestamp_str.is_empty() {
            vec![
                Line::from(vec![
                    Span::styled(format!("<{}>", author), Style::default().fg(msg.color).add_modifier(Modifier::BOLD)),
                    Span::raw(" "),
                    Span::styled(timestamp_str, Style::default().fg(Color::DarkGray)),
                ]),
                Line::from(spans),
            ]
        } else {
            vec![
                Line::from(Span::styled(format!("<{}>", author), Style::default().fg(msg.color).add_modifier(Modifier::BOLD))),
                Line::from(spans),
            ]
        };
        f.render_widget(Paragraph::new(text).wrap(ratatui::widgets::Wrap { trim: true }), text_area);
    }
}

pub fn draw_chat_main(f: &mut Frame, app: &mut App, area: Rect, focused: bool) {
    let input_str = app.get_current_input().to_string(); // Clone to avoid borrow conflict
    
    // Calculate approximate input height based on content and available width
    let input_inner_width = area.width.saturating_sub(2); // Account for borders
    let estimated_lines = if input_inner_width > 0 && !input_str.is_empty() {
        // Simple estimation: count characters and divide by width, plus count newlines
        let char_lines = (input_str.len() as u16 + input_inner_width - 1) / input_inner_width;
        let newline_count = input_str.matches('\n').count() as u16;
        (char_lines + newline_count).max(1)
    } else {
        1
    };
    
    // Constrain input height to reasonable bounds (min 3, max 8 lines + borders)
    let input_height = (estimated_lines + 2).clamp(3, 10);
    
    // Split area vertically: messages above, input below
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3), // message list
            Constraint::Length(input_height), // dynamic input box
        ])
        .split(area);

    // Use the clean helper method to get the title
    let title = app.get_current_chat_title();

    draw_message_list(f, app, chunks[0], focused, &title);

    // Create styled input text with white text and colored @mentions
    let mut input_spans = Vec::new();
    let mut last = 0;
    let mention_re = regex::Regex::new(r"@([a-zA-Z0-9_]+)").unwrap();
    
    for m in mention_re.find_iter(&input_str) {
        let start = m.start();
        let end = m.end();
        
        // Add text before the mention in white
        if start > last {
            input_spans.push(Span::styled(&input_str[last..start], Style::default().fg(Color::White)));
        }
        
        // Add the mention with user color or default styling
        let mention = &input_str[start+1..end];
        let mention_color = app.chat.channel_userlist.iter().find(|u| u.username == mention).map(|u| u.color.clone().into());
        if let Some(mcolor) = mention_color {
            input_spans.push(Span::styled(format!("@{}", mention), Style::default().fg(Color::Black).bg(mcolor).add_modifier(Modifier::BOLD)));
        } else {
            input_spans.push(Span::styled(format!("@{}", mention), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
        }
        last = end;
    }
    
    // Add remaining text after last mention in white
    if last < input_str.len() {
        input_spans.push(Span::styled(&input_str[last..], Style::default().fg(Color::White)));
    }
    
    // If no spans were created (no mentions), create a single white span
    if input_spans.is_empty() && !input_str.is_empty() {
        input_spans.push(Span::styled(&input_str, Style::default().fg(Color::White)));
    }

    let char_count = input_str.chars().count();
    let input_title = format!("{} / 500", char_count);

    let input = Paragraph::new(Line::from(input_spans))
        .block(Block::default().borders(Borders::ALL).title(input_title).border_style(
            if focused {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            }
        ))
        .wrap(Wrap { trim: true });
    f.render_widget(input, chunks[1]);
    
    if focused {
        // Improved cursor positioning for multiline input
        let input_area = chunks[1];
        let inner_area = Block::default().borders(Borders::ALL).inner(input_area);
        
        if inner_area.width > 0 {
            let cursor_pos = input_str.len();
            let text_up_to_cursor = &input_str[..cursor_pos];
            
            // More accurate cursor positioning that accounts for wrapping
            let mut current_line = 0u16;
            let mut current_col = 0u16;
            
            for ch in text_up_to_cursor.chars() {
                if ch == '\n' {
                    current_line += 1;
                    current_col = 0;
                } else {
                    current_col += 1;
                    // Handle wrapping when line exceeds width
                    if current_col >= inner_area.width {
                        current_line += 1;
                        current_col = 0;
                    }
                }
            }
            
            let cursor_y = inner_area.y + current_line;
            let cursor_x = inner_area.x + current_col;
            
            // Ensure cursor is within bounds
            if cursor_y < inner_area.y + inner_area.height && cursor_x < inner_area.x + inner_area.width {
                f.set_cursor_position((cursor_x, cursor_y));
            }
        } else {
            // Empty input or no width - place cursor at start
            f.set_cursor_position((inner_area.x, inner_area.y));
        }
    }
    
    // Draw mention suggestions popup if present
    if focused {
        draw_mention_suggestion_popup(f, app, chunks[1], area);
        draw_emoji_suggestion_popup(f, app, chunks[1], area);
    }
    
    // Draw scrollbar if there are more messages than fit - use message area only
    let messages = app.get_current_message_list();
    let total_msgs = messages.len();
    let max_rows = app.chat.last_chat_rows.unwrap_or(0);
    
    if total_msgs > max_rows && chunks[0].width > 2 {
        let msg_area = chunks[0]; // Use message area, not full area
        let bar_x = msg_area.x + msg_area.width - 1;
        let bar_y = msg_area.y;
        let bar_height = msg_area.height;
        
        let thumb_height = ((max_rows as f32 / total_msgs as f32) * bar_height as f32).ceil().max(1.0) as u16;
        let max_offset = total_msgs.saturating_sub(max_rows);
        let offset = app.chat.chat_scroll_offset.min(max_offset);
        
        // Invert thumb position: offset=0 => bottom, max_offset => top
        let thumb_pos = if max_offset > 0 {
            ((1.0 - (offset as f32 / max_offset as f32)) * (bar_height - thumb_height) as f32).round() as u16
        } else { 
            bar_height.saturating_sub(thumb_height) 
        };
        
        for i in 0..bar_height {
            let y = bar_y + i;
            let symbol = if i >= thumb_pos && i < thumb_pos + thumb_height {
                "█"
            } else {
                "│"
            };
            f.render_widget(Paragraph::new(symbol), Rect::new(bar_x, y, 1, 1));
        }
    }
}

pub fn draw_user_list(f: &mut Frame, app: &mut App, area: Rect, focused: bool) {
    let border_style = if focused {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let block = Block::default().borders(Borders::ALL).title("Users [Ctrl+U]").border_style(border_style);
    f.render_widget(block.clone(), area);

    let inner_area = block.inner(area);
    if inner_area.width == 0 || inner_area.height == 0 { return; }

    const AVATAR_PIXEL_SIZE: u32 = 16;
    let (font_w, font_h) = app.profile.picker.font_size();
    let (font_w, font_h) = if font_w == 0 || font_h == 0 { (8, 16) } else { (font_w, font_h) };
    let avatar_cell_width = (AVATAR_PIXEL_SIZE as f32 / font_w as f32).ceil() as u16;
    let avatar_cell_height = (AVATAR_PIXEL_SIZE as f32 / font_h as f32).ceil() as u16;
    let row_height = avatar_cell_height.max(1);

    let mut current_y = inner_area.y;
    use std::collections::BTreeMap;
    use common::UserRole;
    // Group users by role
    let mut role_map: BTreeMap<UserRole, Vec<_>> = BTreeMap::new();
    for user in app.chat.channel_userlist.iter() {
        role_map.entry(user.role.clone()).or_default().push(user.clone()); // Clone users to avoid borrowing issues
    }
    // Collect roles and reverse
    let mut roles: Vec<_> = role_map.keys().cloned().collect();
    roles.reverse();
    // Status order: Connected, Busy, Away, Offline
    // For selection logic
    let list_state = app.chat.user_list_state.clone();
    let selected_index = list_state.selected();
    let mut idx = 0;
    for role in roles {
        let users = role_map.get(&role).cloned().unwrap_or_default();
        // Draw role header
        if current_y + row_height > inner_area.y + inner_area.height { break; }
        let header = Block::default()
            .borders(Borders::TOP)
            .title_alignment(ratatui::layout::Alignment::Center)
            .title(format!("{:?}", role))
            .border_style(Style::default().fg(Color::DarkGray)) // Set border color to gray
            .style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)); // Set text color to gray
        f.render_widget(header, Rect::new(inner_area.x, current_y, inner_area.width, row_height));
        current_y += row_height;
        for user in users {
            if current_y + row_height > inner_area.y + inner_area.height { break; }
            let row_area = Rect::new(inner_area.x, current_y, inner_area.width, row_height);
            let is_selected = focused && selected_index == Some(idx);
            let text_style = if is_selected {
                Style::default().fg(Color::Black).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(user.color.clone().into())
            };
            if is_selected {
                f.render_widget(Block::default().style(Style::default().bg(Color::Cyan)), row_area);
            }
            let status_symbol = match user.status {
                common::UserStatus::Connected => "●",
                common::UserStatus::Away => "◐",
                common::UserStatus::Busy => "■",
                common::UserStatus::Offline => "○",
            };
            let status_color = match user.status {
                common::UserStatus::Connected => Color::Green,
                common::UserStatus::Away => Color::Yellow,
                common::UserStatus::Busy => Color::Red,
                common::UserStatus::Offline => Color::DarkGray,
            };
            if let Some(state) = get_avatar_protocol(app, &user, AVATAR_PIXEL_SIZE) {
                let row_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(avatar_cell_width), Constraint::Min(0)])
                    .split(row_area);
                let image_widget = StatefulImage::default();
                f.render_stateful_widget(image_widget, row_chunks[0], state);
                let text = Line::from(vec![
                    Span::styled(format!(" {} ", status_symbol), Style::default().fg(status_color)),
                    Span::styled(&user.username, text_style),
                ]);
                f.render_widget(Paragraph::new(text).alignment(ratatui::layout::Alignment::Left), row_chunks[1]);
            } else {
                // Render a blank avatar area for alignment
                let row_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(avatar_cell_width), Constraint::Min(0)])
                    .split(row_area);
                // Optionally, render a placeholder avatar here instead of leaving blank
                // f.render_widget(Paragraph::new(" "), row_chunks[0]);
                let text = Line::from(vec![
                    Span::styled(format!(" {} ", status_symbol), Style::default().fg(status_color)),
                    Span::styled(&user.username, text_style),
                ]);
                f.render_widget(Paragraph::new(text).alignment(ratatui::layout::Alignment::Left), row_chunks[1]);
            }
            current_y += row_height;
            idx += 1;
        }
    }
}

/// Draw the mention suggestion popup below (or above) the input area.
pub fn draw_mention_suggestion_popup(f: &mut Frame, app: &App, input_area: Rect, chat_area: Rect) {
    if app.chat.mention_suggestions.is_empty() { return; }
    let max_name_len = app.chat.mention_suggestions.iter().map(|&i| app.chat.channel_userlist[i].username.len()).max().unwrap_or(8).max(8);
    let popup_width = (max_name_len + 12).min(chat_area.width as usize) as u16;
    let mut lines = vec![];
    for (i, &user_idx) in app.chat.mention_suggestions.iter().enumerate() {
        let user = &app.chat.channel_userlist[user_idx];
        let style = if i == app.chat.mention_selected {
            Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(user.color.clone().into()).bg(Color::Black)
        };
        lines.push(Line::from(Span::styled(format!("{}", user.username), style)));
    }
    let popup_height = (lines.len() as u16).saturating_add(2);
    // Default: below input
    let mut popup_y = input_area.y + input_area.height;
    // If overflow, move above input
    if popup_y + popup_height > chat_area.y + chat_area.height {
        if chat_area.y >= popup_height {
            popup_y = input_area.y.saturating_sub(popup_height);
        } else {
            popup_y = chat_area.y; // Clamp to top
        }
    }
    if popup_y < chat_area.y { popup_y = chat_area.y; }
    let popup_area = Rect::new(
        input_area.x,
        popup_y,
        popup_width,
        popup_height
    );
    let block = Block::default().borders(Borders::ALL).title("Mentions");
    let para = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    f.render_widget(para, popup_area);
}

/// Draw the emoji suggestion popup above the input area.
pub fn draw_emoji_suggestion_popup(f: &mut Frame, app: &App, input_area: Rect, chat_area: Rect) {
    if app.chat.emoji_suggestions.is_empty() { return; }
    
    const GRID_COLS: usize = 4;
    const GRID_ROWS: usize = 5;
    const ITEMS_PER_PAGE: usize = GRID_COLS * GRID_ROWS;
    
    // Calculate which page we're on and visible emojis
    let selected_index = app.chat.emoji_selected;
    let page = selected_index / ITEMS_PER_PAGE;
    let start_idx = page * ITEMS_PER_PAGE;
    let end_idx = (start_idx + ITEMS_PER_PAGE).min(app.chat.emoji_suggestions.len());
    let visible_emojis = &app.chat.emoji_suggestions[start_idx..end_idx];
    
    // Calculate popup size - wider to accommodate 4 columns
    let cell_width = 8; // Width per emoji cell
    let cell_height = 1; // Height per emoji cell
    let popup_width = (GRID_COLS * cell_width + 2) as u16; // +2 for borders
    let popup_height = (GRID_ROWS * cell_height + 2) as u16; // +2 for borders
    
    // Position popup directly above the input area
    let popup_y = input_area.y.saturating_sub(popup_height);
    
    let popup_area = Rect::new(
        input_area.x,
        popup_y,
        popup_width.min(chat_area.width),
        popup_height
    );
    
    // Create the popup block
    let block = Block::default().borders(Borders::ALL).title("Emojis");
    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);
    
    // Draw the emoji grid
    for (i, emoji) in visible_emojis.iter().enumerate() {
        let row = i / GRID_COLS;
        let col = i % GRID_COLS;
        
        let x = inner_area.x + (col * cell_width) as u16;
        let y = inner_area.y + (row * cell_height) as u16;
        let cell_area = Rect::new(x, y, cell_width as u16, cell_height as u16);
        
        // Check if this is the selected emoji
        let global_index = start_idx + i;
        let is_selected = global_index == selected_index;
        
        let style = if is_selected {
            Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White).bg(Color::Black)
        };
        
        // Center the emoji in its cell
        let emoji_text = format!("{:^width$}", emoji, width = cell_width);
        f.render_widget(
            Paragraph::new(emoji_text).style(style),
            cell_area
        );
    }
}
