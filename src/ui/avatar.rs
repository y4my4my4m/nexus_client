//! Avatar protocol and image helpers for the UI.

use base64::Engine;
use image::{DynamicImage, RgbaImage};
use crate::app::App;

// Returns a mutable reference to a cached StatefulProtocol for the user's avatar, creating it if needed.
pub fn get_avatar_protocol<'a>(app: &'a mut App, user: &common::User, size: u32) -> Option<&'a mut ratatui_image::protocol::StatefulProtocol> {
    let key = (user.id, size);
    if !app.avatar_protocol_cache.contains_key(&key) {
        let pic = user.profile_pic.as_ref()?;
        let b64 = if let Some(idx) = pic.find(',') {
            if idx + 1 >= pic.len() { return None; }
            &pic[idx + 1..]
        } else { pic };
        let bytes = base64::engine::general_purpose::STANDARD.decode(b64).ok()?;
        let img = image::load_from_memory(&bytes).ok()?;
        let mut resized = img.resize_exact(size, size, image::imageops::FilterType::Lanczos3).to_rgba8();
        apply_circular_mask(&mut resized);
        let protocol = app.picker.new_resize_protocol(DynamicImage::ImageRgba8(resized));
        app.avatar_protocol_cache.insert(key, protocol);
    }
    app.avatar_protocol_cache.get_mut(&key)
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
