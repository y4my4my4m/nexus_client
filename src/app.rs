use common::{ClientMessage, ServerMessage};
use crate::sound::{SoundManager, SoundType};
use crate::state::{
    ChatState, ForumState, ProfileState, AuthState, NotificationState, UiState,
    AppConfig, AppResult, AppError
};
use crate::services::{ChatService, MessageService, ProfileService, ImageService};
use crate::services::image::{ImageCache, ImageCacheStats};
use crate::model::ChatMessageWithMeta;
use crate::ui::backgrounds::BackgroundManager;
use crate::ui::themes::ThemeManager;
use tokio::sync::mpsc;
use std::sync::Arc;
use crate::desktop_notifications::DesktopNotificationService;

/// Main application state and controller
pub struct App<'a> {
    // Network
    pub to_server: mpsc::UnboundedSender<ClientMessage>,
    
    // State modules
    pub auth: AuthState,
    pub chat: ChatState,
    pub forum: ForumState,
    pub profile: ProfileState,
    pub notifications: NotificationState,
    pub ui: UiState,
    
    // Services
    pub sound_manager: &'a SoundManager,
    pub image_cache: Arc<ImageCache>,
    pub chat_service: ChatService,
    
    // Theme system
    pub background_manager: BackgroundManager, // For animated backgrounds
    pub theme_manager: ThemeManager, // For UI color themes
    
    // Configuration
    pub config: AppConfig,
}

impl<'a> App<'a> {
    pub fn new(to_server: mpsc::UnboundedSender<ClientMessage>, sound_manager: &'a SoundManager) -> Self {
        let image_cache = Arc::new(ImageCache::with_default_config());
        let chat_service = ChatService::with_image_cache(image_cache.clone());
        Self {
            to_server,
            auth: AuthState::default(),
            chat: ChatState::default(),
            forum: ForumState::default(),
            profile: ProfileState::default(),
            notifications: NotificationState::default(),
            ui: UiState::default(),
            sound_manager,
            image_cache,
            chat_service,
            background_manager: BackgroundManager::new(),
            theme_manager: ThemeManager::new(),
            config: AppConfig::default(),
        }
    }

    // --- Core App Methods ---
    
    pub fn send_to_server(&mut self, msg: ClientMessage) {
        if let Err(e) = self.to_server.send(msg) {
            self.set_notification(format!("Failed to send message: {}", e), Some(3000), true);
        }
    }

    pub fn set_notification(&mut self, message: impl Into<String>, ms: Option<u64>, minimal: bool) {
        self.notifications.set_notification(message.into(), ms, minimal, self.ui.tick_count);
    }

    pub fn on_tick(&mut self) {
        self.ui.tick();
        if self.notifications.should_close_notification(self.ui.tick_count) {
            self.notifications.clear_notification();
        }
        
        // Periodic cache cleanup (every 5 minutes worth of ticks)
        if self.ui.tick_count % (5 * 60 * 10) == 0 { // Assuming 10 ticks per second
            if let Some(cleaned) = self.chat_service.cleanup_cache() {
                if cleaned > 0 {
                    tracing::debug!("Cleaned {} expired cache entries", cleaned);
                }
            }
        }
    }

    // --- Input Management ---
    
    pub fn enter_input_mode(&mut self, mode: crate::state::InputMode) {
        self.auth.set_input_mode(mode);
        self.ui.set_mode(crate::state::AppMode::Input);
        self.notifications.clear_notification();
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
        self.chat.set_current_chat_target(target.clone());
        
        // Preload images for the new conversation
        self.chat_service.preload_conversation_images(&self.chat);
    }

    // --- Server Message Handling ---
    
