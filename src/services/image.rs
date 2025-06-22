use crate::state::AppError;
use std::io::Cursor;
use base64::Engine;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// Service for image processing and validation
pub struct ImageService;

impl ImageService {
    pub fn decode_image_bytes(val: &Option<String>) -> Option<Vec<u8>> {
        if let Some(s) = val {
            if s.starts_with("http") {
                None // Not handling URLs here
            } else {
                let b64 = if let Some(idx) = s.find(",") {
                    &s[idx+1..]
                } else {
                    s.as_str()
                };
                base64::engine::general_purpose::STANDARD.decode(b64).ok()
            }
        } else {
            None
        }
    }
    
    pub fn composite_banner_and_pfp(
        banner_bytes: &[u8],
        pfp_bytes: &[u8],
        banner_size: (u32, u32),
        pfp_size: (u32, u32),
        pfp_padding_left: u32,
        username: &str,
    ) -> Result<Vec<u8>, AppError> {
        // Load images
        let banner_img = image::load_from_memory(banner_bytes)
            .map_err(|e| AppError::Image(format!("Failed to load banner: {}", e)))?;
        let pfp_img = image::load_from_memory(pfp_bytes)
            .map_err(|e| AppError::Image(format!("Failed to load profile picture: {}", e)))?;

        // Crop banner to fill the target size (always fills, no empty space)
        let banner_resized = banner_img.resize_to_fill(
            banner_size.0,
            banner_size.1,
            image::imageops::FilterType::Lanczos3,
        );
        let mut composite = banner_resized.to_rgba8();

        // Add black gradient overlay for better text readability
        for y in 0..banner_size.1 {
            for x in 0..banner_size.0 {
                let pixel = composite.get_pixel_mut(x, y);
                // Create a subtle black gradient from top to bottom
                let gradient_factor = (y as f32 / banner_size.1 as f32) * 0.4 + 0.1; // 0.1 to 0.5 opacity
                pixel[0] = (pixel[0] as f32 * (1.0 - gradient_factor)) as u8;
                pixel[1] = (pixel[1] as f32 * (1.0 - gradient_factor)) as u8;
                pixel[2] = (pixel[2] as f32 * (1.0 - gradient_factor)) as u8;
            }
        }

        // Profile picture: center-crop and zoom to fill the circle
        let (target_w, target_h) = pfp_size;
        let pfp_cropped = pfp_img.resize_to_fill(target_w, target_h, image::imageops::FilterType::Lanczos3);
        let mut pfp_rgba = pfp_cropped.to_rgba8();
        // Apply circular mask to profile picture (centered in pfp_rgba)
        let center_x = target_w as f32 / 2.0;
        let center_y = target_h as f32 / 2.0;
        let radius = (target_w.min(target_h) as f32 / 2.0) - 1.0;
        for y in 0..target_h {
            for x in 0..target_w {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let distance = (dx * dx + dy * dy).sqrt();
                
                let pixel = pfp_rgba.get_pixel_mut(x, y);
                if distance > radius {
                    pixel[3] = 0;
                } else if distance > radius - 2.0 {
                    let fade = (radius - distance) / 2.0;
                    pixel[3] = (pixel[3] as f32 * fade) as u8;
                }
            }
        }
        // Add subtle border around circular profile picture
        for y in 0..target_h {
            for x in 0..target_w {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let distance = (dx * dx + dy * dy).sqrt();
                if distance >= radius - 2.0 && distance <= radius {
                    let pixel = pfp_rgba.get_pixel_mut(x, y);
                    pixel[0] = 255;
                    pixel[1] = 255;
                    pixel[2] = 255;
                    pixel[3] = 200;
                }
            }
        }

        // Overlay circular PFP on banner
        let pfp_y = (banner_size.1 - target_h) / 2;
        for y in 0..target_h {
            for x in 0..target_w {
                let banner_x = pfp_padding_left + x;
                let banner_y = pfp_y + y;
                if banner_x < banner_size.0 && banner_y < banner_size.1 {
                    let pfp_pixel = pfp_rgba.get_pixel(x, y);
                    let banner_pixel = composite.get_pixel_mut(banner_x, banner_y);
                    if pfp_pixel[3] > 0 {
                        let alpha = pfp_pixel[3] as f32 / 255.0;
                        let inv_alpha = 1.0 - alpha;
                        banner_pixel[0] = (pfp_pixel[0] as f32 * alpha + banner_pixel[0] as f32 * inv_alpha) as u8;
                        banner_pixel[1] = (pfp_pixel[1] as f32 * alpha + banner_pixel[1] as f32 * inv_alpha) as u8;
                        banner_pixel[2] = (pfp_pixel[2] as f32 * alpha + banner_pixel[2] as f32 * inv_alpha) as u8;
                    }
                }
            }
        }

        // Render username text directly onto the composite image using simple bitmap approach
        if !username.is_empty() {
            Self::draw_simple_text(&mut composite, username, banner_size);
        }

        // Convert to bytes
        let mut buffer = Vec::new();
        composite.write_to(&mut Cursor::new(&mut buffer), image::ImageFormat::Png)
            .map_err(|e| AppError::Image(format!("Failed to encode composite image: {}", e)))?;
        Ok(buffer)
    }
    
