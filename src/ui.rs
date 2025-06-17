use crate::app::{App, AppMode, InputMode};
use crate::banner::get_styled_banner_lines;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn ui(f: &mut Frame, app: &mut App) {
    let size = f.size();

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
        if let Some(profile) = &app.profile_view {
            draw_profile_view_popup(f, profile);
        }
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
        f.set_cursor(chunks[0].x + app.current_input.len() as u16 + 1, chunks[0].y + 1);
    } else if let Some(InputMode::LoginPassword) = &app.input_mode {
        f.set_cursor(chunks[1].x + app.password_input.len() as u16 + 1, chunks[1].y + 1);
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
        f.set_cursor(chunks[0].x + app.current_input.len() as u16 + 1, chunks[0].y + 1);
    } else if let Some(InputMode::RegisterPassword) = &app.input_mode {
        f.set_cursor(chunks[1].x + app.password_input.len() as u16 + 1, chunks[1].y + 1);
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
            Constraint::Length(3), // Bio
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
    // Inline Paragraphs for each field to avoid closure lifetime issues
    let bio_style = if app.profile_edit_focus == Bio {
        Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else { Style::default() };
    f.render_widget(
        Paragraph::new(app.edit_bio.clone())
            .block(Block::default().borders(Borders::ALL).title("Bio").border_style(bio_style))
            .style(bio_style),
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
        Bio => (inner[0].x + app.edit_bio.len() as u16 + 1, inner[0].y + 1),
        Url1 => (inner[1].x + app.edit_url1.len() as u16 + 1, inner[1].y + 1),
        Url2 => (inner[2].x + app.edit_url2.len() as u16 + 1, inner[2].y + 1),
        Url3 => (inner[3].x + app.edit_url3.len() as u16 + 1, inner[3].y + 1),
        Location => (inner[4].x + app.edit_location.len() as u16 + 1, inner[4].y + 1),
        ProfilePic => (inner[5].x + app.edit_profile_pic.len() as u16 + 1, inner[5].y + 1),
        CoverBanner => (inner[6].x + app.edit_cover_banner.len() as u16 + 1, inner[6].y + 1),
        _ => (0, 0),
    };
    if matches!(app.profile_edit_focus, Bio|Url1|Url2|Url3|Location|ProfilePic|CoverBanner) {
        f.set_cursor(cursor.0, cursor.1);
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
    let messages: Vec<ListItem> = app.chat_messages.iter().rev().take(chunks[0].height as usize).rev().map(|msg| {
        ListItem::new(Line::from(vec![
            Span::styled(format!("<{}>: ", msg.author), Style::default().fg(msg.color).add_modifier(Modifier::BOLD)),
            Span::raw(msg.content.clone()),
        ]))
    }).collect();
    let messages_list = List::new(messages).block(Block::default().borders(Borders::ALL).title("Live Chat // #general").border_style(border_style));
    f.render_widget(messages_list, chunks[0]);
    let input = Paragraph::new(app.current_input.as_str()).style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[1]);
    f.set_cursor(chunks[1].x + app.current_input.len() as u16 + 1, chunks[1].y + 1);
}

fn draw_user_list(f: &mut Frame, app: &App, area: Rect, focused: bool) {
    let border_style = if focused {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let items: Vec<ListItem> = app.connected_users.iter().map(|user| {
        ListItem::new(Line::from(vec![
            Span::styled(&user.username, Style::default().fg(user.color)),
            Span::raw(format!(" ({:?})", user.role)),
        ]))
    }).collect();
    let mut list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Users [U]").border_style(border_style));
    if focused {
        list = list.highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black));
    } else {
        list = list.highlight_style(Style::default());
    }
    f.render_stateful_widget(list, area, &mut app.forum_list_state.clone());
}

