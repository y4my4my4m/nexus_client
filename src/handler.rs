// client/src/handler.rs

use crate::app::{App, AppMode, InputMode};
use crate::sound::SoundType;
use common::{ClientMessage, SerializableColor};
use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::Color;

pub fn handle_key_event(key: KeyEvent, app: &mut crate::app::App) {
    if key.kind != event::KeyEventKind::Press { return; }

    // app.sound_manager.play(SoundType::Click);

    if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
        app.should_quit = true; return;
    }
    
    if app.notification.is_some() {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => app.notification = None,
            _ => {}
        }
        return;
    }

    // Close profile view popup on any key, and do nothing else
    if app.show_profile_view_popup {
        app.show_profile_view_popup = false;
        app.profile_view = None;
        return;
    }

    match app.mode {
        AppMode::Login | AppMode::Register => handle_auth_mode(key, app),
        AppMode::Input => handle_input_mode(key, app),
        _ => handle_main_app_mode(key, app),
    }
}

fn handle_auth_mode(key: KeyEvent, app: &mut App) {
    let is_login = app.mode == AppMode::Login;
    
    match key.code {
        KeyCode::Char(c) => {
            if let Some(im) = &app.input_mode {
                match im {
                    InputMode::LoginUsername | InputMode::RegisterUsername => app.current_input.push(c),
                    InputMode::LoginPassword | InputMode::RegisterPassword => app.password_input.push(c),
                    _ => {}
                }
            }
        },
        KeyCode::Backspace => {
            if let Some(im) = &app.input_mode {
                match im {
                    InputMode::LoginUsername | InputMode::RegisterUsername => { app.current_input.pop(); },
                    InputMode::LoginPassword | InputMode::RegisterPassword => { app.password_input.pop(); },
                    _ => {}
                }
            }
        },
        KeyCode::Tab => {
            let focus_order = if is_login {
                [InputMode::LoginUsername, InputMode::LoginPassword, InputMode::AuthSubmit, InputMode::AuthSwitch]
            } else {
                [InputMode::RegisterUsername, InputMode::RegisterPassword, InputMode::AuthSubmit, InputMode::AuthSwitch]
            };
            let current_idx = focus_order.iter().position(|m| Some(m) == app.input_mode.as_ref()).unwrap_or(0);
            let next_idx = (current_idx + 1) % focus_order.len();
            app.input_mode = Some(focus_order[next_idx].clone());
        },
        KeyCode::BackTab => {
            let focus_order = if is_login {
                [InputMode::LoginUsername, InputMode::LoginPassword, InputMode::AuthSubmit, InputMode::AuthSwitch]
            } else {
                [InputMode::RegisterUsername, InputMode::RegisterPassword, InputMode::AuthSubmit, InputMode::AuthSwitch]
            };
            let current_idx = focus_order.iter().position(|m| Some(m) == app.input_mode.as_ref()).unwrap_or(0);
            let next_idx = (current_idx + focus_order.len() - 1) % focus_order.len();
            app.input_mode = Some(focus_order[next_idx].clone());
        },
        KeyCode::Enter => {
            match &app.input_mode {
                Some(InputMode::LoginUsername) => app.input_mode = Some(InputMode::LoginPassword),
                Some(InputMode::LoginPassword) => app.input_mode = Some(InputMode::AuthSubmit),
                Some(InputMode::RegisterUsername) => app.input_mode = Some(InputMode::RegisterPassword),
                Some(InputMode::RegisterPassword) => app.input_mode = Some(InputMode::AuthSubmit),
                Some(InputMode::AuthSubmit) => {
                    let username = app.current_input.clone();
                    let password = app.password_input.clone();
                    if username.is_empty() || password.is_empty() {
                        app.set_notification("Fields cannot be empty.", None, false);
                        return;
                    }
                    if app.mode == AppMode::Login {
                        app.send_to_server(ClientMessage::Login { username, password });
                    } else {
                        app.send_to_server(ClientMessage::Register { username, password });
                    }
                    // Clear fields only after sending
                    app.current_input.clear();
                    app.password_input.clear();
                },
                Some(InputMode::AuthSwitch) => {
                    app.current_input.clear();
                    app.password_input.clear();
                    if app.mode == AppMode::Login {
                        app.mode = AppMode::Register;
                        app.input_mode = Some(InputMode::RegisterUsername);
                    } else {
                        app.mode = AppMode::Login;
                        app.input_mode = Some(InputMode::LoginUsername);
                    }
                },
                _ => {}
            }
        },
        KeyCode::Esc => app.should_quit = true,
        _ => {}
    }
}

