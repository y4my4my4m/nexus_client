use ratatui::{
    Frame,
    layout::Rect,
    style::{Style, Color, Modifier},
    widgets::Paragraph,
};
use crate::app::App;
use super::{Theme, ThemeColors, AccentColors};

pub struct GeometryTheme;

impl Theme for GeometryTheme {
    fn name(&self) -> &'static str {
        "Geometry"
    }

    fn draw_background(&self, f: &mut Frame, app: &App, area: Rect) {
        let tick = app.ui.tick_count;
        let w = area.width as f32;
        let h = area.height as f32;
        let cx = area.x as f32 + w / 2.0;
        let cy = area.y as f32 + h / 2.0;

        // Matrix maze background: vertical and horizontal lines
        for i in 0..w as u16 {
            if i % 7 == (tick % 7) as u16 {
                for y in 0..area.height {
                    let cell = Rect::new(area.x + i, area.y + y, 1, 1);
                    f.render_widget(
                        Paragraph::new("│").style(Style::default().fg(Color::Black)),
                        cell,
                    );
                }
            }
        }
        for j in 0..h as u16 {
            if j % 5 == ((tick / 2) % 5) as u16 {
                for x in 0..area.width {
                    let cell = Rect::new(area.x + x, area.y + j, 1, 1);
                    f.render_widget(
                        Paragraph::new("─").style(Style::default().fg(Color::Black)),
                        cell,
                    );
                }
            }
        }

        // Rotating wireframe pyramid (triangle base)
        let pyramid_height = h.min(w) * 0.35;
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

        // Draw base edges
        for i in 0..3 {
            let (x0, y0) = base_points[i];
            let (x1, y1) = base_points[(i + 1) % 3];
            draw_line(f, area, x0, y0, x1, y1, Color::Cyan);
        }
        // Draw sides
        for &(bx, by) in &base_points {
            draw_line(f, area, bx, by, apex.0, apex.1, Color::Magenta);
        }
        // Draw apex
        let apex_cell = to_cell(area, apex.0, apex.1);
        if let Some((ax, ay)) = apex_cell {
            f.render_widget(
                Paragraph::new("▲").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Rect::new(ax, ay, 1, 1),
            );
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

// Helper: Bresenham's line algorithm for terminal cells
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

// Helper: map float coordinates to terminal cell (u16)
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
