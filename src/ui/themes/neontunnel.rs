use ratatui::{Frame, layout::Rect, style::{Style, Color, Modifier}, widgets::Paragraph};
use crate::app::App;
use super::{Theme, ThemeColors, AccentColors};

pub struct NeonTunnelTheme;

impl Theme for NeonTunnelTheme {
    fn name(&self) -> &'static str { "NeonTunnel" }
    fn draw_background(&self, f: &mut Frame, app: &App, area: Rect) {
        let tick = app.ui.tick_count;
        let w = area.width as f32;
        let h = area.height as f32;
        let cx = area.x as f32 + w / 2.0;
        let cy = area.y as f32 + h / 2.0;
        let t = tick as f32 * 0.07;
        let num_rings = 18;
        for i in 0..num_rings {
            let z = (i as f32 * 0.25 + t) % 5.0;
            let scale = 1.0 / (z + 0.7);
            let radius = w.min(h) * 0.38 * scale;
            let color = match i % 6 {
                0 => Color::Cyan,
                1 => Color::Magenta,
                2 => Color::Yellow,
                3 => Color::Green,
                4 => Color::LightBlue,
                _ => Color::LightMagenta,
            };
            let segs = 24;
            for j in 0..segs {
                let theta0 = j as f32 * std::f32::consts::TAU / segs as f32;
                let theta1 = (j + 1) as f32 * std::f32::consts::TAU / segs as f32;
                let x0 = cx + radius * theta0.cos();
                let y0 = cy + radius * theta0.sin();
                let x1 = cx + radius * theta1.cos();
                let y1 = cy + radius * theta1.sin();
                draw_line(f, area, x0, y0, x1, y1, color);
            }
        }
        // Center vanishing point
        f.render_widget(
            Paragraph::new("✦").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Rect::new(cx as u16, cy as u16, 1, 1),
        );
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
// --- Drawing helpers ---
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
