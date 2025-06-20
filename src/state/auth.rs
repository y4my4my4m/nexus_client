use common::User;
use ratatui::widgets::ListState;

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

/// State management for authentication
pub struct AuthState {
    pub current_user: Option<User>,
    pub current_input: String,
    pub password_input: String,
    pub input_mode: Option<InputMode>,
}

impl Default for AuthState {
    fn default() -> Self {
        Self {
            current_user: None,
            current_input: String::new(),
            password_input: String::new(),
            input_mode: Some(InputMode::LoginUsername),
        }
    }
}

impl AuthState {
    pub fn is_logged_in(&self) -> bool {
        self.current_user.is_some()
    }
    
    pub fn login(&mut self, user: User) {
        self.current_user = Some(user);
        self.clear_inputs();
    }
    
    pub fn logout(&mut self) {
        self.current_user = None;
        self.clear_inputs();
        self.input_mode = Some(InputMode::LoginUsername);
    }
    
    pub fn clear_inputs(&mut self) {
        self.current_input.clear();
        self.password_input.clear();
    }
    
    pub fn set_input_mode(&mut self, mode: InputMode) {
        self.input_mode = Some(mode);
        self.clear_inputs();
    }
}