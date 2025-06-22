use common::{User, UserProfile};
use uuid::Uuid;
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use std::collections::HashMap;

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

/// State management for user profile functionality
pub struct ProfileState {
    // Profile editing
    pub edit_bio: String,
    pub edit_url1: String,
    pub edit_url2: String,
    pub edit_url3: String,
    pub edit_location: String,
    pub edit_profile_pic: String,
    pub edit_cover_banner: String,
    pub profile_edit_focus: ProfileEditFocus,
    pub profile_edit_error: Option<String>,
    pub profile_requested_by_user: bool,
    
    // Profile viewing
    pub profile_view: Option<UserProfile>,
    pub show_profile_view_popup: bool,
    
    // Image rendering
    pub picker: Picker,
    pub profile_image_state: Option<Box<dyn StatefulProtocol>>,
    pub profile_banner_image_state: Option<Box<dyn StatefulProtocol>>,
    pub avatar_protocol_cache: HashMap<(Uuid, u32), Box<dyn StatefulProtocol>>,
    
    // User actions
    pub show_user_actions: bool,
    pub user_actions_selected: usize,
    pub user_actions_target: Option<usize>,
}

impl ProfileState {
    pub fn new() -> Self {
        let picker = Picker::from_termios().unwrap_or_else(|_| {
            Picker::new((16, 16))
        });

        Self {
            edit_bio: String::new(),
            edit_url1: String::new(),
            edit_url2: String::new(),
            edit_url3: String::new(),
            edit_location: String::new(),
            edit_profile_pic: String::new(),
            edit_cover_banner: String::new(),
            profile_edit_focus: ProfileEditFocus::Bio,
            profile_edit_error: None,
            profile_requested_by_user: false,
            profile_view: None,
            show_profile_view_popup: false,
            picker,
            profile_image_state: None,
            profile_banner_image_state: None,
            avatar_protocol_cache: HashMap::new(),
            show_user_actions: false,
            user_actions_selected: 0,
            user_actions_target: None,
        }
    }
    
    pub fn load_profile_for_editing(&mut self, profile: &UserProfile) {
        self.edit_bio = profile.bio.as_deref().unwrap_or("").to_string();
        self.edit_url1 = profile.url1.as_deref().unwrap_or("").to_string();
        self.edit_url2 = profile.url2.as_deref().unwrap_or("").to_string();
        self.edit_url3 = profile.url3.as_deref().unwrap_or("").to_string();
        self.edit_location = profile.location.as_deref().unwrap_or("").to_string();
        self.edit_profile_pic = profile.profile_pic.as_deref().unwrap_or("").to_string();
        self.edit_cover_banner = profile.cover_banner.as_deref().unwrap_or("").to_string();
        self.profile_edit_error = None;
    }
    
    pub fn clear_edit_state(&mut self) {
        self.edit_bio.clear();
        self.edit_url1.clear();
        self.edit_url2.clear();
        self.edit_url3.clear();
        self.edit_location.clear();
        self.edit_profile_pic.clear();
        self.edit_cover_banner.clear();
        self.profile_edit_error = None;
        self.profile_edit_focus = ProfileEditFocus::Bio;
    }
    
    pub fn set_profile_for_viewing(&mut self, profile: UserProfile) {
        self.profile_view = Some(profile);
        self.show_profile_view_popup = true;
    }
    
    pub fn close_profile_view(&mut self) {
        self.show_profile_view_popup = false;
        self.profile_view = None;
    }
    
    pub fn invalidate_avatar_cache(&mut self, user_id: Uuid) {
        self.avatar_protocol_cache.retain(|(uid, _), _| *uid != user_id);
    }
}