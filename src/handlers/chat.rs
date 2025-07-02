use crate::app::App;
use crate::sound::SoundType;
use nexus_tui_common::ClientMessage;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle chat-related input
pub fn handle_chat_input(key: KeyEvent, app: &mut App) {
    // Handle popups first
    if handle_chat_popups(key, app) {
        return;
    }

    match app.chat.chat_focus {
        crate::state::ChatFocus::Sidebar => handle_sidebar_input(key, app),
        crate::state::ChatFocus::Messages => handle_message_input(key, app),
        crate::state::ChatFocus::Users => handle_user_list_input(key, app),
        crate::state::ChatFocus::DMInput => handle_dm_input(key, app),
    }
}

fn handle_chat_popups(key: KeyEvent, app: &mut App) -> bool {
    // Handle profile view popup
    if app.profile.show_profile_view_popup {
        app.profile.close_profile_view();
        return true;
    }

    // Handle user actions popup
    if app.profile.show_user_actions {
        match key.code {
            KeyCode::Up => {
                app.sound_manager.play(SoundType::Scroll);
                if app.profile.user_actions_selected > 0 {
                    app.profile.user_actions_selected -= 1;
                }
            }
            KeyCode::Down => {
                app.sound_manager.play(SoundType::Scroll);
                if app.profile.user_actions_selected < 2 {
                    app.profile.user_actions_selected += 1;
                }
            }
            KeyCode::Enter => {
                handle_user_action(app);
            }
            KeyCode::Esc => {
                app.profile.show_user_actions = false;
            }
            _ => {}
        }
        return true;
    }

    // Handle server invite selection popup
    if app.ui.show_server_invite_selection {
        match key.code {
            KeyCode::Up => {
                if app.ui.server_invite_selected > 0 {
                    app.ui.server_invite_selected -= 1;
                }
            }
            KeyCode::Down => {
                if app.ui.server_invite_selected < app.chat.servers.len().saturating_sub(1) {
                    app.ui.server_invite_selected += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(target_user_id) = app.ui.server_invite_target_user {
                    if let Some(server) = app.chat.servers.get(app.ui.server_invite_selected) {
                        app.send_to_server(ClientMessage::SendServerInvite {
                            to_user_id: target_user_id,
                            server_id: server.id,
                        });
                        
                        let username = app.chat.channel_userlist.iter()
                            .find(|u| u.id == target_user_id)
                            .map(|u| u.username.clone())
                            .unwrap_or_else(|| "User".to_string());
                        
                        app.set_notification(&format!("Sent server invite to {}!", username), Some(2000), false);
                    }
                }
                app.ui.show_server_invite_selection = false;
                app.ui.server_invite_target_user = None;
                app.ui.server_invite_selected = 0;
            }
            KeyCode::Esc => {
                app.ui.show_server_invite_selection = false;
                app.ui.server_invite_target_user = None;
                app.ui.server_invite_selected = 0;
            }
            _ => {}
        }
        return true;
    }

    false
}

fn handle_sidebar_input(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Tab => {
            if app.chat.show_user_list {
                app.chat.chat_focus = crate::state::ChatFocus::Messages;
            } else {
                app.chat.chat_focus = crate::state::ChatFocus::Messages;
            }
        }
        KeyCode::BackTab => {
            if app.chat.show_user_list {
                app.chat.chat_focus = crate::state::ChatFocus::Users;
            } else {
                app.chat.chat_focus = crate::state::ChatFocus::Messages;
            }
        }
        KeyCode::Char('u') if key.modifiers == KeyModifiers::CONTROL => {
            app.chat.show_user_list = !app.chat.show_user_list;
        }
        KeyCode::Left | KeyCode::Right => {
            // Switch between servers and DMs tabs
            app.chat.sidebar_tab = match app.chat.sidebar_tab {
                crate::state::SidebarTab::Servers => crate::state::SidebarTab::DMs,
                crate::state::SidebarTab::DMs => crate::state::SidebarTab::Servers,
            };
            app.sound_manager.play(SoundType::ChangeChannel);
            app.select_and_load_first_chat();
        }
        KeyCode::Down => {
            match app.chat.sidebar_tab {
                crate::state::SidebarTab::Servers => move_server_selection(app, 1),
                crate::state::SidebarTab::DMs => move_dm_selection(app, 1),
            }
            select_current_sidebar_target(app);
        }
        KeyCode::Up => {
            match app.chat.sidebar_tab {
                crate::state::SidebarTab::Servers => move_server_selection(app, -1),
                crate::state::SidebarTab::DMs => move_dm_selection(app, -1),
            }
            select_current_sidebar_target(app);
        }
        KeyCode::Enter => {
            app.chat.chat_focus = crate::state::ChatFocus::Messages;
        }
        KeyCode::Esc => {
            app.ui.set_mode(crate::state::AppMode::MainMenu);
        }
        _ => {}
    }
}