    pub fn validate_image_data(data: &str) -> Result<(), AppError> {
        if data.trim().is_empty() {
            return Ok(());
        }
        
        let trimmed = data.trim();
        
        // If it's a file path, check if the file exists and is readable
        if !trimmed.starts_with("data:") && !trimmed.starts_with("http") {
            // It's likely a file path
            if std::path::Path::new(trimmed).exists() {
                // Try to read and validate the file
                match std::fs::read(trimmed) {
                    Ok(bytes) => {
                        if bytes.len() > 1024 * 1024 {
                            return Err(AppError::Image(format!("File '{}' is too large (>1MB)", trimmed)));
                        }
                        // Try to load as image to validate format
                        image::load_from_memory(&bytes)
                            .map_err(|e| AppError::Image(format!("Invalid image file '{}': {}", trimmed, e)))?;
                    }
                    Err(e) => {
                        return Err(AppError::Image(format!("Cannot read file '{}': {}", trimmed, e)));
                    }
                }
            } else {
                // Check if it might be raw base64 (without data URL prefix)
                if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(trimmed) {
                    image::load_from_memory(&bytes)
                        .map_err(|e| AppError::Image(format!("Invalid base64 image data: {}", e)))?;
                } else {
                    return Err(AppError::Image(format!("'{}' is not a valid file path, URL, or base64 data", trimmed)));
                }
            }
        } else {
            // It's a data URL or HTTP URL - try to decode and validate
            if let Some(bytes) = Self::decode_image_bytes(&Some(trimmed.to_string())) {
                image::load_from_memory(&bytes)
                    .map_err(|e| AppError::Image(format!("Invalid image data: {}", e)))?;
            } else {
                return Err(AppError::Image("Failed to decode image data".to_string()));
            }
        }
        
        Ok(())
    }

