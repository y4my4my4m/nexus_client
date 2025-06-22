//! Main UI module. Re-exports submodules and provides the main entry point.

pub mod banner;
pub mod auth;
pub mod main_menu;
pub mod forums;
pub mod settings;
pub mod chat;
pub mod popups;
pub mod avatar;
pub mod time_format;

use ratatui::Frame;
use common::UserRole;
use crate::app::{App, AppMode, InputMode};
use crate::ui::banner::{draw_full_banner, draw_min_banner};
use crate::ui::auth::{draw_login, draw_register};
use crate::ui::main_menu::draw_main_menu;
use crate::ui::forums::{draw_forum_list, draw_thread_list, draw_post_view};
use crate::ui::settings::{draw_settings, draw_profile_edit_page, draw_color_picker};
use crate::ui::chat::draw_chat;
use crate::ui::popups::{draw_input_popup, draw_notification_popup, draw_minimal_notification_popup, draw_profile_view_popup, draw_user_actions_popup, draw_server_actions_popup, draw_server_invite_selection_popup};


pub fn ui(f: &mut Frame, app: &mut App) {
    let size = f.area();
    let (banner_height, use_full_banner) = match app.ui.mode {
        AppMode::Login | AppMode::Register | AppMode::MainMenu => (9, true),
        _ => (3, false),
    };
    let chunks = ratatui::layout::Layout::default()
        .constraints([
            ratatui::layout::Constraint::Length(banner_height), // Banner height
            ratatui::layout::Constraint::Min(0),                // Main Content
            ratatui::layout::Constraint::Length(3),             // Footer
        ])
        .split(size);

    if use_full_banner {
        draw_full_banner(f, app, chunks[0]);
    } else {
        draw_min_banner(f, app, chunks[0]);
    }

    let help_text = match app.ui.mode {
        AppMode::Login | AppMode::Register => "[Esc] QUIT | [F2] Preferences\n[Tab]/[Shift+Tab] Change Focus | [Enter] Select/Submit",
        _ => "[Tab] Change Focus | [F2] Prefs | [↑↓] Nav\n[PgUp/PgDn] Scroll | [Enter] Sel | [Esc] Back"
    };
    let status_text = if let Some(user) = &app.auth.current_user {
        if user.role == UserRole::Admin {
            format!("Logged in as: {} ({:?})", user.username, user.role)
        } else {
            format!("Logged in as: {}", user.username)
        }
    } else { "Not Logged In".to_string() };
    
    // Split footer into two sections: help text and status
    let footer_chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage(67), // Help text area
            ratatui::layout::Constraint::Percentage(33), // Status area
        ])
        .split(chunks[2]);
    
    // Render help text with multiline support and wrapping
    f.render_widget(
        ratatui::widgets::Paragraph::new(help_text)
            .wrap(ratatui::widgets::Wrap { trim: true })
            .block(ratatui::widgets::Block::default().borders(ratatui::widgets::Borders::TOP)),
        footer_chunks[0],
    );
    
    // Render status text right-aligned
    f.render_widget(
        ratatui::widgets::Paragraph::new(ratatui::text::Span::styled(
            status_text,
            ratatui::style::Style::default().fg(ratatui::style::Color::Yellow)
        ))
            .alignment(ratatui::layout::Alignment::Right)
            .block(ratatui::widgets::Block::default().borders(ratatui::widgets::Borders::TOP)),
        footer_chunks[1],
    );

    let main_area = chunks[1];
    match app.ui.mode {
        AppMode::Login => draw_login(f, app, main_area),
        AppMode::Register => draw_register(f, app, main_area),
        AppMode::MainMenu => draw_main_menu(f, app, main_area),
        AppMode::Settings => draw_settings(f, app, main_area),
        AppMode::ForumList => draw_forum_list(f, app, main_area),
        AppMode::ThreadList => draw_thread_list(f, app, main_area),
        AppMode::PostView => draw_post_view(f, app, main_area),
        AppMode::Chat => draw_chat(f, app, main_area),
        AppMode::Input => {
            let underlying_mode = match app.auth.input_mode {
                Some(InputMode::NewForumName) | Some(InputMode::NewForumDescription) => Some(AppMode::ForumList),
                Some(InputMode::NewThreadTitle) | Some(InputMode::NewThreadContent) => Some(AppMode::ForumList),
                Some(InputMode::NewPostContent) => Some(AppMode::PostView),
                Some(InputMode::UpdatePassword) => Some(AppMode::Settings),
                _ => None,
            };
            if let Some(mode) = underlying_mode {
                match mode {
                    AppMode::ForumList => draw_forum_list(f, app, main_area),
                    AppMode::PostView => draw_post_view(f, app, main_area),
                    AppMode::Settings => draw_settings(f, app, main_area),
                    _ => {}
                }
            }
            draw_input_popup(f, app);
        }
        AppMode::EditProfile => draw_profile_edit_page(f, app, main_area),
        AppMode::ColorPicker => draw_color_picker(f, app, main_area),
        AppMode::Preferences => crate::ui::settings::draw_preferences(f, app, main_area),
    }

    if let Some((notification, _, minimal)) = &app.notifications.current_notification {
        if *minimal {
            draw_minimal_notification_popup(f, notification.clone());
        } else {
            draw_notification_popup(f, notification.clone());
        }
    }
    if app.profile.show_profile_view_popup {
        if let Some(profile) = app.profile.profile_view.clone() {
            draw_profile_view_popup(f, app, &profile);
        }
    }
    if app.profile.show_user_actions {
        draw_user_actions_popup(f, app);
    }
    if app.ui.show_server_actions {
        draw_server_actions_popup(f, app);
    }
    if app.ui.show_server_invite_selection {
        draw_server_invite_selection_popup(f, app);
    }
    if app.ui.show_quit_confirm {
        crate::ui::popups::draw_quit_confirm_popup(f, app);
        return;
    }
}
