// client/src/banner.rs

use figlet_rs::FIGfont;
use rand::prelude::*;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use std::path::PathBuf;

#[derive(Clone)]
struct BufferChar {
    char: char,
    style: Style,
}

const BASE_PATH: &str = env!("CARGO_MANIFEST_DIR");

pub fn get_styled_banner_lines(width: u16, tick_count: u64) -> Vec<Line<'static>> {
    // let standard_font = FIGfont::standard().unwrap();
    // let figlet_text = standard_font.convert("NEXUS").unwrap();

    // let custom_font = FIGfont::from_file(PathBuf::from(BASE_PATH).join("assets/fig/cosmike.flf").to_str().unwrap()).unwrap();
    // let figlet_text = custom_font.convert("NEXUS").unwrap();

    let font_data = include_str!("../assets/fig/alligator2.flf");
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
                style: Style::default(),//.bg(Color::Rgb(30, 0, 30)),
            };
            width as usize
        ];
        banner_height
    ];

    let start_y = 1; // Start drawing after the top padding
    let start_x = (width as usize - figlet_width) / 2;

    for (y, line) in figlet_lines.iter().enumerate() {
        for (x, char) in line.chars().enumerate() {
            if let Some(cell) = buffer
                .get_mut(start_y + y)
                .and_then(|row| row.get_mut(start_x + x))
            {
                if char != ' ' {
                    cell.char = char;
                    cell.style = Style::default()
                        .fg(Color::Magenta)
                        // .bg(Color::Rgb(30, 0, 30));
                }
            }
        }
    }
    
    let mut rng = thread_rng();
    for (y, row) in buffer.iter_mut().enumerate() {
        // let scanline_pos = (tick_count) % (banner_height as u64);
        // if y as u64 == scanline_pos {
        //     for cell in row.iter_mut() {
        //         cell.style = cell.style.bg(Color::Rgb(50, 0, 50));
        //     }
        // }

        for (x, cell) in row.iter_mut().enumerate() {
            let glitch_chance = 0.0005 + (tick_count as f64 * 0.01 + (y as f64 * 0.5) + (x as f64 * 0.01)).cos().powi(2) * 0.001;
            if rng.gen_bool(glitch_chance) {
                cell.style = cell.style.bg(Color::Black).fg(Color::Rgb(255, 100, 255));
                if rng.gen_bool(0.5) {
                    cell.char = ['█', '▓', '▒', '░'].choose(&mut rng).unwrap_or(&' ').clone();
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