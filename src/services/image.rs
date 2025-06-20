use crate::state::AppError;
use base64::Engine;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Cached image data with metadata
#[derive(Clone)]
pub struct CachedImage {
    pub data: Vec<u8>,
    pub content_type: String,
    pub size: usize,
    pub last_accessed: u64,
}

/// Image caching system for avatars and banners
pub struct ImageCache {
    cache: HashMap<String, CachedImage>,
    max_size_bytes: usize,
    current_size_bytes: usize,
    max_entries: usize,
}

impl ImageCache {
    pub fn new(max_size_mb: usize, max_entries: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_size_bytes: max_size_mb * 1024 * 1024,
            current_size_bytes: 0,
            max_entries,
        }
    }
    
    /// Generate cache key for user avatar
    pub fn avatar_key(user_id: Uuid, size: u32) -> String {
        format!("avatar:{}:{}", user_id, size)
    }
    
    /// Generate cache key for user banner
    pub fn banner_key(user_id: Uuid) -> String {
        format!("banner:{}", user_id)
    }
    
    /// Get cached image data
    pub fn get(&mut self, key: &str) -> Option<&CachedImage> {
        if let Some(cached) = self.cache.get_mut(key) {
            // Update last accessed time
            cached.last_accessed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            Some(cached)
        } else {
            None
        }
    }
    
    /// Store image in cache
    pub fn put(&mut self, key: String, data: Vec<u8>, content_type: String) -> Result<(), AppError> {
        let size = data.len();
        
        // Check if we need to make space
        while (self.current_size_bytes + size > self.max_size_bytes) || 
              (self.cache.len() >= self.max_entries) {
            self.evict_lru()?;
        }
        
        let cached = CachedImage {
            data,
            content_type,
            size,
            last_accessed: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        
        // Remove old entry if it exists
        if let Some(old) = self.cache.remove(&key) {
            self.current_size_bytes = self.current_size_bytes.saturating_sub(old.size);
        }
        
        self.current_size_bytes += size;
        self.cache.insert(key, cached);
        
        Ok(())
    }
    
    /// Remove least recently used item
    fn evict_lru(&mut self) -> Result<(), AppError> {
        if self.cache.is_empty() {
            return Err(AppError::Image("Cache is full but empty".to_string()));
        }
        
        // Find the LRU item
        let lru_key = self.cache
            .iter()
            .min_by_key(|(_, cached)| cached.last_accessed)
            .map(|(key, _)| key.clone())
            .ok_or_else(|| AppError::Image("Failed to find LRU item".to_string()))?;
        
        if let Some(removed) = self.cache.remove(&lru_key) {
            self.current_size_bytes = self.current_size_bytes.saturating_sub(removed.size);
        }
        
        Ok(())
    }
    
    /// Clear cache for specific user (when profile updated)
    pub fn invalidate_user(&mut self, user_id: Uuid) {
        let keys_to_remove: Vec<String> = self.cache
            .keys()
            .filter(|key| key.contains(&user_id.to_string()))
            .cloned()
            .collect();
        
        for key in keys_to_remove {
            if let Some(removed) = self.cache.remove(&key) {
                self.current_size_bytes = self.current_size_bytes.saturating_sub(removed.size);
            }
        }
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entries: self.cache.len(),
            size_bytes: self.current_size_bytes,
            size_mb: self.current_size_bytes as f32 / (1024.0 * 1024.0),
            max_entries: self.max_entries,
            max_size_mb: self.max_size_bytes as f32 / (1024.0 * 1024.0),
        }
    }
    
    /// Clear entire cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.current_size_bytes = 0;
    }
}

#[derive(Debug)]
pub struct CacheStats {
    pub entries: usize,
    pub size_bytes: usize,
    pub size_mb: f32,
    pub max_entries: usize,
    pub max_size_mb: f32,
}

/// Service for image validation and processing
pub struct ImageService;

impl ImageService {
    /// Validate image data size and format
    pub fn validate_image_data(base64_data: &str) -> Result<(), AppError> {
        if base64_data.is_empty() {
            return Ok(()); // Empty is valid (no image)
        }
        
        // Decode base64 to check size
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(base64_data.trim())
            .map_err(|e| AppError::Image(format!("Invalid base64: {}", e)))?;
        
        // Check file size (1MB limit)
        const MAX_SIZE: usize = 1024 * 1024;
        if decoded.len() > MAX_SIZE {
            return Err(AppError::Image(format!(
                "Image too large: {} bytes (max: {} bytes)",
                decoded.len(),
                MAX_SIZE
            )));
        }
        
        // Validate image format
        Self::detect_image_type(&decoded)?;
        
        Ok(())
    }
    
    /// Decode base64 image with caching
    pub fn decode_cached_image(
        base64_data: &str,
        cache: &mut ImageCache,
        cache_key: &str,
    ) -> Result<Vec<u8>, AppError> {
        // Check cache first
        if let Some(cached) = cache.get(cache_key) {
            return Ok(cached.data.clone());
        }
        
        // Decode and cache
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(base64_data.trim())
            .map_err(|e| AppError::Image(format!("Invalid base64: {}", e)))?;
        
        // Determine content type
        let content_type = Self::detect_image_type(&decoded)?;
        
        // Cache the decoded data
        cache.put(cache_key.to_string(), decoded.clone(), content_type)?;
        
        Ok(decoded)
    }
    
    /// Detect image content type from bytes
    fn detect_image_type(data: &[u8]) -> Result<String, AppError> {
        if data.len() < 4 {
            return Err(AppError::Image("Image data too short".to_string()));
        }
        
        // Check PNG signature
        if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            return Ok("image/png".to_string());
        }
        
        // Check JPEG signature
        if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return Ok("image/jpeg".to_string());
        }
        
        // Check GIF signature
        if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
            return Ok("image/gif".to_string());
        }
        
        // Check WebP signature
        if data.len() >= 12 && data[0..4] == [0x52, 0x49, 0x46, 0x46] && data[8..12] == [0x57, 0x45, 0x42, 0x50] {
            return Ok("image/webp".to_string());
        }
        
        Ok("application/octet-stream".to_string())
    }
}