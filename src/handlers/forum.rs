use crate::app::App;
use crate::sound::SoundType;
use nexus_tui_common::ClientMessage;
use crossterm::event::{KeyCode, KeyEvent};

/// Handle forum-related input (forum list, thread list, post view)
pub fn handle_forum_input(key: KeyEvent, app: &mut App) {
    match app.ui.mode {
        crate::state::AppMode::ForumList => handle_forum_list_input(key, app),
        crate::state::AppMode::ThreadList => handle_thread_list_input(key, app),
        crate::state::AppMode::PostView => handle_post_view_input(key, app),
        _ => {}
    }
}

fn handle_forum_list_input(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Down => {
            if !app.forum.forums.is_empty() {
                app.sound_manager.play(SoundType::ChangeChannel);
                let current = app.forum.forum_list_state.selected().unwrap_or(0);
                let next = (current + 1) % app.forum.forums.len();
                app.forum.forum_list_state.select(Some(next));
            }
        }
        KeyCode::Up => {
            if !app.forum.forums.is_empty() {
                app.sound_manager.play(SoundType::ChangeChannel);
                let current = app.forum.forum_list_state.selected().unwrap_or(0);
                let next = (current + app.forum.forums.len() - 1) % app.forum.forums.len();
                app.forum.forum_list_state.select(Some(next));
            }
        }
        KeyCode::Enter => {
            if let Some(idx) = app.forum.forum_list_state.selected() {
                if let Some(forum) = app.forum.forums.get(idx) {
                    app.forum.select_forum(forum.id);
                    app.ui.set_mode(crate::state::AppMode::ThreadList);
                }
            }
        }
        KeyCode::Char('n') | KeyCode::Char('N') => {
            // Admin-only: Create new forum
            if let Some(user) = &app.auth.current_user {
                if user.role == nexus_tui_common::UserRole::Admin {
                    app.enter_input_mode(crate::state::InputMode::NewForumName);
                }
            }
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            // Admin-only: Delete selected forum
            if let Some(user) = &app.auth.current_user {
                if user.role == nexus_tui_common::UserRole::Admin {
                    if let Some(idx) = app.forum.forum_list_state.selected() {
                        if let Some(forum) = app.forum.forums.get(idx) {
                            app.send_to_server(ClientMessage::DeleteForum { forum_id: forum.id });
                            app.set_notification("Forum deletion requested", Some(2000), false);
                        }
                    }
                }
            }
        }
        KeyCode::Esc => {
            app.ui.set_mode(crate::state::AppMode::MainMenu);
        }
        _ => {}
    }
}

fn handle_thread_list_input(key: KeyEvent, app: &mut App) {
    use crossterm::event::KeyModifiers;
    
    match key.code {
        KeyCode::Down => {
            if let Some(forum) = app.forum.get_current_forum() {
                if !forum.threads.is_empty() {
                    app.sound_manager.play(SoundType::ChangeChannel);
                    let current = app.forum.thread_list_state.selected().unwrap_or(0);
                    let next = (current + 1) % forum.threads.len();
                    app.forum.thread_list_state.select(Some(next));
                }
            }
        }
        KeyCode::Up => {
            if let Some(forum) = app.forum.get_current_forum() {
                if !forum.threads.is_empty() {
                    app.sound_manager.play(SoundType::ChangeChannel);
                    let current = app.forum.thread_list_state.selected().unwrap_or(0);
                    let next = (current + forum.threads.len() - 1) % forum.threads.len();
                    app.forum.thread_list_state.select(Some(next));
                }
            }
        }
        KeyCode::Enter => {
            if let Some(idx) = app.forum.thread_list_state.selected() {
                if let Some(forum) = app.forum.get_current_forum() {
                    if let Some(thread) = forum.threads.get(idx) {
                        app.forum.select_thread(thread.id);
                        app.ui.set_mode(crate::state::AppMode::PostView);
                    }
                }
            }
        }
        KeyCode::Char('n') | KeyCode::Char('N') => {
            app.enter_input_mode(crate::state::InputMode::NewThreadTitle);
        }
        KeyCode::Char('d') | KeyCode::Char('D') if key.modifiers.contains(KeyModifiers::ALT) => {
            // Admin-only: Delete selected thread
            if let Some(user) = &app.auth.current_user {
                if user.role == nexus_tui_common::UserRole::Admin {
                    if let Some(idx) = app.forum.thread_list_state.selected() {
                        if let Some(forum) = app.forum.get_current_forum() {
                            if let Some(thread) = forum.threads.get(idx) {
                                app.send_to_server(ClientMessage::DeleteThread(thread.id));
                                app.set_notification("Thread deletion requested", Some(2000), false);
                            }
                        }
                    }
                }
            }
        }
        KeyCode::Esc => {
            app.ui.set_mode(crate::state::AppMode::ForumList);
        }
        _ => {}
    }
}

