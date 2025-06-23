use ratatui::{Frame, layout::Rect, style::{Style, Color, Modifier}, widgets::Paragraph};
use crate::app::App;
use super::{Theme, ThemeColors, AccentColors};

pub struct CyberGridTheme;

impl Theme for CyberGridTheme {
    fn name(&self) -> &'static str { "CyberGrid" }
    fn draw_background(&self, f: &mut Frame, app: &App, area: Rect) {
        let tick = app.ui.tick_count;
        let w = area.width as f32;
        let h = area.height as f32;
        let cx = area.x as f32 + w / 2.0;
        let cy = area.y as f32 + h / 2.0;
        let grid_size = 12;
        let t = tick as f32 * 0.04;
        for i in 0..=grid_size {
            let frac = i as f32 / grid_size as f32;
            let x = cx + (frac - 0.5) * w * 0.9;
            let y = cy + (frac - 0.5) * h * 0.9;
            // Vertical lines
            draw_line(f, area, x, cy - h * 0.45, x, cy + h * 0.45, Color::Cyan);
            // Horizontal lines
            draw_line(f, area, cx - w * 0.45, y, cx + w * 0.45, y, Color::Magenta);
        }
        // Flickering nodes
        for i in 0..=grid_size {
            for j in 0..=grid_size {
                let fx = cx + (i as f32 / grid_size as f32 - 0.5) * w * 0.9;
                let fy = cy + (j as f32 / grid_size as f32 - 0.5) * h * 0.9;
                let flicker = ((tick + (i * 13 + j * 7) as u64) % 7) < 2;
                if flicker {
                    let color = match (i + j) % 3 {
                        0 => Color::Yellow,
                        1 => Color::Cyan,
                        _ => Color::Magenta,
                    };
                    if let Some((tx, ty)) = to_cell(area, fx, fy) {
                        f.render_widget(
                            Paragraph::new("●").style(Style::default().fg(color).add_modifier(Modifier::BOLD)),
                            Rect::new(tx, ty, 1, 1),
                        );
                    }
                }
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

// --- Drawing helpers (copied from geometry.rs) ---
fn draw_line(
    f: &mut Frame,
    area: Rect,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    color: Color,
) {
    let (mut x0, mut y0) = (x0.round() as i32, y0.round() as i32);
    let (x1, y1) = (x1.round() as i32, y1.round() as i32);
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut x = x0;
    let mut y = y0;
    loop {
        if let Some((tx, ty)) = to_cell(area, x as f32, y as f32) {
            f.render_widget(
                Paragraph::new("•").style(Style::default().fg(color)),
                Rect::new(tx, ty, 1, 1),
            );
        }
        if x == x1 && y == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy {
            if x == x1 { break; }
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            if y == y1 { break; }
            err += dx;
            y += sy;
        }
    }
}
fn to_cell(area: Rect, x: f32, y: f32) -> Option<(u16, u16)> {
    let tx = x.round() as i32;
    let ty = y.round() as i32;
    if tx >= area.x as i32
        && ty >= area.y as i32
        && tx < (area.x + area.width) as i32
        && ty < (area.y + area.height) as i32
    {
        Some((tx as u16, ty as u16))
    } else {
        None
    }
}
