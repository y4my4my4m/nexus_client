//! Main menu UI screen with cyberpunk aesthetics.

use ratatui::{Frame, layout::{Rect, Layout, Constraint, Direction, Alignment}, style::{Style, Color, Modifier}, widgets::{Block, List, ListItem, Borders, Paragraph, BorderType}, text::{Line, Span}};
use crate::app::App;

pub fn draw_main_menu(f: &mut Frame, app: &mut App, area: Rect) {
    // Draw animated background using the current theme
    let theme = app.theme_manager.get_current_theme();
    theme.draw_background(f, app, area);
    
    // Calculate responsive layout based on available height
    let available_height = area.height;
    
    // More aggressive space utilization - reduce title height to minimal
    let title_height = if available_height < 15 { 0 } else { 2 }; // Reduced from 6-8 to 2
    let status_height = if available_height < 20 { 0 } else { 6 };
    
    // Create main layout with better proportions
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(title_height),
            Constraint::Min(12), // Ensure minimum space for menu
            Constraint::Length(status_height),
        ])
        .margin(1)
        .split(area);

    // Draw enhanced title section
    if title_height > 0 {
        draw_enhanced_title(f, app, main_layout[0]);
    }

    // Draw enhanced menu with better styling
    draw_enhanced_menu(f, app, main_layout[1]);

    // Draw enhanced status section
    if status_height > 0 {
        draw_enhanced_status(f, app, main_layout[2]);
    }
    
    // Add floating UI elements
    draw_floating_elements(f, app, area);
    
    // Draw flowing arrows at the bottom of the screen
    draw_bottom_flowing_arrows(f, app, area);
}

