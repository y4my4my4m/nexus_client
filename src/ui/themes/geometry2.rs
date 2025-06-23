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
        use rand::{Rng, SeedableRng};
        use rand::rngs::StdRng;
        let tick = app.ui.tick_count;
        let w = area.width as f32;
        let h = area.height as f32;
        let cx = area.x as f32 + w / 2.0;
        let cy = area.y as f32 + h / 2.0;
        let mut rng = StdRng::seed_from_u64(tick as u64 / 2);

        // Camera: chaotic demo-scene movement
        let t = tick as f32 * 0.025;
        let cam_radius = 3.5 + (t * 0.7).sin() * 1.2 + (t * 0.13).cos() * 0.7;
        let cam_yaw = t * 0.8 + (t * 0.23).sin() * 1.2 + (t * 0.11).cos() * 0.7;
        let cam_pitch = 0.8 + (t * 0.17).cos() * 0.5 + (t * 0.31).sin() * 0.3;
        let cam_x = cam_radius * cam_yaw.cos() * cam_pitch.cos();
        let cam_y = cam_radius * cam_yaw.sin() * cam_pitch.cos();
        let cam_z = cam_radius * cam_pitch.sin() + (t * 0.7).cos() * 0.5;
        let camera = [cam_x, cam_y, cam_z];
        let look_at = [0.0, 0.0, 0.0];
        let up = [0.0, 0.0, 1.0];
        let fov = 1.1 + (t * 0.09).sin() * 0.4;
        let view = look_at_matrix(camera, look_at, up);
        let proj = perspective_matrix(fov, w / h, 0.1, 12.0);

        // Demo-scene: field of animated triangles/lines
        let num_tris = 18;
        for i in 0..num_tris {
            // Each triangle has its own orbit, scale, and color cycling
            let phase = t + i as f32 * 0.7;
            let r = 1.2 + (phase * 0.8).sin() * 0.7 + (phase * 0.33).cos() * 0.3;
            let base_angle = phase * 1.2 + (phase * 0.17).cos() * 0.7;
            let z = (phase * 0.9).sin() * 1.2 + (phase * 0.5).cos() * 0.7;
            let mut tri3d = vec![];
            for j in 0..3 {
                let theta = base_angle + j as f32 * (2.0 * std::f32::consts::PI / 3.0);
                let x = r * theta.cos();
                let y = r * theta.sin();
                tri3d.push([x, y, z]);
            }
            // Color cycling
            let color = match i % 6 {
                0 => Color::Cyan,
                1 => Color::Magenta,
                2 => Color::Yellow,
                3 => Color::Green,
                4 => Color::LightBlue,
                _ => Color::LightMagenta,
            };
            // Flicker effect
            let flicker = rng.gen_bool(0.92) || (tick % 3 != 0);
            if !flicker { continue; }
            // Project to 2D
            let mut pts2d = Vec::new();
            for v in &tri3d {
                let p = mat4_mul_vec3(&proj, &mat4_mul_vec3(&view, v));
                let sx = cx + p[0] * w * 0.38;
                let sy = cy - p[1] * h * 0.38;
                pts2d.push((sx, sy));
            }
            // Draw triangle edges
            for j in 0..3 {
                let (x0, y0) = pts2d[j];
                let (x1, y1) = pts2d[(j + 1) % 3];
                draw_line(f, area, x0, y0, x1, y1, color);
            }
            // Optionally, draw a glowing dot at the centroid
            let centroid = (
                (pts2d[0].0 + pts2d[1].0 + pts2d[2].0) / 3.0,
                (pts2d[0].1 + pts2d[1].1 + pts2d[2].1) / 3.0,
            );
            if let Some((cx, cy)) = to_cell(area, centroid.0, centroid.1) {
                f.render_widget(
                    Paragraph::new("•").style(Style::default().fg(color).add_modifier(Modifier::BOLD)),
                    Rect::new(cx, cy, 1, 1),
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

// --- 3D math helpers ---
fn look_at_matrix(eye: [f32; 3], center: [f32; 3], up: [f32; 3]) -> [[f32; 4]; 4] {
    let f = normalize([
        center[0] - eye[0],
        center[1] - eye[1],
        center[2] - eye[2],
    ]);
    let s = normalize(cross(f, up));
    let u = cross(s, f);
    [
        [s[0], u[0], -f[0], 0.0],
        [s[1], u[1], -f[1], 0.0],
        [s[2], u[2], -f[2], 0.0],
        [
            -dot(s, eye),
            -dot(u, eye),
            dot(f, eye),
            1.0,
        ],
    ]
}
fn perspective_matrix(fov: f32, aspect: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
    let f = 1.0 / (fov / 2.0).tan();
    [
        [f / aspect, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, (far + near) / (near - far), -1.0],
        [0.0, 0.0, (2.0 * far * near) / (near - far), 0.0],
    ]
}
fn mat4_mul_vec3(m: &[[f32; 4]; 4], v: &[f32; 3]) -> [f32; 3] {
    let x = m[0][0] * v[0] + m[1][0] * v[1] + m[2][0] * v[2] + m[3][0];
    let y = m[0][1] * v[0] + m[1][1] * v[1] + m[2][1] * v[2] + m[3][1];
    let z = m[0][2] * v[0] + m[1][2] * v[1] + m[2][2] * v[2] + m[3][2];
    let w = m[0][3] * v[0] + m[1][3] * v[1] + m[2][3] * v[2] + m[3][3];
    if w.abs() > 1e-5 {
        [x / w, y / w, z / w]
    } else {
        [x, y, z]
    }
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 1e-5 {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        v
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
