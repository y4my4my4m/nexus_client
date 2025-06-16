use common::{ChatMessage, ClientMessage, Forum, ServerMessage};
use ratatui::widgets::ListState;
use tokio::sync::mpsc;

#[derive(PartialEq, Debug)]
pub enum AppMode {
    MainMenu,
    ForumList,
    ThreadList,
    PostView,
    Chat,
}

pub struct App {
    pub username: String,
    pub mode: AppMode,
    pub main_menu_state: ListState,
    pub forum_list_state: ListState,
    pub thread_list_state: ListState,
    pub forums: Vec<Forum>,
    pub current_forum_index: Option<usize>,
    pub current_thread_index: Option<usize>,
    pub chat_messages: Vec<ChatMessage>,
    pub chat_input: String,
    pub tick_count: u64,
    pub should_quit: bool,
    pub to_server: mpsc::UnboundedSender<ClientMessage>,
}

impl App {
    pub fn new(username: String, to_server: mpsc::UnboundedSender<ClientMessage>) -> App {
        let mut main_menu_state = ListState::default();
        main_menu_state.select(Some(0));
        let mut forum_list_state = ListState::default();
        forum_list_state.select(Some(0));

        App {
            username, // <-- INITIALIZE IT
            mode: AppMode::MainMenu,
            main_menu_state,
            forum_list_state,
            thread_list_state: ListState::default(),
            forums: vec![],
            current_forum_index: None,
            current_thread_index: None,
            chat_messages: vec![ChatMessage {
                author: "SYSTEM".to_string(),
                content: "Connecting to the Nexus...".to_string(),
                color: ratatui::style::Color::Yellow,
            }],
            chat_input: String::new(),
            tick_count: 0,
            should_quit: false,
            to_server,
        }
    }

    pub fn on_tick(&mut self) {
        self.tick_count += 1;
    }

    pub fn handle_server_message(&mut self, msg: ServerMessage) {
        match msg {
            ServerMessage::Forums(forums) => {
                self.forums = forums;
            }
            ServerMessage::NewChatMessage(chat_message) => {
                self.chat_messages.push(chat_message);
                if self.chat_messages.len() > 100 {
                    self.chat_messages.remove(0);
                }
            }
        }
    }

    // Notice the &mut self
    pub fn send_to_server(&mut self, msg: ClientMessage) {
        if let Err(e) = self.to_server.send(msg) {
            self.chat_messages.push(ChatMessage {
                author: "CLIENT_ERROR".to_string(),
                content: format!("Failed to send message to server: {}", e),
                color: ratatui::style::Color::Red,
            });
        }
    }
}