    /// Draw simple text on the image using a basic bitmap approach
    fn draw_simple_text(image: &mut image::RgbaImage, text: &str, banner_size: (u32, u32)) {
        if text.is_empty() {
            return;
        }

        let char_width = 8;  // Keep at 8 for good visibility
        let char_height = 8; // Reduce back to 8 to fix vertical stretching
        let text_padding = 20; // Move text higher up
        
        // Calculate text dimensions
        let text_width = text.len() as u32 * char_width;
        let text_height = char_height;
        
        // Position text higher up (closer to middle rather than bottom)
        let pfp_width: u32 = 64; // Same as pfp_size in composite function
        let pfp_padding_left: u32 = 30;
        let text_x: u32 = pfp_padding_left + pfp_width + 20; // 20 pixels after profile pic
        let text_y: u32 = (banner_size.1 / 2).saturating_sub(text_height / 2); // Center vertically
        
        // Draw more opaque black background for better contrast
        let bg_padding: u32 = 6; // Increased padding
        let bg_x: u32 = text_x.saturating_sub(bg_padding);
        let bg_y: u32 = text_y.saturating_sub(bg_padding);
        let bg_width: u32 = text_width + (bg_padding * 2);
        let bg_height: u32 = text_height + (bg_padding * 2);
        
        // Fill background with more opaque black (85% opacity)
        for y in bg_y..bg_y + bg_height {
            for x in bg_x..bg_x + bg_width {
                if x < banner_size.0 && y < banner_size.1 {
                    let pixel = image.get_pixel_mut(x, y);
                    // Blend with more opaque black (85% opacity)
                    let alpha = 0.85;
                    let inv_alpha = 1.0 - alpha;
                    pixel[0] = (0.0 * alpha + pixel[0] as f32 * inv_alpha) as u8;
                    pixel[1] = (0.0 * alpha + pixel[1] as f32 * inv_alpha) as u8;
                    pixel[2] = (0.0 * alpha + pixel[2] as f32 * inv_alpha) as u8;
                }
            }
        }
        
        // Draw each character using a simple bitmap approach
        for (i, ch) in text.chars().enumerate() {
            let char_x = text_x + (i as u32 * char_width);
            
            // Get the bitmap pattern for this character
            let bitmap = Self::get_char_bitmap(ch);
            
            // Draw the character bitmap (no scaling to fix stretching)
            for (row, &pattern) in bitmap.iter().enumerate() {
                for col in 0..8 { // Use full 8 bits for wider characters
                    if pattern & (1 << (7 - col)) != 0 { // Adjust bit order for 8-bit width
                        let px = char_x + col;
                        let py = text_y + row as u32; // No vertical scaling
                        
                        if px < banner_size.0 && py < banner_size.1 {
                            let pixel = image.get_pixel_mut(px, py);
                            pixel[0] = 255; // White
                            pixel[1] = 255;
                            pixel[2] = 255;
                            pixel[3] = 255;
                        }
                    }
                }
            }
        }
    }
    
