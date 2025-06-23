use ratatui::{
    Frame,
    layout::Rect,
    style::{Style, Color, Modifier},
    widgets::Paragraph,
};
use crate::app::App;
use crate::ui::backgrounds::Background;
use crate::ui::themes::{Theme, ThemeColors, AccentColors};

pub struct GeometryBackground;

impl Background for GeometryBackground {
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

        // --- 3D Rotating, Zoomed-In Wireframe Pyramid ---
        let t = tick as f32 * 0.045;
        let zoom = 1.25; // much bigger
        let base_radius = w.min(h) * 0.38 * zoom;
        let pyramid_height = w.min(h) * 0.7 * zoom;
        // 3D rotation angles
        let yaw = t * 0.7;
        let pitch = (t * 0.5).sin() * 0.7 + 0.7;
        // 3D points: base (triangle) and apex
        let mut pts3d = vec![];
        for i in 0..3 {
            let theta = i as f32 * std::f32::consts::TAU / 3.0;
            let x = base_radius * theta.cos();
            let y = base_radius * theta.sin();
            let z = 0.0;
            pts3d.push([x, y, z]);
        }
        let apex3d = [0.0, 0.0, -pyramid_height];
        // 3D rotation (yaw, then pitch)
        let rot_yaw = |p: [f32;3]| {
            let (sy, cy) = yaw.sin_cos();
            [
                p[0]*cy - p[2]*sy,
                p[1],
                p[0]*sy + p[2]*cy,
            ]
        };
        let rot_pitch = |p: [f32;3]| {
            let (sp, cp) = pitch.sin_cos();
            [
                p[0],
                p[1]*cp - p[2]*sp,
                p[1]*sp + p[2]*cp,
            ]
        };
        let project = |p: [f32;3]| {
            // Perspective projection
            let persp = 1.2 / (p[2] * 0.018 + 2.8);
            [cx + p[0] * persp, cy + p[1] * persp]
        };
        let pts2d: Vec<_> = pts3d.iter().map(|&p| project(rot_pitch(rot_yaw(p)))).collect();
        let apex2d = project(rot_pitch(rot_yaw(apex3d)));
        // Draw base edges
        for i in 0..3 {
            let [x0, y0] = pts2d[i];
            let [x1, y1] = pts2d[(i + 1) % 3];
            draw_line(f, area, x0, y0, x1, y1, Color::Cyan);
        }
        // Draw sides
        for &[x, y] in &pts2d {
            draw_line(f, area, x, y, apex2d[0], apex2d[1], Color::Magenta);
        }
        // Draw apex
        if let Some((ax, ay)) = to_cell(area, apex2d[0], apex2d[1]) {
            f.render_widget(
                Paragraph::new("▲").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Rect::new(ax, ay, 1, 1),
            );
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
