use crate::app::{App, AppMode, InputMode};
use crate::banner::get_styled_banner_lines;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use ratatui_image::StatefulImage;
use base64::Engine;
use image::{DynamicImage, RgbaImage};

// Returns a mutable reference to a cached StatefulProtocol for the user's avatar, creating it if needed.
fn get_avatar_protocol<'a>(app: &'a mut App, user: &common::User, size: u32) -> Option<&'a mut ratatui_image::protocol::StatefulProtocol> {
    let key = (user.id, size);
    if !app.avatar_protocol_cache.contains_key(&key) {
        let pic = user.profile_pic.as_ref()?;
        let b64 = if let Some(idx) = pic.find(',') {
            if idx + 1 >= pic.len() { return None; }
            &pic[idx + 1..]
        } else { pic };
        let bytes = base64::engine::general_purpose::STANDARD.decode(b64).ok()?;
        let img = image::load_from_memory(&bytes).ok()?;
        let mut resized = img.resize_exact(size, size, image::imageops::FilterType::Lanczos3).to_rgba8();
        apply_circular_mask(&mut resized);
        let protocol = app.picker.new_resize_protocol(DynamicImage::ImageRgba8(resized));
        app.avatar_protocol_cache.insert(key, protocol);
    }
    app.avatar_protocol_cache.get_mut(&key)
}

// Helper: Apply a circular alpha mask to an RgbaImage in-place
fn apply_circular_mask(img: &mut RgbaImage) {
    let (w, h) = (img.width() as i32, img.height() as i32);
    let cx = w / 2;
    let cy = h / 2;
    let r = w.min(h) as f32 / 2.0;
    for y in 0..h {
        for x in 0..w {
            let dx = x - cx;
            let dy = y - cy;
            let dist = ((dx * dx + dy * dy) as f32).sqrt();
            if dist > r {
                let p = img.get_pixel_mut(x as u32, y as u32);
                p[3] = 0; // Set alpha to 0 (transparent)
            }
        }
    }
}

pub fn ui(f: &mut Frame, app: &mut App) {
    let size = f.area();

    let chunks = Layout::default()
        .constraints([
            Constraint::Length(8), // Banner height
            Constraint::Min(0),    // Main Content
            Constraint::Length(3), // Footer
        ])
        .split(size);

    let banner_lines = get_styled_banner_lines(size.width, app.tick_count);
    let banner = Paragraph::new(banner_lines)
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(banner, chunks[0]);

    let help_text = match app.mode {
        AppMode::Login | AppMode::Register => "| [Tab]/[Shift+Tab] Change Focus | [Enter] Select/Submit | [Esc] QUIT |",
        _ => "| [Q]uit | [↑↓] Navigate | [Enter] Select | [Esc] Back |"
    };
    let status_text = if let Some(user) = &app.current_user {
        format!("Logged in as: {} ({:?})", user.username, user.role)
    } else { "Not Logged In".to_string() };
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw(help_text), Span::raw(" | "),
            Span::styled(status_text, Style::default().fg(Color::Yellow)),
        ])).block(Block::default().borders(Borders::TOP)),
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
}

fn draw_centered_rect(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = Layout::default().direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2), Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ]).split(r);
    Layout::default().direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2), Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ]).split(popup_layout[1])[1]
}

