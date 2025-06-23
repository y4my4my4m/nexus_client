pub mod auth;
pub mod chat;
pub mod profile;
pub mod forum;
pub mod navigation;

use crate::app::App;
use crossterm::event::KeyEvent;

/// Main input handler dispatcher
pub fn handle_key_event(key: KeyEvent, app: &mut App) {
    // Handle server error popup first (highest priority)
    if app.ui.show_server_error {
        handle_server_error_input(key, app);
        return;
    }

    // Handle quit confirmation dialog
    if app.ui.show_quit_confirm {
        handle_quit_confirm_input(key, app);
        return;
    }

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

fn handle_quit_confirm_input(key: KeyEvent, app: &mut App) {
    use crossterm::event::{KeyCode, KeyModifiers};

    match key.code {
        KeyCode::Left | KeyCode::Right => {
            app.sound_manager.play(crate::sound::SoundType::Scroll);
            app.ui.quit_confirm_selected = if app.ui.quit_confirm_selected == 0 { 1 } else { 0 };
        }
        KeyCode::Enter => {
            if app.ui.quit_confirm_selected == 0 {
                // Yes - quit the application
                app.sound_manager.play(crate::sound::SoundType::PopupClose);
                app.ui.quit();
            } else {
                // No - cancel quit
                app.sound_manager.play(crate::sound::SoundType::PopupClose);
            }
            app.ui.show_quit_confirm = false;
        }
        KeyCode::Esc => {
            // Cancel quit
            app.sound_manager.play(crate::sound::SoundType::PopupClose);
            app.ui.show_quit_confirm = false;
        }
        // Handle Ctrl+C again to close the dialog
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.sound_manager.play(crate::sound::SoundType::PopupClose);
            app.ui.show_quit_confirm = false;
        }
        _ => {}
    }
}

/// Handle server error popup input
fn handle_server_error_input(key: KeyEvent, app: &mut App) {
    use crossterm::event::{KeyCode, KeyModifiers};

    match key.code {
        KeyCode::Enter => {
            // Request connection retry
            app.sound_manager.play(crate::sound::SoundType::PopupClose);
            app.ui.should_retry_connection = true;
            app.ui.hide_server_error();
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Allow Ctrl+C to quit the application
            app.ui.quit();
        }
        // Remove ESC handler - don't allow closing the popup with ESC
        _ => {}
    }
}