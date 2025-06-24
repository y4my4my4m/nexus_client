use ratatui::{Frame, layout::Rect, style::{Style, Color, Modifier}, widgets::Paragraph};
use crate::app::App;
use super::{Theme, ThemeColors, AccentColors};
use ratatui::prelude::Alignment;
use crate::ui::themes::ThemeMainMenuLayout;
use ratatui::layout::Constraint;

pub struct CyberpunkTheme;
impl Theme for CyberpunkTheme {
    fn name(&self) -> &'static str { "Cyberpunk" }
    fn colors(&self) -> ThemeColors {
        ThemeColors {
            primary: Color::Cyan,
            secondary: Color::Magenta,
            background: Color::Black,
            text: Color::White,
            selected_bg: Color::LightCyan,
            selected_fg: Color::Black,
        }
    }
    fn accents(&self) -> AccentColors {
        AccentColors {
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Cyan,
        }
    }
    fn border_color(&self, tick: u64) -> Color {
        match (tick / 8) % 3 {
            0 => Color::Cyan,
            1 => Color::Magenta,
            _ => Color::Yellow,
        }
    }
    fn selected_style(&self) -> Style {
        Style::default().fg(Color::Black).bg(Color::LightCyan).add_modifier(Modifier::BOLD)
    }
    fn text_style(&self) -> Style {
        Style::default().fg(Color::White)
    }
    fn draw_top_banner(&self, f: &mut ratatui::Frame, app: &crate::app::App, area: ratatui::layout::Rect) {
        let tick = app.ui.tick_count;
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
            ratatui::widgets::Paragraph::new(top_border_chars.iter().collect::<String>())
                .style(ratatui::style::Style::default().fg(ratatui::style::Color::Cyan).add_modifier(ratatui::style::Modifier::BOLD)),
            area
        );
    }
    fn draw_bottom_banner(&self, f: &mut ratatui::Frame, app: &crate::app::App, area: ratatui::layout::Rect) {
        let tick = app.ui.tick_count;
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
        let bottom_area = ratatui::layout::Rect::new(area.x, area.y + area.height - 1, area.width, 1);
        f.render_widget(
            ratatui::widgets::Paragraph::new(flow_chars)
                .style(ratatui::style::Style::default().fg(ratatui::style::Color::Green).add_modifier(ratatui::style::Modifier::BOLD)),
            bottom_area
        );
    }
    fn draw_main_menu(&self, f: &mut ratatui::Frame, main_menu_state: &mut ratatui::widgets::ListState, tick: u64, area: ratatui::layout::Rect) {
        use ratatui::{widgets::{Block, List, ListItem, Borders, Paragraph, BorderType}, style::{Style, Color, Modifier}, text::{Line, Span}, layout::{Layout, Constraint, Direction}};
        let menu_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60),
                Constraint::Percentage(40),
            ])
            .split(area);
        let menu_items = [
            ("Forums", "  ╔════════════════╗\n  ║ ░▒▓█ DATA █▓▒░ ║\n  ╚════════════════╝", "Neural archive matrices"),
            ("Chat", "  ╔══════════════╗\n  ║  ◄► COMM ◄►  ║\n  ╚══════════════╝", "Real-time neural link"),
            ("Settings", "  ╔══════════════╗\n  ║ ⚙  CONFIG ⚙  ║\n  ╚══════════════╝", "System parameters"),
            ("Logout", "  ╔═══════════════╗\n  ║ ◄◄ DISCONNECT ║\n  ╚═══════════════╝", "Terminate session"),
        ];
        let items: Vec<ListItem> = menu_items.iter().enumerate().map(|(i, &(name, icon, desc))| {
            let is_selected = Some(i) == main_menu_state.selected();
            let selection_glow = if is_selected { (tick / 5) % 8 } else { 0 };
            if is_selected {
                let icon_lines: Vec<&str> = icon.lines().collect();
                let icon_height = icon_lines.len();
                let mut lines = vec![];
                for (j, icon_line) in icon_lines.iter().enumerate() {
                    let glow_color = match selection_glow {
                        0..=1 => Color::Cyan,
                        2..=3 => Color::LightCyan,
                        4..=5 => Color::Blue,
                        _ => Color::LightBlue,
                    };
                    if j == icon_height / 2 {
                        // Keep ▶ in the same horizontal position as unselected
                        lines.push(Line::from(vec![
                            Span::styled("  ▶ ", Style::default().fg(Color::LightMagenta).add_modifier(Modifier::BOLD)),
                            Span::styled(icon_line.trim_start(), Style::default().fg(glow_color).add_modifier(Modifier::BOLD)),
                        ]));
                    } else {
                        lines.push(Line::from(vec![
                            Span::styled("    ", Style::default()),
                            Span::styled(icon_line.trim_start(), Style::default().fg(glow_color).add_modifier(Modifier::BOLD)),
                        ]));
                    }
                }
                // └─ and desc, aligned with icon
                lines.push(Line::from(vec![
                    Span::styled("    ", Style::default()),
                    Span::styled("└─ ", Style::default().fg(Color::Yellow)),
                    Span::styled(desc, Style::default().fg(Color::LightBlue).add_modifier(Modifier::ITALIC)),
                ]));
                lines.push(Line::from(Span::raw("")));
                ListItem::new(lines)
            } else {
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled("  ▶ ", Style::default().fg(Color::DarkGray)),
                        Span::styled(name, Style::default().fg(Color::White)),
                        Span::styled(" - ", Style::default().fg(Color::DarkGray)),
                        Span::styled(desc, Style::default().fg(Color::Gray)),
                    ]),
                    Line::from(Span::raw("")),
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
        f.render_stateful_widget(list, menu_layout[0], main_menu_state);
        // Info panel with dynamic content
        let selected = main_menu_state.selected().unwrap_or(0);
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
            menu_layout[1]
        );
    }
    fn draw_settings_menu(&self, f: &mut ratatui::Frame, settings_list_state: &mut ratatui::widgets::ListState, tick: u64, area: ratatui::layout::Rect) {
        use ratatui::{widgets::{Block, List, ListItem, Borders, Paragraph, BorderType}, style::{Style, Color, Modifier}, text::{Line, Span}, layout::{Layout, Constraint, Direction}};
        let settings_items = [
            ("Change Password", "  ╔═══════════════╗\n  ║ ⚡ SECURITY ⚡║\n  ╚═══════════════╝", "Update authentication key"),
            ("Change Color", "  ╔═══════════════╗\n  ║ 🎨 IDENTITY 🎨║\n  ╚═══════════════╝", "Customize user signature"),
            ("Edit Profile", "  ╔═══════════════╗\n  ║ 👤 PERSONA 👤 ║\n  ╚═══════════════╝", "Modify profile data"),
            ("Preferences", "  ╔═══════════════╗\n  ║  ⚙  SYSTEM ⚙  ║\n  ╚═══════════════╝", "Configure client settings"),
        ];
        let layout = if area.width >= 80 {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
                .split(area)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(8), Constraint::Length(10)])
                .split(area)
        };
        let items: Vec<ListItem> = settings_items.iter().enumerate().map(|(i, &(name, icon, desc))| {
            let is_selected = Some(i) == settings_list_state.selected();
            let selection_glow = if is_selected { (tick / 5) % 8 } else { 0 };
            if is_selected {
                let icon_lines: Vec<&str> = icon.lines().collect();
                let mut lines = vec![
                    Line::from(vec![
                        Span::styled(">>> ", Style::default().fg(Color::LightMagenta).add_modifier(Modifier::BOLD)),
                        Span::styled(name, Style::default().fg(Color::Black).bg(Color::LightCyan).add_modifier(Modifier::BOLD)),
                        Span::styled(" <<<", Style::default().fg(Color::LightMagenta).add_modifier(Modifier::BOLD)),
                    ]),
                ];
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
                lines.push(Line::from(Span::raw("")));
                ListItem::new(lines)
            } else {
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled("  ▶ ", Style::default().fg(Color::DarkGray)),
                        Span::styled(name, Style::default().fg(Color::White)),
                        Span::styled(" - ", Style::default().fg(Color::DarkGray)),
                        Span::styled(desc, Style::default().fg(Color::Gray)),
                    ]),
                    Line::from(Span::raw("")),
                ])
            }
        }).collect();
        let list_border_color = match (tick / 10) % 4 {
            0 => Color::Cyan,
            1 => Color::Blue,
            2 => Color::LightBlue,
            _ => Color::DarkGray,
        };
        let list_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(list_border_color))
            .title("▼ SYSTEM CONFIGURATION ▼")
            .title_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));
        let list = List::new(items).block(list_block);
        f.render_stateful_widget(list, layout[0], settings_list_state);
        // Info panel
        let selected = settings_list_state.selected().unwrap_or(0);
        let info_lines = match selected {
            0 => vec![
                Line::from(vec![Span::styled("SECURITY UPDATE", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))]),
                Line::from(Span::raw("")),
                Line::from(vec![Span::styled("▶ Password Authentication", Style::default().fg(Color::White))]),
                Line::from(vec![Span::styled("▶ Encryption Key Rotation", Style::default().fg(Color::White))]),
                Line::from(vec![Span::styled("▶ Session Invalidation", Style::default().fg(Color::White))]),
                Line::from(vec![Span::styled("▶ Access Control Update", Style::default().fg(Color::White))]),
                Line::from(Span::raw("")),
                Line::from(vec![Span::styled("Security Level: ", Style::default().fg(Color::Gray)), Span::styled("HIGH", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))]),
            ],
            1 => vec![
                Line::from(vec![Span::styled("VISUAL IDENTITY", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))]),
                Line::from(Span::raw("")),
                Line::from(vec![Span::styled("▶ Color Palette Selection", Style::default().fg(Color::White))]),
                Line::from(vec![Span::styled("▶ Theme Configuration", Style::default().fg(Color::White))]),
                Line::from(vec![Span::styled("▶ Visual Signature", Style::default().fg(Color::White))]),
                Line::from(vec![Span::styled("▶ Display Preferences", Style::default().fg(Color::White))]),
                Line::from(Span::raw("")),
                Line::from(vec![Span::styled("Current Theme: ", Style::default().fg(Color::Gray)), Span::styled("CYBERPUNK", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))]),
                Line::from(vec![Span::styled("Press F8: ", Style::default().fg(Color::Gray)), Span::styled("Cycle Theme", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
            ],
            2 => vec![
                Line::from(vec![Span::styled("USER PROFILE", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
                Line::from(Span::raw("")),
                Line::from(vec![Span::styled("▶ Personal Information", Style::default().fg(Color::White))]),
                Line::from(vec![Span::styled("▶ Avatar Management", Style::default().fg(Color::White))]),
                Line::from(vec![Span::styled("▶ Bio & Social Links", Style::default().fg(Color::White))]),
                Line::from(vec![Span::styled("▶ Privacy Settings", Style::default().fg(Color::White))]),
                Line::from(Span::raw("")),
                Line::from(vec![Span::styled("Profile Status: ", Style::default().fg(Color::Gray)), Span::styled("ACTIVE", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))]),
            ],
            3 => vec![
                Line::from(vec![Span::styled("CLIENT CONFIG", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
                Line::from(Span::raw("")),
                Line::from(vec![Span::styled("▶ Audio Settings", Style::default().fg(Color::White))]),
                Line::from(vec![Span::styled("▶ Visual Effects", Style::default().fg(Color::White))]),
                Line::from(vec![Span::styled("▶ Notifications", Style::default().fg(Color::White))]),
                Line::from(vec![Span::styled("▶ Performance Tuning", Style::default().fg(Color::White))]),
                Line::from(Span::raw("")),
                Line::from(vec![Span::styled("Active Theme: ", Style::default().fg(Color::Gray)), Span::styled("CYBERPUNK", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
                Line::from(vec![Span::styled("Press F8: ", Style::default().fg(Color::Gray)), Span::styled("Cycle Theme", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
            ],
            _ => vec![Line::from("")],
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
        let mut info_content = info_lines;
        info_content.push(Line::from(""));
        info_content.push(Line::from("[↑↓] Select  [Enter] Edit  [Esc] Back"));
        f.render_widget(Paragraph::new(info_content).block(info_block).alignment(Alignment::Left), layout[1]);
    }
    fn draw_floating_elements(&self, f: &mut Frame, app: &App, area: Rect) {
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
    fn main_menu_layout(&self, area: Rect) -> ThemeMainMenuLayout {
        let available_height = area.height;
        let title_height = if available_height < 15 { 0 } else { 2 };
        let status_height = if available_height < 20 { 0 } else { 6 };
        ThemeMainMenuLayout {
            constraints: vec![
                Constraint::Length(title_height),
                Constraint::Min(12),
                Constraint::Length(status_height),
            ],
            show_top_banner: title_height > 0,
            show_status: status_height > 0,
        }
    }
}