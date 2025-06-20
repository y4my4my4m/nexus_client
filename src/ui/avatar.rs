//! Avatar protocol and image helpers for the UI.

use base64::Engine;
use image::{DynamicImage, RgbaImage, GenericImageView};
use crate::app::App;

// Returns a mutable reference to a cached StatefulProtocol for the user's avatar, creating it if needed.
pub fn get_avatar_protocol<'a>(app: &'a mut App, user: &common::User, size: u32) -> Option<&'a mut ratatui_image::protocol::StatefulProtocol> {
    let key = (user.id, size);
    if !app.profile.avatar_protocol_cache.contains_key(&key) {
        let pic = user.profile_pic.as_ref()?;
        let b64 = if let Some(idx) = pic.find(',') {
            if idx + 1 >= pic.len() { return None; }
            &pic[idx + 1..]
        } else { pic };
        let bytes = base64::engine::general_purpose::STANDARD.decode(b64).ok()?;
        let img = image::load_from_memory(&bytes).ok()?;
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
        apply_circular_mask(&mut square);
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
