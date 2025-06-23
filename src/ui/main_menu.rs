//! Main menu UI screen with cyberpunk aesthetics.

use ratatui::{Frame, layout::{Rect, Layout, Constraint, Direction, Alignment}, style::{Style, Color, Modifier}, widgets::{Block, List, ListItem, Borders, Paragraph, BorderType}, text::{Line, Span}};
use crate::app::App;

pub fn draw_main_menu(f: &mut Frame, app: &mut App, area: Rect) {
    // Calculate responsive layout based on available height
    let available_height = area.height;
    
    let title_height = if available_height < 20 { 0 } else { 4 }; // Compact or full title
    let status_height = if available_height < 25 { 0 } else { 4 }; // Hide status in very small windows
    
    // Create a responsive layout that prioritizes menu items
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(title_height),  // Menu title section (responsive)
            Constraint::Min(0),
            Constraint::Length(status_height), // Bottom info/status (can be hidden)
        ])
        .horizontal_margin(2)
        .split(area);

    let tick = app.ui.tick_count;
    let blinking_signal = if tick % 30 < 15 {
        Span::styled("◉", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
    } else {
        Span::styled(" ", Style::default())
    };

    let title_lines = if available_height < 20 {
        // Hide title completely for very small windows to prioritize menu items
        vec![]
    } else {
        // Full title for larger windows - use separate lines for better control
        vec![
            Line::from(vec![
                Span::styled("N E X U S", Style::default().fg(Color::LightMagenta).add_modifier(Modifier::BOLD)),
                Span::styled("  //  ", Style::default().fg(Color::Yellow)),
                Span::styled("NEURAL INTERFACE", Style::default().fg(Color::LightBlue)),
            ]),
            Line::from(vec![
                Span::styled(">> ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled("QUANTUM COMMUNICATION GRID", Style::default().fg(Color::White)),
                Span::styled(" <<", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("┌─ ", Style::default().fg(Color::Yellow)),
                Span::styled("SELECT PROTOCOL", Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD)),
                Span::styled(" ─┐", Style::default().fg(Color::Yellow)),
            ]),
        ]
    };

    // Only render title block if there are title lines
    let has_title = !title_lines.is_empty();
    if has_title {
        let title_block = Paragraph::new(title_lines)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            );
        f.render_widget(title_block, main_layout[0]);
    }

    // Blinking status indicator positioned at top-right corner of the title area
    if tick % 30 < 15 && has_title {
        let signal_area = Rect {
            x: main_layout[0].x + main_layout[0].width - 3,
            y: main_layout[0].y + 1,
            width: 1,
            height: 1,
        };
        f.render_widget(
            Paragraph::new(Span::styled("◉", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
            signal_area
        );
    }

    // Cyberpunk menu items with futuristic styling
    let menu_items = [
        ("Forums", "🌐 DATA ARCHIVES", "Access quantum archive matrices"),
        ("Chat", "💬 REAL-TIME COMM", "Enter neural chat networks"),
        ("Settings", "⚙️ SYSTEM CONFIG", "Modify interface parameters"),
        ("Logout", "🚪 DISCONNECT", "Terminate session protocol"),
    ];

    let items: Vec<ListItem> = menu_items.iter().enumerate().map(|(i, &(_, title, desc))| {
        let is_selected = Some(i) == app.ui.main_menu_state.selected();
        
        if is_selected {
            // Selected item with glowing effect
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(">>> ", Style::default().fg(Color::LightMagenta).add_modifier(Modifier::BOLD)),
                    Span::styled(title, Style::default().fg(Color::Black).bg(Color::LightCyan).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::styled("    └─ ", Style::default().fg(Color::Yellow)),
                    Span::styled(desc, Style::default().fg(Color::LightBlue).add_modifier(Modifier::ITALIC)),
                ]),
                Line::from(Span::raw("")), // Spacing
            ])
        } else {
            // Unselected items with subtle styling
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled("  ▶ ", Style::default().fg(Color::DarkGray)),
                    Span::styled(title, Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("    └─ ", Style::default().fg(Color::DarkGray)),
                    Span::styled(desc, Style::default().fg(Color::Gray)),
                ]),
                Line::from(Span::raw("")), // Spacing
            ])
        }
    }).collect();

    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default());

    let list = List::new(items)
        .block(list_block);

    f.render_stateful_widget(list, main_layout[1], &mut app.ui.main_menu_state);

    if status_height > 0 {
        let status_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),  // User profile
                Constraint::Percentage(50),  // System status
            ])
            .split(main_layout[2]);

        if let Some(user) = &app.auth.current_user {
            // User profile block
            let user_text = vec![
                Line::from(vec![
                    Span::styled("ID: ", Style::default().fg(Color::Gray)),
                    Span::styled(&user.username, Style::default().fg(user.color.clone().into()).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::styled("ROLE: ", Style::default().fg(Color::Gray)),
                    Span::styled(format!("{:?}", user.role), Style::default().fg(Color::LightMagenta)),
                ]),
            ];

            let user_block = Paragraph::new(user_text)
                .alignment(Alignment::Left)
                .block(
                    Block::default()
                        .title("USER PROFILE")
                        .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Yellow))
                );

            // System status block
            let system_text = vec![
                Line::from(vec![
                    Span::styled("NEURAL LINK: ", Style::default().fg(Color::Gray)),
                    Span::styled("ACTIVE", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::styled("UPLINK: ", Style::default().fg(Color::Gray)),
                    Span::styled("SECURE", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                ]),
            ];

            let system_block = Paragraph::new(system_text)
                .alignment(Alignment::Left)
                .block(
                    Block::default()
                        .title("SYSTEM STATUS")
                        .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Green))
                );

            f.render_widget(user_block, status_layout[0]);
            f.render_widget(system_block, status_layout[1]);
        } else {
            // Single disconnected status block when not logged in
            let disconnected_text = vec![
                Line::from(vec![
                    Span::styled("NEURAL LINK: ", Style::default().fg(Color::Gray)),
                    Span::styled("DISCONNECTED", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::styled("STATUS: ", Style::default().fg(Color::Gray)),
                    Span::styled("UNAUTHORIZED", Style::default().fg(Color::Red)),
                ]),
            ];

            let disconnected_block = Paragraph::new(disconnected_text)
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .title("SYSTEM STATUS")
                        .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Red))
                );

            f.render_widget(disconnected_block, main_layout[2]);
        }
    }
}
