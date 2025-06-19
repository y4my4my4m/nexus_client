// client/src/app.rs

use common::{ChatMessage, ClientMessage, Forum, ServerMessage, User, UserProfile};
use crate::sound::{SoundManager, SoundType};
use ratatui::widgets::ListState;
use tokio::sync::mpsc;
use uuid::Uuid;
use std::fs;
use std::io::Cursor;
use std::path::Path;
use base64::engine::Engine as _;
use std::collections::{HashSet, HashMap};
use crate::global_prefs::GlobalPrefs;
use std::sync::Arc;

use ratatui_image::{picker::Picker, protocol::StatefulProtocol};

#[derive(PartialEq, Debug, Clone)]
pub enum AppMode {
    Login, Register, MainMenu, Settings, ForumList, ThreadList, PostView, Chat, Input, EditProfile, ColorPicker, Parameters,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    LoginUsername,
    LoginPassword,
    RegisterUsername,
    RegisterPassword,
    AuthSubmit,
    AuthSwitch,
    NewThreadTitle,
    NewThreadContent,
    NewPostContent,
    UpdatePassword,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatFocus {
    Messages,
    Users,
    DMInput,
    Sidebar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileEditFocus {
    Bio,
    Url1,
    Url2,
    Url3,
    Location,
    ProfilePic,
    ProfilePicDelete,
    CoverBanner,
    CoverBannerDelete,
    Save,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarSection {
    Servers,
    DMs,
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

pub struct App<'a> {
    pub mode: AppMode,
    pub input_mode: Option<InputMode>,
    pub current_input: String,
    pub password_input: String,
    pub notification: Option<(String, Option<u64>, bool)>,
    pub current_user: Option<User>,
    pub main_menu_state: ListState,
    pub forum_list_state: ListState,
    pub thread_list_state: ListState,
    pub settings_list_state: ListState,
    pub user_list_state: ListState,
    pub forums: Vec<Forum>,
    pub current_forum_id: Option<Uuid>,
    pub current_thread_id: Option<Uuid>,
    pub chat_messages: Vec<ChatMessage>,
    pub tick_count: u64,
    pub should_quit: bool,
    pub to_server: mpsc::UnboundedSender<ClientMessage>,
    pub sound_manager: &'a SoundManager,
    pub show_user_list: bool,
    pub connected_users: Vec<User>,
    pub chat_focus: ChatFocus,
    pub dm_input: String,
    pub dm_target: Option<uuid::Uuid>,
    pub show_user_actions: bool,
    pub user_actions_selected: usize,
    pub user_actions_target: Option<usize>,
    pub edit_bio: String,
    pub edit_url1: String,
    pub edit_url2: String,
    pub edit_url3: String,
    pub edit_location: String,
    pub edit_profile_pic: String,
    pub edit_cover_banner: String,
    pub profile_edit_error: Option<String>,
    pub show_profile_view_popup: bool,
    pub profile_edit_focus: ProfileEditFocus,
    pub profile_requested_by_user: bool,

    // --- Image rendering fields ---
    pub picker: Picker,
    pub profile_view: Option<UserProfile>,
    pub profile_image_state: Option<StatefulProtocol>,
    pub profile_banner_image_state: Option<StatefulProtocol>,
    pub avatar_protocol_cache: HashMap<(uuid::Uuid, u32), StatefulProtocol>,

    // --- Mention fields ---
    pub mention_suggestions: Vec<usize>, // store indices into channel_userlist
    pub mention_selected: usize,
    pub mention_prefix: Option<String>,

    // --- Quit confirmation fields ---
    pub show_quit_confirm: bool,
    pub quit_confirm_selected: usize,

    // --- Color picker fields ---
    pub color_picker_selected: usize, // index in color palette

    // --- Server list fields ---
    pub servers: Vec<common::Server>,

    // --- Channel selection fields ---
    pub selected_server: Option<usize>,
    pub selected_channel: Option<usize>,

    // --- Chat scrolling fields ---
    pub chat_scroll_offset: usize, // how many lines up from the latest message
    pub last_chat_rows: Option<usize>, // number of visible chat rows from last render

    // --- Per-channel user list ---
    pub channel_userlist: Vec<User>,

    // --- Per-channel history complete flag ---
    pub channel_history_complete: HashMap<Uuid, bool>,

    // --- Server actions fields ---
    pub show_server_actions: bool, // Show server actions popup
    pub server_actions_selected: usize, // Selected action in server actions popup

    // --- Pending new thread fields ---
    pub pending_new_thread_title: Option<String>, // Title of the new thread pending selection

    // --- DM fields ---
    pub dm_user_list: Vec<User>, // Users you have DMs with
    pub selected_dm_user: Option<usize>, // Index into dm_user_list
    pub dm_messages: Vec<common::DirectMessage>, // Current DM conversation
    pub dm_history_complete: bool, // If all history loaded

    // --- Notification fields ---
    pub notifications: Vec<common::Notification>,
    pub notification_history_complete: bool,

    // --- New fields for tabbed sidebar and unread logic ---
    pub sidebar_tab: SidebarTab,
    pub unread_channels: HashSet<uuid::Uuid>, // channel_id
    pub unread_dm_conversations: HashSet<uuid::Uuid>, // user_id

    // --- Chat input drafts ---
    pub chat_input_drafts: HashMap<ChatTarget, String>,
    pub current_chat_target: Option<ChatTarget>,
}

impl<'a> App<'a> {
    pub fn new(to_server: mpsc::UnboundedSender<ClientMessage>, sound_manager: &'a SoundManager) -> App<'a> {
        // --- CORRECTED: Use the new Picker API ---
        let picker = Picker::from_query_stdio().unwrap_or_else(|e| {
            eprintln!(
                "Failed to query terminal for graphics support: {}. Falling back to ASCII picker.",
                e
            );
            Picker::from_fontsize((16, 16))
        });

        App {
            mode: AppMode::Login,
            input_mode: Some(InputMode::LoginUsername),
            current_input: String::new(),
            password_input: String::new(),
            notification: None,
            current_user: None,
            main_menu_state: ListState::default(),
            forum_list_state: ListState::default(),
            thread_list_state: ListState::default(),
            settings_list_state: ListState::default(),
            user_list_state: ListState::default(),
            forums: vec![],
            current_forum_id: None,
            current_thread_id: None,
            chat_messages: vec![],
            tick_count: 0,
            should_quit: false,
            to_server,
            sound_manager,
            show_user_list: true,
            connected_users: vec![],
            chat_focus: ChatFocus::Messages,
            dm_input: String::new(),
            dm_target: None,
            show_user_actions: false,
            user_actions_selected: 0,
            user_actions_target: None,
            edit_bio: String::new(),
            edit_url1: String::new(),
            edit_url2: String::new(),
            edit_url3: String::new(),
            edit_location: String::new(),
            edit_profile_pic: String::new(),
            edit_cover_banner: String::new(),
            profile_edit_error: None,
            show_profile_view_popup: false,
            profile_edit_focus: ProfileEditFocus::Bio,
            profile_requested_by_user: false,
            picker,
            profile_view: None,
            profile_image_state: None,
            profile_banner_image_state: None,
            avatar_protocol_cache: HashMap::new(),
            mention_suggestions: vec![],
            mention_selected: 0,
            mention_prefix: None,
            show_quit_confirm: false,
            quit_confirm_selected: 0,
            color_picker_selected: 0,
            servers: vec![],
            selected_server: None,
            selected_channel: None,
            chat_scroll_offset: 0,
            last_chat_rows: None,
            channel_userlist: vec![],
            channel_history_complete: HashMap::new(),
            show_server_actions: false,
            server_actions_selected: 0,
            pending_new_thread_title: None,
            dm_user_list: vec![],
            selected_dm_user: None,
            dm_messages: vec![],
            dm_history_complete: false,
            notifications: vec![],
            notification_history_complete: false,
            sidebar_tab: SidebarTab::Servers,
            unread_channels: HashSet::new(),
            unread_dm_conversations: HashSet::new(),
            chat_input_drafts: HashMap::new(),
            current_chat_target: None,
        }
    }

    pub fn set_notification(&mut self, message: impl Into<String>, ms: Option<u64>, minimal: bool) {
        let msg = message.into();
        if msg.to_lowercase().contains("error") {
            self.sound_manager.play(SoundType::Error);
        }
        let close_tick = ms.map(|ms| self.tick_count + (ms / 100));
        self.notification = Some((msg, close_tick, minimal));
    }

    pub fn send_to_server(&mut self, msg: ClientMessage) {
        if let Err(e) = self.to_server.send(msg) {
            self.set_notification(format!("Connection Error: {}", e), None, true);
        }
    }

    pub fn set_profile_for_viewing(&mut self, profile: UserProfile) {
        fn decode_image_bytes(val: &Option<String>) -> Option<Vec<u8>> {
            if let Some(s) = val {
                if s.starts_with("http") {
                    None // Not handling URLs here
                } else {
                    let b64 = if let Some(idx) = s.find(",") {
                        &s[idx+1..]
                    } else {
                        s.as_str()
                    };
                    base64::engine::general_purpose::STANDARD.decode(b64).ok()
                }
            } else {
                None
            }
        }
        let font_size = self.picker.font_size();
        let banner_px_w = 70 * font_size.0;
        let banner_px_h = 7 * font_size.1;
        let banner_size = (banner_px_w as u32, banner_px_h as u32);
        let pfp_size = (32, 32);
        let pfp_padding_left = 16;
        let banner_bytes = decode_image_bytes(&profile.cover_banner)
            .or_else(|| Some(vec![0u8; (banner_size.0 * banner_size.1 * 4) as usize]));
        let pfp_bytes = decode_image_bytes(&profile.profile_pic)
            .or_else(|| Some(vec![0u8; (pfp_size.0 * pfp_size.1 * 4) as usize]));
        // Only show composite if at least one is not fully transparent
        let show_composite = profile.cover_banner.is_some() || profile.profile_pic.is_some();
        if show_composite {
            let composited = Self::composite_banner_and_pfp(
                &banner_bytes.unwrap(),
                &pfp_bytes.unwrap(),
                banner_size,
                pfp_size,
                pfp_padding_left,
            );
            if let Some(composite_bytes) = composited {
                if let Ok(dynamic_image) = image::load_from_memory(&composite_bytes) {
                    self.profile_banner_image_state = Some(self.picker.new_resize_protocol(dynamic_image));
                } else {
                    self.profile_banner_image_state = None;
                }
            } else {
                self.profile_banner_image_state = None;
            }
            self.profile_image_state = None; // Only render the composited image
        } else {
            self.profile_banner_image_state = None;
            self.profile_image_state = None;
        }
        self.profile_view = Some(profile);
        self.show_profile_view_popup = true;
    }
    
    pub fn handle_server_message(&mut self, msg: ServerMessage) {
        match msg {
            ServerMessage::AuthSuccess(user) => {
                self.current_user = Some(user);
                self.mode = AppMode::MainMenu;
                self.input_mode = None;
                self.current_input.clear();
                self.password_input.clear();
                self.main_menu_state.select(Some(0));
                self.sound_manager.play(SoundType::LoginSuccess);
            }
            ServerMessage::AuthFailure(reason) => {
                self.set_notification(format!("Error: {}", reason), None, false);
                self.sound_manager.play(SoundType::LoginFailure);
            }
            ServerMessage::Forums(forums) => {
                self.forums = forums;
                // --- UX: If a new thread was just created, select and enter it ---
                if let (Some(forum_id), Some(ref title)) = (self.current_forum_id, &self.pending_new_thread_title) {
                    if let Some(forum) = self.forums.iter().find(|f| f.id == forum_id) {
                        if let Some((idx, thread)) = forum.threads.iter().enumerate().find(|(_, t)| t.title == *title) {
                            self.thread_list_state.select(Some(idx));
                            self.current_thread_id = Some(thread.id);
                            self.mode = AppMode::PostView;
                            self.pending_new_thread_title = None;
                        }
                    }
                }
            }
            ServerMessage::NewChatMessage(msg) => {
                // Insert the message into the correct channel's message list
                if let (Some(s), Some(c)) = (self.selected_server, self.selected_channel) {
                    if let Some(server) = self.servers.get_mut(s) {
                        if let Some(channel) = server.channels.get_mut(c) {
                            // For local echo, just add a ChatMessage (not ChannelMessage)
                            self.chat_messages.push(common::ChatMessage {
                                author: msg.author.clone(),
                                content: msg.content.clone(),
                                color: msg.color,
                            });
                            self.sound_manager.play(SoundType::ReceiveChannelMessage);
                        }
                    }
                }
            },
            ServerMessage::NewChannelMessage(msg) => {
                let mut is_current = false;
                if let (Some(s), Some(c)) = (self.selected_server, self.selected_channel) {
                    if let Some(server) = self.servers.get(s) {
                        if let Some(channel) = server.channels.get(c) {
                            if channel.id == msg.channel_id {
                                is_current = true;
                            }
                        }
                    }
                }
                if !is_current {
                    self.unread_channels.insert(msg.channel_id);
                }
                for (si, server) in self.servers.iter_mut().enumerate() {
                    // Compute selected channel id for this server index
                    let is_selected = if let (Some(s), Some(c)) = (self.selected_server, self.selected_channel) {
                        if si == s {
                            server.channels.get(c).map(|ch| ch.id)
                        } else { None }
                    } else { None };
                    if let Some(channel) = server.channels.iter_mut().find(|c| c.id == msg.channel_id) {
                        channel.messages.push(msg.clone());
                        if is_selected == Some(msg.channel_id) {
                            self.chat_messages.push(common::ChatMessage {
                                author: msg.author_username.clone(),
                                content: msg.content.clone(),
                                color: msg.author_color,
                            });
                        }
                    }
                }
            }
            ServerMessage::ChannelMessages { channel_id, messages, history_complete } => {
                self.unread_channels.remove(&channel_id);
                self.channel_history_complete.insert(channel_id, history_complete);
                for (si, server) in self.servers.iter_mut().enumerate() {
                    let is_selected = if let (Some(s), Some(c)) = (self.selected_server, self.selected_channel) {
                        if si == s {
                            server.channels.get(c).map(|ch| ch.id)
                        } else { None }
                    } else { None };
                    if let Some(channel) = server.channels.iter_mut().find(|c| c.id == channel_id) {
                        if channel.messages.is_empty() {
                            // Initial load: just set messages and scroll to bottom
                            channel.messages = messages.clone();
                            if is_selected == Some(channel_id) {
                                self.chat_messages = messages.iter().map(|m| common::ChatMessage {
                                    author: m.author_username.clone(),
                                    content: m.content.clone(),
                                    color: m.author_color,
                                }).collect();
                                self.chat_scroll_offset = 0; // Always start at the bottom
                            }
                        } else if !messages.is_empty() && channel.messages.first().unwrap().id != messages.first().unwrap().id {
                            // Scrollback: prepend older messages
                            let mut new_msgs = messages.clone();
                            let added = new_msgs.len();
                            new_msgs.append(&mut channel.messages);
                            channel.messages = new_msgs;
                            if is_selected == Some(channel_id) {
                                self.chat_scroll_offset += added;
                            }
                        } else {
                            // Replace (e.g. channel switch)
                            channel.messages = messages.clone();
                            if is_selected == Some(channel_id) {
                                self.chat_messages = messages.iter().map(|m| common::ChatMessage {
                                    author: m.author_username.clone(),
                                    content: m.content.clone(),
                                    color: m.author_color,
                                }).collect();
                                self.chat_scroll_offset = 0;
                            }
                        }
                        break;
                    }
                }
            }
            ServerMessage::Notification(text, is_error) => {
                let prefix = if is_error { "Error: " } else { "Info: " };
                self.set_notification(format!("{}{}", prefix, text), Some(2000), false);
            }
            ServerMessage::UserList(users) => {
                self.connected_users = users;
            }
            ServerMessage::UserJoined(user) => {
                // Add or update user in channel_userlist
                if let Some(existing) = self.channel_userlist.iter_mut().find(|u| u.id == user.id) {
                    *existing = user;
                } else {
                    self.channel_userlist.push(user);
                }
            }
            ServerMessage::UserLeft(user_id) => {
                // Mark user as offline in channel_userlist
                if let Some(existing) = self.channel_userlist.iter_mut().find(|u| u.id == user_id) {
                    existing.status = common::UserStatus::Offline;
                }
            }
            ServerMessage::DirectMessage(dm) => {
                let is_current = if let Some(ChatTarget::DM { user_id }) = self.current_chat_target {
                    user_id == dm.from && self.sidebar_tab == SidebarTab::DMs
                } else { false };
                if is_current {
                    self.dm_messages.push(dm);
                    self.chat_scroll_offset = 0;
                } else {
                    self.unread_dm_conversations.insert(dm.from);
                    self.set_notification(
                        format!("DM from {}: {}", dm.author_username, dm.content),
                        Some(4000),
                        true,
                    );
                    self.sound_manager.play(SoundType::DirectMessage);
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
            ServerMessage::Profile(profile) => {
                if self.profile_requested_by_user {
                    self.set_profile_for_viewing(profile);
                } else {
                    self.edit_bio = profile.bio.unwrap_or_default();
                    self.edit_url1 = profile.url1.unwrap_or_default();
                    self.edit_url2 = profile.url2.unwrap_or_default();
                    self.edit_url3 = profile.url3.unwrap_or_default();
                    self.edit_location = profile.location.unwrap_or_default();
                    self.edit_profile_pic = profile.profile_pic.unwrap_or_default();
                    self.edit_cover_banner = profile.cover_banner.unwrap_or_default();
                }
                self.profile_requested_by_user = false;
            }
            ServerMessage::UserUpdated(user) => {
                // Update the user in channel_userlist if present
                if let Some(existing) = self.channel_userlist.iter_mut().find(|u| u.id == user.id) {
                    *existing = user.clone();
                }
                // Also update current_user if it's this user
                if let Some(current) = &mut self.current_user {
                    if current.id == user.id {
                        *current = user.clone();
                    }
                }
                // Update chat messages' author info if present
                for msg in &mut self.chat_messages {
                    if msg.author == user.username {
                        msg.color = user.color;
                        // If you add avatar/profile_pic to ChatMessage, update here too
                    }
                }
                // Invalidate avatar cache for this user (all sizes)
                self.avatar_protocol_cache.retain(|(uid, _), _| *uid != user.id);
            }
            ServerMessage::Servers(servers) => {
                self.servers = servers;
                // Default selection: first server and first channel
                let mut should_fetch_channel = false;
                if self.selected_server.is_none() && !self.servers.is_empty() {
                    self.selected_server = Some(0);
                    if !self.servers[0].channels.is_empty() {
                        self.selected_channel = Some(0);
                        self.chat_messages.clear(); // Do not expect pre-populated messages
                        // Only fetch if we're in Chat mode
                        if self.mode == AppMode::Chat {
                            should_fetch_channel = true;
                        }
                    }
                } else if self.mode == AppMode::Chat && self.selected_server.is_some() && self.selected_channel.is_some() {
                    // If already selected, but just entered chat mode, fetch
                    should_fetch_channel = true;
                }
                // Always fetch userlist and messages for selected channel if needed
                if should_fetch_channel {
                    if let (Some(s), Some(c)) = (self.selected_server, self.selected_channel) {
                        let channel_id = self.servers.get(s)
                            .and_then(|server| server.channels.get(c))
                            .map(|channel| channel.id);
                        if let Some(channel_id) = channel_id {
                            self.send_to_server(ClientMessage::GetChannelUserList { channel_id });
                            self.send_to_server(ClientMessage::GetChannelMessages { channel_id, before: None });
                        }
                    }
                }
            }
            ServerMessage::ChannelUserList { channel_id, users } => {
                let mut sorted_users = users;
                sorted_users.sort_by(|a, b| a.username.to_lowercase().cmp(&b.username.to_lowercase()));
                sorted_users.reverse();
                self.channel_userlist = sorted_users;
                // Reset user list selection to first user if list is not empty
                if !self.channel_userlist.is_empty() {
                    self.user_list_state.select(Some(0));
                } else {
                    self.user_list_state.select(None);
                }
            }
            ServerMessage::DMUserList(users) => {
                self.dm_user_list = users;
                if self.selected_dm_user.is_none() && !self.dm_user_list.is_empty() {
                    self.selected_dm_user = Some(0);
                }
            }
            ServerMessage::DirectMessages { user_id, messages, history_complete } => {
                // --- DM scrollback logic: prepend and adjust scroll offset ---
                if let Some(idx) = self.dm_user_list.iter().position(|u| u.id == user_id) {
                    if self.dm_messages.is_empty() {
                        // Initial load: just set messages and scroll to bottom
                        self.dm_messages = messages.clone();
                        self.dm_history_complete = history_complete;
                        self.chat_scroll_offset = 0;
                    } else if !messages.is_empty() && self.dm_messages.first().map(|m| m.id) != messages.first().map(|m| m.id) {
                        // Scrollback: prepend only truly new messages (deduplicate)
                        let first_existing_id = self.dm_messages.first().map(|m| m.id);
                        let mut new_count = 0;
                        for msg in messages.iter().rev() {
                            if Some(msg.id) == first_existing_id {
                                break;
                            }
                            new_count += 1;
                        }
                        let mut unique_new_msgs = Vec::new();
                        for msg in messages.iter().take(new_count) {
                            // Only add if not already present (shouldn't be, but extra safety)
                            if !self.dm_messages.iter().any(|m| m.id == msg.id) {
                                unique_new_msgs.push(msg.clone());
                            }
                        }
                        // Prepend unique new messages
                        if !unique_new_msgs.is_empty() {
                            let added = unique_new_msgs.len();
                            let mut new_msgs = unique_new_msgs;
                            new_msgs.append(&mut self.dm_messages);
                            self.dm_messages = new_msgs;
                            self.dm_history_complete = history_complete;
                            self.chat_scroll_offset += added;
                            // Clamp scroll offset so you can't scroll past the top
                            let total_msgs = self.dm_messages.len();
                            let max_rows = self.last_chat_rows.unwrap_or(20);
                            let max_scroll = total_msgs.saturating_sub(max_rows);
                            if self.chat_scroll_offset > max_scroll {
                                self.chat_scroll_offset = max_scroll;
                            }
                        } else {
                            // No new unique messages, do not change scroll offset
                            self.dm_history_complete = history_complete;
                        }
                    } else {
                        // Replace (e.g. DM switch)
                        self.dm_messages = messages.clone();
                        self.dm_history_complete = history_complete;
                        self.chat_scroll_offset = 0;
                    }
                }
            }
            ServerMessage::Notifications { notifications, history_complete } => {
                self.notifications = notifications;
                self.notification_history_complete = history_complete;
            }
            ServerMessage::NotificationUpdated { notification_id, read } => {
                if let Some(n) = self.notifications.iter_mut().find(|n| n.id == notification_id) {
                    n.read = read;
                }
            }
        }
    }

    pub fn enter_input_mode(&mut self, mode: InputMode) {
        self.input_mode = Some(mode);
        self.mode = AppMode::Input;
        self.current_input.clear();
        self.password_input.clear();
        self.notification = None;
    }

    pub fn on_tick(&mut self) {
        self.tick_count += 1;
        if let Some((_, Some(close_tick), _)) = &self.notification {
            if self.tick_count >= *close_tick {
                self.notification = None;
            }
        }
    }

    pub fn file_or_url_to_base64(val: &str) -> Option<String> {
        let trimmed = val.trim();
        if trimmed.is_empty() {
            return None;
        }
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            return Some(trimmed.to_string());
        }
        if trimmed.len() > 100 && !trimmed.contains('/') && !trimmed.contains(' ') {
            return Some(trimmed.to_string());
        }
        if Path::new(trimmed).exists() {
            match fs::read(trimmed) {
                Ok(bytes) => return Some(base64::engine::general_purpose::STANDARD.encode(bytes)),
                Err(e) => {
                    eprintln!("ERROR: Failed to read file: {e}");
                    return None;
                }
            }
        } else {
            Some(trimmed.to_string())
        }
    }

    /// Composite the banner and profile picture images in memory, overlaying the PFP on the banner.
    /// banner_size: (width, height) in pixels for the banner
    /// pfp_size: (width, height) in pixels for the PFP (should be 32x32)
    /// pfp_padding_left: left padding in pixels
    pub fn composite_banner_and_pfp(
        banner_bytes: &[u8],
        pfp_bytes: &[u8],
        banner_size: (u32, u32),
        pfp_size: (u32, u32),
        pfp_padding_left: u32,
    ) -> Option<Vec<u8>> {
        use image::{DynamicImage, ImageFormat, Rgba, imageops};
        // If banner_bytes is all zero (fully transparent), create a transparent image
        let mut banner_img = if banner_bytes.iter().all(|&b| b == 0) {
            image::RgbaImage::from_pixel(banner_size.0, banner_size.1, Rgba([0, 0, 0, 0]))
        } else {
            let img = image::load_from_memory(banner_bytes).ok()?;
            img.resize_exact(banner_size.0, banner_size.1, imageops::FilterType::Lanczos3).to_rgba8()
        };
        // Add a subtle black gradient to transparent left to right
        for y in 0..banner_size.1 {
            for x in 0..banner_size.0 {
                let px = banner_img.get_pixel_mut(x, y);
                let alpha = (x as f32 / banner_size.0 as f32 * 255.0) as u8;
                *px = Rgba([px[0], px[1], px[2], alpha]);
            }
        }
        // Resize PFP
        let mut pfp_img = if pfp_bytes.iter().all(|&b| b == 0) {
            image::RgbaImage::from_pixel(pfp_size.0, pfp_size.1, Rgba([0, 0, 0, 0]))
        } else {
            let img = image::load_from_memory(pfp_bytes).ok()?;
            img.resize_exact(pfp_size.0, pfp_size.1, imageops::FilterType::Lanczos3).to_rgba8()
        };
        // Apply circular mask to PFP
        let radius = pfp_size.0.min(pfp_size.1) as f32 / 2.0;
        let center = (pfp_size.0 as f32 / 2.0, pfp_size.1 as f32 / 2.0);
        for y in 0..pfp_size.1 {
            for x in 0..pfp_size.0 {
                let dx = x as f32 + 0.5 - center.0;
                let dy = y as f32 + 0.5 - center.1;
                if (dx*dx + dy*dy).sqrt() > radius {
                    let px = pfp_img.get_pixel_mut(x, y);
                    *px = Rgba([0, 0, 0, 0]);
                }
            }
        }
        // Vertically center PFP on banner
        let pfp_y = (banner_size.1.saturating_sub(pfp_size.1)) / 2;
        imageops::overlay(&mut banner_img, &pfp_img, pfp_padding_left.into(), pfp_y.into());
        let mut out_buf = Vec::new();
        DynamicImage::ImageRgba8(banner_img)
            .write_to(&mut Cursor::new(&mut out_buf), ImageFormat::Png)
            .ok()?;
        Some(out_buf)
    }

    pub fn update_profile_banner_composite(&mut self, banner_area_width_cells: u16, banner_area_height_cells: u16) {
        if let Some(profile) = self.profile_view.as_ref() {
            fn decode_image_bytes(val: &Option<String>) -> Option<Vec<u8>> {
                if let Some(s) = val {
                    if s.starts_with("http") {
                        None
                    } else {
                        let b64 = if let Some(idx) = s.find(",") {
                            &s[idx+1..]
                        } else {
                            s.as_str()
                        };
                        base64::engine::general_purpose::STANDARD.decode(b64).ok()
                    }
                } else {
                    None
                }
            }
            let font_size = self.picker.font_size();
            let banner_px_w = banner_area_width_cells as u32 * font_size.0 as u32;
            let banner_px_h = banner_area_height_cells as u32 * font_size.1 as u32;
            let banner_size = (banner_px_w, banner_px_h);
            let pfp_size = (64, 64);
            let pfp_padding_left = 16;
            let banner_bytes = decode_image_bytes(&profile.cover_banner)
                .or_else(|| Some(vec![0u8; (banner_size.0 * banner_size.1 * 4) as usize]));
            let pfp_bytes = decode_image_bytes(&profile.profile_pic)
                .or_else(|| Some(vec![0u8; (pfp_size.0 * pfp_size.1 * 4) as usize]));
            let show_composite = profile.cover_banner.is_some() || profile.profile_pic.is_some();
            if show_composite {
                let composited = Self::composite_banner_and_pfp(
                    &banner_bytes.unwrap(),
                    &pfp_bytes.unwrap(),
                    banner_size,
                    pfp_size,
                    pfp_padding_left,
                );
                if let Some(composite_bytes) = composited {
                    if let Ok(dynamic_image) = image::load_from_memory(&composite_bytes) {
                        self.profile_banner_image_state = Some(self.picker.new_resize_protocol(dynamic_image));
                    } else {
                        self.profile_banner_image_state = None;
                    }
                } else {
                    self.profile_banner_image_state = None;
                }
                self.profile_image_state = None;
            } else {
                self.profile_banner_image_state = None;
                self.profile_image_state = None;
            }
        }
    }

    // --- DM UI stubs ---
    pub fn draw_dm_user_list(f: &mut ratatui::Frame, app: &mut App, area: ratatui::layout::Rect) {
        use ratatui::widgets::{Block, Borders, List, ListItem};
        let items: Vec<ListItem> = app.dm_user_list.iter().map(|u| {
            let status = match u.status {
                common::UserStatus::Connected => "●",
                _ => "○",
            };
            ListItem::new(format!("{} {}", status, u.username))
        }).collect();
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Direct Messages"))
            .highlight_symbol(">> ");
        f.render_stateful_widget(list, area, &mut app.user_list_state.clone());
    }

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
    // --- Shared chat message list logic for channel and DM ---
    pub fn get_current_message_list(&self) -> Vec<crate::model::ChatMessageWithMeta> {
        match &self.current_chat_target {
            Some(ChatTarget::Channel { server_id, channel_id }) => {
                for server in &self.servers {
                    if &server.id == server_id {
                        for channel in &server.channels {
                            if &channel.id == channel_id {
                                return channel.messages.iter().map(|msg| crate::model::ChatMessageWithMeta {
                                    author: msg.author_username.clone(),
                                    content: msg.content.clone(),
                                    color: msg.author_color,
                                    profile_pic: msg.author_profile_pic.clone(),
                                    timestamp: Some(msg.timestamp),
                                }).collect();
                            }
                        }
                    }
                }
                vec![]
            }
            Some(ChatTarget::DM { user_id }) => {
                self.dm_messages.iter().filter(|msg| msg.from == *user_id || msg.to == *user_id).map(|msg| {
                    let (author, color, profile_pic) = if msg.from == *user_id {
                        // Find user in dm_user_list
                        if let Some(user) = self.dm_user_list.iter().find(|u| u.id == msg.from) {
                            (user.username.clone(), user.color, user.profile_pic.clone())
                        } else {
                            ("?".to_string(), ratatui::style::Color::Gray, None)
                        }
                    } else {
                        // Current user
                        let username = self.current_user.as_ref().map(|u| u.username.clone()).unwrap_or("me".to_string());
                        let color = self.current_user.as_ref().map(|u| u.color).unwrap_or(ratatui::style::Color::Cyan);
                        let profile_pic = self.current_user.as_ref().and_then(|u| u.profile_pic.clone());
                        (username, color, profile_pic)
                    };
                    crate::model::ChatMessageWithMeta {
                        author,
                        content: msg.content.clone(),
                        color,
                        profile_pic,
                        timestamp: Some(msg.timestamp),
                    }
                }).collect()
            }
            None => vec![]
        }
    }
}