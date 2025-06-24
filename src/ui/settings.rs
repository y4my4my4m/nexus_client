//! Settings and profile editing UI screens with cyberpunk aesthetics.

use ratatui::{Frame, layout::Rect, style::{Style, Color, Modifier}, widgets::{Block, List, ListItem, Paragraph, Borders, BorderType, Wrap}, text::{Line, Span}, layout::Constraint, layout::Layout};
use ratatui::prelude::{Alignment, Direction};
use crate::app::{App};
use base64::Engine;
use crate::ui::themes::Theme;

pub fn draw_settings(f: &mut Frame, app: &mut App, area: Rect) {
    // Draw animated background using selected background
    if let Some(bg) = app.background_manager.get_current_background() {
        bg.draw_background(f, app, area);
    }

    let tick = app.ui.tick_count;
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Top border
            Constraint::Min(8),     // Settings list
            Constraint::Length(2),  // Bottom border
        ])
        .margin(1)
        .split(area);

    // Draw top banner via theme
    if main_layout[0].height > 0 {
        app.theme_manager.get_current_theme().draw_top_banner(f, app, main_layout[0]);
    }
    app.theme_manager.get_current_theme().draw_settings_menu(
        f,
        &mut app.ui.settings_list_state,
        app.ui.tick_count,
        main_layout[1],
    );
    app.theme_manager.get_current_theme().draw_bottom_banner(f, app, main_layout[2]);
}

fn draw_settings_background(f: &mut Frame, app: &mut App, area: Rect) {
    let tick = app.ui.tick_count;
    
    // Create animated grid pattern similar to main menu but more subtle
    for y in 0..area.height {
        for x in 0..area.width {
            let grid_x = x as usize;
            let grid_y = y as usize;
            let time_offset = (tick / 6) as usize; // Slower animation
            
            // Create subtle moving wave pattern
            let wave1 = ((grid_x + time_offset) % 25 == 0) as u8;
            let wave2 = ((grid_y + time_offset / 3) % 20 == 0) as u8;
            let pulse = ((grid_x + grid_y + time_offset) % 40 < 2) as u8;
            
            let intensity = wave1 + wave2 + pulse;
            
            let (char, color) = match intensity {
                3 => ('┼', Color::DarkGray),
                2 => ('·', Color::DarkGray),
                1 => ('▪', Color::DarkGray),
                _ => {
                    // Very sparse random noise
                    if (grid_x * 11 + grid_y * 13 + time_offset) % 300 == 0 {
                        ('░', Color::DarkGray)
                    } else {
                        (' ', Color::Black)
                    }
                }
            };
            
            if char != ' ' {
                let cell_area = Rect::new(area.x + x, area.y + y, 1, 1);
                f.render_widget(
                    Paragraph::new(char.to_string()).style(Style::default().fg(color)),
                    cell_area
                );
            }
        }
    }
}

fn draw_settings_border(f: &mut Frame, app: &mut App, area: Rect, is_top: bool) {
    let tick = app.ui.tick_count;
    
    let border_chars: String = (0..area.width)
        .map(|x| {
            let phase = (x as u64 + tick / 3) % 25;
            match phase {
                0..=2 => if is_top { '▲' } else { '▼' },
                3..=5 => if is_top { '△' } else { '▽' },
                20..=22 => if is_top { '◣' } else { '◤' },
                23..=24 => if is_top { '◢' } else { '◥' },
                _ => '═',
            }
        })
        .collect();
    
    let border_color = if is_top { Color::LightCyan } else { Color::Green };
    f.render_widget(
        Paragraph::new(border_chars)
            .style(Style::default().fg(border_color).add_modifier(Modifier::BOLD)),
        area
    );
}

