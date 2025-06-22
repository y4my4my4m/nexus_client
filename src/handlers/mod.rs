pub mod auth;
pub mod chat;
pub mod profile;
pub mod forum;
pub mod navigation;

use crate::app::App;
use crossterm::event::KeyEvent;

/// Main input handler dispatcher
pub fn handle_key_event(key: KeyEvent, app: &mut App) {
    // Handle global shortcuts first
    if navigation::handle_global_shortcuts(key, app) {
        return;
    }

    // Check if there's an active notification and close it on any key press
    if app.notifications.current_notification.is_some() {
        app.notifications.clear_notification();
        return; // Consume the key press and don't process further
    }

    match app.ui.mode {
        crate::state::AppMode::Login | crate::state::AppMode::Register => {
            auth::handle_auth_input(key, app);
        }
        crate::state::AppMode::Chat => {
            chat::handle_chat_input(key, app);
        }
        crate::state::AppMode::EditProfile => {
            profile::handle_profile_edit_input(key, app);
        }
        crate::state::AppMode::ForumList | crate::state::AppMode::ThreadList | crate::state::AppMode::PostView => {
            forum::handle_forum_input(key, app);
        }
        crate::state::AppMode::Input => {
            navigation::handle_input_mode(key, app);
        }
        _ => {
            navigation::handle_general_navigation(key, app);
        }
    }
}