// client/src/app.rs

use common::{ClientMessage, ServerMessage, ClientConfig};
use crate::sound::{SoundManager, SoundType};
use crate::state::{
    ChatState, ForumState, ProfileState, AuthState, NotificationState, UiState,
    AppConfig, AppResult, AppError
};
use crate::services::{ChatService, MessageService, ProfileService, ImageService, ImageCache};
use crate::model::ChatMessageWithMeta;
use tokio::sync::mpsc;

/// Main application state and controller
pub struct App<'a> {
    // State modules
    pub chat: ChatState,
    pub forum: ForumState,
    pub profile: ProfileState,
    pub auth: AuthState,
    pub notification: NotificationState,
    pub notifications: NotificationState, // Add this for compatibility
    pub ui: UiState,
    
    // Configuration and caching
    pub config: ClientConfig,
    pub image_cache: ImageCache,
    
    // External dependencies
    pub tx: mpsc::UnboundedSender<ClientMessage>,
    pub sound_manager: SoundManager,
}

impl<'a> App<'a> {
    pub fn new(tx: mpsc::UnboundedSender<ClientMessage>) -> Self {
        // Load client configuration
        let config = ClientConfig::load_or_default(&ClientConfig::default_path());
        
        // Initialize image cache with config values
        let image_cache = ImageCache::new(
            config.performance.image_cache_size_mb,
            config.performance.max_cached_images,
        );
        
        let notification = NotificationState::default();
        
        Self {
            chat: ChatState::default(),
            forum: ForumState::default(),
            profile: ProfileState::new(),
            auth: AuthState::default(),
            notification: notification.clone(),
            notifications: notification, // Use same instance for compatibility
            ui: UiState::new(),
            config,
            image_cache,
            tx,
            sound_manager: SoundManager::new(),
        }
    }

    // --- Core App Methods ---
    
    pub fn send_to_server(&mut self, msg: ClientMessage) {
        if let Err(e) = self.tx.send(msg) {
            self.set_notification(format!("Failed to send message: {}", e), None, false);
        }
    }

    pub fn set_notification(&mut self, message: impl Into<String>, ms: Option<u64>, minimal: bool) {
        let timeout = ms.unwrap_or(self.config.notifications.notification_timeout_ms);
        self.notification.set_notification(message, Some(timeout), minimal, self.ui.tick_count);
    }

    pub fn on_tick(&mut self) {
        self.ui.tick();
        if self.notification.should_close_notification(self.ui.tick_count) {
            self.notification.clear_notification();
        }
    }

    // --- Input Management ---
    
    pub fn enter_input_mode(&mut self, mode: crate::state::InputMode) {
        self.auth.set_input_mode(mode);
        self.ui.set_mode(crate::state::AppMode::Input);
        self.notification.clear_notification();
    }

    // --- Chat Methods ---
    
    pub fn get_current_message_list(&self) -> Vec<ChatMessageWithMeta> {
        ChatService::build_message_list(&self.chat, self.auth.current_user.as_ref())
    }

    pub fn get_current_input(&self) -> &str {
        self.chat.get_current_input()
    }

    pub fn set_current_input(&mut self, value: String) {
        self.chat.set_current_input(value);
    }

    pub fn clear_current_input(&mut self) {
        self.chat.clear_current_input();
    }

    pub fn set_current_chat_target(&mut self, target: crate::state::ChatTarget) {
        self.chat.set_current_chat_target(target);
    }

    // --- Server Message Handling ---
    
