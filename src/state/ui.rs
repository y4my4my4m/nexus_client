use ratatui::widgets::ListState;
use uuid::Uuid;

#[derive(PartialEq, Debug, Clone)]
pub enum AppMode {
    Login, 
    Register, 
    MainMenu, 
    Settings, 
    ForumList, 
    ThreadList, 
    PostView, 
    Chat, 
    Input, 
    EditProfile, 
    ColorPicker, 
    Preferences,
}

/// State management for UI-specific state
pub struct UiState {
    pub mode: AppMode,
    pub should_quit: bool,
    pub tick_count: u64,
    
    // List states for various UI components
    pub main_menu_state: ListState,
    pub settings_list_state: ListState,
    
    // Color picker
    pub color_picker_selected: usize,
    
    // Preferences navigation
    pub preferences_selected: usize,
    
    // Server actions
    pub show_server_actions: bool,
    pub server_actions_selected: usize,
    
    // Server invites
    pub show_server_invite_selection: bool,
    pub server_invite_selected: usize,
    pub server_invite_target_user: Option<Uuid>,
    
    // Quit confirmation
    pub show_quit_confirm: bool,
    pub quit_confirm_selected: usize,
    
    // Server error popup
    pub show_server_error: bool,
    pub server_error_message: String,
    
    // Connected users (for legacy compatibility)
    pub connected_users: Vec<common::User>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            mode: AppMode::Login,
            should_quit: false,
            tick_count: 0,
            main_menu_state: ListState::default(),
            settings_list_state: ListState::default(),
            color_picker_selected: 0,
            preferences_selected: 0,
            show_server_actions: false,
            server_actions_selected: 0,
            show_server_invite_selection: false,
            server_invite_selected: 0,
            server_invite_target_user: None,
            show_quit_confirm: false,
            quit_confirm_selected: 0,
            show_server_error: false,
            server_error_message: String::new(),
            connected_users: Vec::new(),
        }
    }
}

impl UiState {
    pub fn set_mode(&mut self, mode: AppMode) {
        self.mode = mode;
    }
    
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
    
    pub fn tick(&mut self) {
        self.tick_count += 1;
    }
    
    pub fn reset_selections(&mut self) {
        self.main_menu_state.select(Some(0));
        self.settings_list_state.select(Some(0));
    }
    
    pub fn show_server_error(&mut self, message: String) {
        self.show_server_error = true;
        self.server_error_message = message;
    }
    
    pub fn hide_server_error(&mut self) {
        self.show_server_error = false;
        self.server_error_message.clear();
    }
}