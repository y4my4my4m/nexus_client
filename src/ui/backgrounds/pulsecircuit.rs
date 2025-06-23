use ratatui::{Frame, layout::Rect, style::{Style, Color, Modifier}, widgets::Paragraph};
use crate::app::App;
use crate::ui::backgrounds::Background;
use crate::ui::themes::{Theme, ThemeColors, AccentColors};

pub struct PulseCircuitBackground;

impl Background for PulseCircuitBackground {
    fn name(&self) -> &'static str { "PulseCircuit" }
    fn draw_background(&self, f: &mut Frame, app: &App, area: Rect) {
        let tick = app.ui.tick_count;
        let w = area.width as f32;
        let h = area.height as f32;
        let cx = area.x as f32 + w / 2.0;
        let cy = area.y as f32 + h / 2.0;
        let t = tick as f32 * 0.06;
        let num_traces = 10;
        for i in 0..num_traces {
            let phase = t + i as f32 * 0.7;
            let r = w.min(h) * (0.18 + 0.07 * (phase * 1.2).sin());
            let angle = phase * 0.8 + (phase * 0.17).cos() * 0.7;
            let x0 = cx + r * angle.cos();
            let y0 = cy + r * angle.sin();
            let x1 = cx - r * angle.cos();
            let y1 = cy - r * angle.sin();
            let color = match i % 6 {
                0 => Color::Cyan,
                1 => Color::Magenta,
                2 => Color::Yellow,
                3 => Color::Green,
                4 => Color::LightBlue,
                _ => Color::LightMagenta,
            };
            draw_line(f, area, x0, y0, x1, y1, color);
            // Pulse
            let pulse_pos = (t * 2.0 + i as f32 * 1.3).sin() * 0.5 + 0.5;
            let px = x0 + (x1 - x0) * pulse_pos;
            let py = y0 + (y1 - y0) * pulse_pos;
            if let Some((tx, ty)) = to_cell(area, px, py) {
                f.render_widget(
                    Paragraph::new("●").style(Style::default().fg(color).add_modifier(Modifier::BOLD)),
                    Rect::new(tx, ty, 1, 1),
                );
            }
        }
        // Random glitch sparks
        if (tick % 5) == 0 {
            let gx = cx + (t * 3.0).sin() * w * 0.4;
            let gy = cy + (t * 2.0).cos() * h * 0.4;
            if let Some((tx, ty)) = to_cell(area, gx, gy) {
                f.render_widget(Paragraph::new("✶").style(Style::default().fg(Color::Yellow)), Rect::new(tx, ty, 1, 1));
            }
        }
    }
}
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