fn draw_animated_background(f: &mut Frame, app: &mut App, area: Rect) {
    let tick = app.ui.tick_count;
    
    // Create animated grid pattern
    for y in 0..area.height {
        for x in 0..area.width {
            let grid_x = x as usize;
            let grid_y = y as usize;
            let time_offset = (tick / 4) as usize;
            
            // Create moving wave pattern
            let wave1 = ((grid_x + time_offset) % 20 == 0) as u8;
            let wave2 = ((grid_y + time_offset / 2) % 15 == 0) as u8;
            let pulse = ((grid_x + grid_y + time_offset) % 30 < 3) as u8;
            
            let intensity = wave1 + wave2 + pulse;
            
            let (char, color) = match intensity {
                3 => ('╬', Color::Cyan),
                2 => ('┼', Color::Blue),
                1 => ('·', Color::DarkGray),
                _ => {
                    // Sparse random noise
                    if (grid_x * 7 + grid_y * 11 + time_offset) % 200 == 0 {
                        ('▪', Color::DarkGray)
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
    
    // Add scanning lines effect - fix type conversion
    let scan_line = (tick / 2) % (area.height as u64);
    for x in 0..area.width {
        let scan_area = Rect::new(area.x + x, area.y + scan_line as u16, 1, 1);
        let intensity = if ((x as u64) + tick / 3) % 8 < 2 { Color::Green } else { Color::DarkGray };
        f.render_widget(
            Paragraph::new("▬").style(Style::default().fg(intensity)),
            scan_area
        );
    }
}

fn draw_enhanced_title(f: &mut Frame, app: &mut App, area: Rect) {
    let tick = app.ui.tick_count;
    
    // Just draw the animated top border - no bottom flowing arrows
    let top_border_chars: Vec<char> = (0..area.width)
        .map(|x| {
            let phase = (x as u64 + tick / 2) % 20;
            match phase {
                0..=2 => '█',
                3..=5 => '▓',
                6..=8 => '▒',
                9..=11 => '░',
                _ => '─',
            }
        })
        .collect();
    
    f.render_widget(
        Paragraph::new(top_border_chars.iter().collect::<String>())
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        area
    );
}

fn draw_enhanced_menu(f: &mut Frame, app: &mut App, area: Rect) {
    let tick = app.ui.tick_count;
    
    // Create side-by-side layout for menu and info panel
    let menu_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60),  // Menu items
            Constraint::Percentage(40),  // Info/preview panel
        ])
        .split(area);
    
    // Enhanced menu items with ASCII art icons
    let menu_items = [
        ("Forums", "  ╔════════════════╗\n  ║ ░▒▓█ DATA █▓▒░ ║\n  ╚════════════════╝", "Neural archive matrices"),
        ("Chat", "  ╔══════════════╗\n  ║  ◄► COMM ◄►  ║\n  ╚══════════════╝", "Real-time neural link"),
        ("Settings", "  ╔══════════════╗\n  ║ ⚙  CONFIG ⚙  ║\n  ╚══════════════╝", "System parameters"),
        ("Logout", "  ╔═══════════════╗\n  ║ ◄◄ DISCONNECT ║\n  ╚═══════════════╝", "Terminate session"),
    ];
    
    let items: Vec<ListItem> = menu_items.iter().enumerate().map(|(i, &(name, icon, desc))| {
        let is_selected = Some(i) == app.ui.main_menu_state.selected();
        let selection_glow = if is_selected { (tick / 5) % 8 } else { 0 };
        
        if is_selected {
            // Enhanced selected item with multi-line icon
            let icon_lines: Vec<&str> = icon.lines().collect();
            let mut lines = vec![
                Line::from(vec![
                    Span::styled(">>> ", Style::default().fg(Color::LightMagenta).add_modifier(Modifier::BOLD)),
                    Span::styled(name, Style::default().fg(Color::Black).bg(Color::LightCyan).add_modifier(Modifier::BOLD)),
                    Span::styled(" <<<", Style::default().fg(Color::LightMagenta).add_modifier(Modifier::BOLD)),
                ]),
            ];
            
            // Add icon lines
            for icon_line in icon_lines {
                let glow_color = match selection_glow {
                    0..=1 => Color::Cyan,
                    2..=3 => Color::LightCyan,
                    4..=5 => Color::Blue,
                    _ => Color::LightBlue,
                };
                lines.push(Line::from(vec![
                    Span::styled(icon_line, Style::default().fg(glow_color).add_modifier(Modifier::BOLD))
                ]));
            }
            
            lines.push(Line::from(vec![
                Span::styled("    └─ ", Style::default().fg(Color::Yellow)),
                Span::styled(desc, Style::default().fg(Color::LightBlue).add_modifier(Modifier::ITALIC)),
            ]));
            lines.push(Line::from(Span::raw(""))); // Spacing
            
            ListItem::new(lines)
        } else {
            // Minimized unselected items
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled("  ▶ ", Style::default().fg(Color::DarkGray)),
                    Span::styled(name, Style::default().fg(Color::White)),
                    Span::styled(" - ", Style::default().fg(Color::DarkGray)),
                    Span::styled(desc, Style::default().fg(Color::Gray)),
                ]),
                Line::from(Span::raw("")), // Spacing
            ])
        }
    }).collect();
    
    let menu_border_color = match (tick / 10) % 4 {
        0 => Color::Cyan,
        1 => Color::Blue,
        2 => Color::LightBlue,
        _ => Color::DarkGray,
    };
    
    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(menu_border_color))
        .title("▼ PROTOCOL SELECTION ▼")
        .title_style(Style::default().fg(Color::LightMagenta).add_modifier(Modifier::BOLD));
    
    let list = List::new(items).block(list_block);
    f.render_stateful_widget(list, menu_layout[0], &mut app.ui.main_menu_state);
    
    // Info panel with dynamic content
    draw_info_panel(f, app, menu_layout[1]);
}