fn handle_input_mode(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Enter => {
            let input = app.current_input.clone();
            let prev_input = app.password_input.clone();
            app.current_input.clear();
            app.password_input.clear();
            if let Some(im) = app.input_mode.take() {
                match im {
                    InputMode::NewThreadTitle => {
                        if input.trim().is_empty() {
                            app.set_notification("Thread title cannot be empty.", None, false);
                            app.input_mode = Some(InputMode::NewThreadTitle);
                            return;
                        }
                        app.enter_input_mode(InputMode::NewThreadContent);
                        app.password_input = input;
                    },
                    InputMode::NewThreadContent => {
                        let title = prev_input;
                        let content = input;
                        if title.trim().is_empty() || content.trim().is_empty() {
                            app.set_notification("Thread title and content cannot be empty.", None, false);
                            app.input_mode = Some(InputMode::NewThreadContent);
                            app.password_input = title;
                            return;
                        }
                        if let Some(forum_id) = app.current_forum_id {
                            app.send_to_server(ClientMessage::CreateThread{ forum_id, title: title.clone(), content: content.clone() });
                            app.set_notification("Thread submitted!", Some(1500), false);
                        }
                        app.mode = AppMode::ForumList;
                    },
                    InputMode::NewPostContent => {
                        if input.trim().is_empty() {
                            app.set_notification("Post content cannot be empty.", None, false);
                            app.input_mode = Some(InputMode::NewPostContent);
                            return;
                        }
                        if let Some(thread_id) = app.current_thread_id {
                            app.send_to_server(ClientMessage::CreatePost { thread_id, content: input.clone() });
                            app.set_notification("Reply submitted!", Some(1500), false);
                        }
                        app.mode = AppMode::PostView;
                    }
                    InputMode::UpdatePassword => {
                        app.send_to_server(ClientMessage::UpdatePassword(input));
                        app.mode = AppMode::Settings;
                    }
                    _ => { app.mode = AppMode::MainMenu; }
                }
            }
        }
        KeyCode::Char(c) => app.current_input.push(c),
        KeyCode::Backspace => { app.current_input.pop(); },
        KeyCode::Esc => { app.input_mode = None; app.mode = AppMode::MainMenu; }
        _ => {}
    }
}

