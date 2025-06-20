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
        
        // Resize banner to target size
        let banner_resized = banner_img.resize_exact(
            banner_size.0, 
            banner_size.1, 
            image::imageops::FilterType::Lanczos3
        );
        
        // Resize PFP to target size
        let pfp_resized = pfp_img.resize_exact(
            pfp_size.0, 
            pfp_size.1, 
            image::imageops::FilterType::Lanczos3
        );
        
        // Create composite image
        let mut composite = banner_resized.to_rgba8();
        
        // Overlay PFP on banner
        let pfp_rgba = pfp_resized.to_rgba8();
        let pfp_y = (banner_size.1 - pfp_size.1) / 2; // Center vertically
        
        image::imageops::overlay(&mut composite, &pfp_rgba, pfp_padding_left as i64, pfp_y as i64);
        
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