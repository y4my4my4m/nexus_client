use common::{User, DirectMessage, Server, Channel, ChannelMessage};
use uuid::Uuid;
use std::collections::{HashMap, HashSet};
use ratatui::widgets::ListState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatFocus {
    Messages,
    Users,
    DMInput,
    Sidebar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarTab {
    Servers,
    DMs,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChatTarget {
    Channel { server_id: Uuid, channel_id: Uuid },
    DM { user_id: Uuid },
}

/// State management for chat functionality
pub struct ChatState {
    // Server and channel data
    pub servers: Vec<Server>,
    pub selected_server: Option<usize>,
    pub selected_channel: Option<usize>,
    
    // Chat messages and scrolling
    pub chat_messages: Vec<ChannelMessage>,
    pub chat_scroll_offset: usize,
    pub last_chat_rows: Option<usize>,
    
    // Channel management
    pub channel_userlist: Vec<User>,
    pub channel_history_complete: HashMap<Uuid, bool>,
    pub unread_channels: HashSet<Uuid>,
    
    // Direct messages
    pub dm_user_list: Vec<User>,
    pub selected_dm_user: Option<usize>,
    pub dm_messages: Vec<DirectMessage>,
    pub dm_history_complete: bool,
    pub unread_dm_conversations: HashSet<Uuid>,
    pub dm_input: String,
    pub dm_target: Option<Uuid>,
    
    // UI state
    pub chat_focus: ChatFocus,
    pub sidebar_tab: SidebarTab,
    pub show_user_list: bool,
    pub user_list_state: ListState,
    
    // Input drafts per chat target
    pub chat_input_drafts: HashMap<ChatTarget, String>,
    pub current_chat_target: Option<ChatTarget>,
    
    // Mention system
    pub mention_suggestions: Vec<usize>,
    pub mention_selected: usize,
    pub mention_prefix: Option<String>,
    
    // Emoji system
    pub emoji_suggestions: Vec<String>,
    pub emoji_selected: usize,
    pub emoji_prefix: Option<String>,
}

impl Default for ChatState {
    fn default() -> Self {
        Self {
            servers: Vec::new(),
            selected_server: None,
            selected_channel: None,
            chat_messages: Vec::new(),
            chat_scroll_offset: 0,
            last_chat_rows: None,
            channel_userlist: Vec::new(),
            channel_history_complete: HashMap::new(),
            unread_channels: HashSet::new(),
            dm_user_list: Vec::new(),
            selected_dm_user: None,
            dm_messages: Vec::new(),
            dm_history_complete: false,
            unread_dm_conversations: HashSet::new(),
            dm_input: String::new(),
            dm_target: None,
            chat_focus: ChatFocus::Messages,
            sidebar_tab: SidebarTab::Servers,
            show_user_list: true,
            user_list_state: ListState::default(),
            chat_input_drafts: HashMap::new(),
            current_chat_target: None,
            mention_suggestions: Vec::new(),
            mention_selected: 0,
            mention_prefix: None,
            emoji_suggestions: Vec::new(),
            emoji_selected: 0,
            emoji_prefix: None,
        }
    }
}

impl ChatState {
    pub fn set_current_chat_target(&mut self, target: ChatTarget) {
        self.current_chat_target = Some(target);
    }
    
    pub fn get_current_input(&self) -> &str {
        if let Some(target) = &self.current_chat_target {
            self.chat_input_drafts.get(target).map(|s| s.as_str()).unwrap_or("")
        } else {
            ""
        }
    }
    
    pub fn set_current_input(&mut self, value: String) {
        if let Some(target) = &self.current_chat_target {
            self.chat_input_drafts.insert(target.clone(), value);
        }
    }
    
    pub fn clear_current_input(&mut self) {
        if let Some(target) = &self.current_chat_target {
            self.chat_input_drafts.insert(target.clone(), String::new());
        }
    }
    
    pub fn reset_scroll_offset(&mut self) {
        self.chat_scroll_offset = 0;
    }
    
    pub fn update_scroll_offset(&mut self, offset: usize, max_rows: usize) {
        let total_msgs = match &self.current_chat_target {
            Some(ChatTarget::Channel { .. }) => self.chat_messages.len(),
            Some(ChatTarget::DM { .. }) => self.dm_messages.len(),
            None => 0,
        };
        
        let max_scroll = total_msgs.saturating_sub(max_rows);
        self.chat_scroll_offset = offset.min(max_scroll);
    }
    
    pub fn clear_mention_suggestions(&mut self) {
        self.mention_suggestions.clear();
        self.mention_prefix = None;
        self.mention_selected = 0;
    }
    
    pub fn clear_emoji_suggestions(&mut self) {
        self.emoji_suggestions.clear();
        self.emoji_prefix = None;
        self.emoji_selected = 0;
    }
}