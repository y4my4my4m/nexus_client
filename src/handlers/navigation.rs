use crate::app::App;
use crate::sound::SoundType;
use crate::global_prefs::global_prefs_mut;
use crate::desktop_notifications::DesktopNotificationService;
use common::{ClientMessage, UserColor};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::Color;

/// Handle global shortcuts that work across all modes
pub fn handle_global_shortcuts(key: KeyEvent, app: &mut App) -> bool {
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Show quit confirmation dialog
            app.ui.show_quit_confirm = true;
            app.ui.quit_confirm_selected = 1; // Default to "No"
            app.sound_manager.play(SoundType::PopupOpen);
            return true;
        }
        KeyCode::F(2) => {
            // open the preferences menu
            app.ui.set_mode(crate::state::AppMode::Preferences);
            app.sound_manager.play(SoundType::PopupOpen);
            return true;
        }
        KeyCode::F(5) => {
            if app.ui.mode == crate::state::AppMode::Chat {
                app.ui.show_server_actions = true;
                app.sound_manager.play(SoundType::PopupOpen);
                return true;
            }
        }
        KeyCode::F(6) => {
            if app.ui.mode == crate::state::AppMode::Chat {
                app.set_notification("Refreshing notifications...", Some(500), true);
                app.send_to_server(ClientMessage::GetNotifications { before: None });
                return true;
            }
        }
        _ => {}
    }
    false
}

/// Handle general navigation (main menu, settings, etc.)
pub fn handle_general_navigation(key: KeyEvent, app: &mut App) {
    match app.ui.mode {
        crate::state::AppMode::MainMenu => handle_main_menu_input(key, app),
        crate::state::AppMode::Settings => handle_settings_input(key, app),
        crate::state::AppMode::ColorPicker => handle_color_picker_input(key, app),
        crate::state::AppMode::Preferences => handle_preferences_input(key, app),
        _ => {}
    }
}

/// Handle input mode (popup input dialogs)
pub fn handle_input_mode(key: KeyEvent, app: &mut App) {
    use crate::state::InputMode::*;
    
    match key.code {
        KeyCode::Enter => {
            let input = app.auth.current_input.clone();
            let prev_input = app.auth.password_input.clone();
            app.auth.clear_inputs();
            
            if let Some(im) = app.auth.input_mode.take() {
                match im {
                    NewForumName => {
                        app.sound_manager.play(SoundType::PopupOpen);
                        if input.trim().is_empty() {
                            app.set_notification("Forum name cannot be empty.", None, false);
                            app.auth.set_input_mode(NewForumName);
                            return;
                        }
                        app.enter_input_mode(NewForumDescription);
                        app.auth.password_input = input;
                    }
                    NewForumDescription => {
                        let name = prev_input;
                        let description = input;
                        app.sound_manager.play(SoundType::PopupOpen);
                        
                        if name.trim().is_empty() || description.trim().is_empty() {
                            app.set_notification("Forum name and description cannot be empty.", None, false);
                            app.auth.set_input_mode(NewForumName);
                            app.auth.password_input = name;
                            return;
                        }
                        
                        app.send_to_server(ClientMessage::CreateForum {
                            name: name.clone(),
                            description: description.clone(),
                        });
                        app.set_notification("Forum creation requested!", Some(1500), false);
                        app.ui.set_mode(crate::state::AppMode::ForumList);
                    }
                    NewThreadTitle => {
                        app.sound_manager.play(SoundType::PopupOpen);
                        if input.trim().is_empty() {
                            app.set_notification("Thread title cannot be empty.", None, false);
                            app.auth.set_input_mode(NewThreadTitle);
                            return;
                        }
                        app.enter_input_mode(NewThreadContent);
                        app.auth.password_input = input;
                    }
                    NewThreadContent => {
                        let title = prev_input;
                        let content = input;
                        app.sound_manager.play(SoundType::PopupOpen);
                        
                        if title.trim().is_empty() || content.trim().is_empty() {
                            app.set_notification("Thread title and content cannot be empty.", None, false);
                            app.auth.set_input_mode(NewThreadTitle);
                            app.auth.password_input = title;
                            return;
                        }
                        
                        if let Some(forum_id) = app.forum.current_forum_id {
                            app.forum.pending_new_thread_title = Some(title.clone());
                            app.send_to_server(ClientMessage::CreateThread {
                                forum_id,
                                title: title.clone(),
                                content: content.clone(),
                            });
                            app.set_notification("Thread submitted!", Some(1500), false);
                        }
                        app.ui.set_mode(crate::state::AppMode::ThreadList);
                    }
                    NewPostContent => {
                        app.sound_manager.play(SoundType::PopupOpen);
                        if input.trim().is_empty() {
                            app.set_notification("Post content cannot be empty.", None, false);
                            app.auth.set_input_mode(NewPostContent);
                            return;
                        }
                        
                        if let Some(thread_id) = app.forum.current_thread_id {
                            // Check if this is a reply to a specific post
                            if let Some(reply_to_id) = app.forum.reply_to_post_id {
                                app.send_to_server(ClientMessage::CreatePostReply {
                                    thread_id,
                                    content: input.clone(),
                                    reply_to: reply_to_id,
                                });
                                app.set_notification("Reply submitted!", Some(1500), false);
                                app.forum.set_reply_target(None); // Clear reply target
                            } else {
                                app.send_to_server(ClientMessage::CreatePost {
                                    thread_id,
                                    content: input.clone(),
                                });
                                app.set_notification("Post submitted!", Some(1500), false);
                            }
                        }
                        app.ui.set_mode(crate::state::AppMode::PostView);
                    }
                    UpdatePassword => {
                        app.sound_manager.play(SoundType::PopupOpen);
                        app.send_to_server(ClientMessage::UpdatePassword(input));
                        app.ui.set_mode(crate::state::AppMode::Settings);
                    }
                    _ => {
                        app.ui.set_mode(crate::state::AppMode::MainMenu);
                    }
                }
            }
        }
        KeyCode::Char(c) => {
            app.auth.current_input.push(c);
        }
        KeyCode::Backspace => {
            app.auth.current_input.pop();
        }
        KeyCode::Esc => {
            app.auth.input_mode = None;
            app.ui.set_mode(crate::state::AppMode::MainMenu);
        }
        _ => {}
    }
}

