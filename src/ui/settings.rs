//! Settings and profile editing UI screens.

use ratatui::{Frame, layout::Rect, style::{Style, Color, Modifier}, widgets::{Block, List, ListItem, Paragraph, Borders, BorderType, Wrap}, text::{Line, Span}, layout::Constraint, layout::Layout};
use ratatui::prelude::{Alignment, Direction};
use crate::app::{App};

pub fn draw_settings(f: &mut Frame, app: &mut App, area: Rect) {
    let items = vec![
        ListItem::new("Change Password"),
        ListItem::new("Change User Color (Cycle)"),
        ListItem::new("Edit Profile"),
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