fn draw_login(f: &mut Frame, app: &mut App, area: Rect) {
    let outer_block = Block::default().title("Login").borders(Borders::ALL);
    f.render_widget(outer_block, area);
    let chunks = Layout::default().margin(2).constraints([
        Constraint::Length(3), Constraint::Length(3), Constraint::Min(1)
    ]).split(area);

    let username_style = if matches!(app.input_mode, Some(InputMode::LoginUsername)) {
        Style::default().fg(Color::Yellow)
    } else { Style::default() };
    f.render_widget(
        Paragraph::new(app.current_input.as_str())
            .block(Block::default().borders(Borders::ALL).title("Username")).style(username_style),
        chunks[0],
    );
    let password_style = if matches!(app.input_mode, Some(InputMode::LoginPassword)) {
        Style::default().fg(Color::Yellow)
    } else { Style::default() };
    f.render_widget(
        Paragraph::new("*".repeat(app.password_input.len()))
            .block(Block::default().borders(Borders::ALL).title("Password")).style(password_style),
        chunks[1],
    );

    let button_area = Layout::default().margin(1).constraints([Constraint::Length(3)]).split(chunks[2])[0];
    let button_chunks = Layout::default().direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)]).split(button_area);

    let submit_style = if matches!(app.input_mode, Some(InputMode::AuthSubmit)) {
        Style::default().bg(Color::Cyan).fg(Color::Black)
    } else { Style::default() };
    f.render_widget(Paragraph::new(Span::styled("[ SUBMIT ]", submit_style)).alignment(Alignment::Center), button_chunks[0]);

    let switch_style = if matches!(app.input_mode, Some(InputMode::AuthSwitch)) {
        Style::default().bg(Color::Magenta).fg(Color::Black)
    } else { Style::default() };
    f.render_widget(Paragraph::new(Span::styled("[ To Register ]", switch_style)).alignment(Alignment::Center), button_chunks[1]);

    if let Some(InputMode::LoginUsername) = &app.input_mode {
        f.set_cursor_position((chunks[0].x + app.current_input.len() as u16 + 1, chunks[0].y + 1));
    } else if let Some(InputMode::LoginPassword) = &app.input_mode {
        f.set_cursor_position((chunks[1].x + app.password_input.len() as u16 + 1, chunks[1].y + 1));
    }
}

fn draw_register(f: &mut Frame, app: &mut App, area: Rect) {
    let outer_block = Block::default().title("Register").borders(Borders::ALL);
    f.render_widget(outer_block, area);
    let chunks = Layout::default().margin(2).constraints([
        Constraint::Length(3), Constraint::Length(3), Constraint::Min(1)
    ]).split(area);
    let username_style = if matches!(app.input_mode, Some(InputMode::RegisterUsername)) {
        Style::default().fg(Color::Yellow)
    } else { Style::default() };
    f.render_widget(
        Paragraph::new(app.current_input.as_str())
            .block(Block::default().borders(Borders::ALL).title("Choose Username")).style(username_style),
        chunks[0],
    );
    let password_style = if matches!(app.input_mode, Some(InputMode::RegisterPassword)) {
        Style::default().fg(Color::Yellow)
    } else { Style::default() };
    f.render_widget(
        Paragraph::new("*".repeat(app.password_input.len()))
            .block(Block::default().borders(Borders::ALL).title("Choose Password")).style(password_style),
        chunks[1],
    );

    let button_area = Layout::default().margin(1).constraints([Constraint::Length(3)]).split(chunks[2])[0];
    let button_chunks = Layout::default().direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)]).split(button_area);

    let submit_style = if matches!(app.input_mode, Some(InputMode::AuthSubmit)) {
        Style::default().bg(Color::Cyan).fg(Color::Black)
    } else { Style::default() };
    f.render_widget(Paragraph::new(Span::styled("[ SUBMIT ]", submit_style)).alignment(Alignment::Center), button_chunks[0]);

    let switch_style = if matches!(app.input_mode, Some(InputMode::AuthSwitch)) {
        Style::default().bg(Color::Magenta).fg(Color::Black)
    } else { Style::default() };
    f.render_widget(Paragraph::new(Span::styled("[ To Login ]", switch_style)).alignment(Alignment::Center), button_chunks[1]);

    if let Some(InputMode::RegisterUsername) = &app.input_mode {
        f.set_cursor_position((chunks[0].x + app.current_input.len() as u16 + 1, chunks[0].y + 1));
    } else if let Some(InputMode::RegisterPassword) = &app.input_mode {
        f.set_cursor_position((chunks[1].x + app.password_input.len() as u16 + 1, chunks[1].y + 1));
    }
}
fn draw_main_menu(f: &mut Frame, app: &mut App, area: Rect) {
    let items = vec![
        ListItem::new("Forums"), ListItem::new("Chat"), ListItem::new("Settings"),
        ListItem::new(Line::styled("Logout", Style::default().fg(Color::Red))),
    ];
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Main Menu"))
        .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black)).highlight_symbol(">> ");
    f.render_stateful_widget(list, area, &mut app.main_menu_state);
}

