//! Popups: notifications, DM input, profile view, user actions, etc.

use ratatui::{Frame, layout::{Rect, Layout, Constraint, Direction}, style::{Style, Color}, widgets::{Block, Paragraph, Borders, BorderType, Clear, Wrap}, text::{Line, Span}, layout::Alignment};
use crate::app::App;
use ratatui::style::Modifier;

pub fn draw_centered_rect(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
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

pub fn draw_dm_input_popup(f: &mut Frame, app: &App) {
    let username = app.chat.dm_target.and_then(|uid| app.chat.channel_userlist.iter().find(|u| u.id == uid)).map(|u| u.username.as_str()).unwrap_or("");
    let title = format!("DM to {}", username);
    let popup_area = draw_centered_rect(f.area(), 80, 30);
    let input_str = &app.chat.dm_input;
    // Calculate popup size based on content
    let base_area = draw_centered_rect(f.area(), 50, 20);
    let input_inner_width = base_area.width.saturating_sub(2); // Account for borders
    
    // Simple estimation for height calculation
    let estimated_lines = if input_inner_width > 0 && !input_str.is_empty() {
        let char_lines = (input_str.len() as u16 + input_inner_width - 1) / input_inner_width;
        let newline_count = input_str.matches('\n').count() as u16;
        (char_lines + newline_count).max(1)
    } else {
        1
    };
    
    // Adjust popup height if needed for multiline content
    let min_height = (estimated_lines + 4).clamp(8, 25); // +4 for borders and title
    let height_percent = if base_area.height < min_height {
        ((min_height as f32 / f.area().height as f32) * 100.0).min(80.0) as u16
    } else {
        20
    };
    
    let area = draw_centered_rect(f.area(), 50, height_percent);
    let block = Block::default().title(Line::from(vec![
        Span::raw("Send Direct Message to "),
        Span::styled(username, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ])).borders(Borders::ALL).border_type(BorderType::Double);
    
    let input_field = Paragraph::new(app.chat.dm_input.as_str()).wrap(Wrap { trim: true }).block(block);
    f.render_widget(Clear, area);
    f.render_widget(input_field, area);
    
    // Calculate cursor position for multiline input
    let inner_area = Block::default().borders(Borders::ALL).inner(area);
    if inner_area.width > 0 && !app.chat.dm_input.is_empty() {
        let cursor_pos = app.chat.dm_input.len();
        let text_up_to_cursor = &app.chat.dm_input[..cursor_pos];
        
        // Count newlines and estimate position
        let newlines = text_up_to_cursor.matches('\n').count() as u16;
        let last_line = text_up_to_cursor.split('\n').last().unwrap_or("");
        let col_in_line = last_line.len() as u16;
        let estimated_col = col_in_line % inner_area.width;
        let estimated_line = newlines + (col_in_line / inner_area.width);
        
        let cursor_y = inner_area.y + estimated_line;
        let cursor_x = inner_area.x + estimated_col;
        
        // Ensure cursor is within bounds
        if cursor_y < inner_area.y + inner_area.height && cursor_x < inner_area.x + inner_area.width {
            f.set_cursor_position((cursor_x, cursor_y));
        }
    } else {
        // Empty input - place cursor at start
        f.set_cursor_position((inner_area.x, inner_area.y));
    }
    
    // Draw mention suggestions popup if present
    crate::ui::chat::draw_mention_suggestion_popup(f, app, area, f.area());
}

pub fn draw_input_popup(f: &mut Frame, app: &App) {
    let title = match app.auth.input_mode {
        Some(crate::state::InputMode::NewThreadTitle) => "New Thread Title",
        Some(crate::state::InputMode::NewThreadContent) => "New Thread Content",
        Some(crate::state::InputMode::NewPostContent) => "Reply Content",
        Some(crate::state::InputMode::UpdatePassword) => "New Password",
        _ => "Input"
    };
    
    // Calculate popup size based on content
    let input_str = if matches!(app.auth.input_mode, Some(crate::state::InputMode::UpdatePassword)) {
        "*".repeat(app.auth.current_input.len())
    } else { 
        app.auth.current_input.clone() 
    };
    
    let base_area = draw_centered_rect(f.area(), 60, 25);
    let input_inner_width = base_area.width.saturating_sub(2); // Account for borders
    
    // Simple estimation for height calculation 
    let estimated_lines = if input_inner_width > 0 && !input_str.is_empty() {
        let char_lines = (input_str.len() as u16 + input_inner_width - 1) / input_inner_width;
        let newline_count = input_str.matches('\n').count() as u16;
        (char_lines + newline_count).max(1)
    } else {
        1
    };
    
    // Adjust popup height if needed for multiline content
    let min_height = (estimated_lines + 4).clamp(8, 30); // +4 for borders and title
    let height_percent = if base_area.height < min_height {
        ((min_height as f32 / f.area().height as f32) * 100.0).min(80.0) as u16
    } else {
        25
    };
    
    let area = draw_centered_rect(f.area(), 60, height_percent);
    let block = Block::default().title(title).borders(Borders::ALL).border_type(BorderType::Double);
    let input_field = Paragraph::new(input_str.clone()).wrap(Wrap { trim: true }).block(block);
    f.render_widget(Clear, area);
    f.render_widget(input_field, area);
    
    // Calculate cursor position for multiline input
    let inner_area = Block::default().borders(Borders::ALL).inner(area);
    if inner_area.width > 0 && !app.auth.current_input.is_empty() {
        let cursor_pos = app.auth.current_input.len();
        let display_text = if matches!(app.auth.input_mode, Some(crate::state::InputMode::UpdatePassword)) {
            "*".repeat(cursor_pos)
        } else {
            app.auth.current_input[..cursor_pos].to_string()
        };
        
        // Count newlines and estimate position
        let newlines = display_text.matches('\n').count() as u16;
        let last_line = display_text.split('\n').last().unwrap_or("");
        let col_in_line = last_line.len() as u16;
        let estimated_col = col_in_line % inner_area.width;
        let estimated_line = newlines + (col_in_line / inner_area.width);
        
        let cursor_y = inner_area.y + estimated_line;
        let cursor_x = inner_area.x + estimated_col;
        
        // Ensure cursor is within bounds
        if cursor_y < inner_area.y + inner_area.height && cursor_x < inner_area.x + inner_area.width {
            f.set_cursor_position((cursor_x, cursor_y));
        }
    } else {
        // Empty input - place cursor at start
        let inner_area = Block::default().borders(Borders::ALL).inner(area);
        f.set_cursor_position((inner_area.x, inner_area.y));
    }
}

pub fn draw_notification_popup(f: &mut Frame, text: String) {
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

pub fn draw_minimal_notification_popup(f: &mut Frame, text: String) {
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

pub fn draw_profile_view_popup(f: &mut Frame, app: &mut App, profile: &common::UserProfile) {
    let area = draw_centered_rect(f.area(), 70, 60);
    f.render_widget(Clear, area);
    
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Banner (increased height)
            Constraint::Min(0),    // Rest
        ])
        .split(area);

    // --- Full-width banner with PFP and Username ---
    let banner_area = layout[0];
    
    // Update the composite image to match the banner area dimensions
    app.update_profile_banner_composite();

    // --- Render banner background: full width, cropped to fill ---
    if let Some(state) = &mut app.profile.profile_banner_image_state {
        // Create a block with borders for the banner
        let banner_block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(&profile.username, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
            .style(Style::default());
        
        // Get the inner area (excluding borders) for the image
        let image_area = banner_block.inner(banner_area);
        
        // Render the block first
        f.render_widget(banner_block, banner_area);
        
        // Render image to fill the inner area completely
        let image_widget = ratatui_image::StatefulImage::default()
            .resize(ratatui_image::Resize::Crop(None)); // Crop to fill instead of fit
        f.render_stateful_widget(image_widget, image_area, state);
        
        // Overlay username text with enhanced styling for better visibility
        let username_area = Rect {
            x: image_area.x + image_area.width.saturating_sub(profile.username.len() as u16 + 2),
            y: image_area.y + image_area.height.saturating_sub(1),
            width: profile.username.len() as u16 + 2,
            height: 1,
        };
        
        let username_text = Paragraph::new(Span::styled(
            &profile.username, 
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        )).alignment(Alignment::Right);
        
        f.render_widget(username_text, username_area);
    } else {
        // Fallback: solid color banner with username
        let banner_bg = Color::Blue;
        let banner_block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(&profile.username, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
            .style(Style::default().bg(banner_bg));
        f.render_widget(banner_block, banner_area);
    }

    // --- Rest of profile info below banner ---
    let content_area = layout[1];
    let mut lines = vec![];
    
    if let Some(bio) = &profile.bio {
        if !bio.is_empty() {
            let mut bio_lines: Vec<&str> = bio.lines().collect();
            if bio_lines.len() > 10 {
                bio_lines.truncate(9);
                bio_lines.push("...");
            }
            lines.push(Line::from(vec![Span::styled("Bio: ", Style::default().fg(Color::Cyan))]));
            for line in bio_lines {
                lines.push(Line::from(Span::raw(line)));
            }
            lines.push(Line::from("")); // Add spacing
        }
    }
    
    if let Some(loc) = &profile.location { 
        if !loc.is_empty() { 
            lines.push(Line::from(vec![
                Span::styled("📍 Location: ", Style::default().fg(Color::Cyan)), 
                Span::raw(loc)
            ])); 
        } 
    }
    
    if let Some(url1) = &profile.url1 { 
        if !url1.is_empty() { 
            lines.push(Line::from(vec![
                Span::styled("🔗 URL1: ", Style::default().fg(Color::Cyan)), 
                Span::raw(url1)
            ])); 
        } 
    }
    
    if let Some(url2) = &profile.url2 { 
        if !url2.is_empty() { 
            lines.push(Line::from(vec![
                Span::styled("🔗 URL2: ", Style::default().fg(Color::Cyan)), 
                Span::raw(url2)
            ])); 
        } 
    }
    
    if let Some(url3) = &profile.url3 { 
        if !url3.is_empty() { 
            lines.push(Line::from(vec![
                Span::styled("🔗 URL3: ", Style::default().fg(Color::Cyan)), 
                Span::raw(url3)
            ])); 
        } 
    }
    
    lines.push(Line::from(vec![
        Span::styled("👑 Role: ", Style::default().fg(Color::Cyan)), 
        Span::styled(format!("{:?}", profile.role), Style::default().fg(Color::Yellow))
    ]));
    
    let content = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL).title("Profile Details"));
    f.render_widget(content, content_area);
}

pub fn draw_user_actions_popup(f: &mut Frame, app: &App) {
    let area = draw_centered_rect(f.area(), 40, 20);
    f.render_widget(Clear, area);
    let user = app.profile.user_actions_target.and_then(|idx| app.chat.channel_userlist.get(idx));
    let username = user.map(|u| u.username.as_str()).unwrap_or("<unknown>");
    let actions = ["Show Profile", "Send DM", "Invite to Server"];
    let mut lines = vec![];
    for (i, action) in actions.iter().enumerate() {
        let style = if app.profile.user_actions_selected == i {
            Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(ratatui::style::Modifier::BOLD)
        } else {
            Style::default()
        };
        lines.push(Line::from(Span::styled(*action, style)));
    }
    let block = Block::default()
        .title(Span::styled(username, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
        .style(Style::default())
        .borders(Borders::ALL);
    let para = Paragraph::new(lines).block(block).alignment(Alignment::Left);
    f.render_widget(para, area);
}

pub fn draw_server_actions_popup(f: &mut Frame, app: &App) {
    let area = draw_centered_rect(f.area(), 40, 20);
    f.render_widget(Clear, area);
    let server_name = app.chat.selected_server.and_then(|s| app.chat.servers.get(s)).map(|srv| srv.name.as_str()).unwrap_or("<server>");
    let is_owner = app.chat.selected_server
        .and_then(|s| app.chat.servers.get(s))
        .and_then(|srv| app.auth.current_user.as_ref().map(|u| u.id == srv.owner))
        .unwrap_or(false);
    let mut actions = vec!["View full user list", "Send invite code"];
    if is_owner {
        actions.push("Server settings");
    }
    let mut lines = vec![];
    for (i, action) in actions.iter().enumerate() {
        let style = if app.ui.server_actions_selected == i {
            Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        lines.push(Line::from(Span::styled(*action, style)));
    }
    let block = Block::default()
        .title(Span::styled(server_name, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
        .style(Style::default())
        .borders(Borders::ALL);
    let para = Paragraph::new(lines).block(block).alignment(Alignment::Left);
    f.render_widget(para, area);
}

pub fn draw_quit_confirm_popup(f: &mut Frame, app: &App) {
    // Try to ensure the popup is tall enough for all content (message + buttons + paddings)
    let mut percent_y = 18u16;
    let percent_x = 40u16;
    let pad_above_msg = 1;
    let pad_between_msg_btn = 1;
    let pad_below_btn = 1;
    let content_lines = pad_above_msg + 1 + pad_between_msg_btn + 1 + pad_below_btn;
    let mut area = draw_centered_rect(f.area(), percent_x, percent_y);
    let mut popup_height = area.height.saturating_sub(2); // minus borders
    // If not enough height, increase percent_y up to 60%
    while popup_height < content_lines && percent_y < 60 {
        percent_y += 5;
        area = draw_centered_rect(f.area(), percent_x, percent_y);
        popup_height = area.height.saturating_sub(2);
    }
    let block = Block::default()
        .title("Are you sure?")
        .borders(Borders::ALL)
        .border_type(BorderType::Double);
    let extra = popup_height.saturating_sub(content_lines);
    let pad_top = extra / 2;
    let pad_bottom = extra - pad_top;
    let mut lines = Vec::new();
    for _ in 0..pad_top { lines.push(Line::from("")); }
    for _ in 0..pad_above_msg { lines.push(Line::from("")); }
    lines.push(Line::from(Span::styled(
        "Do you really want to quit?",
        Style::default().add_modifier(Modifier::BOLD),
    )));
    for _ in 0..pad_between_msg_btn { lines.push(Line::from("")); }
    let yes_style = if app.ui.quit_confirm_selected == 0 {
        Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green)
    };
    let no_style = if app.ui.quit_confirm_selected == 1 {
        Style::default().fg(Color::Black).bg(Color::Red).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Red)
    };
    let buttons = vec![
        Span::styled("[ Yes ]", yes_style),
        Span::raw("  "),
        Span::styled("[ No ]", no_style),
    ];
    lines.push(Line::from(buttons));
    for _ in 0..pad_below_btn { lines.push(Line::from("")); }
    for _ in 0..pad_bottom { lines.push(Line::from("")); }
    let para = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(Clear, area);
    f.render_widget(para, area);
}

pub fn draw_server_invite_selection_popup(f: &mut Frame, app: &App) {
    let area = draw_centered_rect(f.area(), 50, 30);
    f.render_widget(Clear, area);
    
    let user = app.ui.server_invite_target_user
        .and_then(|uid| app.chat.channel_userlist.iter().find(|u| u.id == uid));
    let username = user.map(|u| u.username.as_str()).unwrap_or("<unknown>");
    
    let mut lines = vec![];
    lines.push(Line::from(vec![
        Span::raw("Select server to invite "),
        Span::styled(username, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(" to:"),
    ]));
    lines.push(Line::from(""));
    
    for (i, server) in app.chat.servers.iter().enumerate() {
        let style = if app.ui.server_invite_selected == i {
            Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        lines.push(Line::from(Span::styled(&server.name, style)));
    }
    
    let block = Block::default()
        .title("Invite to Server")
        .style(Style::default())
        .borders(Borders::ALL);
    let para = Paragraph::new(lines).block(block).alignment(Alignment::Left);
    f.render_widget(para, area);
}