fn handle_post_view_input(key: KeyEvent, app: &mut App) {
    use crossterm::event::KeyModifiers;
    
    match key.code {
        // Post navigation
        KeyCode::Up => {
            app.forum.move_post_selection(-1);
            app.sound_manager.play(SoundType::ChangeChannel);
        }
        KeyCode::Down => {
            app.forum.move_post_selection(1);
            app.sound_manager.play(SoundType::ChangeChannel);
        }
        // Manual scrolling
        KeyCode::PageUp => {
            app.forum.scroll_posts(-1, 3); // Scroll up by 3 posts
            app.sound_manager.play(SoundType::Scroll);
        }
        KeyCode::PageDown => {
            app.forum.scroll_posts(1, 3); // Scroll down by 3 posts
            app.sound_manager.play(SoundType::Scroll);
        }
        // Home/End for quick navigation
        KeyCode::Home => {
            if let Some(thread) = app.forum.get_current_thread() {
                if !thread.posts.is_empty() {
                    app.forum.selected_post_index = Some(0);
                    app.forum.scroll_offset = 0;
                    app.forum.selected_reply_index = None;
                    app.sound_manager.play(SoundType::ChangeChannel);
                }
            }
        }
        KeyCode::End => {
            if let Some(thread) = app.forum.get_current_thread() {
                if !thread.posts.is_empty() {
                    let last_idx = thread.posts.len() - 1;
                    app.forum.selected_post_index = Some(last_idx);
                    app.forum.auto_scroll_to_selected_post();
                    app.forum.selected_reply_index = None;
                    app.sound_manager.play(SoundType::ChangeChannel);
                }
            }
        }
        // Reply navigation (left/right when replies exist)
        KeyCode::Left => {
            if app.forum.selected_reply_index.is_some() {
                app.forum.move_reply_selection(-1);
                app.sound_manager.play(SoundType::ChangeChannel);
            }
        }
        KeyCode::Right => {
            if let Some(post) = app.forum.get_selected_post() {
                let replies = app.forum.get_replies_to_post(post.id);
                if !replies.is_empty() {
                    if app.forum.selected_reply_index.is_none() {
                        app.forum.selected_reply_index = Some(0);
                    } else {
                        app.forum.move_reply_selection(1);
                    }
                    app.sound_manager.play(SoundType::ChangeChannel);
                }
            }
        }
        // Enter: Navigate to selected reply or jump to original post in context mode
        KeyCode::Enter => {
            if app.forum.show_reply_context {
                // Jump to the original post that the current post replied to
                if app.forum.jump_to_replied_post() {
                    app.sound_manager.play(SoundType::PopupOpen);
                } else {
                    app.sound_manager.play(SoundType::Error);
                }
            } else if let Some(reply_post) = app.forum.get_selected_reply_post() {
                // Find the index of the reply post and scroll to it
                if let Some(thread) = app.forum.get_current_thread() {
                    if let Some((idx, _)) = thread.posts.iter().enumerate().find(|(_, p)| p.id == reply_post.id) {
                        app.forum.selected_post_index = Some(idx);
                        app.forum.selected_reply_index = None;
                        app.forum.scroll_to_post(idx);
                        app.sound_manager.play(SoundType::PopupOpen);
                    }
                }
            }
        }
        KeyCode::Char('c') | KeyCode::Char('C') => {
            // Toggle context view to see what post this replied to
            app.forum.toggle_reply_context();
            if app.forum.show_reply_context {
                app.sound_manager.play(SoundType::PopupOpen);
            } else {
                app.sound_manager.play(SoundType::PopupClose);
            }
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            if key.modifiers.contains(KeyModifiers::ALT) {
                // Alt+R: General post (not replying to anyone)
                app.forum.set_reply_target(None);
                app.enter_input_mode(crate::state::InputMode::NewPostContent);
            } else {
                // R: Reply to the currently selected post
                if let Some(post) = app.forum.get_selected_post() {
                    app.forum.set_reply_target(Some(post.id));
                } else {
                    app.forum.set_reply_target(None);
                }
                app.enter_input_mode(crate::state::InputMode::NewPostContent);
            }
        }
        KeyCode::Char('d') | KeyCode::Char('D') if key.modifiers.contains(KeyModifiers::ALT) => {
            // Admin-only: Delete selected post
            if let Some(user) = &app.auth.current_user {
                if user.role == nexus_tui_common::UserRole::Admin {
                    if let Some(post) = app.forum.get_selected_post() {
                        app.send_to_server(ClientMessage::DeletePost(post.id));
                        app.set_notification("Post deletion requested", Some(2000), false);
                    }
                }
            }
        }
        KeyCode::Esc => {
            // Clear reply selection if active, otherwise go back to thread list
            if app.forum.selected_reply_index.is_some() {
                app.forum.selected_reply_index = None;
                app.sound_manager.play(SoundType::PopupClose);
            } else {
                app.ui.set_mode(crate::state::AppMode::ThreadList);
            }
        }
        _ => {}
    }
}