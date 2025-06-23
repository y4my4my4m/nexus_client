use ratatui::{Frame, layout::Rect, style::{Style, Color, Modifier}, widgets::Paragraph};
use crate::app::App;
use super::{Theme, ThemeColors, AccentColors};

pub struct PlasmaWaveTheme;

impl Theme for PlasmaWaveTheme {
    fn name(&self) -> &'static str { "PlasmaWave" }
    fn draw_background(&self, f: &mut Frame, app: &App, area: Rect) {
        let tick = app.ui.tick_count;
        let w = area.width as f32;
        let h = area.height as f32;
        for y in 0..area.height {
            for x in 0..area.width {
                let xf = x as f32 / w;
                let yf = y as f32 / h;
                let t = tick as f32 * 0.07;
                let v = ((xf * 8.0 + t).sin() + (yf * 8.0 - t * 0.7).cos() + ((xf + yf) * 6.0 + t * 0.5).sin()) * 0.5;
                let color = if v > 0.7 {
                    Color::Cyan
                } else if v > 0.3 {
                    Color::Magenta
                } else if v > -0.2 {
                    Color::Yellow
                } else if v > -0.6 {
                    Color::Green
                } else {
                    Color::LightBlue
                };
                // Glitch overlay
                let glitch = ((tick + (x as u64 * 13 + y as u64 * 7)) % 97) < 2;
                let ch = if glitch { "▒" } else { "█" };
                f.render_widget(
                    Paragraph::new(ch).style(Style::default().fg(color)),
                    Rect::new(area.x + x, area.y + y, 1, 1),
                );
            }
        }
    }
    fn get_primary_colors(&self) -> ThemeColors {
        ThemeColors {
            primary: Color::Cyan,
            secondary: Color::Magenta,
            background: Color::Black,
            text: Color::White,
            selected_bg: Color::LightCyan,
            selected_fg: Color::Black,
        }
    }
    fn get_border_colors(&self, tick: u64) -> Color {
        match (tick / 8) % 3 {
            0 => Color::Cyan,
            1 => Color::Magenta,
            _ => Color::Yellow,
        }
    }
    fn get_accent_colors(&self) -> AccentColors {
        AccentColors {
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Cyan,
        }
    }
}
