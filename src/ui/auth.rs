//! Authentication (login/register) UI screens.

use ratatui::{Frame, layout::{Rect, Layout, Constraint}, style::{Style, Color}, widgets::{Block, Paragraph, Borders}, text::{Span}};
use crate::app::{App, InputMode};
use ratatui::prelude::{Alignment, Direction};

pub fn draw_login(f: &mut Frame, app: &mut App, area: Rect) {
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
        f.set_cursor_position((chunks[0].x + app.current_input.len() as u16 + 1, chunks[0].y + 1));
    } else if let Some(InputMode::LoginPassword) = &app.input_mode {
        f.set_cursor_position((chunks[1].x + app.password_input.len() as u16 + 1, chunks[1].y + 1));
    }
}

pub fn draw_register(f: &mut Frame, app: &mut App, area: Rect) {
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
        f.set_cursor_position((chunks[0].x + app.current_input.len() as u16 + 1, chunks[0].y + 1));
    } else if let Some(InputMode::RegisterPassword) = &app.input_mode {
        f.set_cursor_position((chunks[1].x + app.password_input.len() as u16 + 1, chunks[1].y + 1));
    }
}
