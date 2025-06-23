use ratatui::{Frame, layout::Rect, style::{Style, Color, Modifier}, widgets::Paragraph};
use crate::app::App;
use crate::ui::backgrounds::Background;

pub struct HackerGlyphsBackground;

impl Background for HackerGlyphsBackground {
    fn name(&self) -> &'static str { "HackerGlyphs" }
    fn draw_background(&self, f: &mut Frame, app: &App, area: Rect) {
        let tick = app.ui.tick_count;
        let w = area.width as f32;
        let h = area.height as f32;
        let cx = area.x as f32 + w / 2.0;
        let cy = area.y as f32 + h / 2.0;
        let t = tick as f32 * 0.045;
        let glyphs = ["λ", "Σ", "Ψ", "Ω", "Ξ", "Φ", "Δ", "π", "β", "μ", "∑", "∇", "∂", "∫", "∴", "∞"];
        let num = 22;
        for i in 0..num {
            let phase = t + i as f32 * 0.5;
            let r = w.min(h) * (0.18 + 0.22 * (phase * 0.7).sin());
            let angle = phase * 1.2 + (phase * 0.17).cos() * 1.7;
            let x = cx + r * angle.cos();
            let y = cy + r * angle.sin();
            let idx = ((tick / 2 + i as u64 * 13) % glyphs.len() as u64) as usize;
            let color = match i % 6 {
                0 => Color::Cyan,
                1 => Color::Magenta,
                2 => Color::Yellow,
                3 => Color::Green,
                4 => Color::LightBlue,
                _ => Color::LightMagenta,
            };
            let flicker = ((tick + i as u64 * 7) % 9) < 2;
            let style = if flicker {
                Style::default().fg(color).add_modifier(Modifier::BOLD | Modifier::RAPID_BLINK)
            } else {
                Style::default().fg(color)
            };
            if let Some((tx, ty)) = to_cell(area, x, y) {
                f.render_widget(
                    Paragraph::new(glyphs[idx]).style(style),
                    Rect::new(tx, ty, 1, 1),
                );
            }
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
