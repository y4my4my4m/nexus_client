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

use ratatui_image::{picker::Picker, protocol::StatefulProtocol};

#[derive(PartialEq, Debug, Clone)]
pub enum AppMode {
    Login, Register, MainMenu, Settings, ForumList, ThreadList, PostView, Chat, Input, EditProfile,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileEditFocus {
    Bio,
    Url1,
    Url2,
    Url3,
    Location,
    ProfilePic,
    CoverBanner,
    Save,
    Cancel,
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
        let banner_bytes = decode_image_bytes(&profile.cover_banner);
        let pfp_bytes = decode_image_bytes(&profile.profile_pic);
        if let (Some(banner), Some(pfp)) = (banner_bytes, pfp_bytes) {
            // Remove unused variables banner_cells_w and banner_cells_h
            let font_size = self.picker.font_size();
            // Fallback: use 70 cols x 7 rows as a reasonable default
            let banner_px_w = 70 * font_size.0;
            let banner_px_h = 7 * font_size.1;
            let banner_size = (banner_px_w as u32, banner_px_h as u32);
            let pfp_size = (32, 32);
            let pfp_padding_left = 16;
            let composited = Self::composite_banner_and_pfp(&banner, &pfp, banner_size, pfp_size, pfp_padding_left);
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
            // Fallback: render separately as before
            fn decode_image_field(picker: &Picker, val: &Option<String>) -> Option<StatefulProtocol> {
                if let Some(s) = val {
                    if s.starts_with("http") {
                        None
                    } else {
                        let b64 = if let Some(idx) = s.find(",") {
                            &s[idx+1..]
                        } else {
                            s.as_str()
                        };
                        if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(b64) {
                            if let Ok(dynamic_image) = image::load_from_memory(&bytes) {
                                return Some(picker.new_resize_protocol(dynamic_image));
                            }
                        }
                        None
                    }
                } else {
                    None
                }
            }
            self.profile_image_state = decode_image_field(&self.picker, &profile.profile_pic);
            self.profile_banner_image_state = decode_image_field(&self.picker, &profile.cover_banner);
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
                self.send_to_server(ClientMessage::GetUserList);
            }
            ServerMessage::AuthFailure(reason) => {
                self.set_notification(format!("Error: {}", reason), None, false);
                self.sound_manager.play(SoundType::LoginFailure);
            }
            ServerMessage::Forums(forums) => self.forums = forums,
            ServerMessage::NewChatMessage(msg) => {
                if self.chat_messages.len() > 200 { self.chat_messages.remove(0); }
                self.chat_messages.push(msg)
            },
            ServerMessage::Notification(text, is_error) => {
                let prefix = if is_error { "Error: " } else { "Info: " };
                self.set_notification(format!("{}{}", prefix, text), Some(2000), false);
            }
            ServerMessage::UserList(users) => {
                self.connected_users = users;
            }
            ServerMessage::UserJoined(user) => {
                if !self.connected_users.iter().any(|u| u.id == user.id) {
                    self.connected_users.push(user);
                }
            }
            ServerMessage::UserLeft(user_id) => {
                self.connected_users.retain(|u| u.id != user_id);
            }
            ServerMessage::DirectMessage { from, content } => {
                self.set_notification(
                    format!("DM from {}: {}", from.username, content),
                    Some(4000),
                    true,
                );
                self.sound_manager.play(SoundType::DirectMessage);
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
        // Resize banner
        let banner_img = image::load_from_memory(banner_bytes).ok()?;
        let mut banner_img = banner_img.resize_exact(banner_size.0, banner_size.1, imageops::FilterType::Lanczos3).to_rgba8();
        // Add a subtle black gradient to transparent left to right
        for y in 0..banner_size.1 {
            for x in 0..banner_size.0 {
                let px = banner_img.get_pixel_mut(x, y);
                let alpha = (x as f32 / banner_size.0 as f32 * 255.0) as u8;
                *px = Rgba([px[0], px[1], px[2], alpha]);
            }
        }
        // Resize PFP
        let pfp_img = image::load_from_memory(pfp_bytes).ok()?;
        let mut pfp_img = pfp_img.resize_exact(pfp_size.0, pfp_size.1, imageops::FilterType::Lanczos3).to_rgba8();
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
            if let (Some(banner_str), Some(pfp_str)) = (profile.cover_banner.as_ref(), profile.profile_pic.as_ref()) {
                fn decode_image_bytes(val: &str) -> Option<Vec<u8>> {
                    if val.starts_with("http") {
                        None
                    } else {
                        let b64 = if let Some(idx) = val.find(",") {
                            &val[idx+1..]
                        } else {
                            val
                        };
                        base64::engine::general_purpose::STANDARD.decode(b64).ok()
                    }
                }
                let banner_bytes = decode_image_bytes(banner_str);
                let pfp_bytes = decode_image_bytes(pfp_str);
                if let (Some(banner), Some(pfp)) = (banner_bytes, pfp_bytes) {
                    let font_size = self.picker.font_size();
                    let banner_px_w = banner_area_width_cells as u32 * font_size.0 as u32;
                    let banner_px_h = banner_area_height_cells as u32 * font_size.1 as u32;
                    let banner_size = (banner_px_w, banner_px_h);
                    let pfp_size = (64, 64);
                    let pfp_padding_left = 16;
                    let composited = Self::composite_banner_and_pfp(&banner, &pfp, banner_size, pfp_size, pfp_padding_left);
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
                }
            }
        }
    }
}