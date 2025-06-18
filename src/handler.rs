// client/src/handler.rs

use crate::app::{App, AppMode, InputMode};
use crate::sound::SoundType;
use crate::global_prefs::global_prefs_mut;
use common::{ClientMessage, SerializableColor};
use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::Color;

pub fn handle_key_event(key: KeyEvent, app: &mut crate::app::App) {
    if key.kind != event::KeyEventKind::Press { return; }

    // --- Allow F2 to open preferences from anywhere ---
    if key.code == KeyCode::F(2) {
        app.mode = AppMode::Parameters;
        return;
    }

    if app.show_quit_confirm {
        match key.code {
            KeyCode::Left | KeyCode::Tab => {
                app.quit_confirm_selected = (app.quit_confirm_selected + 1) % 2;
                app.sound_manager.play(SoundType::Scroll);
            },
            KeyCode::Right => {
                app.quit_confirm_selected = (app.quit_confirm_selected + 1) % 2;
                app.sound_manager.play(SoundType::Scroll);
            },
            KeyCode::Enter => {
                if app.quit_confirm_selected == 0 {
                    app.sound_manager.play(SoundType::Select);
                    app.should_quit = true;
                }
                app.sound_manager.play(SoundType::PopupClose);
                app.show_quit_confirm = false;
            },
            KeyCode::Esc => {
                app.show_quit_confirm = false;
                app.sound_manager.play(SoundType::PopupClose);
            },
            _ => {}
        }
        return;
    }

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
        AppMode::Login | AppMode::Register => {
            // REMOVE F2 HANDLING HERE, now global
            handle_auth_mode(key, app)
        },
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
                        app.sound_manager.play(SoundType::LoginFailure);
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
                        app.sound_manager.play(SoundType::PopupOpen);
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
                        app.sound_manager.play(SoundType::PopupOpen);
                        if title.trim().is_empty() || content.trim().is_empty() {
                            app.set_notification("Thread title and content cannot be empty.", None, false);
                            app.input_mode = Some(InputMode::NewThreadContent);
                            app.password_input = title;
                            return;
                        }
                        if let Some(forum_id) = app.current_forum_id {
                            app.pending_new_thread_title = Some(title.clone());
                            app.send_to_server(ClientMessage::CreateThread{ forum_id, title: title.clone(), content: content.clone() });
                            app.set_notification("Thread submitted!", Some(1500), false);
                        }
                        app.mode = AppMode::ThreadList;
                    },
                    InputMode::NewPostContent => {
                        app.sound_manager.play(SoundType::PopupOpen);
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
                        app.sound_manager.play(SoundType::PopupOpen);
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
    // Only allow quit popup if not in any input/edit mode or DM input
    let in_input = matches!(app.mode,
        AppMode::Input | AppMode::EditProfile | AppMode::Chat
    );
    if key.code == KeyCode::Char('q') && !in_input {
        app.sound_manager.play(SoundType::PopupOpen);
        app.show_quit_confirm = true;
        app.quit_confirm_selected = 1; // Default to No
        return;
    }
    match app.mode {
        AppMode::Settings => match key.code {
            KeyCode::Down => { 
                app.sound_manager.play(SoundType::Scroll); 
                let max = if app.current_user.is_some() { 4 } else { 3 };
                let cur = app.settings_list_state.selected().unwrap_or(0);
                app.settings_list_state.select(Some((cur + 1) % max));
            }
            KeyCode::Up => { 
                app.sound_manager.play(SoundType::Scroll); 
                let max = if app.current_user.is_some() { 4 } else { 3 };
                let cur = app.settings_list_state.selected().unwrap_or(0);
                app.settings_list_state.select(Some((cur + max - 1) % max));
            }
            KeyCode::Enter => if let Some(s) = app.settings_list_state.selected() {
                match s {
                    0 => app.enter_input_mode(InputMode::UpdatePassword),
                    1 => {
                        // Enter color picker mode
                        // Set initial selection to current user color if possible
                        if let Some(user) = &app.current_user {
                            let palette = [
                                Color::Cyan, Color::Green, Color::Yellow, Color::Red, Color::Magenta, Color::Blue, Color::White, Color::LightCyan, Color::LightGreen, Color::LightYellow, Color::LightRed, Color::LightMagenta, Color::LightBlue, Color::Gray, Color::DarkGray, Color::Black
                            ];
                            app.color_picker_selected = palette.iter().position(|&c| c == user.color).unwrap_or(0);
                        } else {
                            app.color_picker_selected = 0;
                        }
                        app.mode = AppMode::ColorPicker;
                    },
                    2 => {
                        if let Some(user) = &app.current_user {
                            app.profile_requested_by_user = false;
                            app.send_to_server(common::ClientMessage::GetProfile { user_id: user.id });
                            app.profile_edit_focus = crate::app::ProfileEditFocus::Bio;
                            app.mode = AppMode::EditProfile;
                        }
                    },
                    3 => {
                        app.mode = AppMode::Parameters;
                    },
                    _ => {}
                }
            },
            KeyCode::Esc => app.mode = AppMode::MainMenu,
            KeyCode::Char('p') => {
                app.mode = AppMode::Parameters;
            },
            _ => {}
        },
        AppMode::ColorPicker => match key.code {
            KeyCode::Left => {
                let palette_len = 16;
                if app.color_picker_selected == 0 {
                    app.color_picker_selected = palette_len - 1;
                } else {
                    app.color_picker_selected -= 1;
                }
            },
            KeyCode::Right => {
                let palette_len = 16;
                app.color_picker_selected = (app.color_picker_selected + 1) % palette_len;
            },
            KeyCode::Enter => {
                let palette = [
                    Color::Cyan, Color::Green, Color::Yellow, Color::Red, Color::Magenta, Color::Blue, Color::White, Color::LightCyan, Color::LightGreen, Color::LightYellow, Color::LightRed, Color::LightMagenta, Color::LightBlue, Color::Gray, Color::DarkGray, Color::Black
                ];
                let new_color = palette[app.color_picker_selected];
                if let Some(user) = &mut app.current_user {
                    user.color = new_color;
                    app.send_to_server(ClientMessage::UpdateColor(SerializableColor(new_color)));
                }
                app.mode = AppMode::Settings;
            },
            KeyCode::Esc => {
                app.mode = AppMode::Settings;
            },
            _ => {}
        },
        AppMode::MainMenu => match key.code {
            KeyCode::Down => {
                app.sound_manager.play(SoundType::Scroll);
                app.main_menu_state.select(Some(app.main_menu_state.selected().map_or(0, |s| (s + 1) % 4)));
            }
            KeyCode::Up => {
                app.sound_manager.play(SoundType::Scroll);
                app.main_menu_state.select(Some(app.main_menu_state.selected().map_or(3, |s| (s + 3) % 4)));
            },
            KeyCode::Enter => if let Some(s) = app.main_menu_state.selected() {
                match s {
                    0 => { app.send_to_server(ClientMessage::GetForums); app.mode = AppMode::ForumList; app.forum_list_state.select(Some(0)); },
                    1 => {
                        app.mode = AppMode::Chat;
                        app.current_input.clear();
                        app.send_to_server(ClientMessage::GetServers); // Ensure servers are requested after login
                    },
                    2 => { app.mode = AppMode::Settings; app.settings_list_state.select(Some(0)); },
                    3 => { app.send_to_server(ClientMessage::Logout); app.current_user = None; app.mode = AppMode::Login; app.input_mode = Some(InputMode::LoginUsername); app.current_input.clear(); app.password_input.clear(); },
                    _ => {}
                }
            },
            _ => {}
        },
        AppMode::ForumList => match key.code {
            KeyCode::Down => if !app.forums.is_empty() { 
                app.sound_manager.play(SoundType::Scroll);
                app.forum_list_state.select(Some(app.forum_list_state.selected().map_or(0, |s| (s + 1) % app.forums.len())));
            },
            KeyCode::Up => if !app.forums.is_empty() { 
                app.sound_manager.play(SoundType::Scroll);
                app.forum_list_state.select(Some(app.forum_list_state.selected().map_or(app.forums.len() - 1, |s| (s + app.forums.len() - 1) % app.forums.len())));
            },
            KeyCode::Enter => if let Some(idx) = app.forum_list_state.selected() {
                if let Some(forum) = app.forums.get(idx) {
                    app.current_forum_id = Some(forum.id);
                    app.thread_list_state.select(Some(0));
                    app.mode = AppMode::ThreadList;
                }
            },
            KeyCode::Esc => app.mode = AppMode::MainMenu,
            _ => {}
        },
        AppMode::ThreadList => match key.code {
            KeyCode::Down => if let Some(forum) = app.current_forum_id.and_then(|id| app.forums.iter().find(|f| f.id == id)) {
                app.sound_manager.play(SoundType::Scroll);
                if !forum.threads.is_empty() { app.thread_list_state.select(Some(app.thread_list_state.selected().map_or(0, |s| (s + 1) % forum.threads.len()))); }
            },
            KeyCode::Up => if let Some(forum) = app.current_forum_id.and_then(|id| app.forums.iter().find(|f| f.id == id)) {
                app.sound_manager.play(SoundType::Scroll);
                if !forum.threads.is_empty() { app.thread_list_state.select(Some(app.thread_list_state.selected().map_or(forum.threads.len() - 1, |s| (s + forum.threads.len() - 1) % forum.threads.len()))); }
            },
            KeyCode::Enter => if let Some(idx) = app.thread_list_state.selected() {
                if let Some(forum) = app.current_forum_id.and_then(|id| app.forums.iter().find(|f| f.id == id)) {
                    if let Some(thread) = forum.threads.get(idx) { app.current_thread_id = Some(thread.id); app.mode = AppMode::PostView; }
                }
            },
            KeyCode::Char('n') | KeyCode::Char('N') => {
                app.enter_input_mode(InputMode::NewThreadTitle);
            },
            KeyCode::Esc => app.mode = AppMode::ForumList,
             _ => {}
        },
        AppMode::PostView => match key.code {
            KeyCode::Char('r') | KeyCode::Char('R') => app.enter_input_mode(InputMode::NewPostContent),
            KeyCode::Esc => app.mode = AppMode::ThreadList,
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
                    ProfilePic => ProfilePicDelete,
                    ProfilePicDelete => CoverBanner,
                    CoverBanner => CoverBannerDelete,
                    CoverBannerDelete => Save,
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
                    ProfilePicDelete => ProfilePic,
                    CoverBanner => ProfilePicDelete,
                    CoverBannerDelete => CoverBanner,
                    Save => CoverBannerDelete,
                    Cancel => Save,
                };
            },
            KeyCode::Enter => {
                use crate::app::ProfileEditFocus::*;
                match app.profile_edit_focus {
                    Save => {
                        // On save, process profile_pic and cover_banner
                        app.sound_manager.play(SoundType::Save);
                        let profile_pic = Some(App::file_or_url_to_base64(&app.edit_profile_pic).unwrap_or_default());
                        let cover_banner = Some(App::file_or_url_to_base64(&app.edit_cover_banner).unwrap_or_default());
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
                    }
                    Bio => {
                        app.edit_bio.push('\n');
                    }
                    ProfilePicDelete => {
                        app.edit_profile_pic = String::new();
                        app.profile_edit_focus = ProfilePic;
                    }
                    CoverBannerDelete => {
                        app.edit_cover_banner = String::new();
                        app.profile_edit_focus = CoverBanner;
                    }
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
                        app.sound_manager.play(SoundType::Scroll);
                        if app.user_actions_selected > 0 {
                            app.user_actions_selected -= 1;
                        }
                    },
                    KeyCode::Down => {
                        app.sound_manager.play(SoundType::Scroll);
                        if app.user_actions_selected < 1 {
                            app.user_actions_selected += 1;
                        }
                    },
                    KeyCode::Enter => {
                        if let Some(idx) = app.user_actions_target {
                            app.sound_manager.play(SoundType::PopupOpen);
                            let user = app.channel_userlist.get(idx);
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
            if app.show_server_actions {
                match key.code {
                    KeyCode::Up => {
                        if app.server_actions_selected > 0 {
                            app.server_actions_selected -= 1;
                            app.sound_manager.play(SoundType::Scroll);
                        }
                    },
                    KeyCode::Down => {
                        let max = if let Some(s) = app.selected_server {
                            let is_owner = app.current_user.as_ref().map(|u| u.id) == app.servers.get(s).map(|srv| srv.owner);
                            if is_owner { 3 } else { 2 }
                        } else { 2 };
                        if app.server_actions_selected + 1 < max {
                            app.server_actions_selected += 1;
                            app.sound_manager.play(SoundType::Scroll);
                        }
                    },
                    KeyCode::Enter => {
                        // TODO: Implement server action logic here
                        app.show_server_actions = false;
                    },
                    KeyCode::Esc => {
                        app.show_server_actions = false;
                    },
                    _ => {}
                }
                return;
            }
            match app.chat_focus {
                crate::app::ChatFocus::Sidebar => match key.code {
                    KeyCode::Tab => {
                        if app.show_user_list {
                            app.chat_focus = crate::app::ChatFocus::Messages;
                            if !app.connected_users.is_empty() && app.forum_list_state.selected().is_none() {
                                app.forum_list_state.select(Some(0));
                            }
                        } else {
                            app.chat_focus = crate::app::ChatFocus::Messages;
                        }
                    },
                    KeyCode::BackTab => {
                        if app.show_user_list {
                            app.chat_focus = crate::app::ChatFocus::Users;
                            if !app.connected_users.is_empty() && app.forum_list_state.selected().is_none() {
                                app.forum_list_state.select(Some(0));
                            }
                        } else {
                            app.chat_focus = crate::app::ChatFocus::Messages;
                        }
                    },
                    KeyCode::Char('u') if key.modifiers == KeyModifiers::CONTROL => {
                        app.show_user_list = !app.show_user_list;
                    },
                    KeyCode::Down => {
                        move_sidebar_selection(app, 1);
                    },
                    KeyCode::Up => {
                        move_sidebar_selection(app, -1);
                    },
                    KeyCode::Enter => {
                        if app.selected_server.is_some() && app.selected_channel.is_none() {
                            app.show_server_actions = true;
                            app.server_actions_selected = 0;
                            app.sound_manager.play(SoundType::PopupOpen);
                        } else {
                            app.chat_focus = crate::app::ChatFocus::Messages;
                        }
                    },
                    KeyCode::Esc => app.mode = AppMode::MainMenu,
                    _ => {}
                },
                crate::app::ChatFocus::Messages => match key.code {
                    KeyCode::Tab => {
                        if app.show_user_list {
                            app.chat_focus = crate::app::ChatFocus::Users;
                            if !app.connected_users.is_empty() {
                                app.forum_list_state.select(Some(0));
                            }
                        } else {
                            app.chat_focus = crate::app::ChatFocus::Sidebar;
                        }
                    },
                    KeyCode::BackTab => {
                        app.chat_focus = crate::app::ChatFocus::Sidebar;
                    },
                    KeyCode::Char('u') if key.modifiers == KeyModifiers::CONTROL => {
                        app.show_user_list = !app.show_user_list;
                    },
                    KeyCode::PageUp => {
                        app.sound_manager.play(SoundType::Scroll);
                        let (max_rows, total_msgs, channel_id, history_complete, oldest_msg_id) = if let (Some(s), Some(c)) = (app.selected_server, app.selected_channel) {
                            if let Some(server) = app.servers.get(s) {
                                if let Some(channel) = server.channels.get(c) {
                                    let max_rows = app.last_chat_rows.unwrap_or(20);
                                    let channel_id = channel.id;
                                    let history_complete = *app.channel_history_complete.get(&channel_id).unwrap_or(&false);
                                    let oldest_msg_id = channel.messages.first().map(|m| m.id);
                                    (max_rows, channel.messages.len(), Some(channel_id), history_complete, oldest_msg_id)
                                } else { (20, 0, None, false, None) }
                            } else { (20, 0, None, false, None) }
                        } else {
                            let max_rows = app.last_chat_rows.unwrap_or(20);
                            (max_rows, app.chat_messages.len(), None, false, None)
                        };
                        let max_scroll_offset = total_msgs.saturating_sub(max_rows);
                        // Clamp scroll offset
                        app.chat_scroll_offset = (app.chat_scroll_offset + max_rows).min(max_scroll_offset);
                        // If we've scrolled to the top and history isn't complete, fetch more
                        if app.chat_scroll_offset == max_scroll_offset && !history_complete {
                            if let (Some(channel_id), Some(before_id)) = (channel_id, oldest_msg_id) {
                                app.send_to_server(ClientMessage::GetChannelMessages { channel_id, before: Some(before_id) });
                            }
                        }
                    },
                    KeyCode::PageDown => {
                        let max_rows = app.last_chat_rows.unwrap_or(20);
                        // Clamp scroll offset
                        if app.chat_scroll_offset >= max_rows {
                            app.chat_scroll_offset -= max_rows;
                        } else {
                            app.chat_scroll_offset = 0;
                        }
                        app.sound_manager.play(SoundType::Scroll);
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
                                if let Some(&user_idx) = app.mention_suggestions.get(app.mention_selected) {
                                    let suggestion = &app.channel_userlist[user_idx].username;
                                    if let Some(idx) = app.current_input.rfind(&format!("@{prefix}")) {
                                        app.current_input.replace_range(idx..(idx + 1 + prefix.len()), &format!("@{} ", suggestion));
                                        app.mention_suggestions.clear();
                                        app.mention_prefix = None;
                                    }
                                }
                            }
                        } else if !app.current_input.is_empty() {
                            if let (Some(s), Some(c)) = (app.selected_server, app.selected_channel) {
                                if let Some(server) = app.servers.get(s) {
                                    if let Some(channel) = server.channels.get(c) {
                                        let content = app.current_input.drain(..).collect();
                                        app.send_to_server(ClientMessage::SendChannelMessage {
                                            channel_id: channel.id,
                                            content,
                                        });
                                        app.sound_manager.play(SoundType::SendChannelMessage);
                                    }
                                }
                            }
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
                            let mut suggestions: Vec<usize> = app.channel_userlist.iter().enumerate()
                                .filter(|(_, u)| u.username.to_lowercase().starts_with(&prefix.to_lowercase()))
                                .map(|(i, _)| i)
                                .collect();
                            suggestions.sort_by_key(|&i| app.channel_userlist[i].username.to_lowercase());
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
                            let mut suggestions: Vec<usize> = app.channel_userlist.iter().enumerate()
                                .filter(|(_, u)| u.username.to_lowercase().starts_with(&prefix.to_lowercase()))
                                .map(|(i, _)| i)
                                .collect();
                            suggestions.sort_by_key(|&i| app.channel_userlist[i].username.to_lowercase());
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
                    KeyCode::Tab => {
                        app.chat_focus = crate::app::ChatFocus::Sidebar;
                    },
                    KeyCode::BackTab => {
                        app.chat_focus = crate::app::ChatFocus::Messages;
                    },
                    KeyCode::Char('u') if key.modifiers == KeyModifiers::CONTROL => {
                        app.show_user_list = !app.show_user_list;
                        app.chat_focus = if app.show_user_list {
                            crate::app::ChatFocus::Users
                        } else {
                            crate::app::ChatFocus::Messages
                        };
                    },
                    KeyCode::Down => {
                        let len = app.channel_userlist.len();
                        if len > 0 {
                            app.sound_manager.play(SoundType::Scroll);
                            let sel = app.user_list_state.selected().unwrap_or(0);
                            app.user_list_state.select(Some((sel + 1) % len));
                        }
                    },
                    KeyCode::Up => {
                        let len = app.channel_userlist.len();
                        if len > 0 {
                            app.sound_manager.play(SoundType::Scroll);
                            let sel = app.user_list_state.selected().unwrap_or(0);
                            app.user_list_state.select(Some((sel + len - 1) % len));
                        }
                    },
                    KeyCode::Enter => {
                        if let Some(idx) = app.user_list_state.selected() {
                            app.sound_manager.play(SoundType::PopupOpen);
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
            match key.code {
                KeyCode::F(5) => {
                    app.show_server_actions = true;
                    app.sound_manager.play(SoundType::PopupOpen);
                },
                KeyCode::F(6) => {
                    app.set_notification("Called", Some(500), true);
                    if let Some(current_user) = app.current_user.as_ref() {
                        app.send_to_server(ClientMessage::GetDirectMessages { user_id: current_user.id, before: None });
                    }
                    // if app.show_dm_list && app.selected_dm_user.is_some() {
                    // if let Some(dm_idx) = app.selected_dm_user {
                    //     if let Some(user) = app.dm_user_list.get(dm_idx) {
                    //         app.dm_target = Some(user.id);
                    //         app.send_to_server(ClientMessage::GetDirectMessages { user_id: user.id, before: None });
                    //         app.chat_focus = crate::app::ChatFocus::Messages;
                    //     }
                    // }
                },
                _ => {}
            }
        },
        AppMode::Parameters => match key.code {
            KeyCode::Char(' ') | KeyCode::Enter => {
                let mut prefs = global_prefs_mut();
                prefs.sound_effects_enabled = !prefs.sound_effects_enabled;
                prefs.save();
            },
            KeyCode::Esc => {
                // Return to previous menu
                if app.current_user.is_some() {
                    app.mode = AppMode::Settings;
                } else {
                    app.mode = if app.input_mode == Some(InputMode::RegisterUsername) || app.input_mode == Some(InputMode::RegisterPassword) { AppMode::Register } else { AppMode::Login };
                }
            },
            _ => {}
        },
        _ => {}
    }

}

    // If DM list is open and a DM user is selected, move within DM users
    // if app.show_dm_list && app.selected_dm_user.is_some() {
    //     let len = app.dm_user_list.len();
    //     if len > 0 {
    //         let idx = app.selected_dm_user.unwrap();
    //         let new_idx = if direction == 1 {
    //             (idx + 1) % len
    //         } else {
    //             (idx + len - 1) % len
    //         };
    //         app.selected_dm_user = Some(new_idx);
    //     }
    // } else {

fn move_sidebar_selection(app: &mut App, direction: i32) {
    // direction: 1 for down, -1 for up
    if let Some(s) = app.selected_server {
        if let Some(server) = app.servers.get(s) {
            if let Some(c) = app.selected_channel {
                let next_c = if direction == 1 {
                    if c + 1 < server.channels.len() { Some(c + 1) } else { None }
                } else {
                    if c > 0 { Some(c - 1) } else { None }
                };
                if let Some(new_c) = next_c {
                    app.selected_channel = Some(new_c);
                } else if direction == 1 && s + 1 < app.servers.len() {
                    app.selected_server = Some(s + 1);
                    app.selected_channel = None;
                } else if direction == -1 && s > 0 {
                    app.selected_server = Some(s - 1);
                    if let Some(prev_server) = app.servers.get(s - 1) {
                        if !prev_server.channels.is_empty() {
                            app.selected_channel = Some(prev_server.channels.len() - 1);
                        }
                    }
                } else if direction == 1 {
                    // do nothing, end of list
                }
            } else if direction == 1 && !server.channels.is_empty() {
                app.selected_channel = Some(0);
            } else if direction == 1 && s + 1 < app.servers.len() {
                app.selected_server = Some(s + 1);
                app.selected_channel = None;
            } else if direction == -1 && s > 0 {
                app.selected_server = Some(s - 1);
                if let Some(prev_server) = app.servers.get(s - 1) {
                    if !prev_server.channels.is_empty() {
                        app.selected_channel = Some(prev_server.channels.len() - 1);
                    }
                }
            }
        }
    } else if direction == 1 && !app.servers.is_empty() {
        app.selected_server = Some(0);
        app.selected_channel = None;
    }
    // If a channel is selected, request latest messages for that channel
    if let (Some(s), Some(c)) = (app.selected_server, app.selected_channel) {
        app.chat_scroll_offset = 0; // Reset scroll when switching channels
        if let Some(server) = app.servers.get(s) {
            if let Some(channel) = server.channels.get(c) {
                let channel_id = channel.id;
                app.send_to_server(ClientMessage::GetChannelMessages { channel_id, before: None });
                app.send_to_server(ClientMessage::GetChannelUserList { channel_id });
            }
        }
    } else if app.selected_server.is_some() {
        // If only server is selected, select first channel if exists
        if let Some(s) = app.selected_server {
            if let Some(server) = app.servers.get(s) {
                if !server.channels.is_empty() {
                    app.selected_channel = Some(0);
                    let channel_id = server.channels[0].id;
                    app.send_to_server(ClientMessage::GetChannelMessages { channel_id, before: None });
                    app.send_to_server(ClientMessage::GetChannelUserList { channel_id });
                    app.user_list_state.select(Some(0)); // Reset user list selection
                }
            }
        }
    }
    // play sound
    app.sound_manager.play(SoundType::ChangeChannel);
}