fn handle_message_input(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Tab => {
            if app.chat.show_user_list {
                app.chat.chat_focus = crate::state::ChatFocus::Users;
            } else {
                app.chat.chat_focus = crate::state::ChatFocus::Sidebar;
            }
        }
        KeyCode::BackTab => {
            app.chat.chat_focus = crate::state::ChatFocus::Sidebar;
        }
        KeyCode::PageUp => {
            app.sound_manager.play(SoundType::Scroll);
            handle_scroll_up(app);
        }
        KeyCode::PageDown => {
            app.sound_manager.play(SoundType::Scroll);
            handle_scroll_down(app);
        }
        KeyCode::Down => {
            // Handle mention suggestions first, then emoji suggestions
            if !app.chat.mention_suggestions.is_empty() {
                app.chat.mention_selected = (app.chat.mention_selected + 1) % app.chat.mention_suggestions.len();
            } else if !app.chat.emoji_suggestions.is_empty() {
                // Grid navigation: move down one row (add 4 to index)
                const GRID_COLS: usize = 4;
                const GRID_ROWS: usize = 5;
                const ITEMS_PER_PAGE: usize = GRID_COLS * GRID_ROWS;
                
                let current_row = (app.chat.emoji_selected % ITEMS_PER_PAGE) / GRID_COLS;
                let current_col = (app.chat.emoji_selected % ITEMS_PER_PAGE) % GRID_COLS;
                let current_page = app.chat.emoji_selected / ITEMS_PER_PAGE;
                
                if current_row < GRID_ROWS - 1 {
                    // Move down within current page
                    let new_index = current_page * ITEMS_PER_PAGE + (current_row + 1) * GRID_COLS + current_col;
                    if new_index < app.chat.emoji_suggestions.len() {
                        app.chat.emoji_selected = new_index;
                    }
                } else {
                    // Move to next page, first row
                    let next_page_start = (current_page + 1) * ITEMS_PER_PAGE;
                    if next_page_start < app.chat.emoji_suggestions.len() {
                        let new_index = next_page_start + current_col;
                        app.chat.emoji_selected = new_index.min(app.chat.emoji_suggestions.len() - 1);
                    }
                }
            } else if app.chat.chat_scroll_offset > 0 {
                app.chat.chat_scroll_offset -= 1;
            }
        }
        KeyCode::Up => {
            // Handle mention suggestions first, then emoji suggestions
            if !app.chat.mention_suggestions.is_empty() {
                if app.chat.mention_selected == 0 {
                    app.chat.mention_selected = app.chat.mention_suggestions.len() - 1;
                } else {
                    app.chat.mention_selected -= 1;
                }
            } else if !app.chat.emoji_suggestions.is_empty() {
                // Grid navigation: move up one row (subtract 4 from index)
                const GRID_COLS: usize = 4;
                const GRID_ROWS: usize = 5;
                const ITEMS_PER_PAGE: usize = GRID_COLS * GRID_ROWS;
                
                let current_row = (app.chat.emoji_selected % ITEMS_PER_PAGE) / GRID_COLS;
                let current_col = (app.chat.emoji_selected % ITEMS_PER_PAGE) % GRID_COLS;
                let current_page = app.chat.emoji_selected / ITEMS_PER_PAGE;
                
                if current_row > 0 {
                    // Move up within current page
                    let new_index = current_page * ITEMS_PER_PAGE + (current_row - 1) * GRID_COLS + current_col;
                    app.chat.emoji_selected = new_index;
                } else if current_page > 0 {
                    // Move to previous page, last row
                    let prev_page_start = (current_page - 1) * ITEMS_PER_PAGE;
                    let new_index = prev_page_start + (GRID_ROWS - 1) * GRID_COLS + current_col;
                    app.chat.emoji_selected = new_index.min(app.chat.emoji_suggestions.len() - 1);
                }
            } else {
                // Scroll up one line and fetch more messages if needed
                let max_rows = app.chat.last_chat_rows.unwrap_or(20);
                let total_msgs = app.get_current_message_list().len();
                let max_scroll = total_msgs.saturating_sub(max_rows);
                
                if app.chat.chat_scroll_offset < max_scroll {
                    app.chat.chat_scroll_offset += 1;
                    
                    // Check if we need to fetch more messages when scrolling up
                    if crate::services::ChatService::should_fetch_more_messages(&app.chat, max_rows) {
                        match &app.chat.current_chat_target {
                            Some(crate::state::ChatTarget::Channel { server_id: _, channel_id }) => {
                                if let Some(oldest_msg) = app.chat.chat_messages.first() {
                                    app.send_to_server(ClientMessage::GetChannelMessages {
                                        channel_id: *channel_id,
                                        before: Some(oldest_msg.timestamp),
                                    });
                                }
                            }
                            Some(crate::state::ChatTarget::DM { user_id }) => {
                                if let Some(oldest) = app.chat.dm_messages.first() {
                                    app.send_to_server(ClientMessage::GetDirectMessages {
                                        user_id: *user_id,
                                        before: Some(oldest.timestamp),
                                    });
                                }
                            }
                            None => {}
                        }
                    }
                }
            }
        }
        KeyCode::Left => {
            // Handle emoji grid navigation
            if !app.chat.emoji_suggestions.is_empty() {
                const GRID_COLS: usize = 4;
                const ITEMS_PER_PAGE: usize = GRID_COLS * 5;
                
                let current_col = (app.chat.emoji_selected % ITEMS_PER_PAGE) % GRID_COLS;
                let current_page = app.chat.emoji_selected / ITEMS_PER_PAGE;
                let current_row = (app.chat.emoji_selected % ITEMS_PER_PAGE) / GRID_COLS;
                
                if current_col > 0 {
                    // Move left within current row
                    app.chat.emoji_selected -= 1;
                } else {
                    // Wrap to rightmost column of same row
                    let new_index = current_page * ITEMS_PER_PAGE + current_row * GRID_COLS + (GRID_COLS - 1);
                    if new_index < app.chat.emoji_suggestions.len() {
                        app.chat.emoji_selected = new_index;
                    }
                }
            }
        }
        KeyCode::Right => {
            // Handle emoji grid navigation
            if !app.chat.emoji_suggestions.is_empty() {
                const GRID_COLS: usize = 4;
                const ITEMS_PER_PAGE: usize = GRID_COLS * 5;
                
                let current_col = (app.chat.emoji_selected % ITEMS_PER_PAGE) % GRID_COLS;
                let current_page = app.chat.emoji_selected / ITEMS_PER_PAGE;
                let current_row = (app.chat.emoji_selected % ITEMS_PER_PAGE) / GRID_COLS;
                
                if current_col < GRID_COLS - 1 {
                    // Move right within current row
                    let new_index = app.chat.emoji_selected + 1;
                    if new_index < app.chat.emoji_suggestions.len() {
                        app.chat.emoji_selected = new_index;
                    }
                } else {
                    // Wrap to leftmost column of same row
                    let new_index = current_page * ITEMS_PER_PAGE + current_row * GRID_COLS;
                    app.chat.emoji_selected = new_index;
                }
            }
        }
        KeyCode::Enter => {
            if !app.chat.mention_suggestions.is_empty() {
                app.apply_selected_mention();
            } else if !app.chat.emoji_suggestions.is_empty() {
                app.apply_selected_emoji();
            } else if let Err(e) = app.send_message() {
                app.set_notification(format!("Failed to send message: {}", e), Some(2000), false);
            }
        }
        KeyCode::Char('u') if key.modifiers == KeyModifiers::CONTROL => {
            app.chat.show_user_list = !app.chat.show_user_list;
        }
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                return;
            }
            let mut current = app.get_current_input().to_string();
            current.push(c);
            app.set_current_input(current);
            app.update_mention_suggestions();
            app.update_emoji_suggestions();
        }
        KeyCode::Backspace => {
            let mut current = app.get_current_input().to_string();
            current.pop();
            app.set_current_input(current);
            app.update_mention_suggestions();
            app.update_emoji_suggestions();
        }
        KeyCode::Esc => {
            app.ui.set_mode(crate::state::AppMode::MainMenu);
        }
        _ => {}
    }
}

