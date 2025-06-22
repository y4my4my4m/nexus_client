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
    }
    
    pub fn clear_pending_thread(&mut self) {
        self.pending_new_thread_title = None;
    }
}