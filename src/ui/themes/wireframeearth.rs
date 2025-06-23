use ratatui::{Frame, layout::Rect, style::{Style, Color, Modifier}, widgets::Paragraph};
use crate::app::App;
use super::{Theme, ThemeColors, AccentColors};

pub struct WireframeEarthTheme;

impl Theme for WireframeEarthTheme {
    fn name(&self) -> &'static str { "WireframeEarth" }
    fn draw_background(&self, f: &mut Frame, app: &App, area: Rect) {
        let tick = app.ui.tick_count;
        let w = area.width as f32;
        let h = area.height as f32;
        // Ensure the globe stays centered in the background
        // Adjust the center coordinates to ensure the camera targets the Earth's center
        let radius = (w.min(h) * 0.4).max(20.0); // Ensure a minimum radius
        let cx = area.x as f32 + area.width as f32 / 2.0;
        let cy = area.y as f32 + area.height as f32 / 2.0;
        // Remove the 90-degree tilt and align the camera with the equatorial plane
        let camera_tilt = 0.0; // No tilt
        // Add a natural tilt to the Earth (23.5 degrees)
        let earth_tilt_x = 23.5_f32.to_radians(); // Tilt around X axis
        let earth_tilt_y = 15.0_f32.to_radians(); // Additional tilt around Y axis
        let t = tick as f32 * 0.006;
        let yaw = t * 0.35 + 0.7;
        let lat_lines = 12;
        let lon_lines = 12;
        let points_per_line = 64;
        let front_color = Color::Gray;
        let back_color = Color::DarkGray;
        // Latitude lines (horizontal circles)
        for i in 1..lat_lines {
            let phi = std::f32::consts::PI * (i as f32 / lat_lines as f32 - 0.5);
            let mut prev: Option<(f32, f32, f32)> = None;
            for j in 0..=points_per_line {
                let theta = 2.0 * std::f32::consts::PI * (j as f32 / points_per_line as f32) + yaw;
                let x = radius * phi.cos() * theta.cos();
                let y = radius * phi.cos() * theta.sin();
                let z = radius * phi.sin();
                // Apply yaw (Z axis), then Earth's diagonal tilt (X and Y axes)
                let (x, y, z) = rotate3d_z_then_xy(x, y, z, yaw, earth_tilt_x, earth_tilt_y);
                let sx = cx + x;
                let sy = cy - z; // Adjust for camera tilt
                if let Some((px, py, pz)) = prev {
                    let color = if z > 0.0 && pz > 0.0 { front_color } else { back_color };
                    draw_line(f, area, px, py, sx, sy, color);
                }
                prev = Some((sx, sy, z));
            }
        }
        // Longitude lines (vertical meridians)
        for j in 0..lon_lines {
            let theta = 2.0 * std::f32::consts::PI * (j as f32 / lon_lines as f32) + yaw;
            let mut prev: Option<(f32, f32, f32)> = None;
            for i in 0..=points_per_line {
                let phi = std::f32::consts::PI * (i as f32 / points_per_line as f32 - 0.5);
                let x = radius * phi.cos() * theta.cos();
                let y = radius * phi.cos() * theta.sin();
                let z = radius * phi.sin();
                // Apply yaw (Z axis), then Earth's diagonal tilt (X and Y axes)
                let (x, y, z) = rotate3d_z_then_xy(x, y, z, yaw, earth_tilt_x, earth_tilt_y);
                let sx = cx + x;
                let sy = cy - z; // Adjust for camera tilt
                if let Some((px, py, pz)) = prev {
                    let color = if z > 0.0 && pz > 0.0 { front_color } else { back_color };
                    draw_line(f, area, px, py, sx, sy, color);
                }
                prev = Some((sx, sy, z));
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

fn draw_line(f: &mut Frame, area: Rect, x0: f32, y0: f32, x1: f32, y1: f32, color: Color) {
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

// New: rotate around Z axis, then tilt around X and Y axes
fn rotate3d_z_then_xy(x: f32, y: f32, z: f32, yaw: f32, tilt_x: f32, tilt_y: f32) -> (f32, f32, f32) {
    // Rotate around Z axis (equatorial spin)
    let (sz, cz) = yaw.sin_cos();
    let x1 = x * cz - y * sz;
    let y1 = x * sz + y * cz;
    let z1 = z;
    // Tilt around X axis
    let (sx, cx) = tilt_x.sin_cos();
    let y2 = y1 * cx - z1 * sx;
    let z2 = y1 * sx + z1 * cx;
    // Tilt around Y axis
    let (sy, cy) = tilt_y.sin_cos();
    let x3 = x1 * cy + z2 * sy;
    let z3 = -x1 * sy + z2 * cy;
    (x3, y2, z3)
}

