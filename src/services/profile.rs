use crate::state::AppResult;
use base64::Engine;
use common::UserProfile;
use std::fs;
use std::path::Path;

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

    pub fn file_or_url_to_base64(val: &str) -> Option<String> {
        if val.trim().is_empty() {
            return None;
        }

        // Check if it's already base64 encoded
        if base64::engine::general_purpose::STANDARD.decode(val.trim()).is_ok() {
            return Some(val.trim().to_string());
        }

        // Try to read as file path
        if let Ok(data) = fs::read(val.trim()) {
            return Some(base64::engine::general_purpose::STANDARD.encode(&data));
        }

        None
    }
}