    /// Get a simple bitmap pattern for a character (6x8 pixels)
    fn get_char_bitmap(ch: char) -> [u8; 8] {
        match ch {
            'A' | 'a' => [0b011100, 0b100010, 0b100010, 0b111110, 0b100010, 0b100010, 0b100010, 0b000000],
            'B' | 'b' => [0b111100, 0b100010, 0b100010, 0b111100, 0b100010, 0b100010, 0b111100, 0b000000],
            'C' | 'c' => [0b011100, 0b100010, 0b100000, 0b100000, 0b100000, 0b100010, 0b011100, 0b000000],
            'D' | 'd' => [0b111100, 0b100010, 0b100010, 0b100010, 0b100010, 0b100010, 0b111100, 0b000000],
            'E' | 'e' => [0b111110, 0b100000, 0b100000, 0b111100, 0b100000, 0b100000, 0b111110, 0b000000],
            'F' | 'f' => [0b111110, 0b100000, 0b100000, 0b111100, 0b100000, 0b100000, 0b100000, 0b000000],
            'G' | 'g' => [0b011100, 0b100010, 0b100000, 0b101110, 0b100010, 0b100010, 0b011100, 0b000000],
            'H' | 'h' => [0b100010, 0b100010, 0b100010, 0b111110, 0b100010, 0b100010, 0b100010, 0b000000],
            'I' | 'i' => [0b111110, 0b001000, 0b001000, 0b001000, 0b001000, 0b001000, 0b111110, 0b000000],
            'J' | 'j' => [0b111110, 0b000010, 0b000010, 0b000010, 0b000010, 0b100010, 0b011100, 0b000000],
            'K' | 'k' => [0b100010, 0b100100, 0b101000, 0b110000, 0b101000, 0b100100, 0b100010, 0b000000],
            'L' | 'l' => [0b100000, 0b100000, 0b100000, 0b100000, 0b100000, 0b100000, 0b111110, 0b000000],
            'M' | 'm' => [0b100010, 0b110110, 0b101010, 0b101010, 0b100010, 0b100010, 0b100010, 0b000000],
            'N' | 'n' => [0b100010, 0b110010, 0b101010, 0b101010, 0b100110, 0b100010, 0b100010, 0b000000],
            'O' | 'o' => [0b011100, 0b100010, 0b100010, 0b100010, 0b100010, 0b100010, 0b011100, 0b000000],
            'P' | 'p' => [0b111100, 0b100010, 0b100010, 0b111100, 0b100000, 0b100000, 0b100000, 0b000000],
            'Q' | 'q' => [0b011100, 0b100010, 0b100010, 0b100010, 0b101010, 0b100100, 0b011010, 0b000000],
            'R' | 'r' => [0b111100, 0b100010, 0b100010, 0b111100, 0b101000, 0b100100, 0b100010, 0b000000],
            'S' | 's' => [0b011100, 0b100010, 0b100000, 0b011100, 0b000010, 0b100010, 0b011100, 0b000000],
            'T' | 't' => [0b111110, 0b001000, 0b001000, 0b001000, 0b001000, 0b001000, 0b001000, 0b000000],
            'U' | 'u' => [0b100010, 0b100010, 0b100010, 0b100010, 0b100010, 0b100010, 0b011100, 0b000000],
            'V' | 'v' => [0b100010, 0b100010, 0b100010, 0b100010, 0b100010, 0b010100, 0b001000, 0b000000],
            'W' | 'w' => [0b100010, 0b100010, 0b100010, 0b101010, 0b101010, 0b110110, 0b100010, 0b000000],
            'X' | 'x' => [0b100010, 0b100010, 0b010100, 0b001000, 0b010100, 0b100010, 0b100010, 0b000000],
            'Y' | 'y' => [0b100010, 0b100010, 0b100010, 0b010100, 0b001000, 0b001000, 0b001000, 0b000000],
            'Z' | 'z' => [0b111110, 0b000010, 0b000100, 0b001000, 0b010000, 0b100000, 0b111110, 0b000000],
            '0' => [0b011100, 0b100010, 0b100110, 0b101010, 0b110010, 0b100010, 0b011100, 0b000000],
            '1' => [0b001000, 0b011000, 0b001000, 0b001000, 0b001000, 0b001000, 0b111110, 0b000000],
            '2' => [0b011100, 0b100010, 0b000010, 0b000100, 0b001000, 0b010000, 0b111110, 0b000000],
            '3' => [0b011100, 0b100010, 0b000010, 0b001100, 0b000010, 0b100010, 0b011100, 0b000000],
            '4' => [0b000100, 0b001100, 0b010100, 0b100100, 0b111110, 0b000100, 0b000100, 0b000000],
            '5' => [0b111110, 0b100000, 0b111100, 0b000010, 0b000010, 0b100010, 0b011100, 0b000000],
            '6' => [0b011100, 0b100010, 0b100000, 0b111100, 0b100010, 0b100010, 0b011100, 0b000000],
            '7' => [0b111110, 0b000010, 0b000100, 0b001000, 0b010000, 0b100000, 0b100000, 0b000000],
            '8' => [0b011100, 0b100010, 0b100010, 0b011100, 0b100010, 0b100010, 0b011100, 0b000000],
            '9' => [0b011100, 0b100010, 0b100010, 0b011110, 0b000010, 0b100010, 0b011100, 0b000000],
            '_' => [0b000000, 0b000000, 0b000000, 0b000000, 0b000000, 0b000000, 0b111110, 0b000000],
            '-' => [0b000000, 0b000000, 0b000000, 0b111110, 0b000000, 0b000000, 0b000000, 0b000000],
            ' ' => [0b000000, 0b000000, 0b000000, 0b000000, 0b000000, 0b000000, 0b000000, 0b000000],
            _ => [0b111110, 0b100010, 0b100010, 0b100010, 0b100010, 0b100010, 0b111110, 0b000000], // Default box for unknown chars
        }
    }

