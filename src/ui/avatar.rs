//! Avatar protocol and image helpers for the UI.

use base64::Engine;
use image::{DynamicImage, RgbaImage, GenericImageView};
use crate::app::App;

// Returns a mutable reference to a cached StatefulProtocol for the user's avatar, creating it if needed.
pub fn get_avatar_protocol<'a>(app: &'a mut App, user: &common::User, size: u32) -> Option<&'a mut Box<dyn ratatui_image::protocol::StatefulProtocol>> {
    let key = (user.id, size);
    if !app.profile.avatar_protocol_cache.contains_key(&key) {
        let pic = user.profile_pic.as_ref()?;
        
        // Handle data URL format (data:image/jpeg;base64,...)
        let b64 = if let Some(idx) = pic.find(',') {
            if idx + 1 >= pic.len() { return None; }
            &pic[idx + 1..]
        } else { pic };
        
        // Decode base64 image data
        let bytes = base64::engine::general_purpose::STANDARD.decode(b64.trim()).ok()?;
        let img = image::load_from_memory(&bytes).ok()?;
        
        // Resize and crop to create a square avatar
        let (orig_w, orig_h) = img.dimensions();
        let scale = f32::max(size as f32 / orig_w as f32, size as f32 / orig_h as f32);
        let new_w = (orig_w as f32 * scale).ceil() as u32;
        let new_h = (orig_h as f32 * scale).ceil() as u32;
        
        // Resize the image
        let resized = img.resize_exact(new_w, new_h, image::imageops::FilterType::Lanczos3).to_rgba8();
        
        // Crop the center square
        let x_offset = ((new_w as i32 - size as i32) / 2).max(0) as u32;
        let y_offset = ((new_h as i32 - size as i32) / 2).max(0) as u32;
        let cropped = image::imageops::crop_imm(&resized, x_offset, y_offset, size, size).to_image();
        let mut square = cropped;
        
        // Apply circular mask for avatar appearance
        apply_circular_mask(&mut square);
        
        // Create protocol for ratatui_image
        let protocol = app.profile.picker.new_resize_protocol(DynamicImage::ImageRgba8(square));
        app.profile.avatar_protocol_cache.insert(key, protocol);
    }
    app.profile.avatar_protocol_cache.get_mut(&key)
}

// Helper: Apply a circular alpha mask to an RgbaImage in-place
pub fn apply_circular_mask(img: &mut RgbaImage) {
    let (w, h) = (img.width() as i32, img.height() as i32);
    let cx = w / 2;
    let cy = h / 2;
    let r = w.min(h) as f32 / 2.0;
    
    for y in 0..h {
        for x in 0..w {
            let dx = x - cx;
            let dy = y - cy;
            let dist = ((dx * dx + dy * dy) as f32).sqrt();
            if dist > r {
                let p = img.get_pixel_mut(x as u32, y as u32);
                p[3] = 0; // Set alpha to 0 (transparent)
            }
        }
    }
}

// Helper function to create a banner with profile picture composite
pub fn create_banner_composite(
    banner_data: &str,
    profile_pic_data: Option<&str>,
    banner_width: u32,
    banner_height: u32,
    pfp_size: u32,
) -> Option<DynamicImage> {
    // Decode banner image
    let banner_b64 = if let Some(idx) = banner_data.find(',') {
        if idx + 1 >= banner_data.len() { return None; }
        &banner_data[idx + 1..]
    } else { banner_data };
    
    let banner_bytes = base64::engine::general_purpose::STANDARD.decode(banner_b64.trim()).ok()?;
    let mut banner_img = image::load_from_memory(&banner_bytes).ok()?;
    
    // Resize banner to fit dimensions while maintaining aspect ratio
    banner_img = banner_img.resize_to_fill(banner_width, banner_height, image::imageops::FilterType::Lanczos3);
    let mut banner_rgba = banner_img.to_rgba8();
    
    // If we have a profile picture, overlay it
    if let Some(pfp_data) = profile_pic_data {
        if let Some(pfp_img) = decode_and_prepare_avatar(pfp_data, pfp_size) {
            // Position the profile picture in the top-left corner with some padding
            let pfp_x = 10;
            let pfp_y = 10;
            
            // Overlay the profile picture onto the banner
            image::imageops::overlay(&mut banner_rgba, &pfp_img, pfp_x as i64, pfp_y as i64);
        }
    }
    
    Some(DynamicImage::ImageRgba8(banner_rgba))
}

// Helper function to decode and prepare avatar image
fn decode_and_prepare_avatar(pic_data: &str, size: u32) -> Option<RgbaImage> {
    let b64 = if let Some(idx) = pic_data.find(',') {
        if idx + 1 >= pic_data.len() { return None; }
        &pic_data[idx + 1..]
    } else { pic_data };
    
    let bytes = base64::engine::general_purpose::STANDARD.decode(b64.trim()).ok()?;
    let img = image::load_from_memory(&bytes).ok()?;
    
    // Resize and crop to create a square avatar
    let (orig_w, orig_h) = img.dimensions();
    let scale = f32::max(size as f32 / orig_w as f32, size as f32 / orig_h as f32);
    let new_w = (orig_w as f32 * scale).ceil() as u32;
    let new_h = (orig_h as f32 * scale).ceil() as u32;
    
    let resized = img.resize_exact(new_w, new_h, image::imageops::FilterType::Lanczos3).to_rgba8();
    
    // Crop the center square
    let x_offset = ((new_w as i32 - size as i32) / 2).max(0) as u32;
    let y_offset = ((new_h as i32 - size as i32) / 2).max(0) as u32;
    let cropped = image::imageops::crop_imm(&resized, x_offset, y_offset, size, size).to_image();
    let mut square = cropped;
    
    // Apply circular mask
    apply_circular_mask(&mut square);
    
    Some(square)
}
