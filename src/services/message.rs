use crate::state::AppConfig;

/// Service for message validation and processing
pub struct MessageService;

impl MessageService {
    pub fn validate_message(content: &str) -> Result<String, String> {
        let trimmed = content.trim();
        
        if trimmed.is_empty() {
            return Err("Message cannot be empty".to_string());
        }
        
        let config = AppConfig::default();
        if trimmed.chars().count() > config.max_message_length {
            let truncated: String = trimmed.chars().take(config.max_message_length).collect();
            Ok(truncated)
        } else {
            Ok(trimmed.to_string())
        }
    }
    
    pub fn is_command(content: &str) -> bool {
        content.trim().starts_with('/')
    }
    
    pub fn parse_command(content: &str) -> Option<(String, Vec<String>)> {
        let trimmed = content.trim();
        if !trimmed.starts_with('/') {
            return None;
        }
        
        let parts: Vec<&str> = trimmed[1..].split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }
        
        let command = parts[0].to_string();
        let args = parts[1..].iter().map(|s| s.to_string()).collect();
        
        Some((command, args))
    }
    
    pub fn format_mention(username: &str) -> String {
        format!("@{}", username)
    }
}