    /// Create a profile pic with username overlay for fallback cases
    pub fn create_pfp_with_username(
        pfp_bytes: &[u8],
        username: &str,
        target_size: (u32, u32),
    ) -> Result<Vec<u8>, AppError> {
        // Load profile pic
        let pfp_img = image::load_from_memory(pfp_bytes)
            .map_err(|e| AppError::Image(format!("Failed to load profile picture: {}", e)))?;

        // Create a dark background similar to the banner case
        let mut background = image::RgbaImage::new(target_size.0, target_size.1);
        for y in 0..target_size.1 {
            for x in 0..target_size.0 {
                // Create a subtle dark gradient from dark gray to black
                let gradient_factor = y as f32 / target_size.1 as f32;
                let gray_value = (64.0 * (1.0 - gradient_factor * 0.5)) as u8; // 64 to 32
                background.put_pixel(x, y, image::Rgba([gray_value, gray_value, gray_value, 255]));
            }
        }

        // Resize and position profile pic (centered)
        let pfp_size = 64u32;
        let pfp_cropped = pfp_img.resize_to_fill(pfp_size, pfp_size, image::imageops::FilterType::Lanczos3);
        let mut pfp_rgba = pfp_cropped.to_rgba8();
        
        // Apply circular mask to profile picture
        let center_x = pfp_size as f32 / 2.0;
        let center_y = pfp_size as f32 / 2.0;
        let radius = (pfp_size as f32 / 2.0) - 1.0;
        for y in 0..pfp_size {
            for x in 0..pfp_size {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let distance = (dx * dx + dy * dy).sqrt();
                
                let pixel = pfp_rgba.get_pixel_mut(x, y);
                if distance > radius {
                    pixel[3] = 0;
                } else if distance > radius - 2.0 {
                    let fade = (radius - distance) / 2.0;
                    pixel[3] = (pixel[3] as f32 * fade) as u8;
                }
            }
        }

        // Add white border around circular profile picture
        for y in 0..pfp_size {
            for x in 0..pfp_size {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let distance = (dx * dx + dy * dy).sqrt();
                if distance >= radius - 2.0 && distance <= radius {
                    let pixel = pfp_rgba.get_pixel_mut(x, y);
                    pixel[0] = 255;
                    pixel[1] = 255;
                    pixel[2] = 255;
                    pixel[3] = 200;
                }
            }
        }

        // Overlay circular PFP on background (positioned on left side like in main composite)
        let pfp_padding_left = 30u32; // Same as main composite function
        let pfp_x = pfp_padding_left;
        let pfp_y = (target_size.1 - pfp_size) / 2; // Still center vertically
        for y in 0..pfp_size {
            for x in 0..pfp_size {
                let bg_x = pfp_x + x;
                let bg_y = pfp_y + y;
                if bg_x < target_size.0 && bg_y < target_size.1 {
                    let pfp_pixel = pfp_rgba.get_pixel(x, y);
                    let bg_pixel = background.get_pixel_mut(bg_x, bg_y);
                    if pfp_pixel[3] > 0 {
                        let alpha = pfp_pixel[3] as f32 / 255.0;
                        let inv_alpha = 1.0 - alpha;
                        bg_pixel[0] = (pfp_pixel[0] as f32 * alpha + bg_pixel[0] as f32 * inv_alpha) as u8;
                        bg_pixel[1] = (pfp_pixel[1] as f32 * alpha + bg_pixel[1] as f32 * inv_alpha) as u8;
                        bg_pixel[2] = (pfp_pixel[2] as f32 * alpha + bg_pixel[2] as f32 * inv_alpha) as u8;
                    }
                }
            }
        }

        // Render username text
        if !username.is_empty() {
            Self::draw_simple_text(&mut background, username, target_size);
        }

        // Convert to bytes
        let mut buffer = Vec::new();
        background.write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png)
            .map_err(|e| AppError::Image(format!("Failed to encode profile image: {}", e)))?;
        Ok(buffer)
    }
}

/// Configuration for the image cache
#[derive(Debug, Clone)]
pub struct ImageCacheConfig {
    pub max_cache_size_mb: usize,
    pub max_entries: usize,
    pub default_ttl_seconds: u64,
    pub cleanup_interval_seconds: u64,
}

impl Default for ImageCacheConfig {
    fn default() -> Self {
        Self {
            max_cache_size_mb: 100, // 100MB cache
            max_entries: 1000,
            default_ttl_seconds: 3600, // 1 hour
            cleanup_interval_seconds: 300, // 5 minutes
        }
    }
}

/// Cached image with metadata
#[derive(Debug, Clone)]
pub struct CachedImage {
    pub data: Vec<u8>,
    pub format: ImageFormat,
    pub size_bytes: usize,
    pub timestamp_cached: u64,
    pub ttl_seconds: u64,
    pub access_count: u64,
    pub last_accessed: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    WebP,
    Base64(String), // For base64 encoded images with mime type
}

