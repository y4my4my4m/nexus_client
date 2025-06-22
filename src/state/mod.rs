pub mod chat;
pub mod forum;
pub mod profile;
pub mod auth;
pub mod notification;
pub mod ui;

pub use chat::{ChatState, ChatFocus, SidebarTab, ChatTarget};
pub use forum::ForumState;
pub use profile::{ProfileState, ProfileEditFocus};
pub use auth::{AuthState, InputMode};
pub use notification::NotificationState;
pub use ui::{UiState, AppMode};


/// Configuration constants for the application
pub struct AppConfig {
    pub max_message_length: usize,
    pub scroll_lines_per_page: usize,
    pub notification_timeout_ms: u64,
    pub min_two_column_width: u16,
    pub avatar_pixel_size: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            max_message_length: 500,
            scroll_lines_per_page: 20,
            notification_timeout_ms: 4000,
            min_two_column_width: 110,
            avatar_pixel_size: 32,
        }
    }
}

/// Application error types
#[derive(Debug)]
pub enum AppError {
    Network(String),
    Validation(String),
    IO(std::io::Error),
    Image(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Network(msg) => write!(f, "Network error: {}", msg),
            AppError::Validation(msg) => write!(f, "Validation error: {}", msg),
            AppError::IO(err) => write!(f, "IO error: {}", err),
            AppError::Image(msg) => write!(f, "Image error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

pub type AppResult<T> = Result<T, AppError>;