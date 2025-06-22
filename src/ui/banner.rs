use ratatui::{Frame, layout::Rect, widgets::Paragraph, widgets::Block, widgets::Borders, style::{Color, Style, Modifier}, text::{Line, Span}};
use crate::app::App;
use crate::banner::get_styled_banner_lines;
use crate::global_prefs::global_prefs;
use rand::prelude::*;

pub fn draw_full_banner(f: &mut Frame, app: &App, area: Rect) {
    let banner_lines = get_styled_banner_lines(area.width, app.ui.tick_count);
    let banner = Paragraph::new(banner_lines)
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(banner, area);
}

pub fn draw_min_banner(f: &mut Frame, app: &App, area: Rect) {
    let prefs = global_prefs();
    let glitch_enabled = prefs.minimal_banner_glitch_enabled;
    
    let mut rng = thread_rng();
    let time_factor = app.ui.tick_count as f64 * 0.05; // Slower animation for minimal effect
    
    let banner_text = "NEXUS";
    let mut spans = Vec::new();
    
    for (i, char) in banner_text.chars().enumerate() {
        let char_pos = i as f64 / banner_text.len() as f64;
        
        let mut final_char = char;
        let mut style = Style::default();
        
        // Base gradient color (cyan to magenta but more muted)
        let color = if char_pos < 0.5 {
            Color::Rgb(
                (char_pos * 2.0 * 150.0) as u8 + 50,
                200,
                255 - (char_pos * 2.0 * 50.0) as u8,
            )
        } else {
            Color::Rgb(
                200,
                200 - ((char_pos - 0.5) * 2.0 * 100.0) as u8,
                255,
            )
        };
        style = style.fg(color);
        
        // Apply glitch effects only if enabled in preferences
        if glitch_enabled {
            // Very subtle glitch probability
            let glitch_wave = (time_factor + char_pos * 8.0).sin();
            let base_glitch_chance = 0.001 + (glitch_wave * 0.5 + 0.5).powi(3) * 0.01;
            
            // Minimal glitch effects
            if rng.gen_bool(base_glitch_chance) {
                match rng.gen_range(0..100) {
                    // Color shift (most common - 60% of glitches)
                    0..=59 => {
                        let shift_colors = [
                            Color::Rgb(255, 100, 255), // Bright magenta
                            Color::Rgb(100, 255, 255), // Bright cyan  
                            Color::Rgb(255, 255, 100), // Bright yellow
                            Color::Rgb(150, 255, 150), // Light green
                        ];
                        style = style.fg(shift_colors.choose(&mut rng).unwrap_or(&Color::Magenta).clone());
                    }
                    // Character corruption (20% of glitches)
                    60..=79 => {
                        let glitch_chars = ['█', '▓', '▒', '░', '■', '□', 'X', '#'];
                        final_char = glitch_chars.choose(&mut rng).unwrap_or(&char).clone();
                        style = style.fg(Color::Rgb(255, 50, 255));
                    }
                    // Intensity change (15% of glitches)
                    80..=94 => {
                        if rng.gen_bool(0.5) {
                            style = style.add_modifier(Modifier::BOLD);
                        } else {
                            style = style.add_modifier(Modifier::DIM);
                        }
                    }
                    // Flicker (5% of glitches)
                    _ => {
                        style = style.add_modifier(Modifier::RAPID_BLINK);
                    }
                }
            }
            
            // Very subtle random intensity variations (much less frequent than main banner)
            if rng.gen_bool(0.02) {
                if rng.gen_bool(0.5) {
                    style = style.add_modifier(Modifier::BOLD);
                } else {
                    style = style.add_modifier(Modifier::DIM);
                }
            }
        }
        
        spans.push(Span::styled(final_char.to_string(), style));
    }
    
    // Occasional subtle background tint (only if glitch is enabled)
    let mut block_style = Style::default();
    if glitch_enabled && rng.gen_bool(0.005) {
        block_style = block_style.bg(Color::Rgb(20, 0, 20));
    }
    
    let banner_line = Line::from(spans);
    let banner = Paragraph::new(vec![banner_line])
        .block(Block::default().borders(Borders::ALL).style(block_style))
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(banner, area);
}
