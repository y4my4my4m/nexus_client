use ratatui::{Frame, layout::Rect, style::{Style, Color, Modifier}, widgets::Paragraph};
use crate::app::App;
use super::{Theme, ThemeColors, AccentColors};

pub struct FractalGridTheme;

impl Theme for FractalGridTheme {
    fn name(&self) -> &'static str { "FractalGrid" }
    fn draw_background(&self, f: &mut Frame, app: &App, area: Rect) {
        let tick = app.ui.tick_count;
        let w = area.width as f32;
        let h = area.height as f32;
        let t = tick as f32 * 0.04;
        let depth = 3;
        draw_fractal_grid(f, area, 0.0, 0.0, w, h, depth, t);
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
fn draw_fractal_grid(f: &mut Frame, area: Rect, x: f32, y: f32, w: f32, h: f32, depth: usize, t: f32) {
    if depth == 0 || w < 4.0 || h < 2.0 {
        return;
    }
    let color = match (depth + (t as usize) % 6) % 6 {
        0 => Color::Cyan,
        1 => Color::Magenta,
        2 => Color::Yellow,
        3 => Color::Green,
        4 => Color::LightBlue,
        _ => Color::LightMagenta,
    };
    // Draw border
    for i in 0..w as u16 {
        let fx = x + i as f32;
        let fy1 = y;
        let fy2 = y + h - 1.0;
        if let Some((tx, ty)) = to_cell(area, fx, fy1) {
            f.render_widget(Paragraph::new("─").style(Style::default().fg(color)), Rect::new(tx, ty, 1, 1));
        }
        if let Some((tx, ty)) = to_cell(area, fx, fy2) {
            f.render_widget(Paragraph::new("─").style(Style::default().fg(color)), Rect::new(tx, ty, 1, 1));
        }
    }
    for j in 0..h as u16 {
        let fy = y + j as f32;
        let fx1 = x;
        let fx2 = x + w - 1.0;
        if let Some((tx, ty)) = to_cell(area, fx1, fy) {
            f.render_widget(Paragraph::new("│").style(Style::default().fg(color)), Rect::new(tx, ty, 1, 1));
        }
        if let Some((tx, ty)) = to_cell(area, fx2, fy) {
            f.render_widget(Paragraph::new("│").style(Style::default().fg(color)), Rect::new(tx, ty, 1, 1));
        }
    }
    // Glitch
    if (t * 7.0).sin() > 0.8 && w > 8.0 && h > 4.0 {
        let gx = x + w * 0.5 + (t * 3.0).sin() * (w * 0.2);
        let gy = y + h * 0.5 + (t * 2.0).cos() * (h * 0.2);
        if let Some((tx, ty)) = to_cell(area, gx, gy) {
            f.render_widget(Paragraph::new("▒").style(Style::default().fg(Color::Yellow)), Rect::new(tx, ty, 1, 1));
        }
    }
    // Recurse
    draw_fractal_grid(f, area, x + w * 0.25, y + h * 0.25, w * 0.5, h * 0.5, depth - 1, t + 1.7);
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