impl ImageFormat {
    /// Detect format from data or mime type
    pub fn detect_from_data(data: &[u8]) -> Self {
        if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            ImageFormat::Png
        } else if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
            ImageFormat::Jpeg
        } else if data.starts_with(&[0x47, 0x49, 0x46]) {
            ImageFormat::Gif
        } else if data.len() >= 12 && &data[8..12] == b"WEBP" {
            ImageFormat::WebP
        } else {
            // Fallback to PNG
            ImageFormat::Png
        }
    }

    pub fn from_base64_data_url(data_url: &str) -> Option<(Self, Vec<u8>)> {
        if let Some(comma_pos) = data_url.find(',') {
            let (header, data) = data_url.split_at(comma_pos);
            let data = &data[1..]; // Skip the comma
            
            if let Ok(decoded) = BASE64.decode(data) {
                let format = if header.contains("image/png") {
                    ImageFormat::Png
                } else if header.contains("image/jpeg") || header.contains("image/jpg") {
                    ImageFormat::Jpeg
                } else if header.contains("image/gif") {
                    ImageFormat::Gif
                } else if header.contains("image/webp") {
                    ImageFormat::WebP
                } else {
                    ImageFormat::Base64(header.to_string())
                };
                
                return Some((format, decoded));
            }
        }
        None
    }
}

impl CachedImage {
    pub fn new(data: Vec<u8>, format: ImageFormat, ttl_seconds: Option<u64>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            size_bytes: data.len(),
            data,
            format,
            timestamp_cached: now,
            ttl_seconds: ttl_seconds.unwrap_or(3600),
            access_count: 0,
            last_accessed: now,
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        now > self.timestamp_cached + self.ttl_seconds
    }

    pub fn touch(&mut self) {
        self.access_count += 1;
        self.last_accessed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}

/// Cache key for images
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ImageCacheKey {
    UserAvatar(Uuid),
    UserCoverBanner(Uuid),
    ServerIcon(Uuid),
    ServerBanner(Uuid),
    Custom(String),
}

impl ImageCacheKey {
    pub fn user_avatar(user_id: Uuid) -> Self {
        Self::UserAvatar(user_id)
    }

    pub fn user_cover_banner(user_id: Uuid) -> Self {
        Self::UserCoverBanner(user_id)
    }

    pub fn server_icon(server_id: Uuid) -> Self {
        Self::ServerIcon(server_id)
    }

    pub fn server_banner(server_id: Uuid) -> Self {
        Self::ServerBanner(server_id)
    }

    pub fn custom(key: impl Into<String>) -> Self {
        Self::Custom(key.into())
    }
}

/// Thread-safe image cache with LRU eviction and TTL
pub struct ImageCache {
    cache: Arc<Mutex<HashMap<ImageCacheKey, CachedImage>>>,
    config: ImageCacheConfig,
    current_size_bytes: Arc<Mutex<usize>>,
}

impl ImageCache {
    pub fn new(config: ImageCacheConfig) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            config,
            current_size_bytes: Arc::new(Mutex::new(0)),
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(ImageCacheConfig::default())
    }

    /// Store an image in the cache
    pub fn put(&self, key: ImageCacheKey, image: CachedImage) -> Result<(), String> {
        let mut cache = self.cache.lock().map_err(|e| format!("Cache lock error: {}", e))?;
        let mut current_size = self.current_size_bytes.lock()
            .map_err(|e| format!("Size lock error: {}", e))?;

        // Check if we need to evict entries
        while cache.len() >= self.config.max_entries 
            || (*current_size + image.size_bytes) > (self.config.max_cache_size_mb * 1024 * 1024) {
            
            if let Some(evict_key) = self.find_lru_key(&cache) {
                if let Some(evicted) = cache.remove(&evict_key) {
                    *current_size = current_size.saturating_sub(evicted.size_bytes);
                }
            } else {
                break; // No more entries to evict
            }
        }

        // Add the new image
        *current_size += image.size_bytes;
        cache.insert(key, image);

        Ok(())
    }

