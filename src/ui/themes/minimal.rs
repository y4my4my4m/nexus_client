use ratatui::{Frame, layout::Rect, style::{Style, Color}, widgets::Paragraph};
use crate::app::App;
use super::{Theme, ThemeColors, AccentColors};
use ratatui::style::{Modifier};

pub struct MinimalTheme;
impl Theme for MinimalTheme {
    fn name(&self) -> &'static str { "Minimal" }
    fn colors(&self) -> ThemeColors {
        ThemeColors {
            primary: Color::Yellow,
            secondary: Color::Gray,
            background: Color::Black,
            text: Color::White,
            selected_bg: Color::Cyan,
            selected_fg: Color::Black,
        }
    }
    fn accents(&self) -> AccentColors {
        AccentColors {
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::White,
        }
    }
    fn border_color(&self, tick: u64) -> Color {
        match (tick / 15) % 3 {
            0 => Color::Red,
            1 => Color::Gray,
            _ => Color::DarkGray,
        }
    }
    fn selected_style(&self) -> Style {
        Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)
    }
    fn text_style(&self) -> Style {
        Style::default()
    }
    fn draw_top_banner(&self, _f: &mut ratatui::Frame, _app: &crate::app::App, _area: ratatui::layout::Rect) {
        // Minimal: no top banner
    }
    fn draw_bottom_banner(&self, _f: &mut ratatui::Frame, _app: &crate::app::App, _area: ratatui::layout::Rect) {
        // Minimal: no bottom banner
    }
    fn draw_main_menu(&self, f: &mut ratatui::Frame, main_menu_state: &mut ratatui::widgets::ListState, tick: u64, area: ratatui::layout::Rect) {
        use ratatui::{widgets::{Block, List, ListItem, Borders}, style::{Style, Color}, text::{Line, Span}};
        let menu_items = ["Forums", "Chat", "Settings", "Logout"];
        let items: Vec<ListItem> = menu_items.iter().enumerate().map(|(i, &name)| {
            let is_selected = Some(i) == main_menu_state.selected();
            let style = if is_selected {
                Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(ratatui::style::Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(Span::styled(name, style)))
        }).collect();
        let list_block = Block::default()
            .borders(Borders::ALL)
            .title("Menu")
            .border_style(Style::default());
        let list = List::new(items).block(list_block);
        f.render_stateful_widget(list, area, main_menu_state);
    }
    fn draw_settings_menu(&self, f: &mut ratatui::Frame, settings_list_state: &mut ratatui::widgets::ListState, tick: u64, area: ratatui::layout::Rect) {
        use ratatui::{widgets::{Block, List, ListItem, Borders, Paragraph}, style::{Style, Color}, text::{Line, Span}, layout::{Layout, Constraint, Direction}};
        let settings_items = ["Change Password", "Change Color", "Edit Profile", "Preferences"];
        let items: Vec<ListItem> = settings_items.iter().enumerate().map(|(i, &name)| {
            let is_selected = Some(i) == settings_list_state.selected();
            let style = if is_selected {
                Style::default().fg(Color::Black).bg(Color::White).add_modifier(ratatui::style::Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(Span::styled(name, style)))
        }).collect();
        // Layout: left = list, right = info panel
        let layout = if area.width >= 60 {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(6), Constraint::Length(6)])
                .split(area)
        };
        let list_block = Block::default()
            .borders(Borders::ALL)
            .title("Settings")
            .border_style(Style::default().fg(Color::White));
        let list = List::new(items).block(list_block);
        f.render_stateful_widget(list, layout[0], settings_list_state);
        // Info panel
        let selected = settings_list_state.selected().unwrap_or(0);
        let info_lines = match selected {
            0 => vec![
                Line::from("Change your password for better security."),
                Line::from("Recommended: Use a strong, unique password."),
            ],
            1 => vec![
                Line::from("Change the color theme of the app."),
                Line::from("Try different palettes for accessibility."),
            ],
            2 => vec![
                Line::from("Edit your profile information."),
                Line::from("Update your bio, avatar, and links."),
            ],
            3 => vec![
                Line::from("Configure app preferences."),
                Line::from("Sound, notifications, and more."),
            ],
            _ => vec![Line::from("")],
        };
        let info_block = Block::default()
            .borders(Borders::ALL)
            .title("Info")
            .border_style(Style::default().fg(Color::Gray));
        let mut info_content = info_lines;
        info_content.push(Line::from(""));
        info_content.push(Line::from("[↑↓] Select  [Enter] Edit  [Esc] Back"));
        f.render_widget(Paragraph::new(info_content).block(info_block).alignment(ratatui::layout::Alignment::Left), layout[1]);
    }
    fn draw_floating_elements(&self, _f: &mut Frame, _app: &App, _area: Rect) {
        // Minimal: no floating elements
    }
}