fn handle_user_list_input(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Tab => {
            app.chat.chat_focus = crate::state::ChatFocus::Sidebar;
        }
        KeyCode::BackTab => {
            app.chat.chat_focus = crate::state::ChatFocus::Messages;
        }
        KeyCode::Char('u') if key.modifiers == KeyModifiers::CONTROL => {
            app.chat.show_user_list = !app.chat.show_user_list;
            app.chat.chat_focus = if app.chat.show_user_list {
                crate::state::ChatFocus::Users
            } else {
                crate::state::ChatFocus::Messages
            };
        }
        KeyCode::Down => {
            let len = app.chat.channel_userlist.len();
            if len > 0 {
                app.sound_manager.play(SoundType::Scroll);
                let sel = app.chat.user_list_state.selected().unwrap_or(0);
                app.chat.user_list_state.select(Some((sel + 1) % len));
            }
        }
        KeyCode::Up => {
            let len = app.chat.channel_userlist.len();
            if len > 0 {
                app.sound_manager.play(SoundType::Scroll);
                let sel = app.chat.user_list_state.selected().unwrap_or(0);
                app.chat.user_list_state.select(Some((sel + len - 1) % len));
            }
        }
        KeyCode::Enter => {
            if let Some(idx) = app.chat.user_list_state.selected() {
                app.sound_manager.play(SoundType::PopupOpen);
                app.profile.show_user_actions = true;
                app.profile.user_actions_selected = 0;
                app.profile.user_actions_target = Some(idx);
            }
        }
        KeyCode::Esc => {
            app.ui.set_mode(crate::state::AppMode::MainMenu);
        }
        _ => {}
    }
}

