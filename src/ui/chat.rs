//! Chat and user list UI screens.

use ratatui::{Frame, layout::{Rect, Layout, Constraint, Direction}, style::{Style, Color, Modifier}, widgets::{Block, Paragraph, Borders, List, ListItem, Wrap}, text::{Line, Span}};
use crate::app::{App, ChatFocus};
use crate::ui::avatar::get_avatar_protocol;
use crate::ui::time_format::{format_message_timestamp, format_date_delimiter};
use ratatui_image::StatefulImage;
use ratatui::widgets::ListState;
use chrono::TimeZone;

pub fn draw_chat(f: &mut Frame, app: &mut App, area: Rect) {
    // Sidebar: servers/channels | Main: chat | Optional: user list
    let sidebar_width = 28;
    let show_users = app.show_user_list;
    let focus = app.chat_focus;
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
    draw_sidebar(f, app, chunks[0], focus == ChatFocus::Sidebar);
    draw_chat_main(f, app, chunks[1], focus == ChatFocus::Messages);
    if show_users && chunks.len() > 2 {
        draw_user_list(f, app, chunks[2], focus == ChatFocus::Users);
    }
    if app.chat_focus == ChatFocus::DMInput {
        crate::ui::popups::draw_dm_input_popup(f, app);
    }
}

