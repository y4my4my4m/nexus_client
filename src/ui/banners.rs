use figlet_rs::FIGfont;
use rand::prelude::*;
use ratatui::{Frame, layout::Rect, style::{Color, Style, Modifier}, widgets::{Block, Paragraph, Borders}, text::{Line, Span}};
use crate::app::App;
use crate::global_prefs::global_prefs;

#[derive(Clone)]
struct BufferChar {
    char: char,
    style: Style,
    glitch_type: GlitchType,
}

#[derive(Clone, PartialEq)]
enum GlitchType {
    None,
    DataCorruption,
    RgbShift,
    Scanline,
    Flicker,
    Static,
}

pub fn get_styled_banner_lines(width: u16, tick_count: u64) -> Vec<Line<'static>> {
    
    // Load the FIGfont from file
    let font_data = include_str!("../../assets/fig/alligator2.flf");
    let custom_font = FIGfont::from_content(font_data).unwrap();
    let figlet_text = custom_font.convert("NEXUS").unwrap();
    
    // 1. Create a String that will live for the whole function scope.
    let figlet_string = figlet_text.to_string();
    // 2. Now, borrow from `figlet_string` to create the slices.
    let figlet_lines: Vec<&str> = figlet_string.lines().collect();
    
    let figlet_height = figlet_lines.len();
    let figlet_width = figlet_lines.get(0).map_or(0, |l| l.chars().count());

    let banner_height = figlet_height + 2; // Height of the banner in lines (1 line padding at top and bottom)
    let mut buffer: Vec<Vec<BufferChar>> = vec![
        vec![
            BufferChar {
                char: ' ',
                style: Style::default(),
                glitch_type: GlitchType::None,
            };
            width as usize
        ];
        banner_height
    ];

    let start_y = 1; // Start drawing after the top padding
    let start_x = (width as usize - figlet_width) / 2;

    // Base text rendering with cyberpunk colors
    for (y, line) in figlet_lines.iter().enumerate() {
        for (x, char) in line.chars().enumerate() {
            if let Some(cell) = buffer
                .get_mut(start_y + y)
                .and_then(|row| row.get_mut(start_x + x))
            {
                if char != ' ' {
                    cell.char = char;
                    // Gradient effect from cyan to magenta
                    let color_ratio = x as f32 / figlet_width.max(1) as f32;
                    let base_color = if color_ratio < 0.5 {
                        Color::Rgb(
                            (color_ratio * 2.0 * 255.0) as u8,
                            255,
                            255 - (color_ratio * 2.0 * 100.0) as u8,
                        )
                    } else {
                        Color::Rgb(
                            255,
                            255 - ((color_ratio - 0.5) * 2.0 * 155.0) as u8,
                            255,
                        )
                    };
                    cell.style = Style::default().fg(base_color);
                }
            }
        }
    }
    
    let mut rng = thread_rng();
    let time_factor = tick_count as f64 * 0.1;
    
    // Apply multiple glitch effects
    for (y, row) in buffer.iter_mut().enumerate() {
        let y_norm = y as f64 / banner_height as f64;
        
        // Horizontal scanline effects that move up and down
        let scanline_1 = ((time_factor + y_norm * 10.0).sin() * 0.5 + 0.5) * banner_height as f64;
        let scanline_2 = ((time_factor * 0.7 + y_norm * 15.0).cos() * 0.5 + 0.5) * banner_height as f64;
        let scanline_3 = ((time_factor * 1.3 + y_norm * 8.0).sin() * 0.3 + 0.7) * banner_height as f64;
        
        let is_scanline = (y as f64 - scanline_1).abs() < 1.0 || 
                         (y as f64 - scanline_2).abs() < 0.5 ||
                         (y as f64 - scanline_3).abs() < 2.0;
        
        // Vertical interference pattern
        let vertical_interference = ((time_factor * 2.0).sin() * 3.0 + 5.0) as usize;
        let has_vertical_glitch = y % vertical_interference == 0;
        
        for (x, cell) in row.iter_mut().enumerate() {
            let x_norm = x as f64 / width as f64;
            
            // Base glitch probability with multiple wave patterns
            let wave1 = (time_factor * 0.5 + y_norm * 8.0 + x_norm * 12.0).sin();
            let wave2 = (time_factor * 0.3 + y_norm * 15.0 + x_norm * 7.0).cos();
            let wave3 = (time_factor * 0.8 + y_norm * 5.0).sin();
            
            let base_glitch_chance = 0.001 + (wave1 * wave2 * wave3).powi(2) * 0.008;
            
            // Scanline effects
            if is_scanline {
                cell.style = cell.style
                    .bg(Color::Rgb(20, 0, 20))
                    .add_modifier(Modifier::DIM);
                cell.glitch_type = GlitchType::Scanline;
                
                if rng.gen_bool(0.3) {
                    cell.char = ['─', '═', '▬', '▭'].choose(&mut rng).unwrap_or(&'─').clone();
                    cell.style = cell.style.fg(Color::Rgb(100, 50, 150));
                }
            }
            
            // Vertical interference
            if has_vertical_glitch && rng.gen_bool(0.15) {
                cell.style = cell.style.bg(Color::Rgb(40, 0, 40));
                if cell.char != ' ' {
                    cell.style = cell.style.add_modifier(Modifier::BOLD);
                }
            }
            
            // RGB channel separation effect
            if rng.gen_bool(base_glitch_chance * 2.0) {
                cell.glitch_type = GlitchType::RgbShift;
                let shift_direction = rng.gen_range(0..4);
                match shift_direction {
                    0 => cell.style = cell.style.fg(Color::Red).bg(Color::Black),
                    1 => cell.style = cell.style.fg(Color::Green).bg(Color::Black),
                    2 => cell.style = cell.style.fg(Color::Blue).bg(Color::Black),
                    _ => cell.style = cell.style.fg(Color::Cyan).bg(Color::Rgb(50, 0, 0)),
                }
                
                // Sometimes add shifted characters
                if rng.gen_bool(0.6) {
                    let shift_chars = ['▌', '▐', '█', '▓', '▒', '░', '▄', '▀'];
                    cell.char = shift_chars.choose(&mut rng).unwrap_or(&'█').clone();
                }
            }
            
            // Data corruption effect
            else if rng.gen_bool(base_glitch_chance * 1.5) {
                cell.glitch_type = GlitchType::DataCorruption;
                let corruption_chars = [
                    '░', '▒', '▓', '█', '▌', '▐', '▄', '▀', '■', '□', '▪', '▫',
                    '0', '1', 'X', '#', '@', '%', '&', '*', '+', '-', '=', '|',
                    '/', '\\', '^', '~', '`', '<', '>', '{', '}', '[', ']'
                ];
                cell.char = corruption_chars.choose(&mut rng).unwrap_or(&'█').clone();
                
                // Corrupted colors
                let corruption_colors = [
                    Color::Rgb(255, 0, 255),   // Bright magenta
                    Color::Rgb(0, 255, 255),   // Bright cyan
                    Color::Rgb(255, 255, 0),   // Bright yellow
                    Color::Rgb(0, 255, 0),     // Bright green
                    Color::Rgb(255, 100, 0),   // Orange
                    Color::Rgb(150, 0, 255),   // Purple
                ];
                cell.style = Style::default()
                    .fg(corruption_colors.choose(&mut rng).unwrap_or(&Color::Magenta).clone())
                    .bg(Color::Black)
                    .add_modifier(if rng.gen_bool(0.3) { Modifier::BOLD } else { Modifier::DIM });
            }
            
            // Flickering effect
            else if rng.gen_bool(base_glitch_chance * 0.8) && cell.char != ' ' {
                cell.glitch_type = GlitchType::Flicker;
                if rng.gen_bool(0.5) {
                    cell.style = cell.style.add_modifier(Modifier::RAPID_BLINK);
                } else {
                    cell.style = cell.style
                        .add_modifier(Modifier::DIM)
                        .fg(Color::Rgb(80, 80, 80));
                }
            }
            
            // Static noise effect in empty areas
            else if cell.char == ' ' && rng.gen_bool(base_glitch_chance * 0.3) {
                cell.glitch_type = GlitchType::Static;
                let static_chars = ['·', '‧', '•', '∘', '°', '˚'];
                cell.char = static_chars.choose(&mut rng).unwrap_or(&'·').clone();
                cell.style = Style::default()
                    .fg(Color::Rgb(60, 60, 60))
                    .add_modifier(Modifier::DIM);
            }
            
            // Random intensity variations for the base text
            if cell.char != ' ' && cell.glitch_type == GlitchType::None && rng.gen_bool(0.05) {
                if rng.gen_bool(0.5) {
                    cell.style = cell.style.add_modifier(Modifier::BOLD);
                } else {
                    cell.style = cell.style.add_modifier(Modifier::DIM);
                }
            }
        }
        
        // Horizontal line shifts (cyberpunk data stream effect)
        if rng.gen_bool(0.02) {
            let shift_amount = rng.gen_range(1..4);
            let shift_right = rng.gen_bool(0.5);
            
            if shift_right {
                // Shift right
                for i in (shift_amount..row.len()).rev() {
                    row[i] = row[i - shift_amount].clone();
                }
                for i in 0..shift_amount.min(row.len()) {
                    row[i] = BufferChar {
                        char: ['>', '»', '→', '▶'].choose(&mut rng).unwrap_or(&'>').clone(),
                        style: Style::default().fg(Color::Rgb(255, 50, 50)),
                        glitch_type: GlitchType::DataCorruption,
                    };
                }
            } else {
                // Shift left  
                for i in 0..(row.len() - shift_amount) {
                    row[i] = row[i + shift_amount].clone();
                }
                for i in (row.len() - shift_amount)..row.len() {
                    row[i] = BufferChar {
                        char: ['<', '«', '←', '◀'].choose(&mut rng).unwrap_or(&'<').clone(),
                        style: Style::default().fg(Color::Rgb(50, 255, 50)),
                        glitch_type: GlitchType::DataCorruption,
                    };
                }
            }
        }
    }

    buffer
        .into_iter()
        .map(|row| {
            let mut spans = Vec::new();
            let mut current_style = Style::default();
            let mut current_text = String::new();

            for cell in row {
                if cell.style == current_style {
                    current_text.push(cell.char);
                } else {
                    if !current_text.is_empty() {
                        spans.push(Span::styled(current_text, current_style));
                    }
                    current_style = cell.style;
                    current_text = String::from(cell.char);
                }
            }
            if !current_text.is_empty() {
                spans.push(Span::styled(current_text, current_style));
            }
            Line::from(spans)
        })
        .collect()
}

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