//! Main menu UI screen with cyberpunk aesthetics.

use ratatui::{Frame, layout::{Rect, Layout, Constraint, Direction, Alignment}, style::{Style, Color, Modifier}, widgets::{Block, List, ListItem, Borders, Paragraph, BorderType}, text::{Line, Span}};
use crate::app::App;

pub fn draw_main_menu(f: &mut Frame, app: &mut App, area: Rect) {
    // Draw animated background using selected background
    if let Some(bg) = app.background_manager.get_current_background() {
        bg.draw_background(f, app, area);
    }

    // Calculate responsive layout based on available height
    let available_height = area.height;
    let title_height = if available_height < 15 { 0 } else { 2 };
    let status_height = if available_height < 20 { 0 } else { 6 };
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(title_height),
            Constraint::Min(12),
            Constraint::Length(status_height),
        ])
        .margin(1)
        .split(area);

    // Draw top banner via theme
    if title_height > 0 {
        app.theme_manager.get_current_theme().draw_top_banner(f, app, main_layout[0]);
    }
    // Draw main menu via theme (pass UI state, tick, area)
    app.theme_manager.get_current_theme().draw_main_menu(
        f,
        &mut app.ui.main_menu_state,
        app.ui.tick_count,
        main_layout[1],
    );
    // Draw status section (mutably borrows app, so do not use theme after this)
    if status_height > 0 {
        draw_enhanced_status(f, app, main_layout[2]);
    }
    // Draw bottom banner via theme (do NOT use theme variable)
    app.theme_manager.get_current_theme().draw_bottom_banner(f, app, area);
    draw_floating_elements(f, app, area);
}

fn draw_enhanced_status(f: &mut Frame, app: &mut App, area: Rect) {
    let tick = app.ui.tick_count;
    
    let status_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),  // User info
            Constraint::Percentage(33),  // System status
            Constraint::Percentage(34),  // Network status
        ])
        .split(area);
    
    if let Some(user) = &app.auth.current_user {
        // Enhanced user profile with animations
        let user_text = vec![
            Line::from(vec![
                Span::styled("◢", Style::default().fg(Color::Yellow)),
                Span::styled(" USER: ", Style::default().fg(Color::Gray)),
                Span::styled(&user.username, Style::default().fg(user.color.clone().into()).add_modifier(Modifier::BOLD)),
                Span::styled(" ◣", Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("◢", Style::default().fg(Color::LightMagenta)),
                Span::styled(" ROLE: ", Style::default().fg(Color::Gray)),
                Span::styled(format!("{:?}", user.role), Style::default().fg(Color::LightMagenta)),
                Span::styled(" ◣", Style::default().fg(Color::LightMagenta)),
            ]),
            Line::from(vec![
                Span::styled("◢", Style::default().fg(Color::Green)),
                Span::styled(" SESSION: ", Style::default().fg(Color::Gray)),
                Span::styled("ACTIVE", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(" ◣", Style::default().fg(Color::Green)),
            ]),
        ];
        
        let user_block = Paragraph::new(user_text)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title("◆ USER PROFILE ◆")
                    .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Thick)
                    .border_style(Style::default().fg(Color::Yellow))
            );
        
        // System status with pulsing indicators
        let pulse_char = if tick % 20 < 10 { "●" } else { "○" };
        let system_text = vec![
            Line::from(vec![
                Span::styled("◢ CPU: ", Style::default().fg(Color::Gray)),
                Span::styled(pulse_char, Style::default().fg(Color::Green)),
                Span::styled(" OPTIMAL", Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled("◢ NET: ", Style::default().fg(Color::Gray)),
                Span::styled(pulse_char, Style::default().fg(Color::Green)),
                Span::styled(" SECURE", Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled("◢ ENC: ", Style::default().fg(Color::Gray)),
                Span::styled(pulse_char, Style::default().fg(Color::Green)),
                Span::styled(" AES-256", Style::default().fg(Color::Green)),
            ]),
        ];
        
        let system_block = Paragraph::new(system_text)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title("◆ SYSTEM STATUS ◆")
                    .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Thick)
                    .border_style(Style::default().fg(Color::Green))
            );
        
        // Network status with data flow visualization
        let flow_indicator = match (tick / 5) % 4 {
            0 => "▶──",
            1 => "─▶─",
            2 => "──▶",
            _ => "▶▶▶",
        };
        
        let network_text = vec![
            Line::from(vec![
                Span::styled("◢ UPLINK: ", Style::default().fg(Color::Gray)),
                Span::styled(flow_indicator, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("◢ LATENCY: ", Style::default().fg(Color::Gray)),
                Span::styled("12ms", Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled("◢ BANDWIDTH: ", Style::default().fg(Color::Gray)),
                Span::styled("1Gb/s", Style::default().fg(Color::Green)),
            ]),
        ];
        
        let network_block = Paragraph::new(network_text)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title("◆ NETWORK STATUS ◆")
                    .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Thick)
                    .border_style(Style::default().fg(Color::Cyan))
            );
        
        f.render_widget(user_block, status_layout[0]);
        f.render_widget(system_block, status_layout[1]);
        f.render_widget(network_block, status_layout[2]);
    } else {
        // Disconnected state with warning animations
        let warning_color = if tick % 20 < 10 { Color::Red } else { Color::Yellow };
        let disconnected_text = vec![
            Line::from(vec![
                Span::styled("⚠ NEURAL LINK: ", Style::default().fg(Color::Gray)),
                Span::styled("DISCONNECTED", Style::default().fg(warning_color).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("⚠ STATUS: ", Style::default().fg(Color::Gray)),
                Span::styled("UNAUTHORIZED ACCESS", Style::default().fg(Color::Red)),
            ]),
            Line::from(vec![
                Span::styled("⚠ SECURITY: ", Style::default().fg(Color::Gray)),
                Span::styled("LOCKED", Style::default().fg(Color::Red)),
            ]),
        ];
        
        let disconnected_block = Paragraph::new(disconnected_text)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title("◆ ACCESS DENIED ◆")
                    .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Thick)
                    .border_style(Style::default().fg(warning_color))
            );
        
        f.render_widget(disconnected_block, area);
    }
}

fn draw_floating_elements(f: &mut Frame, app: &mut App, area: Rect) {
    let tick = app.ui.tick_count;
    
    // Floating corner indicators
    let corners = [
        (area.x, area.y, "◢"),
        (area.x + area.width - 1, area.y, "◣"),
        (area.x, area.y + area.height - 1, "◥"),
        (area.x + area.width - 1, area.y + area.height - 1, "◤"),
    ];
    
    for (x, y, corner_char) in corners {
        let corner_color = match (tick / 10 + (x as u64 + y as u64)) % 4 {
            0 => Color::Cyan,
            1 => Color::Magenta,
            2 => Color::Yellow,
            _ => Color::Green,
        };
        
        let corner_area = Rect::new(x, y, 1, 1);
        f.render_widget(
            Paragraph::new(corner_char).style(Style::default().fg(corner_color).add_modifier(Modifier::BOLD)),
            corner_area
        );
    }
    
    // Floating time/tick counter
    let time_area = Rect::new(area.x + area.width - 20, area.y + 1, 18, 1);
    f.render_widget(
        Paragraph::new(format!("◈ TICK: {:06} ◈", tick))
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Right),
        time_area
    );
}