fn handle_dm_input(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Enter => {
            if let Some(target) = app.chat.dm_target {
                let msg = app.chat.dm_input.clone();
                if !msg.trim().is_empty() {
                    app.send_to_server(ClientMessage::SendDirectMessage { to: target, content: msg });
                    app.sound_manager.play(SoundType::MessageSent);
                }
            }
            app.chat.dm_input.clear();
            app.chat.chat_focus = crate::state::ChatFocus::Users;
        }
        KeyCode::Char(c) => {
            app.chat.dm_input.push(c);
        }
        KeyCode::Backspace => {
            app.chat.dm_input.pop();
        }
        KeyCode::Esc => {
            app.chat.dm_input.clear();
            app.chat.chat_focus = crate::state::ChatFocus::Users;
        }
        _ => {}
    }
}

// Helper functions
fn handle_user_action(app: &mut App) {
    if let Some(idx) = app.profile.user_actions_target {
        app.sound_manager.play(SoundType::PopupOpen);
        let user = app.chat.channel_userlist.get(idx);
        
        match app.profile.user_actions_selected {
            0 => { // View Profile
                if let Some(user) = user {
                    app.profile.profile_requested_by_user = true;
                    app.send_to_server(ClientMessage::GetProfile { user_id: user.id });
                }
            }
            1 => { // Send DM
                if let Some(user) = user {
                    app.chat.dm_target = Some(user.id);
                    app.chat.dm_input.clear();
                    app.chat.chat_focus = crate::state::ChatFocus::DMInput;
                }
            }
            2 => { // Invite to Server
                if let Some(user) = user {
                    app.ui.show_server_invite_selection = true;
                    app.ui.server_invite_selected = 0;
                    app.ui.server_invite_target_user = Some(user.id);
                }
            }
            _ => {}
        }
    }
    app.profile.show_user_actions = false;
}

