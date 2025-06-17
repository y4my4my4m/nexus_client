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
    }

    if let Some((notification, _, minimal)) = &app.notification {
        if *minimal {
            draw_minimal_notification_popup(f, notification.clone());
        } else {
            draw_notification_popup(f, notification.clone());
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
    let items = vec![ListItem::new("Change Password"), ListItem::new("Change User Color (Cycle)")];
    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Settings"))
        .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black)).highlight_symbol(">> ");
    f.render_stateful_widget(list, area, &mut app.settings_list_state);
}

fn draw_chat(f: &mut Frame, app: &mut App, area: Rect) {
    if app.show_user_list {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(area);
        draw_chat_main(f, app, chunks[0]);
        draw_user_list(f, app, chunks[1]);
    } else {
        draw_chat_main(f, app, area);
    }
}

fn draw_chat_main(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default().constraints([Constraint::Min(0), Constraint::Length(3)]).split(area);
    let messages: Vec<ListItem> = app.chat_messages.iter().rev().take(chunks[0].height as usize).rev().map(|msg| {
        ListItem::new(Line::from(vec![
            Span::styled(format!("<{}>: ", msg.author), Style::default().fg(msg.color).add_modifier(Modifier::BOLD)),
            Span::raw(msg.content.clone()),
        ]))
    }).collect();
    let messages_list = List::new(messages).block(Block::default().borders(Borders::ALL).title("Live Chat // #general"));
    f.render_widget(messages_list, chunks[0]);
    let input = Paragraph::new(app.current_input.as_str()).style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[1]);
    f.set_cursor(chunks[1].x + app.current_input.len() as u16 + 1, chunks[1].y + 1);
}

fn draw_user_list(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app.connected_users.iter().map(|user| {
        ListItem::new(Line::from(vec![
            Span::styled(&user.username, Style::default().fg(user.color)),
            Span::raw(format!(" ({:?})", user.role)),
        ]))
    }).collect();
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Users [U]"))
        .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black));
    f.render_widget(list, area);
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