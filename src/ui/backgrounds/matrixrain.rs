use ratatui::{Frame, layout::Rect, style::{Style, Color, Modifier}, widgets::Paragraph};
use crate::app::App;
use crate::ui::backgrounds::Background;

pub struct MatrixRainBackground;

impl Background for MatrixRainBackground {
    fn name(&self) -> &'static str { "MatrixRain" }
    fn draw_background(&self, f: &mut Frame, app: &App, area: Rect) {
        let tick = app.ui.tick_count;
        let charset = ["7", "3", "A", "E", "F", "C", "9", "1", "0", "B", "D", "4", "5", "2", "8", "6"];
        for x in 0..area.width {
            let col_seed = (x as u64 * 31 + tick / 2) % 1000;
            let rain_len = 6 + (col_seed % 8) as u16;
            let offset = (tick as u16 + (col_seed % 17) as u16 * 3) % (area.height + rain_len);
            for y in 0..area.height {
                let rain_pos = y + rain_len;
                let is_rain = y + offset >= area.height && y + offset < area.height + rain_len;
                if is_rain {
                    let idx = ((tick / 2 + x as u64 * 13 + y as u64 * 7) % charset.len() as u64) as usize;
                    let ch = charset[idx];
                    let color = match (rain_pos + tick as u16) % 6 {
                        0 => Color::Cyan,
                        1 => Color::Magenta,
                        2 => Color::Yellow,
                        3 => Color::Green,
                        4 => Color::LightBlue,
                        _ => Color::LightMagenta,
                    };
                    let flicker = ((tick + (x as u64 * 13 + y as u64 * 7)) % 11) < 2;
                    let style = if flicker {
                        Style::default().fg(color).add_modifier(Modifier::BOLD | Modifier::RAPID_BLINK)
                    } else {
                        Style::default().fg(color)
                    };
                    f.render_widget(
                        Paragraph::new(ch).style(style),
                        Rect::new(area.x + x, area.y + y, 1, 1),
                    );
                }
            }
        }
    }
}