    pub fn handle_server_message(&mut self, msg: ServerMessage) {
        match msg {
            ServerMessage::AuthSuccess(user) => {
                self.auth.login(user);
                self.ui.set_mode(crate::state::AppMode::MainMenu);
                self.ui.reset_selections();
                self.sound_manager.play(SoundType::LoginSuccess);
            }
            ServerMessage::AuthFailure(reason) => {
                self.set_notification(format!("Error: {}", reason), None, false);
                self.sound_manager.play(SoundType::LoginFailure);
            }
            ServerMessage::Forums(forums) => {
                self.forum.forums = forums;
                // Handle pending thread selection
                if let (Some(_forum_id), Some(ref title)) = (self.forum.current_forum_id, &self.forum.pending_new_thread_title.clone()) {
                    if let Some(forum) = self.forum.get_current_forum() {
                        if let Some((idx, thread)) = forum.threads.iter().enumerate().find(|(_, t)| t.title == *title) {
                            let thread_id = thread.id; // Extract thread_id to avoid borrowing issues
                            self.forum.thread_list_state.select(Some(idx));
                            self.forum.select_thread(thread_id);
                            self.ui.set_mode(crate::state::AppMode::PostView);
                            self.forum.clear_pending_thread();
                        }
                    }
                }
            }
            ServerMessage::Profile(profile) => {
                if self.profile.profile_requested_by_user {
                    self.profile.set_profile_for_viewing(profile);
                } else {
                    self.profile.load_profile_for_editing(&profile);
                }
                self.profile.profile_requested_by_user = false;
            }
            ServerMessage::UserUpdated(user) => {
                // Update user in channel userlist
                if let Some(existing) = self.chat.channel_userlist.iter_mut().find(|u| u.id == user.id) {
                    *existing = user.clone();
                }
                // Update current user if it's this user
                if let Some(current) = &mut self.auth.current_user {
                    if current.id == user.id {
                        *current = user.clone();
                    }
                }
                // Invalidate avatar cache
                self.profile.invalidate_avatar_cache(user.id);
            }
            ServerMessage::Servers(servers) => {
                self.chat.servers = servers;
                if self.ui.mode == crate::state::AppMode::Chat && self.chat.sidebar_tab == crate::state::SidebarTab::Servers {
                    self.select_and_load_first_chat();
                }
            }
            ServerMessage::ChannelUserList { channel_id: _, users } => {
                let mut sorted_users = users;
                sorted_users.sort_by(|a, b| a.username.to_lowercase().cmp(&b.username.to_lowercase()));
                sorted_users.reverse();
                self.chat.channel_userlist = sorted_users;
                
                if !self.chat.channel_userlist.is_empty() {
                    self.chat.user_list_state.select(Some(0));
                } else {
                    self.chat.user_list_state.select(None);
                }
            }
            ServerMessage::DMUserList(users) => {
                self.chat.dm_user_list = users;
                if self.ui.mode == crate::state::AppMode::Chat && self.chat.sidebar_tab == crate::state::SidebarTab::DMs {
                    self.select_and_load_first_chat();
                }
            }
            ServerMessage::DirectMessage(dm) => {
                let current_user_id = self.auth.current_user.as_ref().map(|u| u.id);
                let is_current = if let (Some(crate::state::ChatTarget::DM { user_id }), Some(_my_id)) = (self.chat.current_chat_target.as_ref(), current_user_id) {
                    (user_id == &dm.from || user_id == &dm.to) && self.chat.sidebar_tab == crate::state::SidebarTab::DMs
                } else { false };
                
                if is_current {
                    self.chat.dm_messages.push(dm);
                    self.chat.reset_scroll_offset();
                } else if let Some(my_id) = current_user_id {
                    if dm.to == my_id {
                        self.chat.unread_dm_conversations.insert(dm.from);
                        self.set_notification(
                            format!("DM from {}: {}", dm.author_username, dm.content),
                            Some(4000),
                            true,
                        );
                        self.sound_manager.play(SoundType::DirectMessage);
                    }
                }
            }
            ServerMessage::MentionNotification { from, content } => {
                self.set_notification(
                    format!("Mentioned by {}: {}", from.username, content),
                    Some(4000),
                    true,
                );
                self.sound_manager.play(SoundType::Mention);
            }
            ServerMessage::Notification(text, is_error) => {
                let prefix = if is_error { "Error: " } else { "Info: " };
                self.set_notification(format!("{}{}", prefix, text), Some(2000), false);
            }
            ServerMessage::Notifications { notifications, history_complete } => {
                self.notifications.notifications = notifications;
                self.notifications.notification_history_complete = history_complete;
            }
            ServerMessage::NotificationUpdated { notification_id, read } => {
                self.notifications.update_notification(notification_id, read);
            }
            ServerMessage::ServerInviteReceived(invite) => {
                let message = format!("Server invite from {} to join '{}'", invite.from_user.username, invite.server.name);
                self.set_notification(message, Some(5000), false);
                self.sound_manager.play(SoundType::PopupOpen);
            }
            ServerMessage::ServerInviteResponse { invite_id: _, accepted, user } => {
                let status = if accepted { "accepted" } else { "declined" };
                let message = format!("{} {} your server invite", user.username, status);
                self.set_notification(message, Some(3000), false);
            }
            ServerMessage::UserJoined(user) => {
                if !self.chat.channel_userlist.iter().any(|u| u.id == user.id) {
                    self.chat.channel_userlist.push(user);
                }
            }
            ServerMessage::UserLeft(user_id) => {
                self.chat.channel_userlist.retain(|u| u.id != user_id);
            }
            ServerMessage::NewChannelMessage(msg) => {
                let current_target = &self.chat.current_chat_target;
                let is_current_channel = if let Some(crate::state::ChatTarget::Channel { channel_id, .. }) = current_target {
                    *channel_id == msg.channel_id
                } else { false };
                
                if is_current_channel {
                    self.chat.chat_messages.push(msg);
                    self.chat.reset_scroll_offset();
                    self.sound_manager.play(SoundType::ReceiveChannelMessage);
                } else {
                    self.chat.unread_channels.insert(msg.channel_id);
                }
            }
            ServerMessage::ChannelMessages { channel_id, messages, history_complete } => {
                if let Some(crate::state::ChatTarget::Channel { channel_id: current_channel_id, .. }) = &self.chat.current_chat_target {
                    if *current_channel_id == channel_id {
                        if self.chat.chat_messages.is_empty() {
                            self.chat.chat_messages = messages;
                        } else {
                            // Prepend new messages for history loading
                            let mut all_messages = messages;
                            all_messages.extend(self.chat.chat_messages.drain(..));
                            self.chat.chat_messages = all_messages;
                        }
                        
                        self.chat.channel_history_complete.insert(channel_id, history_complete);
                    }
                }
            }
            ServerMessage::DirectMessages { user_id, messages, history_complete } => {
                if let Some(crate::state::ChatTarget::DM { user_id: current_user_id }) = &self.chat.current_chat_target {
                    if *current_user_id == user_id {
                        if self.chat.dm_messages.is_empty() {
                            self.chat.dm_messages = messages;
                        } else {
                            // Prepend new messages for history loading
                            let mut all_messages = messages;
                            all_messages.extend(self.chat.dm_messages.drain(..));
                            self.chat.dm_messages = all_messages;
                        }
                        
                        self.chat.dm_history_complete = history_complete;
                    }
                }
            }
            _ => {
                // Log unhandled messages for debugging
                println!("Unhandled server message: {:?}", std::any::type_name::<ServerMessage>());
            }
        }
    }

