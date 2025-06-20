use crate::state::{ChatState, ChatTarget};
use crate::model::ChatMessageWithMeta;
use common::{User, DirectMessage, ChannelMessage};
use uuid::Uuid;

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
    
    pub fn build_message_list(
        chat_state: &ChatState,
        current_user: Option<&User>,
    ) -> Vec<ChatMessageWithMeta> {
        match &chat_state.current_chat_target {
            Some(ChatTarget::Channel { .. }) => {
                chat_state.chat_messages.iter().map(|msg| {
                    ChatMessageWithMeta {
                        author: msg.author.clone(),
                        content: msg.content.clone(),
                        color: msg.color,
                        profile_pic: None, // TODO: Add profile pic support
                        timestamp: None,   // TODO: Add timestamp support
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
            Some(ChatTarget::Channel { channel_id, server_id }) => {
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