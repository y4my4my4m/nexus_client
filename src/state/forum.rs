use common::{Forum, Thread};
use uuid::Uuid;
use ratatui::widgets::ListState;

/// State management for forum functionality
pub struct ForumState {
    pub forums: Vec<Forum>,
    pub current_forum_id: Option<Uuid>,
    pub current_thread_id: Option<Uuid>,
    pub pending_new_thread_title: Option<String>,
    
    // UI state
    pub forum_list_state: ListState,
    pub thread_list_state: ListState,
    
    // Post navigation state
    pub selected_post_index: Option<usize>,
    pub selected_reply_index: Option<usize>, // For navigating replies to a post
    pub reply_to_post_id: Option<Uuid>, // Currently selected post to reply to
    pub scroll_offset: usize, // For scrolling to specific posts
}

impl Default for ForumState {
    fn default() -> Self {
        Self {
            forums: Vec::new(),
            current_forum_id: None,
            current_thread_id: None,
            pending_new_thread_title: None,
            forum_list_state: ListState::default(),
            thread_list_state: ListState::default(),
            selected_post_index: None,
            selected_reply_index: None,
            reply_to_post_id: None,
            scroll_offset: 0,
        }
    }
}

impl ForumState {
    pub fn get_current_forum(&self) -> Option<&Forum> {
        self.current_forum_id
            .and_then(|id| self.forums.iter().find(|f| f.id == id))
    }
    
    pub fn get_current_thread(&self) -> Option<&Thread> {
        self.get_current_forum()
            .and_then(|forum| self.current_thread_id
                .and_then(|id| forum.threads.iter().find(|t| t.id == id)))
    }
    
    pub fn select_forum(&mut self, forum_id: Uuid) {
        self.current_forum_id = Some(forum_id);
        self.thread_list_state.select(Some(0));
    }
    
    pub fn select_thread(&mut self, thread_id: Uuid) {
        self.current_thread_id = Some(thread_id);
        // Reset post navigation when entering a thread
        self.selected_post_index = Some(0);
        self.selected_reply_index = None;
        self.reply_to_post_id = None;
        self.scroll_offset = 0;
    }
    
    pub fn clear_pending_thread(&mut self) {
        self.pending_new_thread_title = None;
    }
    
    // Post navigation methods
    pub fn move_post_selection(&mut self, direction: i32) {
        if let Some(thread) = self.get_current_thread() {
            if thread.posts.is_empty() { return; }
            
            let current = self.selected_post_index.unwrap_or(0);
            let new_index = if direction > 0 {
                (current + 1).min(thread.posts.len() - 1)
            } else {
                current.saturating_sub(1)
            };
            self.selected_post_index = Some(new_index);
            self.selected_reply_index = None; // Reset reply selection
            
            // Auto-scroll to keep selected post visible
            self.auto_scroll_to_selected_post();
        }
    }
    
    pub fn scroll_posts(&mut self, direction: i32, amount: usize) {
        if let Some(thread) = self.get_current_thread() {
            if thread.posts.is_empty() { return; }
            
            if direction > 0 {
                // Scroll down (increase offset)
                let max_offset = thread.posts.len().saturating_sub(1);
                self.scroll_offset = (self.scroll_offset + amount).min(max_offset);
            } else {
                // Scroll up (decrease offset)
                self.scroll_offset = self.scroll_offset.saturating_sub(amount);
            }
        }
    }
    
    pub fn auto_scroll_to_selected_post(&mut self) {
        if let (Some(thread), Some(selected_idx)) = (self.get_current_thread(), self.selected_post_index) {
            let posts_len = thread.posts.len();
            if posts_len == 0 { return; }
            
            // Estimate how many posts fit on screen (rough calculation)
            let visible_posts = 5; // Approximate posts visible at once
            
            // If selected post is above visible area, scroll up
            if selected_idx < self.scroll_offset {
                self.scroll_offset = selected_idx;
            }
            // If selected post is below visible area, scroll down
            else if selected_idx >= self.scroll_offset + visible_posts {
                self.scroll_offset = selected_idx.saturating_sub(visible_posts - 1);
            }
            
            // Keep scroll offset within bounds
            let max_offset = posts_len.saturating_sub(visible_posts);
            self.scroll_offset = self.scroll_offset.min(max_offset);
        }
    }
    
    pub fn get_replies_to_post(&self, post_id: Uuid) -> Vec<(usize, &common::Post)> {
        if let Some(thread) = self.get_current_thread() {
            thread.posts.iter().enumerate()
                .filter(|(_, post)| post.reply_to == Some(post_id))
                .collect()
        } else {
            Vec::new()
        }
    }
    
    pub fn move_reply_selection(&mut self, direction: i32) {
        if let Some(post_idx) = self.selected_post_index {
            if let Some(thread) = self.get_current_thread() {
                if let Some(post) = thread.posts.get(post_idx) {
                    let replies = self.get_replies_to_post(post.id);
                    if replies.is_empty() { return; }
                    
                    let current = self.selected_reply_index.unwrap_or(0);
                    let new_index = if direction > 0 {
                        (current + 1) % replies.len()
                    } else {
                        (current + replies.len() - 1) % replies.len()
                    };
                    self.selected_reply_index = Some(new_index);
                }
            }
        }
    }
    
    pub fn set_reply_target(&mut self, post_id: Option<Uuid>) {
        self.reply_to_post_id = post_id;
    }
    
    pub fn scroll_to_post(&mut self, post_index: usize) {
        self.scroll_offset = post_index;
    }
    
    pub fn get_selected_post(&self) -> Option<&common::Post> {
        if let (Some(thread), Some(idx)) = (self.get_current_thread(), self.selected_post_index) {
            thread.posts.get(idx)
        } else {
            None
        }
    }
    
    pub fn get_selected_reply_post(&self) -> Option<&common::Post> {
        if let (Some(post_idx), Some(reply_idx)) = (self.selected_post_index, self.selected_reply_index) {
            if let Some(thread) = self.get_current_thread() {
                if let Some(post) = thread.posts.get(post_idx) {
                    let replies = self.get_replies_to_post(post.id);
                    replies.get(reply_idx).map(|(_, post)| *post)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}