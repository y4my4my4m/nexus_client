use ratatui::{Frame, layout::Rect, style::{Style, Color, Modifier}, widgets::Paragraph};
use crate::app::App;
use super::{Theme, ThemeColors, AccentColors};

pub struct FractalGridTheme;

impl Theme for FractalGridTheme {
    fn name(&self) -> &'static str { "FractalGrid" }
    fn draw_background(&self, f: &mut Frame, app: &App, area: Rect) {
        // Deep animated fractal tunnel with recursive geometry and color cycling
        let tick = app.ui.tick_count;
        let w = area.width as f32;
        let h = area.height as f32;
        let t = tick as f32 * 0.045;
        let depth = 7; // much deeper recursion
        draw_fractal_tunnel(f, area, w, h, depth, t);
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

// Replace draw_fractal_grid with a fractal tunnel effect
fn draw_fractal_tunnel(f: &mut Frame, area: Rect, w: f32, h: f32, _depth: usize, t: f32) {
    let cx = area.x as f32 + w / 2.0;
    let cy = area.y as f32 + h / 2.0;
    // Slower, more 3D tunnel effect
    let tunnel_speed = 0.025; // much slower
    let cam_z = (t * tunnel_speed) % 1.0;
    let num_rings = 36;
    let segs = 64;
    let colors = [Color::Cyan, Color::Magenta, Color::Yellow, Color::White];
    let wobble = ((t * 0.18).sin() * 0.18, (t * 0.13).cos() * 0.12);
    for i in 0..num_rings {
        let z = i as f32 * 1.1 - cam_z * 24.0;
        let scale = 1.0 / (z + 2.2);
        // Perspective: elliptical rings for 3D feel
        let ellipse = 0.85 + (z * 0.04).sin() * 0.13;
        let radius = w.min(h) * 0.48 * scale;
        let color = if i == num_rings - 1 {
            Color::White
        } else {
            colors[i % colors.len()]
        };
        for j in 0..segs {
            let theta0 = j as f32 * std::f32::consts::TAU / segs as f32;
            let theta1 = (j + 1) as f32 * std::f32::consts::TAU / segs as f32;
            let x0 = cx + (radius * theta0.cos() * ellipse) + wobble.0 * w;
            let y0 = cy + (radius * theta0.sin()) + wobble.1 * h;
            let x1 = cx + (radius * theta1.cos() * ellipse) + wobble.0 * w;
            let y1 = cy + (radius * theta1.sin()) + wobble.1 * h;
            draw_line(f, area, x0, y0, x1, y1, color);
        }
    }
    // Draw vanishing point
    f.render_widget(
        Paragraph::new("✦").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Rect::new(cx as u16, cy as u16, 1, 1),
    );
}

fn draw_line(f: &mut Frame, area: Rect, x0: f32, y0: f32, x1: f32, y1: f32, color: Color) {
    if let (Some((tx0, ty0)), Some((tx1, ty1))) = (to_cell(area, x0, y0), to_cell(area, x1, y1)) {
        let dx = (tx1 as i32 - tx0 as i32).abs();
        let dy = (ty1 as i32 - ty0 as i32).abs();
        let sx = if tx0 < tx1 { 1 } else { -1 };
        let sy = if ty0 < ty1 { 1 } else { -1 };
        let mut err = dx - dy;
        let mut x = tx0;
        let mut y = ty0;
        loop {
            f.render_widget(Paragraph::new("█").style(Style::default().fg(color)), Rect::new(x, y, 1, 1));
            if x == tx1 && y == ty1 { break; }
            let e2 = err * 2;
            if e2 > -dy { err -= dy; x = (x as i32 + sx) as u16; }
            if e2 < dx { err += dx; y = (y as i32 + sy) as u16; }
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
