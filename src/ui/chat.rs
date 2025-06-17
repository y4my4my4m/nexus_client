//! Chat and user list UI screens.

use ratatui::{Frame, layout::{Rect, Layout, Constraint, Direction}, style::{Style, Color, Modifier}, widgets::{Block, Paragraph, Borders, Wrap}, text::{Line, Span}};
use crate::app::{App, ChatFocus};
use crate::ui::avatar::get_avatar_protocol;
use ratatui_image::StatefulImage;

pub fn draw_chat(f: &mut Frame, app: &mut App, area: Rect) {
    let show_users = app.show_user_list;
    let focus = app.chat_focus;
    if show_users {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(area);
        draw_chat_main(f, app, chunks[0], focus == crate::app::ChatFocus::Messages);
        draw_user_list(f, app, chunks[1], focus == crate::app::ChatFocus::Users);
    } else {
        draw_chat_main(f, app, area, true);
    }
    if app.chat_focus == crate::app::ChatFocus::DMInput {
        crate::ui::popups::draw_dm_input_popup(f, app);
    }
}

pub fn draw_chat_main(f: &mut Frame, app: &mut App, area: Rect, focused: bool) {
    let border_style = if focused {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let chunks = Layout::default().constraints([Constraint::Min(0), Constraint::Length(3)]).split(area);
    let messages_area = chunks[0];
    let input_area = chunks[1];

    let messages_block = Block::default()
        .borders(Borders::ALL)
        .title("Live Chat // #general")
        .border_style(border_style);
    f.render_widget(messages_block.clone(), messages_area);

    let inner_area = messages_block.inner(messages_area);
    if inner_area.width == 0 || inner_area.height == 0 { return; }

    const AVATAR_PIXEL_SIZE: u32 = 32;
    let (font_w, font_h) = app.picker.font_size();
    let (font_w, font_h) = if font_w == 0 || font_h == 0 { (8, 16) } else { (font_w, font_h) };
    let avatar_cell_width = (AVATAR_PIXEL_SIZE as f32 / font_w as f32).ceil() as u16;
    let avatar_cell_height = (AVATAR_PIXEL_SIZE as f32 / font_h as f32).ceil() as u16;
    let row_height = avatar_cell_height.max(2);

    // Collect display items first to avoid borrow checker issues
    let display_items: Vec<_> = app.chat_messages.iter().rev().map(|msg| {
        let user = app.connected_users.iter().find(|u| u.username == msg.author).cloned();
        let author = msg.author.clone();
        let color = msg.color;
        let content = msg.content.clone();
        (user, author, color, content)
    }).collect();

    let mut current_y = inner_area.y;
    for (user_opt, author, color, content) in display_items.into_iter().rev() {
        let row_area = Rect::new(inner_area.x, current_y, inner_area.width, row_height);
        if let Some(user) = user_opt {
            if let Some(state) = get_avatar_protocol(app, &user, AVATAR_PIXEL_SIZE) {
                let row_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(avatar_cell_width), Constraint::Length(1), Constraint::Min(0)])
                    .split(row_area);
                let image_widget = StatefulImage::default();
                f.render_stateful_widget(image_widget, row_chunks[0], state);
                let text = vec![
                    Line::from(Span::styled(format!("<{}>", author), Style::default().fg(color).add_modifier(Modifier::BOLD))),
                    Line::from(Span::raw(&content)),
                ];
                f.render_widget(Paragraph::new(text).wrap(Wrap { trim: true }), row_chunks[2]);
            } else {
                let row_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(avatar_cell_width), Constraint::Length(1), Constraint::Min(0)])
                    .split(row_area);
                let text = vec![
                    Line::from(vec![
                        Span::raw("○ "),
                        Span::styled(format!("<{}>:", author), Style::default().fg(color).add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from(vec![Span::raw("  "), Span::raw(&content)]),
                ];
                f.render_widget(Paragraph::new(text).wrap(Wrap { trim: true }), row_chunks[2]);
            }
        }
        current_y += row_height + 1;
    }

    let input = Paragraph::new(app.current_input.as_str()).style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, input_area);
    if focused {
        f.set_cursor_position((input_area.x + app.current_input.len() as u16 + 1, input_area.y + 1));
    }
}

pub fn draw_user_list(f: &mut Frame, app: &mut App, area: Rect, focused: bool) {
    let border_style = if focused {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let block = Block::default().borders(Borders::ALL).title("Users [Ctrl+U]").border_style(border_style);
    f.render_widget(block.clone(), area);

    let inner_area = block.inner(area);
    if inner_area.width == 0 || inner_area.height == 0 { return; }

    const AVATAR_PIXEL_SIZE: u32 = 16;
    let (font_w, font_h) = app.picker.font_size();
    let (font_w, font_h) = if font_w == 0 || font_h == 0 { (8, 16) } else { (font_w, font_h) };
    let avatar_cell_width = (AVATAR_PIXEL_SIZE as f32 / font_w as f32).ceil() as u16;
    let avatar_cell_height = (AVATAR_PIXEL_SIZE as f32 / font_h as f32).ceil() as u16;
    let row_height = avatar_cell_height.max(1);

    let list_state = app.forum_list_state.clone();
    let selected_index = list_state.selected();
    let offset = list_state.offset();

    let mut current_y = inner_area.y;
    // Collect connected_users into a temporary vector before the loop
    let users: Vec<_> = app.connected_users.iter().enumerate().skip(offset).map(|(i, user)| (i, user.clone())).collect();
    for (i, user) in users {
        if current_y + row_height > inner_area.y + inner_area.height { break; }
        let row_area = Rect::new(inner_area.x, current_y, inner_area.width, row_height);

        let is_selected = focused && selected_index == Some(i);
        let text_style = if is_selected {
            Style::default().fg(Color::Black).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(user.color)
        };
        if is_selected {
            f.render_widget(Block::default().style(Style::default().bg(Color::Cyan)), row_area);
        }

        if let Some(state) = get_avatar_protocol(app, &user, AVATAR_PIXEL_SIZE) {
            let row_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(avatar_cell_width), Constraint::Min(0)])
                .split(row_area);
            let image_widget = StatefulImage::default();
            f.render_stateful_widget(image_widget, row_chunks[0], state);
            let text = Line::from(vec![
                Span::raw(" "),
                Span::styled(&user.username, text_style),
                Span::styled(format!(" ({:?})", user.role), text_style.remove_modifier(Modifier::BOLD).add_modifier(Modifier::DIM)),
            ]);
            f.render_widget(Paragraph::new(text).alignment(ratatui::layout::Alignment::Left), row_chunks[1]);
        } else {
            let text = Line::from(vec![
                Span::raw(" ○ "),
                Span::styled(&user.username, text_style),
                Span::styled(format!(" ({:?})", user.role), text_style.remove_modifier(Modifier::BOLD).add_modifier(Modifier::DIM)),
            ]);
            f.render_widget(Paragraph::new(text).alignment(ratatui::layout::Alignment::Left), row_area);
        }
        current_y += row_height;
    }
}