fn draw_forum_list(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app.forums.iter().map(|forum| {
        ListItem::new(Line::from(vec![
            Span::styled(format!("{:<30}", forum.name), Style::default().fg(Color::Cyan)),
            Span::raw(forum.description.clone())
        ]))
    }).collect();
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Forums | [N]ew Thread"))
        .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, area, &mut app.forum_list_state);
}

fn draw_thread_list(f: &mut Frame, app: &mut App, area: Rect) {
    let forum = match app.current_forum_id.and_then(|id| app.forums.iter().find(|f| f.id == id)) {
        Some(f) => f,
        None => { f.render_widget(Paragraph::new("Forum not found..."), area); return; }
    };
    let items: Vec<ListItem> = forum.threads.iter().map(|thread| {
        ListItem::new(Line::from(vec![
            Span::styled(format!("{:<40}", thread.title), Style::default().fg(Color::Cyan)),
            Span::raw("by "),
            Span::styled(format!("{}", thread.author.username), Style::default().fg(thread.author.color)),
        ]))
    }).collect();
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(format!("Threads in '{}'", forum.name)))
        .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, area, &mut app.thread_list_state);
}

fn draw_post_view(f: &mut Frame, app: &mut App, area: Rect) {
    let thread = match (app.current_forum_id, app.current_thread_id) {
        (Some(fid), Some(tid)) => app.forums.iter().find(|f| f.id == fid)
            .and_then(|f| f.threads.iter().find(|t| t.id == tid)),
        _ => None,
    };
    if let Some(thread) = thread {
        let title = format!("Reading: {} | [R]eply", thread.title);
        let mut text: Vec<Line> = Vec::new();
        for post in &thread.posts {
            text.push(Line::from(vec![Span::styled(format!("From: {} ", post.author.username), Style::default().fg(post.author.color).add_modifier(Modifier::BOLD))]));
            text.push(Line::from(Span::raw("---------------------------------")));
            text.push(Line::from(Span::raw(&post.content)));
            text.push(Line::from(Span::raw("")));
        }
        let paragraph = Paragraph::new(text).block(Block::default().borders(Borders::ALL).title(title)).wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
    } else {
        f.render_widget(Paragraph::new("Thread not found..."), area);
    }
}

fn draw_settings(f: &mut Frame, app: &mut App, area: Rect) {
    let items = vec![
        ListItem::new("Change Password"),
        ListItem::new("Change User Color (Cycle)"),
        ListItem::new("Edit Profile"),
    ];
    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Settings"))
        .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black)).highlight_symbol(">> ");
    f.render_stateful_widget(list, area, &mut app.settings_list_state);
}