fn draw_dm_input_popup(f: &mut Frame, app: &App) {
    let username = app.dm_target.and_then(|uid| app.connected_users.iter().find(|u| u.id == uid)).map(|u| u.username.as_str()).unwrap_or("");
    let title = if !username.is_empty() {
        format!("Send Direct Message to {}", username)
    } else {
        "Send Direct Message".to_string()
    };
    let area = draw_centered_rect(f.size(), 50, 20);
    let block = Block::default().title(title).borders(Borders::ALL).border_type(BorderType::Double);
    let input_field = Paragraph::new(app.dm_input.as_str()).wrap(Wrap { trim: true }).block(block);
    f.render_widget(Clear, area);
    f.render_widget(input_field, area);
    f.set_cursor(area.x + app.dm_input.len() as u16 + 1, area.y + 1);
}

fn draw_input_popup(f: &mut Frame, app: &mut App) {
    let title = match app.input_mode {
        Some(InputMode::NewThreadTitle) => "New Thread Title",
        Some(InputMode::NewThreadContent) => "New Thread Content",
        Some(InputMode::NewPostContent) => "Reply Content",
        Some(InputMode::UpdatePassword) => "New Password",
        _ => "Input"
    };
    let area = draw_centered_rect(f.size(), 60, 25);
    let block = Block::default().title(title).borders(Borders::ALL).border_type(BorderType::Double);
    let text_to_render = if matches!(app.input_mode, Some(InputMode::UpdatePassword)) {
        "*".repeat(app.current_input.len())
    } else { app.current_input.clone() };
    let input_field = Paragraph::new(text_to_render).wrap(Wrap { trim: true }).block(block);
    f.render_widget(Clear, area);
    f.render_widget(input_field, area);
    f.set_cursor(area.x + app.current_input.len() as u16 + 1, area.y + 1);
}

fn draw_notification_popup(f: &mut Frame, text: String) {
    let area = draw_centered_rect(f.size(), 50, 20);
    let block = Block::default().title("Notification").borders(Borders::ALL).border_type(BorderType::Double);
    // Vertically center the text in the popup
    let popup_height = area.height.saturating_sub(2); // minus borders
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
    let size = f.size();
    // Minimal size: 30x3 or fit text
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

fn draw_profile_view_popup(f: &mut Frame, profile: &common::UserProfile) {
    let area = draw_centered_rect(f.size(), 60, 60);
    let block = Block::default().title(format!("Profile: {}", profile.username)).borders(Borders::ALL).border_type(BorderType::Double);
    let mut lines = vec![
        Line::from(vec![Span::styled("Bio:", Style::default().fg(Color::Yellow)), Span::raw(" "), Span::raw(profile.bio.as_deref().unwrap_or(""))]),
        Line::from(vec![Span::styled("Location:", Style::default().fg(Color::Yellow)), Span::raw(" "), Span::raw(profile.location.as_deref().unwrap_or(""))]),
        Line::from(vec![Span::styled("URL1:", Style::default().fg(Color::Yellow)), Span::raw(" "), Span::raw(profile.url1.as_deref().unwrap_or(""))]),
        Line::from(vec![Span::styled("URL2:", Style::default().fg(Color::Yellow)), Span::raw(" "), Span::raw(profile.url2.as_deref().unwrap_or(""))]),
        Line::from(vec![Span::styled("URL3:", Style::default().fg(Color::Yellow)), Span::raw(" "), Span::raw(profile.url3.as_deref().unwrap_or(""))]),
        Line::from(vec![Span::styled("Profile Pic:", Style::default().fg(Color::Yellow)), Span::raw(" "), Span::raw(profile.profile_pic.as_deref().unwrap_or(""))]),
        Line::from(vec![Span::styled("Cover Banner:", Style::default().fg(Color::Yellow)), Span::raw(" "), Span::raw(profile.cover_banner.as_deref().unwrap_or(""))]),
        Line::from(vec![Span::styled("[Esc] Close", Style::default().fg(Color::Red))]),
    ];
    let p = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
    f.render_widget(Clear, area);
    f.render_widget(p, area);
}