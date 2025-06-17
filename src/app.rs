// client/src/app.rs

use common::{ChatMessage, ClientMessage, Forum, ServerMessage, User};
use crate::sound::{SoundManager, SoundType};
use ratatui::widgets::ListState;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(PartialEq, Debug, Clone)]
pub enum AppMode {
    Login, Register, MainMenu, Settings, ForumList, ThreadList, PostView, Chat, Input,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    // Auth Screen Focus States
    LoginUsername,
    LoginPassword,
    RegisterUsername,
    RegisterPassword,
    AuthSubmit, // Focus on the main [LOGIN] or [REGISTER] button
    AuthSwitch, // Focus on the "Switch to..." button

    // Generic Input Popups
    NewThreadTitle,
    NewThreadContent,
    NewPostContent,
    UpdatePassword,
}

pub struct App<'a> {
    pub mode: AppMode,
    pub input_mode: Option<InputMode>,
    pub current_input: String,
    pub password_input: String,
    pub notification: Option<(String, Option<u64>, bool)>, // (message, Some(tick_to_close), minimal)
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
    // --- User list toggle and data ---
    pub show_user_list: bool,
    pub connected_users: Vec<User>,
}

impl<'a> App<'a> {
    pub fn new(to_server: mpsc::UnboundedSender<ClientMessage>, sound_manager: &'a SoundManager) -> App<'a> {
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
            show_user_list: false,
            connected_users: vec![],
        }
    }

    /// Set a notification with optional auto-close in ms (if ms is Some) and minimal flag
    pub fn set_notification(&mut self, message: impl Into<String>, ms: Option<u64>, minimal: bool) {
        let msg = message.into();
        // Play error or notify sound
        if msg.to_lowercase().contains("error") {
            self.sound_manager.play(SoundType::Error);
        } else {
            self.sound_manager.play(SoundType::Notify);
        }
        let close_tick = ms.map(|ms| self.tick_count + (ms / 100)); // 100ms per tick
        self.notification = Some((msg, close_tick, minimal));
    }

    pub fn send_to_server(&mut self, msg: ClientMessage) {
        if let Err(e) = self.to_server.send(msg) {
            self.set_notification(format!("Connection Error: {}", e), None, true);
        }
    }
    
    pub fn handle_server_message(&mut self, msg: ServerMessage) {
        match msg {
            ServerMessage::AuthSuccess(user) => {
                self.set_notification("AuthSuccess received!", Some(1500), false);
                self.current_user = Some(user);
                self.mode = AppMode::MainMenu;
                self.input_mode = None;
                self.current_input.clear();
                self.password_input.clear();
                self.main_menu_state.select(Some(0));
            }
            ServerMessage::AuthFailure(reason) => {
                self.set_notification(format!("Error: {}", reason), None, false);
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
            },
            ServerMessage::UserJoined(user) => {
                if !self.connected_users.iter().any(|u| u.id == user.id) {
                    self.connected_users.push(user);
                }
            },
            ServerMessage::UserLeft(user_id) => {
                self.connected_users.retain(|u| u.id != user_id);
            },
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
        // Auto-close notification if needed
        if let Some((_, Some(close_tick), _)) = &self.notification {
            if self.tick_count >= *close_tick {
                self.notification = None;
            }
        }
    }
}