fn handle_scroll_up(app: &mut App) {
    let max_rows = app.chat.last_chat_rows.unwrap_or(20);
    
    match &app.chat.current_chat_target {
        Some(crate::state::ChatTarget::Channel { server_id: _, channel_id }) => {
            let total_msgs = app.get_current_message_list().len();
            let max_scroll_offset = total_msgs.saturating_sub(max_rows);
            
            app.chat.chat_scroll_offset = (app.chat.chat_scroll_offset + max_rows).min(max_scroll_offset);
            
            // Fetch more messages if needed
            if crate::services::ChatService::should_fetch_more_messages(&app.chat, max_rows) {
                // Look for the oldest message in the actual chat messages, not channel.messages
                if let Some(oldest_msg) = app.chat.chat_messages.first() {
                    app.send_to_server(ClientMessage::GetChannelMessages {
                        channel_id: *channel_id,
                        before: Some(oldest_msg.timestamp),
                    });
                }
            }
        }
        Some(crate::state::ChatTarget::DM { user_id }) => {
            let total_msgs = app.get_current_message_list().len();
            let max_scroll_offset = total_msgs.saturating_sub(max_rows);
            
            app.chat.chat_scroll_offset = (app.chat.chat_scroll_offset + max_rows).min(max_scroll_offset);
            
            // Fetch more DM messages if needed
            if crate::services::ChatService::should_fetch_more_messages(&app.chat, max_rows) {
                if let Some(oldest) = app.chat.dm_messages.first() {
                    app.send_to_server(ClientMessage::GetDirectMessages {
                        user_id: *user_id,
                        before: Some(oldest.timestamp),
                    });
                }
            }
        }
        None => {}
    }
}

fn handle_scroll_down(app: &mut App) {
    let max_rows = app.chat.last_chat_rows.unwrap_or(20);
    
    if app.chat.chat_scroll_offset >= max_rows {
        app.chat.chat_scroll_offset -= max_rows;
    } else {
        app.chat.chat_scroll_offset = 0;
    }
}

fn move_server_selection(app: &mut App, direction: i32) {
    if app.chat.servers.is_empty() {
        return;
    }

    let current_server_idx = app.chat.selected_server.unwrap_or(0);
    let current_channel_idx = app.chat.selected_channel.unwrap_or(0);
    
    if let Some(current_server) = app.chat.servers.get(current_server_idx) {
        if current_server.channels.is_empty() {
            // No channels in this server, just move to next/prev server
            if direction == 1 {
                app.chat.selected_server = Some((current_server_idx + 1) % app.chat.servers.len());
            } else {
                app.chat.selected_server = Some((current_server_idx + app.chat.servers.len() - 1) % app.chat.servers.len());
            }
            app.chat.selected_channel = Some(0);
            return;
        }

        if direction == 1 {
            // Moving down
            if current_channel_idx < current_server.channels.len() - 1 {
                // Move to next channel in same server
                app.chat.selected_channel = Some(current_channel_idx + 1);
            } else {
                // At last channel, move to next server's first channel
                let next_server_idx = (current_server_idx + 1) % app.chat.servers.len();
                app.chat.selected_server = Some(next_server_idx);
                app.chat.selected_channel = Some(0);
            }
        } else {
            // Moving up (direction == -1)
            if current_channel_idx > 0 {
                // Move to previous channel in same server
                app.chat.selected_channel = Some(current_channel_idx - 1);
            } else {
                // At first channel, move to previous server's last channel
                let prev_server_idx = (current_server_idx + app.chat.servers.len() - 1) % app.chat.servers.len();
                app.chat.selected_server = Some(prev_server_idx);
                
                // Select the last channel of the previous server
                if let Some(prev_server) = app.chat.servers.get(prev_server_idx) {
                    if !prev_server.channels.is_empty() {
                        app.chat.selected_channel = Some(prev_server.channels.len() - 1);
                    } else {
                        app.chat.selected_channel = Some(0);
                    }
                } else {
                    app.chat.selected_channel = Some(0);
                }
            }
        }
    }
}