fn handle_main_menu_input(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Down => {
            app.sound_manager.play(SoundType::Scroll);
            let current = app.ui.main_menu_state.selected().unwrap_or(0);
            app.ui.main_menu_state.select(Some((current + 1) % 4));
        }
        KeyCode::Up => {
            app.sound_manager.play(SoundType::Scroll);
            let current = app.ui.main_menu_state.selected().unwrap_or(0);
            app.ui.main_menu_state.select(Some((current + 3) % 4));
        }
        KeyCode::Enter => {
            if let Some(selection) = app.ui.main_menu_state.selected() {
                match selection {
                    0 => {
                        app.send_to_server(ClientMessage::GetForums);
                        app.ui.set_mode(crate::state::AppMode::ForumList);
                        app.forum.forum_list_state.select(Some(0));
                    }
                    1 => {
                        app.ui.set_mode(crate::state::AppMode::Chat);
                        app.auth.current_input.clear();
                        app.send_to_server(ClientMessage::GetServers);
                        app.send_to_server(ClientMessage::GetDMUserList);
                    }
                    2 => {
                        app.ui.set_mode(crate::state::AppMode::Settings);
                        app.ui.settings_list_state.select(Some(0));
                    }
                    3 => {
                        app.send_to_server(ClientMessage::Logout);
                        app.auth.logout();
                        app.ui.set_mode(crate::state::AppMode::Login);
                        app.auth.clear_inputs();
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

fn handle_settings_input(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Down => {
            app.sound_manager.play(SoundType::Scroll);
            let max = if app.auth.is_logged_in() { 5 } else { 3 }; // Now 5 items when logged in
            let current = app.ui.settings_list_state.selected().unwrap_or(0);
            app.ui.settings_list_state.select(Some((current + 1) % max));
        }
        KeyCode::Up => {
            app.sound_manager.play(SoundType::Scroll);
            let max = if app.auth.is_logged_in() { 5 } else { 3 }; // Now 5 items when logged in
            let current = app.ui.settings_list_state.selected().unwrap_or(0);
            app.ui.settings_list_state.select(Some((current + max - 1) % max));
        }
        KeyCode::Enter => {
            if let Some(selection) = app.ui.settings_list_state.selected() {
                match selection {
                    0 => app.enter_input_mode(crate::state::InputMode::UpdatePassword),
                    1 => {
                        // Enter color picker mode
                        if let Some(user) = &app.auth.current_user {
                            let palette = [
                                Color::Cyan, Color::Green, Color::Yellow, Color::Red,
                                Color::Magenta, Color::Blue, Color::White, Color::LightCyan,
                                Color::LightGreen, Color::LightYellow, Color::LightRed,
                                Color::LightMagenta, Color::LightBlue, Color::Gray,
                                Color::DarkGray, Color::Black
                            ];
                            app.ui.color_picker_selected = palette.iter()
                                .position(|&c| c == user.color.clone().into())
                                .unwrap_or(0);
                        } else {
                            app.ui.color_picker_selected = 0;
                        }
                        app.ui.set_mode(crate::state::AppMode::ColorPicker);
                    }
                    2 => {
                        if let Some(user) = &app.auth.current_user {
                            app.profile.profile_requested_by_user = false;
                            app.send_to_server(ClientMessage::GetProfile { user_id: user.id });
                            app.profile.profile_edit_focus = crate::state::ProfileEditFocus::Bio;
                            app.ui.set_mode(crate::state::AppMode::EditProfile);
                        }
                    }
                    3 => {
                        app.ui.set_mode(crate::state::AppMode::Preferences);
                    }
                    4 => {
                        // Test notification
                        app.set_notification("Testing in-app notifications...", Some(3000), false);
                        DesktopNotificationService::show_info_notification("Testing desktop notifications!");
                    }
                    _ => {}
                }
            }
        }
        KeyCode::Esc => {
            app.ui.set_mode(crate::state::AppMode::MainMenu);
        }
        KeyCode::Char('p') => {
            app.ui.set_mode(crate::state::AppMode::Preferences);
        }
        _ => {}
    }
}

fn handle_color_picker_input(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Left => {
            let palette_len = 16;
            if app.ui.color_picker_selected == 0 {
                app.ui.color_picker_selected = palette_len - 1;
            } else {
                app.ui.color_picker_selected -= 1;
            }
        }
        KeyCode::Right => {
            let palette_len = 16;
            app.ui.color_picker_selected = (app.ui.color_picker_selected + 1) % palette_len;
        }
        KeyCode::Enter => {
            let palette = [
                Color::Cyan, Color::Green, Color::Yellow, Color::Red,
                Color::Magenta, Color::Blue, Color::White, Color::LightCyan,
                Color::LightGreen, Color::LightYellow, Color::LightRed,
                Color::LightMagenta, Color::LightBlue, Color::Gray,
                Color::DarkGray, Color::Black
            ];
            let new_color = palette[app.ui.color_picker_selected];
            
            if let Some(user) = &mut app.auth.current_user {
                user.color = new_color.into();
                app.send_to_server(ClientMessage::UpdateColor(UserColor::from(new_color)));
                app.sound_manager.play(SoundType::Save);
            }
            app.ui.set_mode(crate::state::AppMode::Settings);
        }
        KeyCode::Esc => {
            app.ui.set_mode(crate::state::AppMode::Settings);
        }
        _ => {}
    }
}

fn handle_preferences_input(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Down => {
            app.sound_manager.play(SoundType::Scroll);
            app.ui.preferences_selected = (app.ui.preferences_selected + 1) % 3; // 3 preferences total
        }
        KeyCode::Up => {
            app.sound_manager.play(SoundType::Scroll);
            app.ui.preferences_selected = if app.ui.preferences_selected == 0 { 2 } else { app.ui.preferences_selected - 1 };
        }
        KeyCode::Char(' ') | KeyCode::Enter => {
            app.sound_manager.play(SoundType::Save);
            let mut prefs = global_prefs_mut();
            match app.ui.preferences_selected {
                0 => {
                    // Toggle sound effects
                    prefs.sound_effects_enabled = !prefs.sound_effects_enabled;
                }
                1 => {
                    // Toggle glitch effects
                    prefs.minimal_banner_glitch_enabled = !prefs.minimal_banner_glitch_enabled;
                }
                2 => {
                    // Toggle desktop notifications
                    prefs.desktop_notifications_enabled = !prefs.desktop_notifications_enabled;
                    
                    // Show a test notification when enabling
                    if prefs.desktop_notifications_enabled {
                        DesktopNotificationService::show_info_notification("Desktop notifications enabled!");
                    }
                }
                _ => {}
            }
            prefs.save();
        }
        KeyCode::Esc => {
            // Return to previous menu
            if app.auth.is_logged_in() {
                app.ui.set_mode(crate::state::AppMode::Settings);
            } else {
                let is_register = matches!(app.auth.input_mode, Some(crate::state::InputMode::RegisterUsername) | Some(crate::state::InputMode::RegisterPassword));
                app.ui.set_mode(if is_register { 
                    crate::state::AppMode::Register 
                } else { 
                    crate::state::AppMode::Login 
                });
            }
        }
        _ => {}
    }
}