use crate::state::AppError;
use std::io::Cursor;
use base64::Engine;

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
    ) -> Result<Vec<u8>, AppError> {
        // Load images
        let banner_img = image::load_from_memory(banner_bytes)
            .map_err(|e| AppError::Image(format!("Failed to load banner: {}", e)))?;
        let pfp_img = image::load_from_memory(pfp_bytes)
            .map_err(|e| AppError::Image(format!("Failed to load profile picture: {}", e)))?;
        
        // Crop banner to fill the target size (instead of fitting)
        let banner_aspect = banner_img.width() as f32 / banner_img.height() as f32;
        let target_aspect = banner_size.0 as f32 / banner_size.1 as f32;
        
        let banner_resized = if banner_aspect > target_aspect {
            // Banner is wider - crop width
            let new_height = banner_img.height();
            let new_width = (new_height as f32 * target_aspect) as u32;
            let x_offset = (banner_img.width() - new_width) / 2;
            let cropped = banner_img.crop_imm(x_offset, 0, new_width, new_height);
            cropped.resize_exact(banner_size.0, banner_size.1, image::imageops::FilterType::Lanczos3)
        } else {
            // Banner is taller - crop height
            let new_width = banner_img.width();
            let new_height = (new_width as f32 / target_aspect) as u32;
            let y_offset = (banner_img.height() - new_height) / 2;
            let cropped = banner_img.crop_imm(0, y_offset, new_width, new_height);
            cropped.resize_exact(banner_size.0, banner_size.1, image::imageops::FilterType::Lanczos3)
        };
        
        // Create composite image
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
        
        // Create circular masked profile picture
        let pfp_resized = pfp_img.resize_exact(
            pfp_size.0, 
            pfp_size.1, 
            image::imageops::FilterType::Lanczos3
        );
        let mut pfp_rgba = pfp_resized.to_rgba8();
        
        // Apply circular mask to profile picture
        let center_x = pfp_size.0 as f32 / 2.0;
        let center_y = pfp_size.1 as f32 / 2.0;
        let radius = (pfp_size.0.min(pfp_size.1) as f32 / 2.0) - 1.0; // Slightly smaller for clean edges
        
        for y in 0..pfp_size.1 {
            for x in 0..pfp_size.0 {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let distance = (dx * dx + dy * dy).sqrt();
                
                if distance > radius {
                    // Outside circle - make transparent
                    let pixel = pfp_rgba.get_pixel_mut(x, y);
                    pixel[3] = 0; // Set alpha to 0
                } else if distance > radius - 2.0 {
                    // Anti-aliasing edge
                    let pixel = pfp_rgba.get_pixel_mut(x, y);
                    let fade = (radius - distance) / 2.0;
                    pixel[3] = (pixel[3] as f32 * fade) as u8;
                }
            }
        }
        
        // Add subtle border around circular profile picture
        for y in 0..pfp_size.1 {
            for x in 0..pfp_size.0 {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let distance = (dx * dx + dy * dy).sqrt();
                
                if distance >= radius - 2.0 && distance <= radius {
                    let pixel = pfp_rgba.get_pixel_mut(x, y);
                    // Add white border
                    pixel[0] = 255;
                    pixel[1] = 255;
                    pixel[2] = 255;
                    pixel[3] = 200; // Semi-transparent white border
                }
            }
        }
        
        // Overlay circular PFP on banner
        let pfp_y = (banner_size.1 - pfp_size.1) / 2; // Center vertically
        
        // Blend the circular profile picture with proper alpha compositing
        for y in 0..pfp_size.1 {
            for x in 0..pfp_size.0 {
                let banner_x = pfp_padding_left + x;
                let banner_y = pfp_y + y;
                
                if banner_x < banner_size.0 && banner_y < banner_size.1 {
                    let pfp_pixel = pfp_rgba.get_pixel(x, y);
                    let banner_pixel = composite.get_pixel_mut(banner_x, banner_y);
                    
                    if pfp_pixel[3] > 0 {
                        // Alpha blend
                        let alpha = pfp_pixel[3] as f32 / 255.0;
                        let inv_alpha = 1.0 - alpha;
                        
                        banner_pixel[0] = (pfp_pixel[0] as f32 * alpha + banner_pixel[0] as f32 * inv_alpha) as u8;
                        banner_pixel[1] = (pfp_pixel[1] as f32 * alpha + banner_pixel[1] as f32 * inv_alpha) as u8;
                        banner_pixel[2] = (pfp_pixel[2] as f32 * alpha + banner_pixel[2] as f32 * inv_alpha) as u8;
                    }
                }
            }
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
        
        // Try to decode and validate as image
        if let Some(bytes) = Self::decode_image_bytes(&Some(data.to_string())) {
            image::load_from_memory(&bytes)
                .map_err(|e| AppError::Image(format!("Invalid image data: {}", e)))?;
        } else {
            return Err(AppError::Image("Failed to decode image data".to_string()));
        }
        
        Ok(())
    }
}