    /// Retrieve an image from the cache
    pub fn get(&self, key: &ImageCacheKey) -> Result<Option<CachedImage>, String> {
        let mut cache = self.cache.lock().map_err(|e| format!("Cache lock error: {}", e))?;
        
        if let Some(mut image) = cache.get(key).cloned() {
            if image.is_expired() {
                // Remove expired image
                let mut current_size = self.current_size_bytes.lock()
                    .map_err(|e| format!("Size lock error: {}", e))?;
                *current_size = current_size.saturating_sub(image.size_bytes);
                cache.remove(key);
                return Ok(None);
            }

            // Update access information
            image.touch();
            cache.insert(key.clone(), image.clone());
            Ok(Some(image))
        } else {
            Ok(None)
        }
    }

    /// Check if an image exists in cache (without updating access time)
    pub fn contains_key(&self, key: &ImageCacheKey) -> bool {
        if let Ok(cache) = self.cache.lock() {
            if let Some(image) = cache.get(key) {
                !image.is_expired()
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Remove an image from the cache
    pub fn remove(&self, key: &ImageCacheKey) -> Result<Option<CachedImage>, String> {
        let mut cache = self.cache.lock().map_err(|e| format!("Cache lock error: {}", e))?;
        
        if let Some(image) = cache.remove(key) {
            let mut current_size = self.current_size_bytes.lock()
                .map_err(|e| format!("Size lock error: {}", e))?;
            *current_size = current_size.saturating_sub(image.size_bytes);
            Ok(Some(image))
        } else {
            Ok(None)
        }
    }

    /// Clear all cached images
    pub fn clear(&self) -> Result<(), String> {
        let mut cache = self.cache.lock().map_err(|e| format!("Cache lock error: {}", e))?;
        let mut current_size = self.current_size_bytes.lock()
            .map_err(|e| format!("Size lock error: {}", e))?;
        
        cache.clear();
        *current_size = 0;
        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> Result<ImageCacheStats, String> {
        let cache = self.cache.lock().map_err(|e| format!("Cache lock error: {}", e))?;
        let current_size = *self.current_size_bytes.lock()
            .map_err(|e| format!("Size lock error: {}", e))?;

        let mut expired_count = 0;
        let mut total_access_count = 0;

        for image in cache.values() {
            if image.is_expired() {
                expired_count += 1;
            }
            total_access_count += image.access_count;
        }

        Ok(ImageCacheStats {
            total_entries: cache.len(),
            total_size_bytes: current_size,
            total_size_mb: current_size as f64 / (1024.0 * 1024.0),
            expired_entries: expired_count,
            total_access_count,
            hit_ratio: 0.0, // Would need to track misses to calculate
        })
    }

    /// Cleanup expired entries
    pub fn cleanup_expired(&self) -> Result<usize, String> {
        let mut cache = self.cache.lock().map_err(|e| format!("Cache lock error: {}", e))?;
        let mut current_size = self.current_size_bytes.lock()
            .map_err(|e| format!("Size lock error: {}", e))?;

        let mut to_remove = Vec::new();
        
        for (key, image) in cache.iter() {
            if image.is_expired() {
                to_remove.push(key.clone());
            }
        }

        let removed_count = to_remove.len();
        for key in to_remove {
            if let Some(image) = cache.remove(&key) {
                *current_size = current_size.saturating_sub(image.size_bytes);
            }
        }

        Ok(removed_count)
    }

    /// Find the LRU key for eviction
    fn find_lru_key(&self, cache: &HashMap<ImageCacheKey, CachedImage>) -> Option<ImageCacheKey> {
        cache.iter()
            .min_by_key(|(_, image)| image.last_accessed)
            .map(|(key, _)| key.clone())
    }

    /// Process a base64 image string and cache it
    pub fn process_and_cache_base64(
        &self, 
        key: ImageCacheKey, 
        base64_data: &str,
        ttl_seconds: Option<u64>
    ) -> Result<CachedImage, String> {
        if let Some((format, data)) = ImageFormat::from_base64_data_url(base64_data) {
            let image = CachedImage::new(data, format, ttl_seconds);
            self.put(key, image.clone())?;
            Ok(image)
        } else {
            Err("Invalid base64 image data".to_string())
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct ImageCacheStats {
    pub total_entries: usize,
    pub total_size_bytes: usize,
    pub total_size_mb: f64,
    pub expired_entries: usize,
    pub total_access_count: u64,
    pub hit_ratio: f64,
}