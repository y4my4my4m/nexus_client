use crate::state::{ChatState, ChatTarget};
use crate::model::ChatMessageWithMeta;
use common::User;

/// Business logic for chat functionality
pub struct ChatService;

impl ChatService {
    pub fn get_mention_suggestions(input: &str, users: &[User]) -> Vec<String> {
        let cursor = input.len();
        let upto = &input[..cursor];
        
        if let Some(idx) = upto.rfind('@') {
            let after_at = &upto[(idx + 1)..];
            if after_at.chars().all(|ch| ch.is_alphanumeric() || ch == '_') && !after_at.is_empty() {
                let prefix = after_at.to_lowercase();
                let mut suggestions: Vec<String> = users
                    .iter()
                    .filter(|u| u.username.to_lowercase().starts_with(&prefix))
                    .map(|u| u.username.clone())
                    .collect();
                suggestions.sort();
                return suggestions;
            }
        }
        
        Vec::new()
    }
    
    pub fn apply_mention_suggestion(input: &str, suggestion: &str, prefix: &str) -> String {
        let mut result = input.to_string();
        if let Some(idx) = input.rfind(&format!("@{}", prefix)) {
            result.replace_range(idx..(idx + 1 + prefix.len()), &format!("@{} ", suggestion));
        }
        result
    }
    
    /// Get emoji suggestions based on input text
    pub fn get_emoji_suggestions(input: &str) -> Vec<String> {
        let cursor = input.len();
        let upto = &input[..cursor];
        
        if let Some(idx) = upto.rfind(':') {
            let after_colon = &upto[(idx + 1)..];
            if after_colon.chars().all(|ch| ch.is_alphabetic() || ch == '_') && !after_colon.is_empty() {
                let prefix = after_colon.to_lowercase();
                let mut suggestions: Vec<String> = emojis::iter()
                    .filter_map(|emoji| {
                        // Check if any shortcode matches the prefix
                        for shortcode in emoji.shortcodes() {
                            if shortcode.to_lowercase().starts_with(&prefix) {
                                return Some(emoji.as_str().to_string());
                            }
                        }
                        None
                    })
                    .collect();
                
                // Remove duplicates and limit to reasonable number
                suggestions.sort();
                suggestions.dedup();
                suggestions.truncate(10);
                return suggestions;
            }
        }
        
        Vec::new()
    }
    
    /// Apply emoji suggestion to input text
    pub fn apply_emoji_suggestion(input: &str, emoji: &str, prefix: &str) -> String {
        let mut result = input.to_string();
        if let Some(idx) = input.rfind(&format!(":{}", prefix)) {
            result.replace_range(idx..(idx + 1 + prefix.len()), &format!("{} ", emoji));
        }
        result
    }
    
    /// Check if input contains a complete emoji shortcode and return the emoji
    pub fn check_for_exact_emoji_match(input: &str) -> Option<(String, usize, usize)> {
        // Look for pattern :emoji_name: (complete with closing colon)
        if let Some(end_pos) = input.rfind(':') {
            if let Some(start_pos) = input[..end_pos].rfind(':') {
                let emoji_name = &input[(start_pos + 1)..end_pos];
                if !emoji_name.is_empty() && emoji_name.chars().all(|ch| ch.is_alphabetic() || ch == '_') {
                    // Check if this matches any emoji shortcode exactly
                    for emoji in emojis::iter() {
                        for shortcode in emoji.shortcodes() {
                            if shortcode.to_lowercase() == emoji_name.to_lowercase() {
                                return Some((emoji.as_str().to_string(), start_pos, end_pos + 1));
                            }
                        }
                    }
                }
            }
        }
        None
    }
    
    pub fn build_message_list(
        chat_state: &ChatState,
        current_user: Option<&User>,
    ) -> Vec<ChatMessageWithMeta> {
        match &chat_state.current_chat_target {
            Some(ChatTarget::Channel { .. }) => {
                chat_state.chat_messages.iter().map(|msg| {
                    ChatMessageWithMeta {
                        author: msg.author_username.clone(),
                        content: msg.content.clone(),
                        color: msg.author_color,
                        profile_pic: msg.author_profile_pic.clone(),
                        timestamp: Some(msg.timestamp),
                    }
                }).collect()
            }
            Some(ChatTarget::DM { .. }) => {
                chat_state.dm_messages.iter().map(|msg| {
                    let (author, color, profile_pic) = if let Some(user) = current_user {
                        if msg.from != user.id {
                            // Find user in dm_user_list
                            if let Some(dm_user) = chat_state.dm_user_list.iter().find(|u| u.id == msg.from) {
                                (dm_user.username.clone(), dm_user.color, dm_user.profile_pic.clone())
                            } else {
                                ("?".to_string(), ratatui::style::Color::Gray, None)
                            }
                        } else {
                            // Current user
                            (user.username.clone(), user.color, user.profile_pic.clone())
                        }
                    } else {
                        ("?".to_string(), ratatui::style::Color::Gray, None)
                    };
                    
                    ChatMessageWithMeta {
                        author,
                        content: msg.content.clone(),
                        color,
                        profile_pic,
                        timestamp: Some(msg.timestamp),
                    }
                }).collect()
            }
            None => Vec::new(),
        }
    }
    
    pub fn should_fetch_more_messages(
        chat_state: &ChatState,
        max_rows: usize,
    ) -> bool {
        match &chat_state.current_chat_target {
            Some(ChatTarget::Channel { channel_id, server_id: _ }) => {
                let history_complete = chat_state.channel_history_complete
                    .get(channel_id)
                    .copied()
                    .unwrap_or(false);
                let total_msgs = chat_state.chat_messages.len();
                let max_scroll_offset = total_msgs.saturating_sub(max_rows);
                
                !history_complete && 
                (chat_state.chat_scroll_offset >= max_scroll_offset.saturating_sub(max_rows / 2) || 
                 total_msgs <= max_rows * 2)
            }
            Some(ChatTarget::DM { .. }) => {
                let total_msgs = chat_state.dm_messages.len();
                let max_scroll_offset = total_msgs.saturating_sub(max_rows);
                
                !chat_state.dm_history_complete && 
                (chat_state.chat_scroll_offset >= max_scroll_offset.saturating_sub(max_rows / 2) || 
                 total_msgs <= max_rows * 2)
            }
            None => false,
        }
    }
}