pub fn draw_sidebar(f: &mut Frame, app: &mut App, area: Rect, focused: bool) {
    let border_style = if focused {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let block = Block::default().borders(Borders::ALL).title("Servers").border_style(border_style);
    f.render_widget(block.clone(), area);
    let inner = block.inner(area);
    if inner.width == 0 || inner.height == 0 { return; }
    // List servers and channels
    let mut items = Vec::new();
    for (si, server) in app.servers.iter().enumerate() {
        let selected_server = app.selected_server == Some(si);
        let server_style = if selected_server {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else { Style::default().fg(Color::Gray) };
        items.push(ListItem::new(Line::from(vec![Span::styled(format!("● {}", server.name), server_style)])));
        if selected_server {
            for (ci, channel) in server.channels.iter().enumerate() {
                let selected_channel = app.selected_channel == Some(ci);
                let channel_style = if selected_channel {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else { Style::default() };
                items.push(ListItem::new(Line::from(vec![Span::styled(format!("  # {}", channel.name), channel_style)])));
            }
        }
    }
    let mut list_state = ListState::default();
    // Highlight selected server/channel
    let mut idx = 0;
    for (si, server) in app.servers.iter().enumerate() {
        if app.selected_server == Some(si) {
            idx += 1 + app.selected_channel.unwrap_or(0);
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

pub fn draw_chat_main(f: &mut Frame, app: &mut App, area: Rect, focused: bool) {
    let border_style = if focused {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let chunks = Layout::default().constraints([Constraint::Min(0), Constraint::Length(3)]).split(area);
    let messages_area = chunks[0];
    let input_area = chunks[1];

    let channel_name = if let (Some(s), Some(c)) = (app.selected_server, app.selected_channel) {
        app.servers.get(s).and_then(|srv| srv.channels.get(c)).map(|ch| ch.name.clone()).unwrap_or_else(|| "?".to_string())
    } else { "?".to_string() };
    let messages_block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Chat // #{}", channel_name))
        .border_style(border_style);
    f.render_widget(messages_block.clone(), messages_area);

    let inner_area = messages_block.inner(messages_area);
    if inner_area.width == 0 || inner_area.height == 0 { return; }

    const AVATAR_PIXEL_SIZE: u32 = 32;
    let (font_w, font_h) = app.picker.font_size();
    let (font_w, font_h) = if font_w == 0 || font_h == 0 { (8, 16) } else { (font_w, font_h) };
    let avatar_cell_width = (AVATAR_PIXEL_SIZE as f32 / font_w as f32).ceil() as u16;
    let avatar_cell_height = (AVATAR_PIXEL_SIZE as f32 / font_h as f32).ceil() as u16;
    let row_height = avatar_cell_height.max(2);

    // Calculate how many messages fit
    let max_rows = (inner_area.height as usize) / (row_height as usize + 1);
    app.last_chat_rows = Some(max_rows);
    let (display_items, is_channel_chat) = if let (Some(s), Some(c)) = (app.selected_server, app.selected_channel) {
        if let Some(server) = app.servers.get(s) {
            if let Some(channel) = server.channels.get(c) {
                let total_msgs = channel.messages.len();
                let end_idx = total_msgs.saturating_sub(app.chat_scroll_offset);
                let start_idx = end_idx.saturating_sub(max_rows);
                let items: Vec<_> = channel.messages.iter().skip(start_idx).take(end_idx - start_idx).map(|msg| {
                    // Add timestamp to tuple
                    (Some(msg.author_username.clone()), msg.author_color, msg.content.clone(), msg.author_profile_pic.clone(), Some(msg.timestamp))
                }).collect();
                (items, true)
            } else { (vec![], true) }
        } else { (vec![], true) }
    } else {
        let total_msgs = app.chat_messages.len();
        let end_idx = total_msgs.saturating_sub(app.chat_scroll_offset);
        let start_idx = end_idx.saturating_sub(max_rows);
        let items: Vec<_> = app.chat_messages.iter().skip(start_idx).take(end_idx - start_idx).map(|msg| {
            (Some(msg.author.clone()), msg.color, msg.content.clone(), None, None)
        }).collect();
        (items, false)
    };

    let now = chrono::Local::now();
    let mut last_date: Option<chrono::NaiveDate> = None;
    let mut current_y = inner_area.y;
    for (author_opt, color, content, profile_pic, timestamp_opt) in display_items.into_iter() {
        if current_y + row_height > inner_area.y + inner_area.height { break; }
        // Insert date delimiter only when the date changes (not for the first message)
        if let Some(ts) = timestamp_opt {
            let dt = chrono::Local.timestamp_opt(ts, 0).single();
            if let Some(dt) = dt {
                let msg_date = dt.date_naive();
                if let Some(last) = last_date {
                    if last != msg_date {
                        // Draw a Block with Borders::TOP and the date as the title
                        let header = Block::default()
                            .borders(Borders::TOP)
                            .title_alignment(ratatui::layout::Alignment::Center)
                            .title(format_date_delimiter(ts))
                            .border_style(Style::default().fg(Color::DarkGray))
                            .style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC));
                        f.render_widget(header, Rect::new(inner_area.x, current_y, inner_area.width, row_height));
                        current_y += row_height;
                    }
                }
                last_date = Some(msg_date);
            }
        }
        let row_area = Rect::new(inner_area.x, current_y, inner_area.width, row_height);
        // Avatar/profile pic rendering
        let avatar_area = Rect::new(row_area.x, row_area.y, avatar_cell_width, avatar_cell_height);
        let text_area = Rect::new(row_area.x + avatar_cell_width + 1, row_area.y, row_area.width - avatar_cell_width - 1, row_height);
        if is_channel_chat {
            if let Some(ref author) = author_opt {
                // Find the user first, then drop the borrow before calling get_avatar_protocol
                let user_opt = app.connected_users.iter().find(|u| u.username == *author).cloned();
                if let Some(user) = user_opt {
                    if let Some(state) = get_avatar_protocol(app, &user, AVATAR_PIXEL_SIZE) {
                        let image_widget = StatefulImage::default();
                        f.render_stateful_widget(image_widget, avatar_area, state);
                    }
                } else if let Some(ref pic) = profile_pic {
                    // Fallback: build a fake User with nil id
                    let user = common::User {
                        id: uuid::Uuid::nil(),
                        username: author.to_string(),
                        color,
                        role: common::UserRole::User,
                        profile_pic: Some(pic.to_string()),
                        cover_banner: None,
                        status: common::UserStatus::Offline, // Add status for fallback user
                    };
                    if let Some(state) = get_avatar_protocol(app, &user, AVATAR_PIXEL_SIZE) {
                        let image_widget = StatefulImage::default();
                        f.render_stateful_widget(image_widget, avatar_area, state);
                    }
                }
            }
        } else {
            // For global chat, show a fallback icon
            let fallback = Line::from(Span::styled("○", Style::default().fg(Color::Gray)));
            f.render_widget(Paragraph::new(fallback), avatar_area);
        }
        // Mention parsing and coloring
        let mut spans = Vec::new();
        let mut last = 0;
        let content_str = &content;
        let mention_re = regex::Regex::new(r"@([a-zA-Z0-9_]+)").unwrap();
        for m in mention_re.find_iter(content_str) {
            let start = m.start();
            let end = m.end();
            if start > last {
                spans.push(Span::raw(&content_str[last..start]));
            }
            let mention = &content_str[start+1..end];
            // Try to find the mentioned user in connected_users (for color)
            let mention_color = app.channel_userlist.iter().find(|u| u.username == mention).map(|u| u.color);
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
        let author = author_opt.unwrap_or_else(|| "?".to_string());
        // Format timestamp if present
        let timestamp_str = if let Some(ts) = timestamp_opt {
            format_message_timestamp(ts, now.clone())
        } else {
            "".to_string()
        };
        let text = if !timestamp_str.is_empty() {
            vec![
                Line::from(vec![
                    Span::styled(format!("<{}>", author), Style::default().fg(color).add_modifier(Modifier::BOLD)),
                    Span::raw(" "),
                    Span::styled(format!("{}", timestamp_str), Style::default().fg(Color::DarkGray)),
                ]),
                Line::from(spans),
            ]
        } else {
            vec![
                Line::from(Span::styled(format!("<{}>", author), Style::default().fg(color).add_modifier(Modifier::BOLD))),
                Line::from(spans),
            ]
        };
        f.render_widget(Paragraph::new(text).wrap(Wrap { trim: true }), text_area);
        current_y += row_height + 1;
    }

    let input = Paragraph::new(app.current_input.as_str()).style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, input_area);
    if focused {
        f.set_cursor_position((input_area.x + app.current_input.len() as u16 + 1, input_area.y + 1));
    }

    // Draw mention suggestions popup if present
    if focused {
        if !app.mention_suggestions.is_empty() {
            let max_name_len = app.mention_suggestions.iter().map(|&i| app.channel_userlist[i].username.len()).max().unwrap_or(8).max(8);
            let popup_width = (max_name_len + 12).min(input_area.width as usize) as u16;
            let mut lines = vec![];
            for (i, &user_idx) in app.mention_suggestions.iter().enumerate() {
                let user = &app.channel_userlist[user_idx];
                let style = if i == app.mention_selected {
                    Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(user.color).bg(Color::Black)
                };
                lines.push(Line::from(Span::styled(format!("{}", user.username), style)));
            }
            // Height: lines + 2 (for borders/title)
            let popup_height = (lines.len() as u16).saturating_add(2);
            // Default: below input
            let mut popup_y = input_area.y + input_area.height;
            // If overflow, move above input
            if popup_y + popup_height > area.y + area.height {
                if input_area.y >= popup_height {
                    popup_y = input_area.y - popup_height;
                } else {
                    popup_y = area.y; // Clamp to top
                }
            }
            // Clamp popup_y to not go above chat area
            if popup_y < area.y { popup_y = area.y; }
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
    }

    // Draw scrollbar if there are more messages than fit
    if is_channel_chat {
        if let (Some(s), Some(c)) = (app.selected_server, app.selected_channel) {
            if let Some(server) = app.servers.get(s) {
                if let Some(channel) = server.channels.get(c) {
                    let total_msgs = channel.messages.len();
                    if total_msgs > max_rows && inner_area.width > 2 {
                        let bar_x = inner_area.x + inner_area.width - 1;
                        let bar_y = inner_area.y;
                        let bar_height = inner_area.height;
                        let thumb_height = ((max_rows as f32 / total_msgs as f32) * bar_height as f32).ceil().max(1.0) as u16;
                        let max_offset = total_msgs.saturating_sub(max_rows);
                        let offset = app.chat_scroll_offset.min(max_offset);
                        // Invert thumb position: offset=0 => bottom, max_offset => top
                        let thumb_pos = if max_offset > 0 {
                            ((1.0 - (offset as f32 / max_offset as f32)) * (bar_height - thumb_height) as f32).round() as u16
                        } else { bar_height.saturating_sub(thumb_height) };
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
            }
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
    let (font_w, font_h) = app.picker.font_size();
    let (font_w, font_h) = if font_w == 0 || font_h == 0 { (8, 16) } else { (font_w, font_h) };
    let avatar_cell_width = (AVATAR_PIXEL_SIZE as f32 / font_w as f32).ceil() as u16;
    let avatar_cell_height = (AVATAR_PIXEL_SIZE as f32 / font_h as f32).ceil() as u16;
    let row_height = avatar_cell_height.max(1);

    let mut current_y = inner_area.y;
    use std::collections::BTreeMap;
    use common::UserRole;
    // Group users by role
    let mut role_map: BTreeMap<UserRole, Vec<_>> = BTreeMap::new();
    for user in app.channel_userlist.iter() {
        role_map.entry(user.role.clone()).or_default().push(user.clone());
    }
    // Collect roles and reverse
    let mut roles: Vec<_> = role_map.keys().cloned().collect();
    roles.sort();
    roles.reverse();
    // Status order: Connected, Busy, Away, Offline
    fn status_order(status: &common::UserStatus) -> u8 {
        match status {
            common::UserStatus::Connected => 0,
            common::UserStatus::Busy => 1,
            common::UserStatus::Away => 2,
            common::UserStatus::Offline => 3,
        }
    }
    // For selection logic
    let list_state = app.user_list_state.clone();
    let selected_index = list_state.selected();
    let offset = list_state.offset();
    let mut idx = 0;
    for role in roles {
        let mut users = role_map.get(&role).cloned().unwrap_or_default();
        users.sort_by(|a, b| {
            status_order(&a.status).cmp(&status_order(&b.status))
                .then_with(|| a.username.to_lowercase().cmp(&b.username.to_lowercase()))
        });
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
                Style::default().fg(user.color)
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