fn draw_settings_info_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let tick = app.ui.tick_count;
    let selected = app.ui.settings_list_state.selected().unwrap_or(0);
    let current_theme_name = app.theme_manager.get_theme_name();
    
    let info_content = if app.auth.is_logged_in() {
        match selected {
            0 => vec![
                Line::from(vec![Span::styled("SECURITY UPDATE", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))]),
                Line::from(Span::raw("")),
                Line::from(vec![Span::styled("▶ Password Authentication", Style::default().fg(Color::White))]),
                Line::from(vec![Span::styled("▶ Encryption Key Rotation", Style::default().fg(Color::White))]),
                Line::from(vec![Span::styled("▶ Session Invalidation", Style::default().fg(Color::White))]),
                Line::from(vec![Span::styled("▶ Access Control Update", Style::default().fg(Color::White))]),
                Line::from(Span::raw("")),
                Line::from(vec![Span::styled("Security Level: ", Style::default().fg(Color::Gray)), 
                             Span::styled("HIGH", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))]),
            ],
            1 => vec![
                Line::from(vec![Span::styled("VISUAL IDENTITY", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))]),
                Line::from(Span::raw("")),
                Line::from(vec![Span::styled("▶ Color Palette Selection", Style::default().fg(Color::White))]),
                Line::from(vec![Span::styled("▶ Theme Configuration", Style::default().fg(Color::White))]),
                Line::from(vec![Span::styled("▶ Visual Signature", Style::default().fg(Color::White))]),
                Line::from(vec![Span::styled("▶ Display Preferences", Style::default().fg(Color::White))]),
                Line::from(Span::raw("")),
                Line::from(vec![Span::styled("Current Theme: ", Style::default().fg(Color::Gray)),
                             Span::styled(current_theme_name.to_uppercase(), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))]),
                Line::from(vec![Span::styled("Press F8: ", Style::default().fg(Color::Gray)),
                             Span::styled("Cycle Theme", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
            ],
            2 => vec![
                Line::from(vec![Span::styled("USER PROFILE", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
                Line::from(Span::raw("")),
                Line::from(vec![Span::styled("▶ Personal Information", Style::default().fg(Color::White))]),
                Line::from(vec![Span::styled("▶ Avatar Management", Style::default().fg(Color::White))]),
                Line::from(vec![Span::styled("▶ Bio & Social Links", Style::default().fg(Color::White))]),
                Line::from(vec![Span::styled("▶ Privacy Settings", Style::default().fg(Color::White))]),
                Line::from(Span::raw("")),
                Line::from(vec![Span::styled("Profile Status: ", Style::default().fg(Color::Gray)),
                             Span::styled("ACTIVE", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))]),
            ],
            _ => vec![
                Line::from(vec![Span::styled("CLIENT CONFIG", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
                Line::from(Span::raw("")),
                Line::from(vec![Span::styled("▶ Audio Settings", Style::default().fg(Color::White))]),
                Line::from(vec![Span::styled("▶ Visual Effects", Style::default().fg(Color::White))]),
                Line::from(vec![Span::styled("▶ Notifications", Style::default().fg(Color::White))]),
                Line::from(vec![Span::styled("▶ Performance Tuning", Style::default().fg(Color::White))]),
                Line::from(Span::raw("")),
                Line::from(vec![Span::styled("Active Theme: ", Style::default().fg(Color::Gray)),
                             Span::styled(current_theme_name.to_uppercase(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
                Line::from(vec![Span::styled("Press F7: ", Style::default().fg(Color::Gray)),
                             Span::styled("Cycle Background", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
                Line::from(vec![Span::styled("Press F8: ", Style::default().fg(Color::Gray)),
                             Span::styled("Cycle Theme", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
            ],
        }
    } else {
        match selected {
            0 => vec![
                Line::from(vec![Span::styled("SECURITY UPDATE", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))]),
                Line::from(Span::raw("")),
                Line::from(vec![Span::styled("▶ Password Authentication", Style::default().fg(Color::White))]),
                Line::from(vec![Span::styled("▶ Encryption Key Rotation", Style::default().fg(Color::White))]),
                Line::from(vec![Span::styled("▶ Session Management", Style::default().fg(Color::White))]),
                Line::from(Span::raw("")),
                Line::from(vec![Span::styled("Status: ", Style::default().fg(Color::Gray)), 
                             Span::styled("GUEST MODE", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
            ],
            1 => vec![
                Line::from(vec![Span::styled("VISUAL IDENTITY", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))]),
                Line::from(Span::raw("")),
                Line::from(vec![Span::styled("▶ Color Palette Selection", Style::default().fg(Color::White))]),
                Line::from(vec![Span::styled("▶ Theme Configuration", Style::default().fg(Color::White))]),
                Line::from(vec![Span::styled("▶ Local Preferences", Style::default().fg(Color::White))]),
                Line::from(Span::raw("")),
                Line::from(vec![Span::styled("Current Theme: ", Style::default().fg(Color::Gray)),
                             Span::styled(current_theme_name.to_uppercase(), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))]),
                Line::from(vec![Span::styled("Press F8: ", Style::default().fg(Color::Gray)),
                             Span::styled("Cycle Theme", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
            ],
            _ => vec![
                Line::from(vec![Span::styled("CLIENT CONFIG", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
                Line::from(Span::raw("")),
                Line::from(vec![Span::styled("▶ Audio Settings", Style::default().fg(Color::White))]),
                Line::from(vec![Span::styled("▶ Visual Effects", Style::default().fg(Color::White))]),
                Line::from(vec![Span::styled("▶ Local Configuration", Style::default().fg(Color::White))]),
                Line::from(Span::raw("")),
                Line::from(vec![Span::styled("Active Theme: ", Style::default().fg(Color::Gray)),
                             Span::styled(current_theme_name.to_uppercase(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
                Line::from(vec![Span::styled("Press F7: ", Style::default().fg(Color::Gray)),
                            Span::styled("Cycle Background", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
                Line::from(vec![Span::styled("Press F8: ", Style::default().fg(Color::Gray)),
                             Span::styled("Cycle Theme", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
            ],
        }
    };
    
    let pulse_color = match (tick / 8) % 3 {
        0 => Color::Cyan,
        1 => Color::Blue,
        _ => Color::LightBlue,
    };
    
    let info_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(pulse_color))
        .title("◈ CONFIG INFO ◈")
        .title_style(Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD));
    
    f.render_widget(
        Paragraph::new(info_content)
            .block(info_block)
            .alignment(Alignment::Left),
        area
    );
}

pub fn draw_profile_edit_page(f: &mut Frame, app: &mut App, area: Rect) {
    use crate::app::ProfileEditFocus::*;
    let min_two_col_width = 110; // Increased for more breathing room
    let is_narrow = area.width < min_two_col_width;
    // Outer card with more margin
    let block = Block::default()
        .title(Span::styled("✦ Edit Your Profile ✦", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .style(Style::default().bg(Color::Black));
    let card_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .split(area)[0];
    f.render_widget(&block, card_area);
    let inner = block.inner(card_area);
    let padded = Layout::default()
        .direction(Direction::Vertical)
        .margin(2) // More padding inside card
        .constraints([Constraint::Min(0)])
        .split(inner)[0];

    if !is_narrow {
        // --- Two-column layout ---
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(52),
                Constraint::Percentage(48),
            ])
            .margin(0)
            .split(padded);
        let left = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Section header
                Constraint::Length(2), // Padding
                Constraint::Length(5), // Bio
                Constraint::Length(2), // Padding
                Constraint::Length(3), // Location
                Constraint::Length(2), // Padding
                Constraint::Length(3), // URL1
                Constraint::Length(2), // Padding
                Constraint::Length(3), // URL2
                Constraint::Length(2), // Padding
                Constraint::Length(3), // URL3
                Constraint::Min(0),
            ])
            .split(columns[0]);
        // Add left padding to the right column
        let right_area = Rect {
            x: columns[1].x + 2, // 2 chars padding
            y: columns[1].y,
            width: columns[1].width.saturating_sub(2),
            height: columns[1].height,
        };
        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Section header
                Constraint::Length(1), // (i) image info
                Constraint::Length(2), // Padding
                Constraint::Length(5), // Profile Pic preview
                Constraint::Length(2), // Padding
                Constraint::Length(3), // Profile Pic field+delete
                Constraint::Length(2), // Padding
                Constraint::Length(5), // Banner preview
                Constraint::Length(2), // Padding
                Constraint::Length(3), // Banner field+delete
                Constraint::Length(2), // Padding
                Constraint::Length(3), // Save/Cancel
                Constraint::Length(2), // Padding
                Constraint::Min(0),    // Error
            ])
            .split(right_area);
        // --- LEFT COLUMN: Text fields ---
        // Section header
        f.render_widget(Paragraph::new(Span::styled("Personal Info", Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD))).alignment(Alignment::Left), left[0]);
        // Bio
        let bio_style = if app.profile.profile_edit_focus == Bio {
            Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        f.render_widget(
            Paragraph::new(app.profile.edit_bio.as_str())
                .block(Block::default().borders(Borders::ALL).title("📝 Bio").border_style(bio_style))
                .style(bio_style)
                .wrap(Wrap { trim: false }),
            left[2],
        );
        // Location
        let location_style = if app.profile.profile_edit_focus == Location {
            Style::default().fg(Color::Black).bg(Color::LightCyan).add_modifier(Modifier::BOLD)
        } else { Style::default().bg(Color::DarkGray) };
        f.render_widget(
            Paragraph::new(app.profile.edit_location.clone())
                .block(Block::default().borders(Borders::ALL).title("📍 Location").border_style(location_style))
                .style(location_style),
            left[4],
        );
        // URLs
        let url_titles = ["🔗 URL1", "🔗 URL2", "🔗 URL3"];
        let url_fields = [&app.profile.edit_url1, &app.profile.edit_url2, &app.profile.edit_url3];
        let url_focus = [Url1, Url2, Url3];
        for i in 0..3 {
            let style = if app.profile.profile_edit_focus == url_focus[i] {
                Style::default().fg(Color::Black).bg(Color::LightCyan).add_modifier(Modifier::BOLD)
            } else { Style::default().bg(Color::DarkGray) };
            f.render_widget(
                Paragraph::new(url_fields[i].clone())
                    .block(Block::default().borders(Borders::ALL).title(url_titles[i]).border_style(style))
                    .style(style),
                left[6 + i * 2],
            );
        }

        // --- RIGHT COLUMN: Images and actions ---
        // Section header
        f.render_widget(Paragraph::new(Span::styled("Profile Images", Style::default().fg(Color::LightMagenta).add_modifier(Modifier::BOLD))).alignment(Alignment::Left), right[0]);
        f.render_widget(Paragraph::new(Span::styled(
            "(i) Image: local file path, under 1MB, no spaces.",
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)
        )).alignment(Alignment::Left), right[1]);
        // Profile Pic preview
        let pic_style = if app.profile.profile_edit_focus == ProfilePic {
            Style::default().fg(Color::Black).bg(Color::LightMagenta).add_modifier(Modifier::BOLD)
        } else { Style::default().bg(Color::DarkGray) };
        if !app.profile.edit_profile_pic.trim().is_empty() {
            let mut show_placeholder = true;
            if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(app.profile.edit_profile_pic.trim()) {
                if let Ok(img) = image::load_from_memory(&bytes) {
                    let mut protocol = app.profile.picker.new_resize_protocol(img);
                    let image_widget = ratatui_image::StatefulImage::default().resize(ratatui_image::Resize::Fit(None));
                    f.render_stateful_widget(image_widget, right[3], &mut protocol);
                    show_placeholder = false;
                }
            }
            if show_placeholder {
                let preview_block = Block::default().borders(Borders::ALL).title("Profile Pic Preview").style(pic_style);
                f.render_widget(preview_block, right[3]);
            }
        } else {
            let preview_block = Block::default().borders(Borders::ALL).title("Profile Pic Preview").style(pic_style);
            f.render_widget(preview_block, right[3]);
        }
        // Profile Pic field + delete
        let row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
            .split(right[5]);
        f.render_widget(
            Paragraph::new(app.profile.edit_profile_pic.clone())
                .block(Block::default().borders(Borders::ALL).title("🖼️ Path/Base64").border_style(pic_style))
                .style(pic_style),
            row[0],
        );
        let del_style = if app.profile.profile_edit_focus == ProfilePicDelete {
            Style::default().fg(Color::Black).bg(Color::Red).add_modifier(Modifier::BOLD)
        } else { Style::default().fg(Color::Red) };
        f.render_widget(
            Paragraph::new(Span::styled("[ Delete ]", del_style)).alignment(Alignment::Center),
            row[1],
        );
        // Banner preview
        let banner_style = if app.profile.profile_edit_focus == CoverBanner {
            Style::default().fg(Color::Black).bg(Color::LightMagenta).add_modifier(Modifier::BOLD)
        } else { Style::default().bg(Color::DarkGray) };
        if !app.profile.edit_cover_banner.trim().is_empty() {
            let mut show_placeholder = true;
            if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(app.profile.edit_cover_banner.trim()) {
                if let Ok(img) = image::load_from_memory(&bytes) {
                    let mut protocol = app.profile.picker.new_resize_protocol(img);
                    let image_widget = ratatui_image::StatefulImage::default().resize(ratatui_image::Resize::Fit(None));
                    f.render_stateful_widget(image_widget, right[7], &mut protocol);
                    show_placeholder = false;
                }
            }
            if show_placeholder {
                let preview_block = Block::default().borders(Borders::ALL).title("Banner Preview").style(banner_style);
                f.render_widget(preview_block, right[7]);
            }
        } else {
            let preview_block = Block::default().borders(Borders::ALL).title("Banner Preview").style(banner_style);
            f.render_widget(preview_block, right[7]);
        }
        // Banner field + delete
        let row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
            .split(right[9]);
        f.render_widget(
            Paragraph::new(app.profile.edit_cover_banner.clone())
                .block(Block::default().borders(Borders::ALL).title("🖼️ Path/Base64").border_style(banner_style))
                .style(banner_style),
            row[0],
        );
        let del_style = if app.profile.profile_edit_focus == CoverBannerDelete {
            Style::default().fg(Color::Black).bg(Color::Red).add_modifier(Modifier::BOLD)
        } else { Style::default().fg(Color::Red) };
        f.render_widget(
            Paragraph::new(Span::styled("[ Delete ]", del_style)).alignment(Alignment::Center),
            row[1],
        );
        // Save/Cancel buttons
        let save_style = if app.profile.profile_edit_focus == Save {
            Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Green)
        };
        let cancel_style = if app.profile.profile_edit_focus == Cancel {
            Style::default().fg(Color::Black).bg(Color::Red).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Red)
        };
        let buttons = Line::from(vec![
            Span::styled("[ Save ]", save_style),
            Span::raw("   "),
            Span::styled("[ Cancel ]", cancel_style),
        ]);
        f.render_widget(Paragraph::new(buttons).alignment(Alignment::Center), right[11]);
        // Error message with extra padding
        if let Some(err) = &app.profile.profile_edit_error {
            f.render_widget(Paragraph::new(err.as_str()).style(Style::default().fg(Color::Red)), right[13]);
        }
        // Set cursor for focused field (moved inside this block)
        let cursor = match app.profile.profile_edit_focus {
            Bio => {
                let lines: Vec<&str> = app.profile.edit_bio.split('\n').collect();
                let y = left[2].y + lines.len() as u16 - 1 + 1;
                let x = left[2].x + lines.last().map(|l| l.len()).unwrap_or(0) as u16 + 1;
                (x, y)
            },
            Location => (left[4].x + app.profile.edit_location.len() as u16 + 1, left[4].y + 1),
            Url1 => (left[6].x + app.profile.edit_url1.len() as u16 + 1, left[6].y + 1),
            Url2 => (left[8].x + app.profile.edit_url2.len() as u16 + 1, left[8].y + 1),
            Url3 => (left[10].x + app.profile.edit_url3.len() as u16 + 1, left[10].y + 1),
            ProfilePic => (row[0].x + app.profile.edit_profile_pic.len() as u16 + 1, row[0].y + 1),
            CoverBanner => (row[0].x + app.profile.edit_cover_banner.len() as u16 + 1, row[0].y + 1),
            _ => (0, 0),
        };
        if matches!(app.profile.profile_edit_focus, Bio|Location|Url1|Url2|Url3|ProfilePic|CoverBanner) {
            f.set_cursor_position(cursor);
        }
    } else {
        // Single column layout for narrow screens - simplified for brevity
        let preview_block = Block::default().borders(Borders::ALL).title("Profile Edit").style(Style::default());
        f.render_widget(preview_block, padded);
    }
}

pub fn draw_color_picker(f: &mut Frame, app: &mut App, area: Rect) {
    let palette = [
        Color::Cyan, Color::Green, Color::Yellow, Color::Red,
        Color::Magenta, Color::Blue, Color::White, Color::LightCyan,
        Color::LightGreen, Color::LightYellow, Color::LightRed,
        Color::LightMagenta, Color::LightBlue, Color::Gray,
        Color::DarkGray, Color::Black
    ];
    
    let block = Block::default().borders(Borders::ALL).title("Choose Your Color");
    f.render_widget(&block, area);
    let inner = block.inner(area);
    
    let items_per_row = 4;
    let rows = (palette.len() + items_per_row - 1) / items_per_row;
    let item_width = inner.width / items_per_row as u16;
    let item_height = 3;
    
    for (i, &color) in palette.iter().enumerate() {
        let row = i / items_per_row;
        let col = i % items_per_row;
        
        if row >= rows {
            break;
        }
        
        let x = inner.x + col as u16 * item_width;
        let y = inner.y + row as u16 * item_height;
        let width = item_width;
        let height = item_height;
        
        let item_area = ratatui::layout::Rect { x, y, width, height };
        
        let style = if i == app.ui.color_picker_selected {
            Style::default().bg(color).fg(Color::Black).add_modifier(Modifier::BOLD)
        } else {
            Style::default().bg(color)
        };
        
        let text = if i == app.ui.color_picker_selected {
            "[ SELECTED ]"
        } else {
            "         "
        };
        
        f.render_widget(
            Paragraph::new(text)
                .style(style)
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL)),
            item_area,
        );
    }
}

pub fn draw_preferences(f: &mut Frame, app: &mut App, area: Rect) {
    let prefs = &app.prefs;
    
    let block = Block::default().borders(Borders::ALL).title("Preferences");
    f.render_widget(&block, area);
    let inner = block.inner(area);
    
    // Create layout for preferences items
    let items_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Sound Effects
            Constraint::Length(3), // Glitch Effects
            Constraint::Length(3), // Desktop Notifications
            Constraint::Min(0),    // Remaining space
        ])
        .split(inner);
    
    // Sound Effects preference
    let sound_status = if prefs.sound_effects_enabled { "ON" } else { "OFF" };
    let sound_style = if app.ui.preferences_selected == 0 {
        Style::default().fg(Color::Black).bg(Color::LightCyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    
    f.render_widget(
        Paragraph::new(format!("🔊 Sound Effects: {}", sound_status))
            .style(sound_style)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center),
        items_layout[0],
    );
    
    // Glitch Effects preference
    let glitch_status = if prefs.minimal_banner_glitch_enabled { "ON" } else { "OFF" };
    let glitch_style = if app.ui.preferences_selected == 1 {
        Style::default().fg(Color::Black).bg(Color::LightCyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    
    f.render_widget(
        Paragraph::new(format!("✨ Glitch Effects: {}", glitch_status))
            .style(glitch_style)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center),
        items_layout[1],
    );
    
    // Desktop Notifications preference
    let desktop_notif_status = if prefs.desktop_notifications_enabled { "ON" } else { "OFF" };
    let desktop_notif_style = if app.ui.preferences_selected == 2 {
        Style::default().fg(Color::Black).bg(Color::LightCyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    
    f.render_widget(
        Paragraph::new(format!("🔔 Desktop Notifications: {}", desktop_notif_status))
            .style(desktop_notif_style)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center),
        items_layout[2],
    );
    
    // Help text
    if items_layout.len() > 3 {
        let help_text = Paragraph::new("Use [↑↓] to navigate, [Space/Enter] to toggle, [Esc] to go back")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Help"));
        f.render_widget(help_text, items_layout[3]);
    }
}
