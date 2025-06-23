use ratatui::{Frame, layout::Rect, style::{Style, Color, Modifier}, widgets::Paragraph};
use crate::app::App;
use super::{Theme, ThemeColors, AccentColors};

pub struct CyberGridTheme;

impl Theme for CyberGridTheme {
    fn name(&self) -> &'static str { "CyberGrid" }
    fn draw_background(&self, f: &mut Frame, app: &App, area: Rect) {
        // Massive animated 3D wireframe grid with perspective and color cycling
        let tick = app.ui.tick_count;
        let w = area.width as f32;
        let h = area.height as f32;
        let cx = area.x as f32 + w / 2.0;
        let cy = area.y as f32 + h / 2.0;
        let t = tick as f32 * 0.035;
        let grid_w = 32;
        let grid_h = 18;
        let cam_z = 18.0 + (t * 0.7).sin() * 7.0 + (t * 0.13).cos() * 3.0;
        let cam_x = (t * 0.23).sin() * 7.0;
        let cam_y = (t * 0.19).cos() * 5.0;
        let fov = 1.2 + (t * 0.09).sin() * 0.4;
        // Project 3D grid points to 2D
        let mut points = vec![];
        for i in 0..=grid_w {
            for j in 0..=grid_h {
                let x = (i as f32 - grid_w as f32 / 2.0) * 1.2;
                let y = (j as f32 - grid_h as f32 / 2.0) * 0.9;
                let z = ((i as f32 * 0.3 + j as f32 * 0.2 + t * 1.2).sin() * 2.0) + (t * 0.5).cos() * 1.2;
                // Camera transform
                let px = x - cam_x;
                let py = y - cam_y;
                let pz = z - cam_z;
                // Perspective projection
                let scale = fov / (pz + 20.0);
                let sx = cx + px * scale * w * 0.18;
                let sy = cy + py * scale * h * 0.28;
                points.push(((i, j), (sx, sy), scale));
            }
        }
        // Draw grid lines
        for i in 0..=grid_w {
            for j in 0..=grid_h {
                let idx = i * (grid_h + 1) + j;
                if i < grid_w {
                    let idx2 = (i + 1) * (grid_h + 1) + j;
                    let (p1, p2) = (points[idx].1, points[idx2].1);
                    let color = match (i + j + (tick / 3) as usize) % 6 {
                        0 => Color::Cyan,
                        1 => Color::Magenta,
                        2 => Color::Yellow,
                        3 => Color::Green,
                        4 => Color::LightBlue,
                        _ => Color::LightMagenta,
                    };
                    draw_line(f, area, p1.0, p1.1, p2.0, p2.1, color);
                }
                if j < grid_h {
                    let idx2 = i * (grid_h + 1) + (j + 1);
                    let (p1, p2) = (points[idx].1, points[idx2].1);
                    let color = match (i * 2 + j + (tick / 2) as usize) % 6 {
                        0 => Color::Cyan,
                        1 => Color::Magenta,
                        2 => Color::Yellow,
                        3 => Color::Green,
                        4 => Color::LightBlue,
                        _ => Color::LightMagenta,
                    };
                    draw_line(f, area, p1.0, p1.1, p2.0, p2.1, color);
                }
            }
        }
        // Draw animated nodes
        for &((_i, _j), (sx, sy), scale) in &points {
            if scale > 0.01 && scale < 0.25 {
                let flicker = ((tick as f32 * 0.7 + sx + sy) as i32 % 9) < 2;
                let color = if flicker {
                    Color::White
                } else {
                    Color::Cyan
                };
                if let Some((tx, ty)) = to_cell(area, sx, sy) {
                    f.render_widget(
                        Paragraph::new("•").style(Style::default().fg(color).add_modifier(Modifier::BOLD)),
                        Rect::new(tx, ty, 1, 1),
                    );
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
    let (x0, y0) = (x0.round() as i32, y0.round() as i32);
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