fn move_dm_selection(app: &mut App, direction: i32) {
    if app.chat.dm_user_list.is_empty() {
        return;
    }

    // Create the same sorted list as the UI to get the display order
    let mut indexed_users: Vec<(usize, &nexus_tui_common::User)> = app.chat.dm_user_list.iter().enumerate().collect();
    indexed_users.sort_by_key(|(_, u)| (!app.chat.unread_dm_conversations.contains(&u.id), u.username.clone()));
    
    // Find current display index
    let current_display_idx = if let Some(selected_original_idx) = app.chat.selected_dm_user {
        indexed_users.iter().position(|(original_idx, _)| *original_idx == selected_original_idx).unwrap_or(0)
    } else {
        0
    };
    
    // Calculate new display index
    let new_display_idx = if direction == 1 {
        (current_display_idx + 1) % indexed_users.len()
    } else {
        (current_display_idx + indexed_users.len() - 1) % indexed_users.len()
    };
    
    // Convert back to original index
    if let Some((original_idx, _)) = indexed_users.get(new_display_idx) {
        app.chat.selected_dm_user = Some(*original_idx);
    }
}

fn select_current_sidebar_target(app: &mut App) {
    match app.chat.sidebar_tab {
        crate::state::SidebarTab::Servers => {
            if let (Some(s), Some(c)) = (app.chat.selected_server, app.chat.selected_channel) {
                if let (Some(_server), Some(channel)) = (
                    app.chat.servers.get(s),
                    app.chat.servers.get(s).and_then(|srv| srv.channels.get(c))
                ) {
                    let channel_id = channel.id;
                    let server_id = channel.server_id;
                    let target = crate::state::ChatTarget::Channel { server_id, channel_id };
                    
                    // Clear old messages and set new target
                    app.chat.chat_messages.clear();
                    app.set_current_chat_target(target);
                    app.chat.reset_scroll_offset();
                    
                    // Request new data
                    app.send_to_server(ClientMessage::GetChannelMessages { channel_id, before: None });
                    app.send_to_server(ClientMessage::GetChannelUserList { channel_id });
                    
                    // Play sound feedback
                    app.sound_manager.play(SoundType::ChangeChannel);
                }
            }
        }
        crate::state::SidebarTab::DMs => {
            if let Some(idx) = app.chat.selected_dm_user {
                if let Some(user) = app.chat.dm_user_list.get(idx) {
                    let user_id = user.id;
                    let target = crate::state::ChatTarget::DM { user_id };
                    
                    // Clear old messages and set new target
                    app.chat.dm_messages.clear();
                    app.set_current_chat_target(target);
                    app.chat.reset_scroll_offset();
                    
                    // Request new data
                    app.send_to_server(ClientMessage::GetDirectMessages { user_id, before: None });
                    app.chat.unread_dm_conversations.remove(&user_id);
                    
                    // Play sound feedback
                    app.sound_manager.play(SoundType::ChangeChannel);
                }
            }
        }
    }
}