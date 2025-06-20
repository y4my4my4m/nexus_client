// client/src/app.rs

use common::{ClientMessage, ServerMessage};
use crate::sound::{SoundManager, SoundType};
use crate::state::{
    ChatState, ForumState, ProfileState, AuthState, NotificationState, UiState,
    AppConfig, AppResult, AppError
};
use crate::services::{ChatService, MessageService, ProfileService, ImageService};
use crate::model::ChatMessageWithMeta;
use tokio::sync::mpsc;

/// Main application state and controller
pub struct App<'a> {
    // State modules
    pub chat: ChatState,
    pub forum: ForumState,
    pub profile: ProfileState,
    pub auth: AuthState,
    pub notifications: NotificationState,
    pub ui: UiState,
    
    // Configuration
    pub config: AppConfig,
    
    // External dependencies
    pub to_server: mpsc::UnboundedSender<ClientMessage>,
    pub sound_manager: &'a SoundManager,
}

impl<'a> App<'a> {
    pub fn new(to_server: mpsc::UnboundedSender<ClientMessage>, sound_manager: &'a SoundManager) -> Self {
        Self {
            chat: ChatState::default(),
            forum: ForumState::default(),
            profile: ProfileState::new(),
            auth: AuthState::default(),
            notifications: NotificationState::default(),
            ui: UiState::default(),
            config: AppConfig::default(),
            to_server,
            sound_manager,
        }
    }

    // --- Core App Methods ---
    
    pub fn send_to_server(&mut self, msg: ClientMessage) {
        if let Err(e) = self.to_server.send(msg) {
            self.set_notification(format!("Failed to send message: {}", e), None, false);
        }
    }

    pub fn set_notification(&mut self, message: impl Into<String>, ms: Option<u64>, minimal: bool) {
        let timeout = ms.unwrap_or(self.config.notification_timeout_ms);
        self.notifications.set_notification(message, Some(timeout), minimal, self.ui.tick_count);
    }

    pub fn on_tick(&mut self) {
        self.ui.tick();
        if self.notifications.should_close_notification(self.ui.tick_count) {
            self.notifications.clear_notification();
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
                if let (Some(forum_id), Some(ref title)) = (self.forum.current_forum_id, &self.forum.pending_new_thread_title.clone()) {
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
                let server_id = if let Some(server) = self.chat.servers.get(s) {
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
                    let user_id = if let Some(user) = self.chat.dm_user_list.get(idx) {
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

    pub fn update_profile_banner_composite(&mut self) {
        // Create composite banner + profile pic image for profile view popup
        if let Some(profile) = &self.profile.profile_view {
            // Check if we have both banner and profile pic data
            let banner_data = ImageService::decode_image_bytes(&profile.cover_banner);
            let pfp_data = ImageService::decode_image_bytes(&profile.profile_pic);
            
            if let (Some(banner_bytes), Some(pfp_bytes)) = (banner_data, pfp_data) {
                // Create composite image: banner with profile pic overlaid
                // Use dynamic sizing based on terminal width for better full-width display
                let banner_size = (600, 120); // Larger banner for better quality
                let pfp_size = (100, 100);    // Larger profile pic for better quality
                let pfp_padding_left = 30;    // PFP position from left
                
                match ImageService::composite_banner_and_pfp(
                    &banner_bytes,
                    &pfp_bytes,
                    banner_size,
                    pfp_size,
                    pfp_padding_left,
                ) {
                    Ok(composite_bytes) => {
                        // Convert composite to image for rendering
                        if let Ok(composite_img) = image::load_from_memory(&composite_bytes) {
                            let protocol = self.profile.picker.new_resize_protocol(composite_img);
                            self.profile.profile_banner_image_state = Some(protocol);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to create banner composite: {}", e);
                        // Fallback to just banner if composite fails
                        if let Ok(banner_img) = image::load_from_memory(&banner_bytes) {
                            let protocol = self.profile.picker.new_resize_protocol(banner_img);
                            self.profile.profile_banner_image_state = Some(protocol);
                        }
                    }
                }
            } else if let Some(banner_bytes) = ImageService::decode_image_bytes(&profile.cover_banner) {
                // Just banner, no profile pic - still apply gradient overlay
                if let Ok(mut banner_img) = image::load_from_memory(&banner_bytes) {
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
            } else {
                // No images available
                self.profile.profile_banner_image_state = None;
            }
        } else {
            self.profile.profile_banner_image_state = None;
        }
    }
}

// Re-export commonly used types for backward compatibility
pub use crate::state::{AppMode, ChatFocus, ProfileEditFocus};
pub use crate::state::InputMode;