    // --- Performance and Configuration Methods ---
    
    /// Get image cache statistics for debugging
    pub fn get_cache_stats(&self) -> String {
        let stats = self.image_cache.stats();
        format!(
            "Cache: {}/{} entries, {:.1}/{:.1} MB",
            stats.entries, stats.max_entries,
            stats.size_mb, stats.max_size_mb
        )
    }
    
    /// Clear image cache
    pub fn clear_image_cache(&mut self) {
        self.image_cache.clear();
        self.set_notification("Image cache cleared", Some(2000), false);
    }
    
    /// Save current configuration
    pub fn save_config(&self) -> AppResult<()> {
        self.config.save(&ClientConfig::default_path())
            .map_err(|e| AppError::IO(std::io::Error::new(std::io::ErrorKind::Other, format!("{}", e))))?;
        Ok(())
    }
    
    /// Update configuration setting
    pub fn update_config_setting(&mut self, setting: &str, value: &str) -> AppResult<()> {
        match setting {
            "show_avatars" => {
                self.config.ui.show_avatars = value.parse().unwrap_or(true);
            }
            "show_timestamps" => {
                self.config.ui.show_timestamps = value.parse().unwrap_or(true);
            }
            "compact_mode" => {
                self.config.ui.compact_mode = value.parse().unwrap_or(false);
            }
            "audio_enabled" => {
                self.config.audio.enabled = value.parse().unwrap_or(true);
                if !self.config.audio.enabled {
                    self.sound_manager.set_enabled(false);
                }
            }
            "audio_volume" => {
                if let Ok(volume) = value.parse::<f32>() {
                    self.config.audio.volume = volume.clamp(0.0, 1.0);
                    self.sound_manager.set_volume(self.config.audio.volume);
                }
            }
            "cache_size_mb" => {
                if let Ok(size) = value.parse::<usize>() {
                    self.config.performance.image_cache_size_mb = size;
                    // Recreate cache with new size
                    self.image_cache = ImageCache::new(size, self.config.performance.max_cached_images);
                }
            }
            _ => {
                return Err(AppError::Validation(format!("Unknown setting: {}", setting)));
            }
        }
        
        self.save_config()?;
        self.set_notification(&format!("Updated setting: {}", setting), Some(2000), false);
        Ok(())
    }
    