    pub fn handle_server_message(&mut self, msg: ServerMessage) {
        use chrono::prelude::*;
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
            ServerMessage::ForumsLightweight(forums_lightweight) => {
                // Convert lightweight forums to regular forums by creating User objects without profile images
                use common::{Forum, Thread, Post, User};
                let forums = forums_lightweight.into_iter().map(|forum_lite| {
                    let threads = forum_lite.threads.into_iter().map(|thread_lite| {
                        let posts = thread_lite.posts.into_iter().map(|post_lite| {
                            Post {
                                id: post_lite.id,
                                author: User {
                                    id: post_lite.author.id,
                                    username: post_lite.author.username,
                                    color: post_lite.author.color,
                                    role: post_lite.author.role,
                                    profile_pic: None, // No profile images in lightweight version
                                    cover_banner: None, // No cover banners in lightweight version
                                    status: post_lite.author.status,
                                },
                                content: post_lite.content,
                                timestamp: post_lite.timestamp,
                                reply_to: post_lite.reply_to,
                            }
                        }).collect();
                        
                        Thread {
                            id: thread_lite.id,
                            title: thread_lite.title,
                            author: User {
                                id: thread_lite.author.id,
                                username: thread_lite.author.username,
                                color: thread_lite.author.color,
                                role: thread_lite.author.role,
                                profile_pic: None, // No profile images in lightweight version
                                cover_banner: None, // No cover banners in lightweight version
                                status: thread_lite.author.status,
                            },
                            posts,
                            timestamp: thread_lite.timestamp,
                        }
                    }).collect();
                    
                    Forum {
                        id: forum_lite.id,
                        name: forum_lite.name,
                        description: forum_lite.description,
                        threads,
                    }
                }).collect();
                
                self.forum.forums = forums;
                
                // Handle pending thread selection (same logic as regular Forums)
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
                
                // Request missing avatars for users that don't have profile pictures
                self.chat_service.request_missing_avatars(&self.chat, &self.to_server);
            }
            ServerMessage::DMUserList(users) => {
                self.chat.dm_user_list = users;
                if self.ui.mode == crate::state::AppMode::Chat && self.chat.sidebar_tab == crate::state::SidebarTab::DMs {
                    self.select_and_load_first_chat();
                }
                
                // Request missing avatars for DM users that don't have profile pictures
                self.chat_service.request_missing_avatars(&self.chat, &self.to_server);
            }
            ServerMessage::DirectMessage(dm) => {
                let current_user_id = self.auth.current_user.as_ref().map(|u| u.id);                
                
                let is_current = if let (Some(crate::state::ChatTarget::DM { user_id }), Some(_my_id)) = (self.chat.current_chat_target.as_ref(), current_user_id) {
                    (user_id == &dm.from || user_id == &dm.to) && self.chat.sidebar_tab == crate::state::SidebarTab::DMs
                } else { false };
                
                // Look up author info by DM sender ID instead of using embedded fields
                let (dm_author_username, sender_profile_pic) = if let Some(dm_user) = self.chat.dm_user_list.iter().find(|u| u.id == dm.from) {
                    (dm_user.username.clone(), dm_user.profile_pic.clone())
                } else if let Some(current_user) = &self.auth.current_user {
                    if current_user.id == dm.from {
                        (current_user.username.clone(), current_user.profile_pic.clone())
                    } else {
                        // Fallback for unknown users
                        (format!("User#{}", dm.from.to_string()[..8].to_uppercase()), None)
                    }
                } else {
                    // Fallback when no current user
                    (format!("User#{}", dm.from.to_string()[..8].to_uppercase()), None)
                };
                
                // Extract needed data before any potential moves
                let dm_from = dm.from;
                let dm_to = dm.to;
                let dm_content = dm.content.clone();
                
                if is_current {
                    self.chat.dm_messages.push(dm);
                    self.chat.reset_scroll_offset();
                } else if let Some(my_id) = current_user_id {
                    if dm_to == my_id {
                        self.chat.unread_dm_conversations.insert(dm_from);
                        self.set_notification(
                            format!("DM from {}: {}", dm_author_username, dm_content),
                            Some(4000),
                            true,
                        );
                        
                        // Desktop notification with profile picture
                        crate::desktop_notifications::DesktopNotificationService::show_dm_notification(
                            &dm_author_username,
                            &dm_content,
                            sender_profile_pic.as_ref(),
                        );
                    }
                }
                
                // Update unread count for sender if not currently viewing their DM
                if let Some(my_id) = current_user_id {
                    if dm_to == my_id && !is_current {
                        self.chat.unread_dm_conversations.insert(dm_from);
                    }
                }
            }
            ServerMessage::MentionNotification { from, content } => {
                self.set_notification(
                    format!("Mentioned by {}: {}", from.username, content),
                    Some(4000),
                    true,
                );
                
                // Show desktop notification for mentions with profile picture
                DesktopNotificationService::show_mention_notification(&from.username, &content, from.profile_pic.as_deref());
                self.sound_manager.play(SoundType::Mention);
            }
            ServerMessage::ForumReplyNotification { thread_id, from_username, message, from_user_profile_pic } => {
                // Check if user is currently viewing this thread - if so, don't show notification
                let is_viewing_thread = if let Some(current_thread_id) = self.forum.current_thread_id {
                    current_thread_id == thread_id && self.ui.mode == crate::state::AppMode::PostView
                } else {
                    false
                };
                
                if !is_viewing_thread {
                    // Show in-app notification
                    self.set_notification(
                        format!("{} replied to your forum post", from_username),
                        Some(4000),
                        true,
                    );
                    
                    // Show desktop notification for forum replies with profile picture (like DMs)
                    DesktopNotificationService::show_dm_notification(
                        &from_username,
                        &message,
                        from_user_profile_pic.as_ref(),
                    );
                    self.sound_manager.play(SoundType::Mention);
                }
            }
            ServerMessage::Notification(text, is_error) => {
                let prefix = if is_error { "Error: " } else { "Info: " };
                self.set_notification(format!("{}{}", prefix, text), Some(2000), false);
                
                // Show desktop notification for important messages
                if is_error {
                    DesktopNotificationService::show_error_notification(&text);
                } else if text.contains("Profile updated successfully") || 
                         text.contains("Server invite") || 
                         text.contains("connected") ||
                         text.contains("disconnected") {
                    DesktopNotificationService::show_info_notification(&text);
                }

                // If this is a profile update success notification, play save sound and return to settings
                if !is_error && text.contains("Profile updated successfully") {
                    self.sound_manager.play(SoundType::Save);
                    if self.ui.mode == crate::state::AppMode::EditProfile {
                        self.ui.set_mode(crate::state::AppMode::Settings);
                    }
                }
            }
            ServerMessage::Notifications { notifications, history_complete } => {
                self.notifications.notifications = notifications.clone();
                self.notifications.notification_history_complete = history_complete;
                
                // Only show desktop notifications for truly new notifications
                // (not when loading notification history)
                // We'll rely on the real-time ForumReplyNotification message instead
            }
            ServerMessage::ServerInviteReceived(invite) => {
                let message = format!("Server invite from {} to join '{}'", invite.from_user.username, invite.server.name);
                self.set_notification(message.clone(), Some(5000), false);
                // Show desktop notification for server invites
                DesktopNotificationService::show_server_invite_notification(&invite.from_user.username, &invite.server.name);
                self.sound_manager.play(SoundType::PopupOpen);
            }
            ServerMessage::ServerInviteResponse { invite_id: _, accepted, user } => {
                let status = if accepted { "accepted" } else { "declined" };
                let message = format!("{} {} your server invite", user.username, status);
                self.set_notification(message, Some(3000), false);
            }
            ServerMessage::UserJoined(user) => {
                // Update existing user or add new user to channel userlist
                if let Some(existing) = self.chat.channel_userlist.iter_mut().find(|u| u.id == user.id) {
                    existing.status = user.status.clone();
                } else {
                    self.chat.channel_userlist.push(user.clone());
                }
                
                // Also update in DM user list if present
                if let Some(existing_dm) = self.chat.dm_user_list.iter_mut().find(|u| u.id == user.id) {
                    existing_dm.status = user.status;
                }
            }
            ServerMessage::UserLeft(user_id) => {
                // Update status to offline instead of removing from list
                if let Some(existing) = self.chat.channel_userlist.iter_mut().find(|u| u.id == user_id) {
                    existing.status = common::UserStatus::Offline;
                }
                
                // Also update in DM user list if present
                if let Some(existing_dm) = self.chat.dm_user_list.iter_mut().find(|u| u.id == user_id) {
                    existing_dm.status = common::UserStatus::Offline;
                }
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
            ServerMessage::ChannelMessagesPaginated { 
                channel_id, 
                messages, 
                has_more, 
                next_cursor: _, 
                prev_cursor: _, 
                total_count: _ 
            } => {
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
                        
                        self.chat.channel_history_complete.insert(channel_id, !has_more);
                        
                        // Preload avatars for new messages
                        self.chat_service.preload_conversation_images(&self.chat);
                    }
                }
            }
            ServerMessage::DirectMessagesPaginated { 
                user_id, 
                messages, 
                has_more, 
                next_cursor: _, 
                prev_cursor: _, 
                total_count: _ 
            } => {
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
                        
                        self.chat.dm_history_complete = !has_more;
                        
                        // Preload avatars for new messages
                        self.chat_service.preload_conversation_images(&self.chat);
                    }
                }
            }
            ServerMessage::CacheStats { total_entries, total_size_mb, hit_ratio, expired_entries } => {
                // Handle cache statistics - could display in debug UI
                tracing::debug!("Cache stats: {} entries, {:.1}MB, {:.1}% hit ratio, {} expired", 
                    total_entries, total_size_mb, hit_ratio * 100.0, expired_entries);
            }
            ServerMessage::ImageCacheInvalidated { keys } => {
                // Remove invalidated images from cache
                for key_str in keys {
                    // Parse key string back to cache key and remove
                    // This would need proper key serialization/deserialization
                    tracing::debug!("Cache invalidated for key: {}", key_str);
                }
            }
            ServerMessage::PerformanceMetrics { query_time_ms, cache_hit_rate, message_count } => {
                // Log performance metrics for monitoring
                tracing::debug!("Query performance: {}ms, cache hit rate: {:.1}%, {} messages", 
                    query_time_ms, cache_hit_rate * 100.0, message_count);
                
                // Could trigger UI indicators for slow queries
                if query_time_ms > 1000 {
                    self.set_notification("Slow network detected", Some(2000), false);
                }
            }
            ServerMessage::UserAvatars { avatars } => {
                // Update avatar cache with received avatars
                for (user_id, profile_pic) in avatars {
                    // Update users in channel userlist
                    if let Some(user) = self.chat.channel_userlist.iter_mut().find(|u| u.id == user_id) {
                        user.profile_pic = profile_pic.clone();
                    }
                    // Update users in DM list
                    if let Some(user) = self.chat.dm_user_list.iter_mut().find(|u| u.id == user_id) {
                        user.profile_pic = profile_pic.clone();
                    }
                    // Invalidate any cached avatar protocols to force reload
                    self.profile.invalidate_avatar_cache(user_id);
                }
            }
            
            // Handle any unmatched server messages
            _ => {
                self.handle_legacy_server_message(msg);
            }
        }
    }

    /// Handle legacy server messages to maintain compatibility
    fn handle_legacy_server_message(&mut self, msg: ServerMessage) {
        // Implementation of existing server message handling logic
        // This would contain all the existing match arms from the original handle_server_message
    }

    // --- Cache Management ---
    
    /// Get image cache statistics for debugging
    pub fn get_cache_stats(&self) -> Option<ImageCacheStats> {
        self.chat_service.get_cache_stats()
    }
    
    /// Force cache cleanup
    pub fn cleanup_cache(&mut self) -> usize {
        self.chat_service.cleanup_cache().unwrap_or(0)
    }
    
    /// Clear all cached images
    pub fn clear_cache(&mut self) -> Result<(), String> {
        self.image_cache.clear()
    }

    // --- Chat Navigation ---
    
    pub fn select_and_load_first_chat(&mut self) {
        match self.chat.sidebar_tab {
            crate::state::SidebarTab::Servers => {
                if self.chat.servers.is_empty() {
                    self.chat.selected_server = None;
                    self.chat.selected_channel = None;
                    self.chat.current_chat_target = None;
                    return;
                }
                
                if self.chat.selected_server.is_none() {
                    self.chat.selected_server = Some(0);
                }
                
                let s = self.chat.selected_server.unwrap_or(0);
                let _server_id = if let Some(server) = self.chat.servers.get(s) {
                    let server_id = server.id;
                    if server.channels.is_empty() {
                        self.chat.selected_channel = None;
                        self.chat.current_chat_target = None;
                        return;
                    }
                    
                    if self.chat.selected_channel.is_none() {
                        self.chat.selected_channel = Some(0);
                    }
                    
                    let c = self.chat.selected_channel.unwrap_or(0);
                    if let Some(channel) = server.channels.get(c) {
                        let target = crate::state::ChatTarget::Channel { 
                            server_id: server.id, 
                            channel_id: channel.id 
                        };
                        let channel_id = channel.id;
                        self.set_current_chat_target(target);
                        self.send_to_server(ClientMessage::GetChannelUserList { channel_id });
                        self.send_to_server(ClientMessage::GetChannelMessages { channel_id, before: None });
                        self.chat.reset_scroll_offset();
                    }
                    server_id
                } else {
                    return;
                };
            }
            crate::state::SidebarTab::DMs => {
                if self.chat.dm_user_list.is_empty() {
                    self.chat.selected_dm_user = None;
                    self.chat.current_chat_target = None;
                    return;
                }
                
                if self.chat.selected_dm_user.is_none() {
                    self.chat.selected_dm_user = Some(0);
                }
                
                if let Some(idx) = self.chat.selected_dm_user {
                    let _user_id = if let Some(user) = self.chat.dm_user_list.get(idx) {
                        let target = crate::state::ChatTarget::DM { user_id: user.id };
                        let user_id = user.id;
                        self.set_current_chat_target(target);
                        self.send_to_server(ClientMessage::GetDirectMessages { user_id, before: None });
                        self.chat.unread_dm_conversations.remove(&user_id);
                        self.chat.reset_scroll_offset();
                        user_id
                    } else {
                        return;
                    };
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
                    // Handle DM commands
                    if let Some((command, _args)) = MessageService::parse_command(&validated_content) {
                        match command.as_str() {
                            "accept" => {
                                self.send_to_server(ClientMessage::AcceptServerInviteFromUser { from_user_id: *user_id });
                                self.set_notification("Server invite accepted!", Some(2000), false);
                                self.sound_manager.play(SoundType::Select);
                                self.clear_current_input();
                                return Ok(());
                            }
                            "decline" => {
                                self.send_to_server(ClientMessage::DeclineServerInviteFromUser { from_user_id: *user_id });
                                self.set_notification("Server invite declined.", Some(2000), false);
                                self.sound_manager.play(SoundType::Select);
                                self.clear_current_input();
                                return Ok(());
                            }
                            _ => {}
                        }
                    }
                    
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
        
        // Process images with proper error handling
        let profile_pic = match ProfileService::file_or_url_to_base64(&self.profile.edit_profile_pic) {
            Ok(result) => result,
            Err(e) => {
                self.profile.profile_edit_error = Some(format!("Profile pic error: {}", e));
                return Err(AppError::Validation(e));
            }
        };
        
        let cover_banner = match ProfileService::file_or_url_to_base64(&self.profile.edit_cover_banner) {
            Ok(result) => result,
            Err(e) => {
                self.profile.profile_edit_error = Some(format!("Cover banner error: {}", e));
                return Err(AppError::Validation(e));
            }
        };
        
        // Clear any previous errors
        self.profile.profile_edit_error = None;
        
        self.send_to_server(ClientMessage::UpdateProfile {
            bio: Some(self.profile.edit_bio.clone()),
            url1: Some(self.profile.edit_url1.clone()),
            url2: Some(self.profile.edit_url2.clone()),
            url3: Some(self.profile.edit_url3.clone()),
            location: Some(self.profile.edit_location.clone()),
            profile_pic,
            cover_banner,
        });
        
        // Don't play success sound or change mode here - wait for server response
        // The server will send a success notification when the profile is actually saved
        
        Ok(())
    }

    // --- Mention System ---
    
    pub fn update_mention_suggestions(&mut self) {
        let input = self.get_current_input().to_string(); // Clone the input to avoid borrow issues
        let suggestions = ChatService::get_mention_suggestions(&input, &self.chat.channel_userlist);
        
        if !suggestions.is_empty() {
            // Convert usernames back to indices for UI compatibility
            let mut indices = Vec::new();
            for suggestion in &suggestions {
                if let Some((idx, _)) = self.chat.channel_userlist.iter().enumerate()
                    .find(|(_, u)| &u.username == suggestion) {
                    indices.push(idx);
                }
            }
            self.chat.mention_suggestions = indices;
            self.chat.mention_selected = 0;
            
            // Extract prefix from input
            if let Some(idx) = input.rfind('@') {
                let after_at = &input[(idx + 1)..];
                if after_at.chars().all(|ch| ch.is_alphanumeric() || ch == '_') && !after_at.is_empty() {
                    self.chat.mention_prefix = Some(after_at.to_string());
                }
            }
        } else {
            self.chat.clear_mention_suggestions();
        }
    }

    pub fn apply_selected_mention(&mut self) {
        if let (Some(prefix), Some(&user_idx)) = (
            &self.chat.mention_prefix.clone(),
            self.chat.mention_suggestions.get(self.chat.mention_selected)
        ) {
            if let Some(user) = self.chat.channel_userlist.get(user_idx) {
                let input = self.get_current_input().to_string();
                let new_input = ChatService::apply_mention_suggestion(&input, &user.username, prefix);
                self.set_current_input(new_input);
                self.chat.clear_mention_suggestions();
            }
        }
    }

    // --- Emoji System ---
    
    pub fn update_emoji_suggestions(&mut self) {
        let input = self.get_current_input().to_string();
        
        // First check for exact emoji matches and auto-transform them
        if let Some((emoji, start_pos, end_pos)) = ChatService::check_for_exact_emoji_match(&input) {
            let mut new_input = input.clone();
            new_input.replace_range(start_pos..end_pos, &emoji);
            self.set_current_input(new_input);
            self.chat.clear_emoji_suggestions();
            return;
        }
        
        // If no exact match, show suggestions
        let suggestions = ChatService::get_emoji_suggestions(&input);
        
        if !suggestions.is_empty() {
            self.chat.emoji_suggestions = suggestions;
            self.chat.emoji_selected = 0;
            
            // Extract prefix from input
            if let Some(idx) = input.rfind(':') {
                let after_colon = &input[(idx + 1)..];
                if after_colon.chars().all(|ch| ch.is_alphabetic() || ch == '_') && !after_colon.is_empty() {
                    self.chat.emoji_prefix = Some(after_colon.to_string());
                }
            }
        } else {
            self.chat.clear_emoji_suggestions();
        }
    }

    pub fn apply_selected_emoji(&mut self) {
        if let (Some(prefix), Some(emoji)) = (
            &self.chat.emoji_prefix.clone(),
            self.chat.emoji_suggestions.get(self.chat.emoji_selected)
        ) {
            let input = self.get_current_input().to_string();
            let new_input = ChatService::apply_emoji_suggestion(&input, emoji, prefix);
            self.set_current_input(new_input);
            self.chat.clear_emoji_suggestions();
        }
    }

    pub fn update_profile_banner_composite(&mut self, banner_area_width_cells: u16, banner_area_height_cells: u16) {
        // Create composite banner + profile pic image for profile view popup
        if let Some(profile) = &self.profile.profile_view {
            // Check if we have both banner and profile pic data
            let banner_data = ImageService::decode_image_bytes(&profile.cover_banner);
            let pfp_data = ImageService::decode_image_bytes(&profile.profile_pic);
            
            if let (Some(banner_bytes), Some(pfp_bytes)) = (banner_data, pfp_data) {
                // Create composite image: banner with profile pic overlaid
                // Use dynamic sizing based on terminal width for better full-width display
                let font_size = self.profile.picker.font_size();
                let banner_px_w = banner_area_width_cells as u32 * font_size.0 as u32;
                let banner_px_h = banner_area_height_cells as u32 * font_size.1 as u32;
                let banner_size = (banner_px_w, banner_px_h);
                let pfp_size = (64, 64);    // Larger profile pic for better quality
                let pfp_padding_left = 30;    // PFP position from left
                
                match ImageService::composite_banner_and_pfp(
                    &banner_bytes,
                    &pfp_bytes,
                    banner_size,
                    pfp_size,
                    pfp_padding_left,
                    &profile.username, // Pass the username for text rendering
                ) {
                    Ok(composite_bytes) => {
                        // Convert composite to image for rendering
                        if let Ok(composite_img) = image::load_from_memory(&composite_bytes) {
                            let protocol = self.profile.picker.new_resize_protocol(composite_img);
                            self.profile.profile_banner_image_state = Some(protocol);
                        }
                    }
                    Err(e) => {
                        // self.set_notification(format!("Failed to create profile banner: {}", e), Some(3000), false);
                        // Fallback to profile pic with username using the new helper function
                        let font_size = self.profile.picker.font_size();
                        let banner_px_w = banner_area_width_cells as u32 * font_size.0 as u32;
                        let banner_px_h = banner_area_height_cells as u32 * font_size.1 as u32;
                        let banner_size = (banner_px_w, banner_px_h);
                        
                        match ImageService::create_pfp_with_username(&pfp_bytes, &profile.username, banner_size) {
                            Ok(fallback_bytes) => {
                                if let Ok(fallback_img) = image::load_from_memory(&fallback_bytes) {
                                    let protocol = self.profile.picker.new_resize_protocol(fallback_img);
                                    self.profile.profile_banner_image_state = Some(protocol);
                                }
                            }
                            Err(_) => {
                                // Last resort: just the profile pic alone
                                if let Ok(pfp_img) = image::load_from_memory(&pfp_bytes) {
                                    let protocol = self.profile.picker.new_resize_protocol(pfp_img);
                                    self.profile.profile_banner_image_state = Some(protocol);
                                }
                            }
                        }
                    }
                }
            } else if let Some(banner_bytes) = ImageService::decode_image_bytes(&profile.cover_banner) {
                // Just banner, no profile pic - still apply gradient overlay
                if let Ok(banner_img) = image::load_from_memory(&banner_bytes) {
                    // Apply black gradient overlay even for banner-only images
                    let mut rgba_img = banner_img.to_rgba8();
                    let (width, height) = rgba_img.dimensions();
                    
                    for y in 0..height {
                        for x in 0..width {
                            let pixel = rgba_img.get_pixel_mut(x, y);
                            // Create a subtle black gradient from top to bottom
                            let gradient_factor = (y as f32 / height as f32) * 0.4 + 0.1; // 0.1 to 0.5 opacity
                            pixel[0] = (pixel[0] as f32 * (1.0 - gradient_factor)) as u8;
                            pixel[1] = (pixel[1] as f32 * (1.0 - gradient_factor)) as u8;
                            pixel[2] = (pixel[2] as f32 * (1.0 - gradient_factor)) as u8;
                        }
                    }
                    
                    let protocol = self.profile.picker.new_resize_protocol(image::DynamicImage::ImageRgba8(rgba_img));
                    self.profile.profile_banner_image_state = Some(protocol);
                }
            } else if let Some(pfp_bytes) = ImageService::decode_image_bytes(&profile.profile_pic) {
                // Just profile pic, no banner - create a default dark background with profile pic overlaid
                let font_size = self.profile.picker.font_size();
                let banner_px_w = banner_area_width_cells as u32 * font_size.0 as u32;
                let banner_px_h = banner_area_height_cells as u32 * font_size.1 as u32;
                let banner_size = (banner_px_w, banner_px_h);
                let pfp_size = (64, 64);
                let pfp_padding_left = 30;
                
                // Create a default dark gradient background
                let mut default_banner = image::RgbaImage::new(banner_size.0, banner_size.1);
                for y in 0..banner_size.1 {
                    for x in 0..banner_size.0 {
                        // Create a subtle dark gradient from dark gray to black
                        let gradient_factor = y as f32 / banner_size.1 as f32;
                        let gray_value = (64.0 * (1.0 - gradient_factor * 0.5)) as u8; // 64 to 32
                        default_banner.put_pixel(x, y, image::Rgba([gray_value, gray_value, gray_value, 255]));
                    }
                }
                
                // Convert to bytes for composite function
                let mut banner_bytes = Vec::new();
                match default_banner.write_to(&mut std::io::Cursor::new(&mut banner_bytes), image::ImageFormat::Png) {
                    Ok(()) => {
                        match ImageService::composite_banner_and_pfp(
                            &banner_bytes,
                            &pfp_bytes,
                            banner_size,
                            pfp_size,
                            pfp_padding_left,
                            &profile.username,
                        ) {
                            Ok(composite_bytes) => {
                                if let Ok(composite_img) = image::load_from_memory(&composite_bytes) {
                                    let protocol = self.profile.picker.new_resize_protocol(composite_img);
                                    self.profile.profile_banner_image_state = Some(protocol);
                                }
                            }
                            Err(e) => {
                                self.set_notification(format!("Failed to create profile banner: {}", e), Some(3000), true);
                                // Fallback to just the profile pic if composite fails
                                if let Ok(pfp_img) = image::load_from_memory(&pfp_bytes) {
                                    let protocol = self.profile.picker.new_resize_protocol(pfp_img);
                                    self.profile.profile_banner_image_state = Some(protocol);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        self.set_notification(format!("Failed to encode profile banner: {}", e), Some(3000), true);
                        // Direct fallback to profile pic
                        if let Ok(pfp_img) = image::load_from_memory(&pfp_bytes) {
                            let protocol = self.profile.picker.new_resize_protocol(pfp_img);
                            self.profile.profile_banner_image_state = Some(protocol);
                        }
                    }
                }
            } else {
                // No images available
                self.profile.profile_banner_image_state = None;
            }
        } else {
            self.profile.profile_banner_image_state = None;
        }
    }

    pub fn get_current_chat_title(&self) -> String {
        match &self.chat.current_chat_target {
            Some(crate::state::ChatTarget::Channel { .. }) => {
                let channel_name = self.chat.selected_server
                    .and_then(|server_idx| self.chat.servers.get(server_idx))
                    .and_then(|server| self.chat.selected_channel
                        .and_then(|channel_idx| server.channels.get(channel_idx))
                        .map(|channel| channel.name.as_str()))
                    .unwrap_or("unknown");
                
                format!("Channel // #{}", channel_name)
            }
            Some(crate::state::ChatTarget::DM { .. }) => {
                let username = self.chat.selected_dm_user
                    .and_then(|dm_idx| self.chat.dm_user_list.get(dm_idx))
                    .map(|user| user.username.as_str())
                    .unwrap_or("unknown");
                
                format!("Conversation // @{}", username)
            }
            None => "Chat".to_string()
        }
    }
}

// Re-export commonly used types for backward compatibility
pub use crate::state::{AppMode, ChatFocus, ProfileEditFocus};
pub use crate::state::InputMode;