fn handle_main_app_mode(key: KeyEvent, app: &mut App) {
    if key.code == KeyCode::Char('q') { app.should_quit = true; return; }
    match app.mode {
        AppMode::MainMenu => match key.code {
            KeyCode::Down => app.main_menu_state.select(Some(app.main_menu_state.selected().map_or(0, |s| (s + 1) % 4))),
            KeyCode::Up => app.main_menu_state.select(Some(app.main_menu_state.selected().map_or(3, |s| (s + 3) % 4))),
            KeyCode::Enter => if let Some(s) = app.main_menu_state.selected() {
                match s {
                    0 => { app.send_to_server(ClientMessage::GetForums); app.mode = AppMode::ForumList; app.forum_list_state.select(Some(0)); },
                    1 => { app.mode = AppMode::Chat; app.current_input.clear(); },
                    2 => { app.mode = AppMode::Settings; app.settings_list_state.select(Some(0)); },
                    3 => { app.send_to_server(ClientMessage::Logout); app.current_user = None; app.mode = AppMode::Login; app.input_mode = Some(InputMode::LoginUsername); app.current_input.clear(); app.password_input.clear(); },
                    _ => {}
                }
            },
            _ => {}
        },
        AppMode::ForumList => match key.code {
            KeyCode::Down => if !app.forums.is_empty() { app.forum_list_state.select(Some(app.forum_list_state.selected().map_or(0, |s| (s + 1) % app.forums.len()))) },
            KeyCode::Up => if !app.forums.is_empty() { app.forum_list_state.select(Some(app.forum_list_state.selected().map_or(app.forums.len() - 1, |s| (s + app.forums.len() - 1) % app.forums.len()))) },
            KeyCode::Enter => if let Some(idx) = app.forum_list_state.selected() {
                if let Some(forum) = app.forums.get(idx) {
                    app.current_forum_id = Some(forum.id);
                    app.thread_list_state.select(Some(0));
                    app.mode = AppMode::ThreadList;
                }
            },
            KeyCode::Char('n') | KeyCode::Char('N') => {
                if app.forum_list_state.selected().is_some() { app.enter_input_mode(InputMode::NewThreadTitle) } 
                else { app.set_notification("Select a forum first to create a thread in.", None, false); }
            },
            KeyCode::Esc => app.mode = AppMode::MainMenu,
            _ => {}
        },
        AppMode::ThreadList => match key.code {
            KeyCode::Down => if let Some(forum) = app.current_forum_id.and_then(|id| app.forums.iter().find(|f| f.id == id)) {
                if !forum.threads.is_empty() { app.thread_list_state.select(Some(app.thread_list_state.selected().map_or(0, |s| (s + 1) % forum.threads.len()))); }
            },
            KeyCode::Up => if let Some(forum) = app.current_forum_id.and_then(|id| app.forums.iter().find(|f| f.id == id)) {
                if !forum.threads.is_empty() { app.thread_list_state.select(Some(app.thread_list_state.selected().map_or(forum.threads.len() - 1, |s| (s + forum.threads.len() - 1) % forum.threads.len()))); }
            },
            KeyCode::Enter => if let Some(idx) = app.thread_list_state.selected() {
                if let Some(forum) = app.current_forum_id.and_then(|id| app.forums.iter().find(|f| f.id == id)) {
                    if let Some(thread) = forum.threads.get(idx) { app.current_thread_id = Some(thread.id); app.mode = AppMode::PostView; }
                }
            },
            KeyCode::Esc => app.mode = AppMode::ForumList,
             _ => {}
        },
        AppMode::PostView => match key.code {
            KeyCode::Char('r') | KeyCode::Char('R') => app.enter_input_mode(InputMode::NewPostContent),
            KeyCode::Esc => app.mode = AppMode::ThreadList,
            _ => {}
        },
        AppMode::Settings => match key.code {
            KeyCode::Down | KeyCode::Up => app.settings_list_state.select(Some(app.settings_list_state.selected().map_or(0, |s| (s + 1) % 3))),
            KeyCode::Enter => if let Some(s) = app.settings_list_state.selected() {
                match s {
                    0 => app.enter_input_mode(InputMode::UpdatePassword),
                    1 => cycle_color(app),
                    2 => {
                        // Fetch latest profile before entering edit mode
                        if let Some(user) = &app.current_user {
                            app.profile_requested_by_user = false; // Not a view, but for edit
                            app.send_to_server(common::ClientMessage::GetProfile { user_id: user.id });
                            // Set a flag to indicate we want to enter edit mode after profile arrives
                            app.profile_edit_focus = crate::app::ProfileEditFocus::Bio;
                            app.mode = AppMode::EditProfile; // Optionally, show a loading state
                        }
                    },
                    _ => {}
                }
            },
            KeyCode::Esc => app.mode = AppMode::MainMenu,
            _ => {}
        },
        AppMode::EditProfile => match key.code {
            KeyCode::Tab => {
                use crate::app::ProfileEditFocus::*;
                app.profile_edit_focus = match app.profile_edit_focus {
                    Bio => Url1,
                    Url1 => Url2,
                    Url2 => Url3,
                    Url3 => Location,
                    Location => ProfilePic,
                    ProfilePic => CoverBanner,
                    CoverBanner => Save,
                    Save => Cancel,
                    Cancel => Bio,
                };
            },
            KeyCode::BackTab => {
                use crate::app::ProfileEditFocus::*;
                app.profile_edit_focus = match app.profile_edit_focus {
                    Bio => Cancel,
                    Url1 => Bio,
                    Url2 => Url1,
                    Url3 => Url2,
                    Location => Url3,
                    ProfilePic => Location,
                    CoverBanner => ProfilePic,
                    Save => CoverBanner,
                    Cancel => Save,
                };
            },
            KeyCode::Enter => {
                use crate::app::ProfileEditFocus::*;
                match app.profile_edit_focus {
                    Save => {
                        // On save, process profile_pic and cover_banner
                        let profile_pic = App::file_or_url_to_base64(&app.edit_profile_pic);
                        let cover_banner = App::file_or_url_to_base64(&app.edit_cover_banner);
                        app.send_to_server(ClientMessage::UpdateProfile {
                            bio: Some(app.edit_bio.clone()),
                            url1: Some(app.edit_url1.clone()),
                            url2: Some(app.edit_url2.clone()),
                            url3: Some(app.edit_url3.clone()),
                            location: Some(app.edit_location.clone()),
                            profile_pic,
                            cover_banner,
                        });
                        app.mode = AppMode::Settings;
                    }
                    Cancel => {
                        app.mode = AppMode::Settings;
                    },
                    Bio => {
                        app.edit_bio.push('\n');
                    },
                    _ => {}
                }
            },
            KeyCode::Esc => {
                app.mode = AppMode::Settings;
            },
            KeyCode::Char(c) => {
                use crate::app::ProfileEditFocus::*;
                match app.profile_edit_focus {
                    Bio => app.edit_bio.push(c),
                    Url1 => app.edit_url1.push(c),
                    Url2 => app.edit_url2.push(c),
                    Url3 => app.edit_url3.push(c),
                    Location => app.edit_location.push(c),
                    ProfilePic => app.edit_profile_pic.push(c),
                    CoverBanner => app.edit_cover_banner.push(c),
                    _ => {}
                }
            },
            KeyCode::Backspace => {
                use crate::app::ProfileEditFocus::*;
                match app.profile_edit_focus {
                    Bio => { app.edit_bio.pop(); },
                    Url1 => { app.edit_url1.pop(); },
                    Url2 => { app.edit_url2.pop(); },
                    Url3 => { app.edit_url3.pop(); },
                    Location => { app.edit_location.pop(); },
                    ProfilePic => { app.edit_profile_pic.pop(); },
                    CoverBanner => { app.edit_cover_banner.pop(); },
                    _ => {}
                }
            },
            _ => {}
        },
        AppMode::Chat => {
            if app.show_user_actions {
                match key.code {
                    KeyCode::Up => {
                        if app.user_actions_selected > 0 {
                            app.user_actions_selected -= 1;
                        }
                    },
                    KeyCode::Down => {
                        if app.user_actions_selected < 1 {
                            app.user_actions_selected += 1;
                        }
                    },
                    KeyCode::Enter => {
                        if let Some(idx) = app.user_actions_target {
                            let user = app.connected_users.get(idx);
                            match app.user_actions_selected {
                                0 => { // View Profile
                                    if let Some(user) = user {
                                        app.profile_requested_by_user = true;
                                        app.send_to_server(ClientMessage::GetProfile { user_id: user.id });
                                    }
                                },
                                1 => { // Send DM
                                    if let Some(user) = user {
                                        app.dm_target = Some(user.id);
                                        app.dm_input.clear();
                                        app.chat_focus = crate::app::ChatFocus::DMInput;
                                    }
                                },
                                _ => {}
                            }
                        }
                        app.show_user_actions = false;
                    },
                    KeyCode::Esc => {
                        app.show_user_actions = false;
                    },
                    _ => {}
                }
                return;
            }
            match app.chat_focus {
                crate::app::ChatFocus::Messages => match key.code {
                    KeyCode::Tab => { 
                        if !app.mention_suggestions.is_empty() {
                            // Tab cycles through suggestions
                            app.mention_selected = (app.mention_selected + 1) % app.mention_suggestions.len();
                        } else {
                            app.chat_focus = crate::app::ChatFocus::Users;
                        }
                    },
                    KeyCode::Down => {
                        if !app.mention_suggestions.is_empty() {
                            app.mention_selected = (app.mention_selected + 1) % app.mention_suggestions.len();
                        }
                    },
                    KeyCode::Up => {
                        if !app.mention_suggestions.is_empty() {
                            if app.mention_selected == 0 {
                                app.mention_selected = app.mention_suggestions.len() - 1;
                            } else {
                                app.mention_selected -= 1;
                            }
                        }
                    },
                    KeyCode::Enter => {
                        if !app.mention_suggestions.is_empty() {
                            // Insert selected mention
                            if let Some(prefix) = &app.mention_prefix {
                                if let Some(suggestion) = app.mention_suggestions.get(app.mention_selected) {
                                    // Replace the last @mention prefix in input
                                    if let Some(idx) = app.current_input.rfind(&format!("@{}", prefix)) {
                                        &app.current_input[(idx + 1 + prefix.len())..];
                                        // Insert mention and a space after
                                        app.current_input.replace_range(idx..(idx + 1 + prefix.len()), &format!("@{} ", suggestion));
                                        app.mention_suggestions.clear();
                                        app.mention_prefix = None;
                                    }
                                }
                            }
                        } else if !app.current_input.is_empty() {
                            let message_content = app.current_input.drain(..).collect();
                            app.send_to_server(ClientMessage::SendChatMessage(message_content));
                        }
                    },
                    KeyCode::Char('u') if key.modifiers == KeyModifiers::CONTROL => {
                        app.show_user_list = !app.show_user_list;
                        app.chat_focus = if app.show_user_list {
                            crate::app::ChatFocus::Users
                        } else {
                            crate::app::ChatFocus::Messages
                        };
                    },
                    KeyCode::Char(c) => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            return;
                        }
                        app.current_input.push(c);
                        // Check for @mention context
                        let cursor = app.current_input.len();
                        let upto = &app.current_input[..cursor];
                        if let Some(idx) = upto.rfind('@') {
                            let after_at = &upto[(idx+1)..];
                            if after_at.chars().all(|ch| ch.is_alphanumeric() || ch == '_' ) && !after_at.is_empty() {
                                let prefix = after_at;
                                let mut suggestions: Vec<String> = app.connected_users.iter()
                                    .map(|u| u.username.clone())
                                    .filter(|u| u.to_lowercase().starts_with(&prefix.to_lowercase()))
                                    .collect();
                                suggestions.sort();
                                if !suggestions.is_empty() {
                                    app.mention_suggestions = suggestions;
                                    app.mention_selected = 0;
                                    app.mention_prefix = Some(prefix.to_string());
                                } else {
                                    app.mention_suggestions.clear();
                                    app.mention_prefix = None;
                                }
                            } else {
                                app.mention_suggestions.clear();
                                app.mention_prefix = None;
                            }
                        } else {
                            app.mention_suggestions.clear();
                            app.mention_prefix = None;
                        }
                    },
                    KeyCode::Backspace => {
                        app.current_input.pop();
                        // Recompute mention suggestions
                        let cursor = app.current_input.len();
                        let upto = &app.current_input[..cursor];
                        if let Some(idx) = upto.rfind('@') {
                            let after_at = &upto[(idx+1)..];
                            if after_at.chars().all(|ch| ch.is_alphanumeric() || ch == '_' ) && !after_at.is_empty() {
                                let prefix = after_at;
                                let mut suggestions: Vec<String> = app.connected_users.iter()
                                    .map(|u| u.username.clone())
                                    .filter(|u| u.to_lowercase().starts_with(&prefix.to_lowercase()))
                                    .collect();
                                suggestions.sort();
                                if !suggestions.is_empty() {
                                    app.mention_suggestions = suggestions;
                                    app.mention_selected = 0;
                                    app.mention_prefix = Some(prefix.to_string());
                                } else {
                                    app.mention_suggestions.clear();
                                    app.mention_prefix = None;
                                }
                            } else {
                                app.mention_suggestions.clear();
                                app.mention_prefix = None;
                            }
                        } else {
                            app.mention_suggestions.clear();
                            app.mention_prefix = None;
                        }
                    },
                    KeyCode::Esc => app.mode = AppMode::MainMenu,
                    _ => {}
                },
                crate::app::ChatFocus::Users => match key.code {
                    KeyCode::Tab => { app.chat_focus = crate::app::ChatFocus::Messages; },
                    KeyCode::Char('u') if key.modifiers == KeyModifiers::CONTROL => {
                        app.show_user_list = !app.show_user_list;
                        app.chat_focus = if app.show_user_list {
                            crate::app::ChatFocus::Users
                        } else {
                            crate::app::ChatFocus::Messages
                        };
                    },
                    KeyCode::Down => {
                        let len = app.connected_users.len();
                        if len > 0 {
                            let sel = app.forum_list_state.selected().unwrap_or(0);
                            app.forum_list_state.select(Some((sel + 1) % len));
                        }
                    },
                    KeyCode::Up => {
                        let len = app.connected_users.len();
                        if len > 0 {
                            let sel = app.forum_list_state.selected().unwrap_or(0);
                            app.forum_list_state.select(Some((sel + len - 1) % len));
                        }
                    },
                    KeyCode::Enter => {
                        if let Some(idx) = app.forum_list_state.selected() {
                            app.show_user_actions = true;
                            app.user_actions_selected = 0;
                            app.user_actions_target = Some(idx);
                        }
                    },
                    KeyCode::Esc => app.mode = AppMode::MainMenu,
                    _ => {}
                },
                crate::app::ChatFocus::DMInput => match key.code {
                    KeyCode::Enter => {
                        if let Some(target) = app.dm_target {
                            let msg = app.dm_input.clone();
                            if !msg.trim().is_empty() {
                                app.send_to_server(ClientMessage::SendDirectMessage { to: target, content: msg });
                                app.sound_manager.play(SoundType::MessageSent);
                            }
                        }
                        app.dm_input.clear();
                        app.chat_focus = crate::app::ChatFocus::Users;
                    },
                    KeyCode::Char(c) => app.dm_input.push(c),
                    KeyCode::Backspace => { app.dm_input.pop(); },
                    KeyCode::Esc => {
                        app.dm_input.clear();
                        app.chat_focus = crate::app::ChatFocus::Users;
                    },
                    _ => {}
                },
            }
        },
        _ => {}
    }

    // Global: close profile view popup
}

fn cycle_color(app: &mut App) {
    if let Some(user) = &mut app.current_user {
        let colors = [Color::Cyan, Color::Green, Color::Yellow, Color::Red, Color::Magenta, Color::Blue, Color::White];
        let current_color_idx = colors.iter().position(|&c| c == user.color).unwrap_or(0);
        let next_color_idx = (current_color_idx + 1) % colors.len();
        let new_color = colors[next_color_idx];
        user.color = new_color;
        app.send_to_server(ClientMessage::UpdateColor(SerializableColor(new_color)));
    }
}
