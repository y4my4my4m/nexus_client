use std::fs;
use std::path::Path;
use base64::Engine;

/// Service for profile validation and processing
pub struct ProfileService;

impl ProfileService {
    pub fn validate_profile_data(
        bio: &str,
        url1: &str,
        url2: &str,
        url3: &str,
        location: &str,
    ) -> Result<(), String> {
        if bio.len() > 500 {
            return Err("Bio must be 500 characters or less".to_string());
        }
        
        for (i, url) in [url1, url2, url3].iter().enumerate() {
            if !url.is_empty() && !Self::is_valid_url(url) {
                return Err(format!("URL{} is not valid", i + 1));
            }
        }
        
        if location.len() > 100 {
            return Err("Location must be 100 characters or less".to_string());
        }
        
        Ok(())
    }
    
    pub fn is_valid_url(url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://")
    }
    
    pub fn file_or_url_to_base64(val: &str) -> Result<Option<String>, String> {
        if val.trim().is_empty() {
            return Ok(None);
        }
        
        let trimmed = val.trim();
        
        // If it's already base64 or a URL, return as-is
        if trimmed.starts_with("data:") || trimmed.starts_with("http") {
            return Ok(Some(trimmed.to_string()));
        }
        
        // Try to read as file path first
        if Path::new(trimmed).exists() {
            match fs::read(trimmed) {
                Ok(bytes) => {
                    if bytes.len() > 1024 * 1024 {
                        return Err(format!("File '{}' is too large (>1MB)", trimmed));
                    }
                    
                    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                    Ok(Some(format!("data:image/png;base64,{}", b64)))
                }
                Err(e) => Err(format!("Failed to read file '{}': {}", trimmed, e))
            }
        } else {
            // If not a file path, try to decode as raw base64 to validate
            match base64::engine::general_purpose::STANDARD.decode(trimmed) {
                Ok(_) => {
                    // If it's valid base64 but missing data URL prefix, add it
                    Ok(Some(format!("data:image/png;base64,{}", trimmed)))
                }
                Err(_) => Err(format!("'{}' is not a valid file path, URL, or base64 data", trimmed))
            }
        }
    }
}