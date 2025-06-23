use ratatui::{Frame, layout::Rect, style::{Style, Color, Modifier}, widgets::Paragraph};
use crate::app::App;
use crate::ui::themes::{Theme, ThemeColors, AccentColors};
use crate::ui::backgrounds::Background;

pub struct CyberpunkBackground;

impl Background for CyberpunkBackground {
    fn name(&self) -> &'static str {
        "Cyberpunk"
    }
    
    fn draw_background(&self, f: &mut Frame, app: &App, area: Rect) {
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
        
        // Add scanning lines effect
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
}