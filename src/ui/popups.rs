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

pub fn draw_input_popup(f: &mut Frame, app: &App) {
    let title = match app.input_mode {
        Some(crate::app::InputMode::NewThreadTitle) => "New Thread Title",
        Some(crate::app::InputMode::NewThreadContent) => "New Thread Content",
        Some(crate::app::InputMode::NewPostContent) => "Reply Content",
        Some(crate::app::InputMode::UpdatePassword) => "New Password",
        _ => "Input"
    };
    let area = draw_centered_rect(f.area(), 60, 25);
    let block = Block::default().title(title).borders(Borders::ALL).border_type(BorderType::Double);
    let text_to_render = if matches!(app.input_mode, Some(crate::app::InputMode::UpdatePassword)) {
        "*".repeat(app.current_input.len())
    } else { app.current_input.clone() };
    let input_field = Paragraph::new(text_to_render).wrap(Wrap { trim: true }).block(block);
    f.render_widget(Clear, area);
    f.render_widget(input_field, area);
    f.set_cursor_position((area.x + app.current_input.len() as u16 + 1, area.y + 1));
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
        let image_widget = ratatui_image::StatefulImage::default().resize(ratatui_image::Resize::Fit(None));
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
            let image_widget = ratatui_image::StatefulImage::default().resize(ratatui_image::Resize::Fit(None));
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
    if let Some(bio) = &profile.bio {
        let mut bio_lines: Vec<&str> = bio.lines().collect();
        if bio_lines.len() > 10 {
            bio_lines.truncate(9);
            bio_lines.push("...");
        }
        lines.push(Line::from(vec![Span::styled("Bio: ", Style::default().fg(Color::Cyan))]));
        for line in bio_lines {
            lines.push(Line::from(Span::raw(line)));
        }
    }
    if let Some(loc) = &profile.location { lines.push(Line::from(vec![Span::styled("Location: ", Style::default().fg(Color::Cyan)), Span::raw(loc)])); }
    if let Some(url1) = &profile.url1 { if !url1.is_empty() { lines.push(Line::from(vec![Span::styled("URL1: ", Style::default().fg(Color::Cyan)), Span::raw(url1)])); } }
    if let Some(url2) = &profile.url2 { if !url2.is_empty() { lines.push(Line::from(vec![Span::styled("URL2: ", Style::default().fg(Color::Cyan)), Span::raw(url2)])); } }
    if let Some(url3) = &profile.url3 { if !url3.is_empty() { lines.push(Line::from(vec![Span::styled("URL3: ", Style::default().fg(Color::Cyan)), Span::raw(url3)])); } }
    lines.push(Line::from(vec![Span::styled("Role: ", Style::default().fg(Color::Cyan)), Span::raw(format!("{:?}", profile.role))]));
    let rest = Paragraph::new(lines).wrap(Wrap { trim: true }).block(Block::default().borders(Borders::ALL));
    f.render_widget(rest, layout[1]);
}

pub fn draw_user_actions_popup(f: &mut Frame, app: &App) {
    let area = draw_centered_rect(f.area(), 40, 20);
    f.render_widget(Clear, area);
    let user = app.user_actions_target.and_then(|idx| app.connected_users.get(idx));
    let username = user.map(|u| u.username.as_str()).unwrap_or("<unknown>");
    let actions = ["Show Profile", "Send DM"];
    let mut lines = vec![Line::from(Span::styled(
        format!("User: {}", username),
        Style::default().fg(Color::Yellow).add_modifier(ratatui::style::Modifier::BOLD),
    ))];
    for (i, action) in actions.iter().enumerate() {
        let style = if app.user_actions_selected == i {
            Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(ratatui::style::Modifier::BOLD)
        } else {
            Style::default()
        };
        lines.push(Line::from(Span::styled(*action, style)));
    }
    let block = Block::default().title("User Actions").borders(Borders::ALL);
    let para = Paragraph::new(lines).block(block).alignment(Alignment::Left);
    f.render_widget(para, area);
}