fn draw_profile_edit_page(f: &mut Frame, app: &mut App, area: Rect) {
    use crate::app::ProfileEditFocus::*;
    let block = Block::default().title("Edit Profile").borders(Borders::ALL).border_type(BorderType::Double);
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(5), // Bio (multiline, taller)
            Constraint::Length(3), // Url1
            Constraint::Length(3), // Url2
            Constraint::Length(3), // Url3
            Constraint::Length(3), // Location
            Constraint::Length(3), // ProfilePic
            Constraint::Length(3), // CoverBanner
            Constraint::Length(2), // Spacer
            Constraint::Length(3), // Save/Cancel
            Constraint::Min(0),    // Error
        ])
        .split(area);
    f.render_widget(block, area);
    // Multiline bio
    let bio_style = if app.profile_edit_focus == Bio {
        Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else { Style::default() };
    f.render_widget(
        Paragraph::new(app.edit_bio.as_str())
            .block(Block::default().borders(Borders::ALL).title("Bio").border_style(bio_style))
            .style(bio_style)
            .wrap(Wrap { trim: false }),
        inner[0],
    );
    let url1_style = if app.profile_edit_focus == Url1 {
        Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else { Style::default() };
    f.render_widget(
        Paragraph::new(app.edit_url1.clone())
            .block(Block::default().borders(Borders::ALL).title("URL1").border_style(url1_style))
            .style(url1_style),
        inner[1],
    );
    let url2_style = if app.profile_edit_focus == Url2 {
        Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else { Style::default() };
    f.render_widget(
        Paragraph::new(app.edit_url2.clone())
            .block(Block::default().borders(Borders::ALL).title("URL2").border_style(url2_style))
            .style(url2_style),
        inner[2],
    );
    let url3_style = if app.profile_edit_focus == Url3 {
        Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else { Style::default() };
    f.render_widget(
        Paragraph::new(app.edit_url3.clone())
            .block(Block::default().borders(Borders::ALL).title("URL3").border_style(url3_style))
            .style(url3_style),
        inner[3],
    );
    let location_style = if app.profile_edit_focus == Location {
        Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else { Style::default() };
    f.render_widget(
        Paragraph::new(app.edit_location.clone())
            .block(Block::default().borders(Borders::ALL).title("Location").border_style(location_style))
            .style(location_style),
        inner[4],
    );
    let pic_style = if app.profile_edit_focus == ProfilePic {
        Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else { Style::default() };
    f.render_widget(
        Paragraph::new(app.edit_profile_pic.clone())
            .block(Block::default().borders(Borders::ALL).title("Profile Pic").border_style(pic_style))
            .style(pic_style),
        inner[5],
    );
    let banner_style = if app.profile_edit_focus == CoverBanner {
        Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else { Style::default() };
    f.render_widget(
        Paragraph::new(app.edit_cover_banner.clone())
            .block(Block::default().borders(Borders::ALL).title("Cover Banner").border_style(banner_style))
            .style(banner_style),
        inner[6],
    );
    // Save/Cancel buttons
    let save_style = if app.profile_edit_focus == Save {
        Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green)
    };
    let cancel_style = if app.profile_edit_focus == Cancel {
        Style::default().fg(Color::Black).bg(Color::Red).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Red)
    };
    let buttons = Line::from(vec![
        Span::styled("[ Save ]", save_style),
        Span::raw("   "),
        Span::styled("[ Cancel ]", cancel_style),
    ]);
    f.render_widget(Paragraph::new(buttons).alignment(Alignment::Center), inner[8]);
    // Error message
    if let Some(err) = &app.profile_edit_error {
        f.render_widget(Paragraph::new(err.as_str()).style(Style::default().fg(Color::Red)), inner[9]);
    }
    // Set cursor for focused field
    let cursor = match app.profile_edit_focus {
        Bio => {
            // Find cursor position in multiline bio
            let lines: Vec<&str> = app.edit_bio.split('\n').collect();
            let y = inner[0].y + lines.len() as u16 - 1 + 1;
            let x = inner[0].x + lines.last().map(|l| l.len()).unwrap_or(0) as u16 + 1;
            (x, y)
        },
        Url1 => (inner[1].x + app.edit_url1.len() as u16 + 1, inner[1].y + 1),
        Url2 => (inner[2].x + app.edit_url2.len() as u16 + 1, inner[2].y + 1),
        Url3 => (inner[3].x + app.edit_url3.len() as u16 + 1, inner[3].y + 1),
        Location => (inner[4].x + app.edit_location.len() as u16 + 1, inner[4].y + 1),
        ProfilePic => (inner[5].x + app.edit_profile_pic.len() as u16 + 1, inner[5].y + 1),
        CoverBanner => (inner[6].x + app.edit_cover_banner.len() as u16 + 1, inner[6].y + 1),
        _ => (0, 0),
    };
    if matches!(app.profile_edit_focus, Bio|Url1|Url2|Url3|Location|ProfilePic|CoverBanner) {
        f.set_cursor_position(cursor);
    }
}

fn draw_chat(f: &mut Frame, app: &mut App, area: Rect) {
    let show_users = app.show_user_list;
    let focus = app.chat_focus;
    if show_users {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(area);
        draw_chat_main(f, app, chunks[0], focus == crate::app::ChatFocus::Messages);
        draw_user_list(f, app, chunks[1], focus == crate::app::ChatFocus::Users);
    } else {
        draw_chat_main(f, app, area, true);
    }
    if app.chat_focus == crate::app::ChatFocus::DMInput {
        draw_dm_input_popup(f, app);
    }
}

fn draw_chat_main(f: &mut Frame, app: &mut App, area: Rect, focused: bool) {
    let border_style = if focused {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let chunks = Layout::default().constraints([Constraint::Min(0), Constraint::Length(3)]).split(area);
    let messages_area = chunks[0];
    let input_area = chunks[1];

    let messages_block = Block::default()
        .borders(Borders::ALL)
        .title("Live Chat // #general")
        .border_style(border_style);
    f.render_widget(messages_block.clone(), messages_area);

    let inner_area = messages_block.inner(messages_area);
    if inner_area.width == 0 || inner_area.height == 0 { return; }

    const AVATAR_PIXEL_SIZE: u32 = 32;
    let (font_w, font_h) = app.picker.font_size();
    let (font_w, font_h) = if font_w == 0 || font_h == 0 { (8, 16) } else { (font_w, font_h) };
    let avatar_cell_width = (AVATAR_PIXEL_SIZE as f32 / font_w as f32).ceil() as u16;
    let avatar_cell_height = (AVATAR_PIXEL_SIZE as f32 / font_h as f32).ceil() as u16;
    let row_height = avatar_cell_height.max(2);

    // Collect display items first to avoid borrow checker issues
    let display_items: Vec<_> = app.chat_messages.iter().rev().map(|msg| {
        let user = app.connected_users.iter().find(|u| u.username == msg.author).cloned();
        let author = msg.author.clone();
        let color = msg.color;
        let content = msg.content.clone();
        (user, author, color, content)
    }).collect();

    let mut current_y = inner_area.y;
    for (user_opt, author, color, content) in display_items.into_iter().rev() {
        let row_area = Rect::new(inner_area.x, current_y, inner_area.width, row_height);
        if let Some(user) = user_opt {
            if let Some(state) = get_avatar_protocol(app, &user, AVATAR_PIXEL_SIZE) {
                let row_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(avatar_cell_width), Constraint::Length(1), Constraint::Min(0)])
                    .split(row_area);
                let image_widget = StatefulImage::default();
                f.render_stateful_widget(image_widget, row_chunks[0], state);
                let text = vec![
                    Line::from(Span::styled(format!("<{}>", author), Style::default().fg(color).add_modifier(Modifier::BOLD))),
                    Line::from(Span::raw(&content)),
                ];
                f.render_widget(Paragraph::new(text).wrap(Wrap { trim: true }), row_chunks[2]);
            } else {
                let row_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(avatar_cell_width), Constraint::Length(1), Constraint::Min(0)])
                    .split(row_area);
                let text = vec![
                    Line::from(vec![
                        Span::raw("○ "),
                        Span::styled(format!("<{}>:", author), Style::default().fg(color).add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from(vec![Span::raw("  "), Span::raw(&content)]),
                ];
                f.render_widget(Paragraph::new(text).wrap(Wrap { trim: true }), row_chunks[2]);
            }
        }
        current_y += row_height + 1;
    }

    let input = Paragraph::new(app.current_input.as_str()).style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, input_area);
    if focused {
        f.set_cursor_position((input_area.x + app.current_input.len() as u16 + 1, input_area.y + 1));
    }
}

fn draw_user_list(f: &mut Frame, app: &mut App, area: Rect, focused: bool) {
    let border_style = if focused {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let block = Block::default().borders(Borders::ALL).title("Users [Ctrl+U]").border_style(border_style);
    f.render_widget(block.clone(), area);

    let inner_area = block.inner(area);
    if inner_area.width == 0 || inner_area.height == 0 { return; }

    const AVATAR_PIXEL_SIZE: u32 = 16;
    let (font_w, font_h) = app.picker.font_size();
    let (font_w, font_h) = if font_w == 0 || font_h == 0 { (8, 16) } else { (font_w, font_h) };
    let avatar_cell_width = (AVATAR_PIXEL_SIZE as f32 / font_w as f32).ceil() as u16;
    let avatar_cell_height = (AVATAR_PIXEL_SIZE as f32 / font_h as f32).ceil() as u16;
    let row_height = avatar_cell_height.max(1);

    let list_state = app.forum_list_state.clone();
    let selected_index = list_state.selected();
    let offset = list_state.offset();

    let mut current_y = inner_area.y;
    // Collect connected_users into a temporary vector before the loop
    let users: Vec<_> = app.connected_users.iter().enumerate().skip(offset).map(|(i, user)| (i, user.clone())).collect();
    for (i, user) in users {
        if current_y + row_height > inner_area.y + inner_area.height { break; }
        let row_area = Rect::new(inner_area.x, current_y, inner_area.width, row_height);

        let is_selected = focused && selected_index == Some(i);
        let text_style = if is_selected {
            Style::default().fg(Color::Black).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(user.color)
        };
        if is_selected {
            f.render_widget(Block::default().style(Style::default().bg(Color::Cyan)), row_area);
        }

        if let Some(state) = get_avatar_protocol(app, &user, AVATAR_PIXEL_SIZE) {
            let row_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(avatar_cell_width), Constraint::Min(0)])
                .split(row_area);
            let image_widget = StatefulImage::default();
            f.render_stateful_widget(image_widget, row_chunks[0], state);
            let text = Line::from(vec![
                Span::raw(" "),
                Span::styled(&user.username, text_style),
                Span::styled(format!(" ({:?})", user.role), text_style.remove_modifier(Modifier::BOLD).add_modifier(Modifier::DIM)),
            ]);
            f.render_widget(Paragraph::new(text).alignment(Alignment::Left), row_chunks[1]);
        } else {
            let text = Line::from(vec![
                Span::raw(" ○ "),
                Span::styled(&user.username, text_style),
                Span::styled(format!(" ({:?})", user.role), text_style.remove_modifier(Modifier::BOLD).add_modifier(Modifier::DIM)),
            ]);
            f.render_widget(Paragraph::new(text).alignment(Alignment::Left), row_area);
        }
        current_y += row_height;
    }
}



fn draw_dm_input_popup(f: &mut Frame, app: &App) {
    let username = app.dm_target.and_then(|uid| app.connected_users.iter().find(|u| u.id == uid)).map(|u| u.username.as_str()).unwrap_or("");
    let title = if !username.is_empty() {
        format!("Send Direct Message to {}", username)
    } else {
        "Send Direct Message".to_string()
    };
    let area = draw_centered_rect(f.area(), 50, 20);
    let block = Block::default().title(title).borders(Borders::ALL).border_type(BorderType::Double);
    let input_field = Paragraph::new(app.dm_input.as_str()).wrap(Wrap { trim: true }).block(block);
    f.render_widget(Clear, area);
    f.render_widget(input_field, area);
    f.set_cursor_position((area.x + app.dm_input.len() as u16 + 1, area.y + 1));
}

fn draw_input_popup(f: &mut Frame, app: &App) {
    let title = match app.input_mode {
        Some(InputMode::NewThreadTitle) => "New Thread Title",
        Some(InputMode::NewThreadContent) => "New Thread Content",
        Some(InputMode::NewPostContent) => "Reply Content",
        Some(InputMode::UpdatePassword) => "New Password",
        _ => "Input"
    };
    let area = draw_centered_rect(f.area(), 60, 25);
    let block = Block::default().title(title).borders(Borders::ALL).border_type(BorderType::Double);
    let text_to_render = if matches!(app.input_mode, Some(InputMode::UpdatePassword)) {
        "*".repeat(app.current_input.len())
    } else { app.current_input.clone() };
    let input_field = Paragraph::new(text_to_render).wrap(Wrap { trim: true }).block(block);
    f.render_widget(Clear, area);
    f.render_widget(input_field, area);
    f.set_cursor_position((area.x + app.current_input.len() as u16 + 1, area.y + 1));
}

fn draw_notification_popup(f: &mut Frame, text: String) {
    let area = draw_centered_rect(f.area(), 50, 20);
    let block = Block::default().title("Notification").borders(Borders::ALL).border_type(BorderType::Double);
    let popup_height = area.height.saturating_sub(2);
    let lines: Vec<&str> = text.lines().collect();
    let text_lines = lines.len() as u16;
    let pad_top = (popup_height.saturating_sub(text_lines)) / 2;
    let pad_bottom = popup_height.saturating_sub(pad_top + text_lines);
    let mut content = Vec::new();
    for _ in 0..pad_top { content.push(Line::raw("")); }
    for l in lines.iter() { content.push(Line::from(*l)); }
    for _ in 0..pad_bottom { content.push(Line::raw("")); }
    let p = Paragraph::new(content).wrap(Wrap { trim: true }).block(block).alignment(Alignment::Center);
    f.render_widget(Clear, area);
    f.render_widget(p, area);
}

fn draw_minimal_notification_popup(f: &mut Frame, text: String) {
    let size = f.area();
    let width = 30u16.max(text.len() as u16 + 2).min(size.width / 2);
    let height = 3u16;
    let x = size.x + size.width - width - 2;
    let y = size.y + 1;
    let area = Rect { x, y, width, height };
    let block = Block::default().borders(Borders::ALL).border_type(BorderType::Plain);
    let p = Paragraph::new(text).block(block).alignment(Alignment::Left);
    f.render_widget(Clear, area);
    f.render_widget(p, area);
}

fn draw_profile_view_popup(f: &mut Frame, app: &mut App, profile: &common::UserProfile) {
    let area = draw_centered_rect(f.area(), 70, 60);
    f.render_widget(Clear, area);
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7), // Banner
            Constraint::Min(0),    // Rest
        ])
        .split(area);

    // --- Banner with PFP and Username ---
    let banner_area = layout[0];
    // Dynamically update the composite image to match the banner area
    app.update_profile_banner_composite(banner_area.width - 2, banner_area.height - 2);
    // Add a border to the top of the banner
    let banner_border = Block::default()
        .borders(Borders::TOP)
        .border_type(BorderType::Double);
    f.render_widget(banner_border, banner_area);

    // Split horizontally: [pfp] [username]
    let banner_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(12), // PFP
            Constraint::Min(10),    // Username
        ])
        .split(banner_area);

    // --- Banner background: crop and stretch to fit ---
    // IMPORTANT: Do NOT use get_styled_banner_lines or any glitch effect here!
    if let Some(state) = &mut app.profile_banner_image_state {
        let banner_block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(&profile.username, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
            .style(Style::default());
        f.render_widget(banner_block, banner_area);
        // Only render the composited image (banner + PFP)
        let offset_area = Rect {
            x: banner_area.x + 1,
            y: banner_area.y + 1,
            width: banner_area.width,
            height: banner_area.height,
        };
        let image_widget = StatefulImage::default().resize(ratatui_image::Resize::Fit(None));
        f.render_stateful_widget(image_widget, offset_area, state);
    } else {
        let banner_bg = Color::Blue;
        let banner_block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(&profile.username, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
            .style(Style::default().bg(banner_bg));
        f.render_widget(banner_block, banner_area);
    }

    // --- PFP inside banner, with cropping/clipping ---
    // Do NOT render a separate PFP image if composited image is present
    if app.profile_banner_image_state.is_none() {
        if let Some(state) = &mut app.profile_image_state {
            let pfp_area = banner_chunks[0];
            let image_widget = StatefulImage::default().resize(ratatui_image::Resize::Fit(None));
            f.render_stateful_widget(image_widget, pfp_area, state);
        } else {
            let pfp_block = Block::default().borders(Borders::ALL).title("Profile Pic");
            let pfp_inner = pfp_block.inner(banner_chunks[0]);
            f.render_widget(pfp_block, banner_chunks[0]);
            let placeholder_text = if let Some(pfp_str) = &profile.profile_pic {
                if pfp_str.starts_with("http") { "[Image URL]" }
                else { "[Invalid Image]" }
            } else {
                "[No Pic]"
            };
            let p = Paragraph::new(placeholder_text).alignment(Alignment::Center);
            f.render_widget(p, pfp_inner);
        }
    }

    // --- Rest of profile info below banner ---
    let mut lines = vec![];
    if let Some(bio) = &profile.bio { lines.push(Line::from(vec![Span::styled("Bio: ", Style::default().fg(Color::Cyan)), Span::raw(bio)])); }
    if let Some(loc) = &profile.location { lines.push(Line::from(vec![Span::styled("Location: ", Style::default().fg(Color::Cyan)), Span::raw(loc)])); }
    if let Some(url1) = &profile.url1 { if !url1.is_empty() { lines.push(Line::from(vec![Span::styled("URL1: ", Style::default().fg(Color::Cyan)), Span::raw(url1)])); } }
    if let Some(url2) = &profile.url2 { if !url2.is_empty() { lines.push(Line::from(vec![Span::styled("URL2: ", Style::default().fg(Color::Cyan)), Span::raw(url2)])); } }
    if let Some(url3) = &profile.url3 { if !url3.is_empty() { lines.push(Line::from(vec![Span::styled("URL3: ", Style::default().fg(Color::Cyan)), Span::raw(url3)])); } }
    lines.push(Line::from(vec![Span::styled("Role: ", Style::default().fg(Color::Cyan)), Span::raw(format!("{:?}", profile.role))]));
    let rest = Paragraph::new(lines).wrap(Wrap { trim: true }).block(Block::default().borders(Borders::ALL));
    f.render_widget(rest, layout[1]);
}

fn draw_user_actions_popup(f: &mut Frame, app: &App) {
    let area = draw_centered_rect(f.area(), 40, 20);
    f.render_widget(Clear, area);
    let user = app.user_actions_target.and_then(|idx| app.connected_users.get(idx));
    let username = user.map(|u| u.username.as_str()).unwrap_or("<unknown>");
    let actions = ["Show Profile", "Send DM"];
    let mut lines = vec![Line::from(Span::styled(
        format!("User: {}", username),
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    ))];
    for (i, action) in actions.iter().enumerate() {
        let style = if app.user_actions_selected == i {
            Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        lines.push(Line::from(Span::styled(*action, style)));
    }
    let block = Block::default().title("User Actions").borders(Borders::ALL);
    let para = Paragraph::new(lines).block(block).alignment(Alignment::Left);
    f.render_widget(para, area);
}