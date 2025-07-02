use crate::state::{ChatState, ChatTarget};
use crate::model::ChatMessageWithMeta;
use crate::services::image::{ImageCache, ImageCacheKey, CachedImage, ImageCacheStats};
use nexus_tui_common::{User, ClientMessage};
use std::sync::Arc;
use tokio::sync::mpsc;

/// Enhanced chat service with pagination and caching capabilities
pub struct ChatService {
    image_cache: Option<Arc<ImageCache>>,
}

impl Default for ChatService {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatService {
    pub fn new() -> Self {
        Self {
            image_cache: None,
        }
    }

    pub fn with_image_cache(image_cache: Arc<ImageCache>) -> Self {
        Self {
            image_cache: Some(image_cache),
        }
    }

    /// Enhanced message fetching logic with intelligent prefetching
    pub fn should_fetch_more_messages_enhanced(
        chat_state: &ChatState,
        max_rows: usize,
        prefetch_threshold: usize,
    ) -> bool {
        match &chat_state.current_chat_target {
            Some(ChatTarget::Channel { channel_id, server_id: _ }) => {
                let history_complete = chat_state.channel_history_complete
                    .get(channel_id)
                    .copied()
                    .unwrap_or(false);
                let total_msgs = chat_state.chat_messages.len();
                let max_scroll_offset = total_msgs.saturating_sub(max_rows);
                
                // Enhanced logic: fetch when approaching the end OR when we have insufficient buffer
                !history_complete && (
                    chat_state.chat_scroll_offset >= max_scroll_offset.saturating_sub(prefetch_threshold) || 
                    total_msgs <= max_rows * 2
                )
            }
            Some(ChatTarget::DM { .. }) => {
                let total_msgs = chat_state.dm_messages.len();
                let max_scroll_offset = total_msgs.saturating_sub(max_rows);
                
                !chat_state.dm_history_complete && (
                    chat_state.chat_scroll_offset >= max_scroll_offset.saturating_sub(prefetch_threshold) || 
                    total_msgs <= max_rows * 2
                )
            }
            None => false,
        }
    }

    /// Calculate optimal message buffer size based on scroll behavior
    pub fn calculate_optimal_buffer_size(
        current_buffer: usize,
        scroll_velocity: f32,
        max_rows: usize,
    ) -> usize {
        let base_buffer = max_rows * 2;
        
        // Increase buffer size based on scroll velocity
        let velocity_multiplier = if scroll_velocity > 10.0 {
            3.0 // Fast scrolling - larger buffer
        } else if scroll_velocity > 5.0 {
            2.5 // Medium scrolling
        } else {
            2.0 // Slow scrolling - standard buffer
        };
        
        let optimal_size = (base_buffer as f32 * velocity_multiplier) as usize;
        optimal_size.min(1000).max(base_buffer) // Cap at 1000, minimum base_buffer
    }

    /// Process and cache user avatar with optimization
    pub fn process_user_avatar(&self, user: &User) -> Option<CachedImage> {
        if let (Some(cache), Some(avatar_data)) = (&self.image_cache, &user.profile_pic) {
            let cache_key = ImageCacheKey::user_avatar(user.id);
            
            // Check if already cached
            if let Ok(Some(cached)) = cache.get(&cache_key) {
                return Some(cached);
            }
            
            // Process and cache new avatar
            if let Ok(cached_image) = cache.process_and_cache_base64(
                cache_key, 
                avatar_data, 
                Some(7200) // 2 hour TTL for avatars
            ) {
                return Some(cached_image);
            }
        }
        None
    }

    /// Batch process multiple user avatars for efficiency
    pub fn batch_process_avatars(&self, users: &[User]) -> Vec<(uuid::Uuid, Option<CachedImage>)> {
        users.iter().map(|user| {
            (user.id, self.process_user_avatar(user))
        }).collect()
    }

    /// Get cache statistics for monitoring
    pub fn get_cache_stats(&self) -> Option<ImageCacheStats> {
        self.image_cache.as_ref()
            .and_then(|cache| cache.stats().ok())
    }

    /// Cleanup expired cache entries
    pub fn cleanup_cache(&self) -> Option<usize> {
        self.image_cache.as_ref()
            .and_then(|cache| cache.cleanup_expired().ok())
    }