fn draw_info_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let tick = app.ui.tick_count;
    let selected = app.ui.main_menu_state.selected().unwrap_or(0);
    
    let info_content = match selected {
        0 => vec![
            Line::from(vec![Span::styled("DATA ARCHIVES", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
            Line::from(Span::raw("")),
            Line::from(vec![Span::styled("▶ Neural Network Forums", Style::default().fg(Color::White))]),
            Line::from(vec![Span::styled("▶ Quantum Discussions", Style::default().fg(Color::White))]),
            Line::from(vec![Span::styled("▶ Code Repositories", Style::default().fg(Color::White))]),
            Line::from(vec![Span::styled("▶ Security Protocols", Style::default().fg(Color::White))]),
            Line::from(Span::raw("")),
            Line::from(vec![Span::styled("Status: ", Style::default().fg(Color::Gray)), 
                         Span::styled("ONLINE", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))]),
        ],
        1 => vec![
            Line::from(vec![Span::styled("REAL-TIME COMM", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))]),
            Line::from(Span::raw("")),
            Line::from(vec![Span::styled("▶ Global Channels", Style::default().fg(Color::White))]),
            Line::from(vec![Span::styled("▶ Private Messages", Style::default().fg(Color::White))]),
            Line::from(vec![Span::styled("▶ Voice Synthesis", Style::default().fg(Color::White))]),
            Line::from(vec![Span::styled("▶ File Transfer", Style::default().fg(Color::White))]),
            Line::from(Span::raw("")),
            Line::from(vec![Span::styled("Encryption: ", Style::default().fg(Color::Gray)),
                         Span::styled("AES-256", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))]),
        ],
        2 => vec![
            Line::from(vec![Span::styled("SYSTEM CONFIG", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
            Line::from(Span::raw("")),
            Line::from(vec![Span::styled("▶ Interface Themes", Style::default().fg(Color::White))]),
            Line::from(vec![Span::styled("▶ Audio Settings", Style::default().fg(Color::White))]),
            Line::from(vec![Span::styled("▶ Security Options", Style::default().fg(Color::White))]),
            Line::from(vec![Span::styled("▶ Network Config", Style::default().fg(Color::White))]),
            Line::from(Span::raw("")),
            Line::from(vec![Span::styled("Access: ", Style::default().fg(Color::Gray)),
                         Span::styled("ADMIN", Style::default().fg(Color::LightMagenta).add_modifier(Modifier::BOLD))]),
        ],
        _ => vec![
            Line::from(vec![Span::styled("DISCONNECT", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))]),
            Line::from(Span::raw("")),
            Line::from(vec![Span::styled("▶ Save Session", Style::default().fg(Color::White))]),
            Line::from(vec![Span::styled("▶ Clear Cache", Style::default().fg(Color::White))]),
            Line::from(vec![Span::styled("▶ Secure Logout", Style::default().fg(Color::White))]),
            Line::from(vec![Span::styled("▶ Emergency Exit", Style::default().fg(Color::Red))]),
            Line::from(Span::raw("")),
            Line::from(vec![Span::styled("Warning: ", Style::default().fg(Color::Red)),
                         Span::styled("Unsaved data will be lost", Style::default().fg(Color::Yellow))]),
        ],
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
        .title("◈ INFO PANEL ◈")
        .title_style(Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD));
    
    f.render_widget(
        Paragraph::new(info_content)
            .block(info_block)
            .alignment(Alignment::Left),
        area
    );
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

fn draw_bottom_flowing_arrows(f: &mut Frame, app: &mut App, area: Rect) {
    let tick = app.ui.tick_count;
    
    // Create flowing arrows at the bottom of the screen
    let flow_chars: String = (0..area.width)
        .map(|x| {
            let flow_pos = (tick / 3 + x as u64) % 30;
            match flow_pos {
                0..=3 => '▶',
                4..=6 => '▷',
                23..=25 => '◁',
                25..=28 => '◀',
                _ => '═',
            }
        })
        .collect();
    
    // Draw the flowing arrows at the very bottom of the screen
    let bottom_area = Rect::new(area.x, area.y + area.height - 1, area.width, 1);
    f.render_widget(
        Paragraph::new(flow_chars)
            .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        bottom_area
    );
}