    // --- Chat Navigation ---
    
    pub fn select_and_load_first_chat(&mut self) {
        // Simplified implementation
        match self.chat.sidebar_tab {
            crate::state::SidebarTab::Servers => {
                if !self.chat.servers.is_empty() && self.chat.selected_server.is_none() {
                    self.chat.selected_server = Some(0);
                }
            }
            crate::state::SidebarTab::DMs => {
                if !self.chat.dm_user_list.is_empty() && self.chat.selected_dm_user.is_none() {
                    self.chat.selected_dm_user = Some(0);
                }
            }
        }
    }

    // --- Message Sending ---
    
    pub fn send_message(&mut self) -> AppResult<()> {
        let content = self.get_current_input().to_string();
        let validated_content = MessageService::validate_message(&content)
            .map_err(|e| AppError::Validation(e))?;
        
        if let Some(target) = &self.chat.current_chat_target.clone() {
            match target {
                crate::state::ChatTarget::Channel { channel_id, .. } => {
                    self.send_to_server(ClientMessage::SendChannelMessage {
                        channel_id: *channel_id,
                        content: validated_content,
                    });
                    self.sound_manager.play(SoundType::SendChannelMessage);
                }
                crate::state::ChatTarget::DM { user_id } => {
                    self.send_to_server(ClientMessage::SendDirectMessage {
                        to: *user_id,
                        content: validated_content,
                    });
                    self.sound_manager.play(SoundType::MessageSent);
                }
            }
            self.clear_current_input();
        }
        
        Ok(())
    }

    // --- Profile Management ---
    
    pub fn save_profile(&mut self) -> AppResult<()> {
        // Validate profile data
        ProfileService::validate_profile_data(
            &self.profile.edit_bio,
            &self.profile.edit_url1,
            &self.profile.edit_url2,
            &self.profile.edit_url3,
            &self.profile.edit_location,
        ).map_err(|e| AppError::Validation(e))?;
        
        // Validate and process images
        ImageService::validate_image_data(&self.profile.edit_profile_pic)?;
        ImageService::validate_image_data(&self.profile.edit_cover_banner)?;
        
        let profile_pic = ProfileService::file_or_url_to_base64(&self.profile.edit_profile_pic);
        let cover_banner = ProfileService::file_or_url_to_base64(&self.profile.edit_cover_banner);
        
        self.send_to_server(ClientMessage::UpdateProfile {
            bio: Some(self.profile.edit_bio.clone()),
            url1: Some(self.profile.edit_url1.clone()),
            url2: Some(self.profile.edit_url2.clone()),
            url3: Some(self.profile.edit_url3.clone()),
            location: Some(self.profile.edit_location.clone()),
            profile_pic,
            cover_banner,
        });
        
        self.sound_manager.play(SoundType::Save);
        self.ui.set_mode(crate::state::AppMode::Settings);
        
        Ok(())
    }

    // Placeholder methods for mention and emoji systems
    pub fn update_mention_suggestions(&mut self) {
        // Implementation would go here
    }
    
    pub fn apply_selected_mention(&mut self) {
        // Implementation would go here
    }
    
    pub fn update_emoji_suggestions(&mut self) {
        // Implementation would go here
    }
    
    pub fn apply_selected_emoji(&mut self) {
        // Implementation would go here
    }
    
    pub fn update_profile_banner_composite(&mut self) {
        // Implementation would go here
    }
}

// Re-export commonly used types for backward compatibility
pub use crate::state::{AppMode, ChatFocus, ProfileEditFocus};
pub use crate::state::InputMode;