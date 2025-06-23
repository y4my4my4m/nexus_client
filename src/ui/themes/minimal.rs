use ratatui::{Frame, layout::Rect, style::{Style, Color}, widgets::Paragraph};
use crate::app::App;
use super::{Theme, ThemeColors, AccentColors};

pub struct MinimalTheme;

impl Theme for MinimalTheme {
    fn name(&self) -> &'static str {
        "Minimal"
    }
    
    fn draw_background(&self, f: &mut Frame, app: &App, area: Rect) {
        let tick = app.ui.tick_count;
        
        // Very subtle background pattern
        for y in 0..area.height {
            for x in 0..area.width {
                let grid_x = x as usize;
                let grid_y = y as usize;
                let time_offset = (tick / 8) as usize; // Much slower animation
                
                // Minimal pattern - just occasional dots
                if (grid_x * 17 + grid_y * 23 + time_offset) % 500 == 0 {
                    let cell_area = Rect::new(area.x + x, area.y + y, 1, 1);
                    f.render_widget(
                        Paragraph::new("·").style(Style::default().fg(Color::DarkGray)),
                        cell_area
                    );
                }
            }
        }
        
        // Very subtle scanning line
        if tick % 100 < 2 {
            let scan_line = (tick / 10) % (area.height as u64);
            for x in 0..area.width {
                let scan_area = Rect::new(area.x + x, area.y + scan_line as u16, 1, 1);
                f.render_widget(
                    Paragraph::new("─").style(Style::default().fg(Color::DarkGray)),
                    scan_area
                );
            }
        }
    }
    
    fn get_primary_colors(&self) -> ThemeColors {
        ThemeColors {
            primary: Color::White,
            secondary: Color::Gray,
            background: Color::Black,
            text: Color::White,
            selected_bg: Color::White,
            selected_fg: Color::Black,
        }
    }
    
    fn get_border_colors(&self, tick: u64) -> Color {
        match (tick / 15) % 3 {
            0 => Color::White,
            1 => Color::Gray,
            _ => Color::DarkGray,
        }
    }
    
    fn get_accent_colors(&self) -> AccentColors {
        AccentColors {
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::White,
        }
    }
}