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
            Constraint::Length(1), // ProfilePic Info
            Constraint::Length(5), // ProfilePic Preview
            Constraint::Length(3), // ProfilePic Field+Delete
            Constraint::Length(5), // CoverBanner Preview
            Constraint::Length(3), // CoverBanner Field+Delete
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
    // Info for profile pic (now its own row)
    let info_line = Paragraph::new(Span::styled(
        "(i) Image must be a local file path, under 1MB, with no spaces in the path/name.",
        Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)
    )).alignment(Alignment::Left);
    f.render_widget(info_line, inner[5]);
    // Profile Pic preview (if any) above the field
    let pic_style = if app.profile_edit_focus == ProfilePic {
        Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else { Style::default() };
    if !app.edit_profile_pic.trim().is_empty() {
        let mut show_placeholder = true;
        if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(app.edit_profile_pic.trim()) {
            if let Ok(img) = image::load_from_memory(&bytes) {
                let mut protocol = app.picker.new_resize_protocol(img);
                let image_widget = ratatui_image::StatefulImage::default().resize(ratatui_image::Resize::Fit(None));
                f.render_stateful_widget(image_widget, inner[6], &mut protocol);
                show_placeholder = false;
            }
        }
        if show_placeholder {
            let preview_block = Block::default().borders(Borders::ALL).title("Profile Pic Preview");
            f.render_widget(preview_block, inner[6]);
        }
    }
    // Always render the field+delete row
    let row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(inner[7]);
    f.render_widget(
        Paragraph::new(app.edit_profile_pic.clone())
            .block(Block::default().borders(Borders::ALL).title("Profile Pic").border_style(pic_style))
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
    // Cover Banner preview (if any) above the field
    let banner_style = if app.profile_edit_focus == CoverBanner {
        Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else { Style::default() };
    if !app.edit_cover_banner.trim().is_empty() {
        let mut show_placeholder = true;
        if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(app.edit_cover_banner.trim()) {
            if let Ok(img) = image::load_from_memory(&bytes) {
                let mut protocol = app.picker.new_resize_protocol(img);
                let image_widget = ratatui_image::StatefulImage::default().resize(ratatui_image::Resize::Fit(None));
                f.render_stateful_widget(image_widget, inner[8], &mut protocol);
                show_placeholder = false;
            }
        }
        if show_placeholder {
            let preview_block = Block::default().borders(Borders::ALL).title("Banner Preview");
            f.render_widget(preview_block, inner[8]);
        }
    }
    // Always render the field+delete row
    let row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(inner[9]);
    f.render_widget(
        Paragraph::new(app.edit_cover_banner.clone())
            .block(Block::default().borders(Borders::ALL).title("Cover Banner").border_style(banner_style))
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
    f.render_widget(Paragraph::new(buttons).alignment(Alignment::Center), inner[10]);
    // Error message
    if let Some(err) = &app.profile_edit_error {
        f.render_widget(Paragraph::new(err.as_str()).style(Style::default().fg(Color::Red)), inner[11]);
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
        ProfilePic => (inner[7].x + app.edit_profile_pic.len() as u16 + 1, inner[7].y + 1),
        CoverBanner => (inner[9].x + app.edit_cover_banner.len() as u16 + 1, inner[9].y + 1),
        _ => (0, 0),
    };
    if matches!(app.profile_edit_focus, Bio|Url1|Url2|Url3|Location|ProfilePic|CoverBanner) {
        f.set_cursor_position(cursor);
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

pub fn draw_parameters_page(f: &mut Frame, app: &mut App, area: Rect) {
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
