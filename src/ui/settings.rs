//! Settings and profile editing UI screens.

use ratatui::{Frame, layout::Rect, style::{Style, Color, Modifier}, widgets::{Block, List, ListItem, Paragraph, Borders, BorderType, Wrap}, text::{Line, Span}, layout::Constraint, layout::Layout};
use ratatui::prelude::{Alignment, Direction};
use crate::app::{App};
use base64::Engine;
use crate::global_prefs;

pub fn draw_settings(f: &mut Frame, app: &mut App, area: Rect) {
    let items = vec![
        ListItem::new("Change Password"),
        ListItem::new("Change User Color"),
        ListItem::new("Edit Profile"),
        ListItem::new("Preferences"),
    ];
    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Settings"))
        .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black)).highlight_symbol(">> ");
    f.render_stateful_widget(list, area, &mut app.settings_list_state);
}

pub fn draw_profile_edit_page(f: &mut Frame, app: &mut App, area: Rect) {
    use crate::app::ProfileEditFocus::*;
    let min_two_col_width = 110; // Increased for more breathing room
    let is_narrow = area.width < min_two_col_width;
    // Outer card with more margin
    let outer_margin = 2;
    let block = Block::default()
        .title(Span::styled("✦ Edit Your Profile ✦", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .style(Style::default().bg(Color::Black));
    let card_area = Layout::default()
        .direction(Direction::Vertical)
        .margin(outer_margin)
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
        let bio_style = if app.profile_edit_focus == Bio {
            Style::default().fg(Color::Black).bg(Color::LightCyan).add_modifier(Modifier::BOLD)
        } else { Style::default().bg(Color::DarkGray) };
        f.render_widget(
            Paragraph::new(app.edit_bio.as_str())
                .block(Block::default().borders(Borders::ALL).title("📝 Bio").border_style(bio_style))
                .style(bio_style)
                .wrap(Wrap { trim: false }),
            left[2],
        );
        // Location
        let location_style = if app.profile_edit_focus == Location {
            Style::default().fg(Color::Black).bg(Color::LightCyan).add_modifier(Modifier::BOLD)
        } else { Style::default().bg(Color::DarkGray) };
        f.render_widget(
            Paragraph::new(app.edit_location.clone())
                .block(Block::default().borders(Borders::ALL).title("📍 Location").border_style(location_style))
                .style(location_style),
            left[4],
        );
        // URLs
        let url_titles = ["🔗 URL1", "🔗 URL2", "🔗 URL3"];
        let url_fields = [&app.edit_url1, &app.edit_url2, &app.edit_url3];
        let url_focus = [Url1, Url2, Url3];
        for i in 0..3 {
            let style = if app.profile_edit_focus == url_focus[i] {
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
        let pic_style = if app.profile_edit_focus == ProfilePic {
            Style::default().fg(Color::Black).bg(Color::LightMagenta).add_modifier(Modifier::BOLD)
        } else { Style::default().bg(Color::DarkGray) };
        if !app.edit_profile_pic.trim().is_empty() {
            let mut show_placeholder = true;
            if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(app.edit_profile_pic.trim()) {
                if let Ok(img) = image::load_from_memory(&bytes) {
                    let mut protocol = app.picker.new_resize_protocol(img);
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
            Paragraph::new(app.edit_profile_pic.clone())
                .block(Block::default().borders(Borders::ALL).title("🖼️ Path/Base64").border_style(pic_style))
                .style(pic_style),
            row[0],
        );
        let del_style = if app.profile_edit_focus == ProfilePicDelete {
            Style::default().fg(Color::Black).bg(Color::Red).add_modifier(Modifier::BOLD)
        } else { Style::default().fg(Color::Red) };
        f.render_widget(
            Paragraph::new(Span::styled("[ Delete ]", del_style)).alignment(Alignment::Center),
            row[1],
        );
        // Banner preview
        let banner_style = if app.profile_edit_focus == CoverBanner {
            Style::default().fg(Color::Black).bg(Color::LightMagenta).add_modifier(Modifier::BOLD)
        } else { Style::default().bg(Color::DarkGray) };
        if !app.edit_cover_banner.trim().is_empty() {
            let mut show_placeholder = true;
            if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(app.edit_cover_banner.trim()) {
                if let Ok(img) = image::load_from_memory(&bytes) {
                    let mut protocol = app.picker.new_resize_protocol(img);
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
            Paragraph::new(app.edit_cover_banner.clone())
                .block(Block::default().borders(Borders::ALL).title("🖼️ Path/Base64").border_style(banner_style))
                .style(banner_style),
            row[0],
        );
        let del_style = if app.profile_edit_focus == CoverBannerDelete {
            Style::default().fg(Color::Black).bg(Color::Red).add_modifier(Modifier::BOLD)
        } else { Style::default().fg(Color::Red) };
        f.render_widget(
            Paragraph::new(Span::styled("[ Delete ]", del_style)).alignment(Alignment::Center),
            row[1],
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
        f.render_widget(Paragraph::new(buttons).alignment(Alignment::Center), right[11]);
        // Error message with extra padding
        if let Some(err) = &app.profile_edit_error {
            f.render_widget(Paragraph::new(err.as_str()).style(Style::default().fg(Color::Red)), right[13]);
        }
        // Set cursor for focused field (moved inside this block)
        let cursor = match app.profile_edit_focus {
            Bio => {
                let lines: Vec<&str> = app.edit_bio.split('\n').collect();
                let y = left[2].y + lines.len() as u16 - 1 + 1;
                let x = left[2].x + lines.last().map(|l| l.len()).unwrap_or(0) as u16 + 1;
                (x, y)
            },
            Location => (left[4].x + app.edit_location.len() as u16 + 1, left[4].y + 1),
            Url1 => (left[6].x + app.edit_url1.len() as u16 + 1, left[6].y + 1),
            Url2 => (left[8].x + app.edit_url2.len() as u16 + 1, left[8].y + 1),
            Url3 => (left[10].x + app.edit_url3.len() as u16 + 1, left[10].y + 1),
            ProfilePic => (row[0].x + app.edit_profile_pic.len() as u16 + 1, row[0].y + 1),
            CoverBanner => (row[0].x + app.edit_cover_banner.len() as u16 + 1, row[0].y + 1),
            _ => (0, 0),
        };
        if matches!(app.profile_edit_focus, Bio|Location|Url1|Url2|Url3|ProfilePic|CoverBanner) {
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
        let bio_style = if app.profile_edit_focus == Bio {
            Style::default().fg(Color::Black).bg(Color::LightCyan).add_modifier(Modifier::BOLD)
        } else { Style::default().bg(Color::DarkGray) };
        f.render_widget(
            Paragraph::new(app.edit_bio.as_str())
                .block(Block::default().borders(Borders::ALL).title("📝 Bio").border_style(bio_style))
                .style(bio_style)
                .wrap(Wrap { trim: false }),
            fields[2],
        );
        // Location
        let location_style = if app.profile_edit_focus == Location {
            Style::default().fg(Color::Black).bg(Color::LightCyan).add_modifier(Modifier::BOLD)
        } else { Style::default().bg(Color::DarkGray) };
        f.render_widget(
            Paragraph::new(app.edit_location.clone())
                .block(Block::default().borders(Borders::ALL).title("📍 Location").border_style(location_style))
                .style(location_style),
            fields[4],
        );
        // URLs
        let url_titles = ["🔗 URL1", "🔗 URL2", "🔗 URL3"];
        let url_fields = [&app.edit_url1, &app.edit_url2, &app.edit_url3];
        let url_focus = [Url1, Url2, Url3];
        for i in 0..3 {
            let style = if app.profile_edit_focus == url_focus[i] {
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
        let pic_style = if app.profile_edit_focus == ProfilePic {
            Style::default().fg(Color::Black).bg(Color::LightMagenta).add_modifier(Modifier::BOLD)
        } else { Style::default().bg(Color::DarkGray) };
        if !app.edit_profile_pic.trim().is_empty() {
            let mut show_placeholder = true;
            if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(app.edit_profile_pic.trim()) {
                if let Ok(img) = image::load_from_memory(&bytes) {
                    let mut protocol = app.picker.new_resize_protocol(img);
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
            Paragraph::new(app.edit_profile_pic.clone())
                .block(Block::default().borders(Borders::ALL).title("🖼️ Path/Base64").border_style(pic_style))
                .style(pic_style),
            row[0],
        );
        let del_style = if app.profile_edit_focus == ProfilePicDelete {
            Style::default().fg(Color::Black).bg(Color::Red).add_modifier(Modifier::BOLD)
        } else { Style::default().fg(Color::Red) };
        f.render_widget(
            Paragraph::new(Span::styled("[ Delete ]", del_style)).alignment(Alignment::Center),
            row[1],
        );
        // Banner preview
        let banner_style = if app.profile_edit_focus == CoverBanner {
            Style::default().fg(Color::Black).bg(Color::LightMagenta).add_modifier(Modifier::BOLD)
        } else { Style::default().bg(Color::DarkGray) };
        if !app.edit_cover_banner.trim().is_empty() {
            let mut show_placeholder = true;
            if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(app.edit_cover_banner.trim()) {
                if let Ok(img) = image::load_from_memory(&bytes) {
                    let mut protocol = app.picker.new_resize_protocol(img);
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
            Paragraph::new(app.edit_cover_banner.clone())
                .block(Block::default().borders(Borders::ALL).title("🖼️ Path/Base64").border_style(banner_style))
                .style(banner_style),
            row[0],
        );
        let del_style = if app.profile_edit_focus == CoverBannerDelete {
            Style::default().fg(Color::Black).bg(Color::Red).add_modifier(Modifier::BOLD)
        } else { Style::default().fg(Color::Red) };
        f.render_widget(
            Paragraph::new(Span::styled("[ Delete ]", del_style)).alignment(Alignment::Center),
            row[1],
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
        f.render_widget(Paragraph::new(buttons).alignment(Alignment::Center), fields[23]);
        // Error message with extra padding
        if let Some(err) = &app.profile_edit_error {
            f.render_widget(Paragraph::new(err.as_str()).style(Style::default().fg(Color::Red)), fields[25]);
        }
    }
}

pub fn draw_color_picker_page(f: &mut Frame, app: &mut App, area: Rect) {
    let colors = [
        Color::Cyan, Color::Green, Color::Yellow, Color::Red, Color::Magenta, Color::Blue, Color::White, Color::LightCyan, Color::LightGreen, Color::LightYellow, Color::LightRed, Color::LightMagenta, Color::LightBlue, Color::Gray, Color::DarkGray, Color::Black
    ];
    let color_names = [
        "Cyan", "Green", "Yellow", "Red", "Magenta", "Blue", "White", "LightCyan", "LightGreen", "LightYellow", "LightRed", "LightMagenta", "LightBlue", "Gray", "DarkGray", "Black"
    ];
    let block = Block::default().title("Pick a Username Color").borders(Borders::ALL).border_type(BorderType::Double);
    f.render_widget(block, area);
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Preview
            Constraint::Length(colors.len() as u16 + 2), // Palette
            Constraint::Min(0),
        ])
        .split(area);
    // Username preview
    let username = app.current_user.as_ref().map(|u| u.username.as_str()).unwrap_or("your_username");
    let preview_color = colors[app.color_picker_selected];
    let preview = Paragraph::new(Span::styled(
        format!("Preview: {}", username),
        Style::default().fg(preview_color).add_modifier(Modifier::BOLD),
    ))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL).title("Preview"));
    f.render_widget(preview, inner[0]);
    // Palette
    let mut palette_lines: Vec<Line> = Vec::new();
    let mut current_line: Vec<Span> = Vec::new();
    let mut current_width = 0u16;
    let max_width = inner[1].width.saturating_sub(4); // account for borders/margins
    for (i, &color) in colors.iter().enumerate() {
        let label = format!(" {} ", color_names[i]);
        let label_width = label.chars().count() as u16;
        let style = if i == app.color_picker_selected {
            Style::default().fg(Color::Black).bg(color).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(color)
        };
        if current_width + label_width > max_width && !current_line.is_empty() {
            palette_lines.push(Line::from(current_line));
            current_line = Vec::new();
            current_width = 0;
        }
        current_line.push(Span::styled(label, style));
        current_width += label_width;
    }
    if !current_line.is_empty() {
        palette_lines.push(Line::from(current_line));
    }
    let palette = Paragraph::new(palette_lines)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Colors (←/→ to pick, Enter=Save, Esc=Cancel)"));
    f.render_widget(palette, inner[1]);
}

pub fn draw_parameters_page(f: &mut Frame, _app: &mut App, area: Rect) {
    use ratatui::widgets::{Block, Borders, Paragraph};
    use ratatui::text::{Span, Line};
    use ratatui::style::{Style, Color, Modifier};
    let block = Block::default().title("Parameters").borders(Borders::ALL);
    f.render_widget(block, area);
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);
    let checked = if global_prefs::global_prefs().sound_effects_enabled { "[x]" } else { "[ ]" };
    let line = Line::from(vec![
        Span::raw("Sound Effects "),
        Span::styled(checked, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ]);
    f.render_widget(Paragraph::new(line), inner[0]);
}