    /// Preload images for current conversation participants
    pub fn preload_conversation_images(&self, chat_state: &ChatState) {
        if let Some(cache) = &self.image_cache {
            match &chat_state.current_chat_target {
                Some(ChatTarget::Channel { .. }) => {
                    // Preload channel user avatars
                    for user in &chat_state.channel_userlist {
                        if let Some(avatar_data) = &user.profile_pic {
                            let cache_key = ImageCacheKey::user_avatar(user.id);
                            if !cache.contains_key(&cache_key) {
                                let _ = cache.process_and_cache_base64(
                                    cache_key,
                                    avatar_data,
                                    Some(7200)
                                );
                            }
                        }
                    }
                }
                Some(ChatTarget::DM { .. }) => {
                    // Preload DM user avatars
                    for user in &chat_state.dm_user_list {
                        if let Some(avatar_data) = &user.profile_pic {
                            let cache_key = ImageCacheKey::user_avatar(user.id);
                            if !cache.contains_key(&cache_key) {
                                let _ = cache.process_and_cache_base64(
                                    cache_key,
                                    avatar_data,
                                    Some(7200)
                                );
                            }
                        }
                    }
                }
                None => {}
            }
        }
    }

    /// Get mention suggestions for users starting with the given input
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
    
    /// Apply the mention suggestion to the input text
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
        let re = regex::Regex::new(r":([a-zA-Z0-9_+-]+):").unwrap();
        if let Some(captures) = re.find(input) {
            let shortcode = &input[captures.start()+1..captures.end()-1];
            if let Some(emoji) = emojis::get_by_shortcode(shortcode) {
                return Some((emoji.as_str().to_string(), captures.start(), captures.end()));
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
                    // Look up user info by sent_by ID
                    let (author, color, profile_pic) = if let Some(user) = chat_state.channel_userlist.iter().find(|u| u.id == msg.sent_by) {
                        (user.username.clone(), user.color.clone().into(), user.profile_pic.clone())
                    } else {
                        // Fallback for unknown users
                        (format!("User#{}", msg.sent_by.to_string()[..8].to_uppercase()), ratatui::style::Color::Gray, None)
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
            Some(ChatTarget::DM { .. }) => {
                chat_state.dm_messages.iter().map(|msg| {
                    let (author, color, profile_pic) = if let Some(user) = current_user {
                        if msg.from != user.id {
                            // Find user in dm_user_list by from ID
                            if let Some(dm_user) = chat_state.dm_user_list.iter().find(|u| u.id == msg.from) {
                                (dm_user.username.clone(), dm_user.color.clone().into(), dm_user.profile_pic.clone())
                            } else {
                                // Fallback for unknown users
                                (format!("User#{}", msg.from.to_string()[..8].to_uppercase()), ratatui::style::Color::Gray, None)
                            }
                        } else {
                            // Current user
                            (user.username.clone(), user.color.clone().into(), user.profile_pic.clone())
                        }
                    } else {
                        // Fallback when no current user
                        (format!("User#{}", msg.from.to_string()[..8].to_uppercase()), ratatui::style::Color::Gray, None)
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
        Self::should_fetch_more_messages_enhanced(chat_state, max_rows, 10)
    }

    /// Request avatars for users that don't have profile pictures loaded
    pub fn request_missing_avatars(&self, chat_state: &ChatState, to_server: &mpsc::UnboundedSender<ClientMessage>) {
        let mut missing_user_ids = std::collections::HashSet::new();
        
        // Check channel users for missing avatars
        for user in &chat_state.channel_userlist {
            if user.profile_pic.is_none() {
                missing_user_ids.insert(user.id);
            }
        }
        
        // Check DM users for missing avatars
        for user in &chat_state.dm_user_list {
            if user.profile_pic.is_none() {
                missing_user_ids.insert(user.id);
            }
        }
        
        // Convert to Vec, limit to reasonable batch size, and send
        let mut unique_user_ids: Vec<_> = missing_user_ids.into_iter().collect();
        unique_user_ids.truncate(20); // Limit to prevent server overload
        
        if !unique_user_ids.is_empty() {
            let _ = to_server.send(ClientMessage::GetUserAvatars { user_ids: unique_user_ids });
        }
    }
}