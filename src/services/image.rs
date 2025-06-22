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