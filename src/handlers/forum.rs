use crate::app::App;
use crate::sound::SoundType;
use common::ClientMessage;
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
                app.sound_manager.play(SoundType::Scroll);
                let current = app.forum.forum_list_state.selected().unwrap_or(0);
                let next = (current + 1) % app.forum.forums.len();
                app.forum.forum_list_state.select(Some(next));
            }
        }
        KeyCode::Up => {
            if !app.forum.forums.is_empty() {
                app.sound_manager.play(SoundType::Scroll);
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
                if user.role == common::UserRole::Admin {
                    app.enter_input_mode(crate::state::InputMode::NewForumName);
                }
            }
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            // Admin-only: Delete selected forum
            if let Some(user) = &app.auth.current_user {
                if user.role == common::UserRole::Admin {
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
                    app.sound_manager.play(SoundType::Scroll);
                    let current = app.forum.thread_list_state.selected().unwrap_or(0);
                    let next = (current + 1) % forum.threads.len();
                    app.forum.thread_list_state.select(Some(next));
                }
            }
        }
        KeyCode::Up => {
            if let Some(forum) = app.forum.get_current_forum() {
                if !forum.threads.is_empty() {
                    app.sound_manager.play(SoundType::Scroll);
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
                if user.role == common::UserRole::Admin {
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
        KeyCode::Char('r') | KeyCode::Char('R') => {
            app.enter_input_mode(crate::state::InputMode::NewPostContent);
        }
        KeyCode::Char('d') | KeyCode::Char('D') if key.modifiers.contains(KeyModifiers::ALT) => {
            // Admin-only: Delete first post in thread (for now, we'll need to add post selection later)
            if let Some(user) = &app.auth.current_user {
                if user.role == common::UserRole::Admin {
                    if let Some(thread) = app.forum.get_current_thread() {
                        if let Some(post) = thread.posts.first() {
                            app.send_to_server(ClientMessage::DeletePost(post.id));
                            app.set_notification("Post deletion requested", Some(2000), false);
                        }
                    }
                }
            }
        }
        KeyCode::Esc => {
            app.ui.set_mode(crate::state::AppMode::ThreadList);
        }
        _ => {}
    }
}