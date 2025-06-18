//! Main UI module. Re-exports submodules and provides the main entry point.

pub mod banner;
pub mod auth;
pub mod main_menu;
pub mod forums;
pub mod settings;
pub mod chat;
pub mod popups;
pub mod avatar;

use ratatui::Frame;
use crate::app::{App, AppMode, InputMode};
use crate::ui::banner::draw_banner;
use crate::ui::auth::{draw_login, draw_register};
use crate::ui::main_menu::draw_main_menu;
use crate::ui::forums::{draw_forum_list, draw_thread_list, draw_post_view};
use crate::ui::settings::{draw_settings, draw_profile_edit_page, draw_color_picker_page};
use crate::ui::chat::draw_chat;
use crate::ui::popups::{draw_input_popup, draw_notification_popup, draw_minimal_notification_popup, draw_profile_view_popup, draw_user_actions_popup};


pub fn ui(f: &mut Frame, app: &mut App) {
    let size = f.area();
    let chunks = ratatui::layout::Layout::default()
        .constraints([
            ratatui::layout::Constraint::Length(8), // Banner height
            ratatui::layout::Constraint::Min(0),    // Main Content
            ratatui::layout::Constraint::Length(3), // Footer
        ])
        .split(size);

    draw_banner(f, app, chunks[0]);

    let help_text = match app.mode {
        AppMode::Login | AppMode::Register => "[Esc] QUIT | [F2] Preferences | [Tab]/[Shift+Tab] Change Focus | [Enter] Select/Submit",
        _ => "[Q]uit | [F2] Preferences | [↑↓] Navigate | [Enter] Select | [Esc] Back"
    };
    let status_text = if let Some(user) = &app.current_user {
        format!("Logged in as: {} ({:?})", user.username, user.role)
    } else { "Not Logged In".to_string() };
    f.render_widget(
        ratatui::widgets::Paragraph::new(ratatui::text::Line::from(vec![
            ratatui::text::Span::raw(help_text), ratatui::text::Span::raw(" | "),
            ratatui::text::Span::styled(status_text, ratatui::style::Style::default().fg(ratatui::style::Color::Yellow)),
        ])).block(ratatui::widgets::Block::default().borders(ratatui::widgets::Borders::TOP)),
        chunks[2],
    );

    let main_area = chunks[1];
    match app.mode {
        AppMode::Login => draw_login(f, app, main_area),
        AppMode::Register => draw_register(f, app, main_area),
        AppMode::MainMenu => draw_main_menu(f, app, main_area),
        AppMode::Settings => draw_settings(f, app, main_area),
        AppMode::ForumList => draw_forum_list(f, app, main_area),
        AppMode::ThreadList => draw_thread_list(f, app, main_area),
        AppMode::PostView => draw_post_view(f, app, main_area),
        AppMode::Chat => draw_chat(f, app, main_area),
        AppMode::Input => {
            let underlying_mode = match app.input_mode {
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
        AppMode::ColorPicker => draw_color_picker_page(f, app, main_area),
        AppMode::Parameters => crate::ui::settings::draw_parameters_page(f, app, main_area),
    }

    if let Some((notification, _, minimal)) = &app.notification {
        if *minimal {
            draw_minimal_notification_popup(f, notification.clone());
        } else {
            draw_notification_popup(f, notification.clone());
        }
    }
    if app.show_profile_view_popup {
        if let Some(profile) = app.profile_view.clone() {
            draw_profile_view_popup(f, app, &profile);
        }
    }
    if app.show_user_actions {
        draw_user_actions_popup(f, app);
    }
    if app.show_quit_confirm {
        crate::ui::popups::draw_quit_confirm_popup(f, app);
        return;
    }
}
