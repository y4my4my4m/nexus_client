use ratatui::{
    Frame,
    layout::Rect,
    style::{Style, Color, Modifier},
    widgets::Paragraph,
};
use crate::app::App;
use super::{Theme, ThemeColors, AccentColors};

pub struct Geometry2Theme;

impl Theme for Geometry2Theme {
    fn name(&self) -> &'static str {
        "Geometry2"
    }

    fn draw_background(&self, f: &mut Frame, app: &App, area: Rect) {
        let tick = app.ui.tick_count;
        let w = area.width as usize;
        let h = area.height as usize;
        let mut bitmap = vec![vec![None; w]; h];

        // Matrix maze background: vertical and horizontal lines (bitmap)
        for i in 0..w {
            if i % 7 == (tick as usize % 7) {
                for y in 0..h {
                    bitmap[y][i] = Some(Color::DarkGray);
                }
            }
        }
        for j in 0..h {
            if j % 5 == ((tick / 2) as usize % 5) {
                for x in 0..w {
                    bitmap[j][x] = Some(Color::DarkGray);
                }
            }
        }

        // Rotating wireframe pyramid (triangle base)
        let cx = w as f32 / 2.0;
        let cy = h as f32 / 2.0;
        let pyramid_height = h.min(w) as f32 * 0.35;
        let base_radius = pyramid_height * 0.7;
        let angle = (tick as f32 / 16.0) % (std::f32::consts::PI * 2.0);

        // 3D pyramid points (projected to 2D)
        let base_points = (0..3).map(|i| {
            let theta = angle + i as f32 * (2.0 * std::f32::consts::PI / 3.0);
            let x = cx + base_radius * theta.cos();
            let y = cy + base_radius * theta.sin() * 0.5;
            (x, y)
        }).collect::<Vec<_>>();
        let apex = (cx, cy - pyramid_height);

        // Draw base edges (bitmap)
        for i in 0..3 {
            let (x0, y0) = base_points[i];
            let (x1, y1) = base_points[(i + 1) % 3];
            rasterize_line(&mut bitmap, x0, y0, x1, y1, Color::Cyan);
        }
        // Draw sides (bitmap)
        for &(bx, by) in &base_points {
            rasterize_line(&mut bitmap, bx, by, apex.0, apex.1, Color::Magenta);
        }
        // Draw apex (bitmap)
        let ax = apex.0.round() as isize;
        let ay = apex.1.round() as isize;
        if ax >= 0 && ay >= 0 && (ax as usize) < w && (ay as usize) < h {
            bitmap[ay as usize][ax as usize] = Some(Color::Yellow);
        }

        // Render the bitmap to the terminal
        for y in 0..h {
            for x in 0..w {
                if let Some(color) = bitmap[y][x] {
                    let cell = Rect::new(area.x + x as u16, area.y + y as u16, 1, 1);
                    let ch = if color == Color::Yellow { "▲" } else { "█" };
                    f.render_widget(
                        Paragraph::new(ch).style(Style::default().fg(color).add_modifier(Modifier::BOLD)),
                        cell,
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

// Rasterize a line into the bitmap using Bresenham's algorithm
fn rasterize_line(bitmap: &mut [Vec<Option<Color>>], x0: f32, y0: f32, x1: f32, y1: f32, color: Color) {
    let (mut x0, mut y0) = (x0.round() as isize, y0.round() as isize);
    let (x1, y1) = (x1.round() as isize, y1.round() as isize);
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let (w, h) = (bitmap[0].len() as isize, bitmap.len() as isize);
    loop {
        if x0 >= 0 && y0 >= 0 && x0 < w && y0 < h {
            bitmap[y0 as usize][x0 as usize] = Some(color);
        }
        if x0 == x1 && y0 == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy {
            if x0 == x1 { break; }
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            if y0 == y1 { break; }
            err += dx;
            y0 += sy;
        }
    }
}
