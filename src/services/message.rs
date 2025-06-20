use crate::state::AppError;

/// Service for message validation and processing
pub struct MessageService;

impl MessageService {
    /// Validate message content
    pub fn validate_message(content: &str) -> Result<String, String> {
        let trimmed = content.trim();
        
        if trimmed.is_empty() {
            return Err("Message cannot be empty".to_string());
        }
        
        if trimmed.len() > 2000 {
            return Err("Message too long (max 2000 characters)".to_string());
        }
        
        Ok(trimmed.to_string())
    }
    
    /// Parse command from message (e.g., "/accept", "/decline")
    pub fn parse_command(content: &str) -> Option<(String, Vec<String>)> {
        let trimmed = content.trim();
        if !trimmed.starts_with('/') {
            return None;
        }
        
        let parts: Vec<&str> = trimmed[1..].split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }
        
        let command = parts[0].to_lowercase();
        let args = parts[1..].iter().map(|s| s.to_string()).collect();
        
        Some((command, args))
    }
    
    /// Extract mentioned usernames from message content
    pub fn extract_mentions(content: &str) -> Vec<String> {
        let re = regex::Regex::new(r"@([a-zA-Z0-9_]+)").unwrap();
        re.captures_iter(content)
            .map(|cap| cap.get(1).unwrap().as_str().to_string())
            .collect()
    }
}