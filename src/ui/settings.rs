//! Settings and profile editing UI screens.

use ratatui::{Frame, layout::Rect, style::{Style, Color, Modifier}, widgets::{Block, List, ListItem, Paragraph, Borders, BorderType, Wrap}, text::{Line, Span}, layout::Constraint, layout::Layout};
use ratatui::prelude::{Alignment, Direction};
use crate::app::{App};
use base64::Engine;
use crate::global_prefs;

pub fn draw_settings(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = ["Change Password", "Change Color", "Edit Profile", "Preferences"].iter().enumerate().map(|(i, &item)| {
        let style = if Some(i) == app.ui.settings_list_state.selected() {
            Style::default().bg(Color::LightCyan).fg(Color::Black)
        } else {
            Style::default()
        };
        ListItem::new(Span::styled(item, style))
    }).collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Settings"));
    f.render_stateful_widget(list, area, &mut app.ui.settings_list_state);
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
        // --- Single-column (stacked) layout for small screens ---
        let fields = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Personal Info header
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
                Constraint::Length(2), // Profile Images header
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
            .split(padded);
        // Section header
        f.render_widget(Paragraph::new(Span::styled("Personal Info", Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD))).alignment(Alignment::Left), fields[0]);
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
            fields[2],
        );
        // Location
        let location_style = if app.profile.profile_edit_focus == Location {
            Style::default().fg(Color::Black).bg(Color::LightCyan).add_modifier(Modifier::BOLD)
        } else { Style::default().bg(Color::DarkGray) };
        f.render_widget(
            Paragraph::new(app.profile.edit_location.clone())
                .block(Block::default().borders(Borders::ALL).title("📍 Location").border_style(location_style))
                .style(location_style),
            fields[4],
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
                fields[6 + i * 2],
            );
        }
        // Section header
        f.render_widget(Paragraph::new(Span::styled("Profile Images", Style::default().fg(Color::LightMagenta).add_modifier(Modifier::BOLD))).alignment(Alignment::Left), fields[12]);
        // (i) image info
        f.render_widget(Paragraph::new(Span::styled(
            "(i) Image: local file path, under 1MB, no spaces.",
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)
        )).alignment(Alignment::Left), fields[13]);
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
                    f.render_stateful_widget(image_widget, fields[15], &mut protocol);
                    show_placeholder = false;
                }
            }
            if show_placeholder {
                let preview_block = Block::default().borders(Borders::ALL).title("Profile Pic Preview").style(pic_style);
                f.render_widget(preview_block, fields[15]);
            }
        } else {
            let preview_block = Block::default().borders(Borders::ALL).title("Profile Pic Preview").style(pic_style);
            f.render_widget(preview_block, fields[15]);
        }
        // Profile Pic field + delete
        let row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
            .split(fields[17]);
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
                    f.render_stateful_widget(image_widget, fields[19], &mut protocol);
                    show_placeholder = false;
                }
            }
            if show_placeholder {
                let preview_block = Block::default().borders(Borders::ALL).title("Banner Preview").style(banner_style);
                f.render_widget(preview_block, fields[19]);
            }
        } else {
            let preview_block = Block::default().borders(Borders::ALL).title("Banner Preview").style(banner_style);
            f.render_widget(preview_block, fields[19]);
        }
        // Banner field + delete
        let row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
            .split(fields[21]);
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
        f.render_widget(Paragraph::new(buttons).alignment(Alignment::Center), fields[23]);
        // Error message with extra padding
        if let Some(err) = &app.profile.profile_edit_error {
            f.render_widget(Paragraph::new(err.as_str()).style(Style::default().fg(Color::Red)), fields[25]);
        }
        
        // Set cursor for focused field in single-column layout
        let cursor = match app.profile.profile_edit_focus {
            Bio => {
                let lines: Vec<&str> = app.profile.edit_bio.split('\n').collect();
                let y = fields[2].y + lines.len() as u16 - 1 + 1;
                let x = fields[2].x + lines.last().map(|l| l.len()).unwrap_or(0) as u16 + 1;
                (x, y)
            },
            Location => (fields[4].x + app.profile.edit_location.len() as u16 + 1, fields[4].y + 1),
            Url1 => (fields[6].x + app.profile.edit_url1.len() as u16 + 1, fields[6].y + 1),
            Url2 => (fields[8].x + app.profile.edit_url2.len() as u16 + 1, fields[8].y + 1),
            Url3 => (fields[10].x + app.profile.edit_url3.len() as u16 + 1, fields[10].y + 1),
            ProfilePic => (fields[17].x + app.profile.edit_profile_pic.len() as u16 + 1, fields[17].y + 1),
            CoverBanner => (fields[21].x + app.profile.edit_cover_banner.len() as u16 + 1, fields[21].y + 1),
            _ => (0, 0),
        };
        if matches!(app.profile.profile_edit_focus, Bio|Location|Url1|Url2|Url3|ProfilePic|CoverBanner) {
            f.set_cursor_position(cursor);
        }
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
    let prefs = global_prefs::global_prefs();
    
    let block = Block::default().borders(Borders::ALL).title("Preferences");
    f.render_widget(&block, area);
    let inner = block.inner(area);
    
    // Create layout for preferences items
    let items_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Sound Effects
            Constraint::Length(3), // Glitch Effects
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
    
    // Help text
    let help_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(inner)[1];
        
    f.render_widget(
        Paragraph::new("Use ↑↓ to navigate, SPACE/ENTER to toggle, ESC to go back")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center),
        help_area,
    );
}
