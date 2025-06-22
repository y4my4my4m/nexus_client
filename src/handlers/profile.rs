use crate::app::App;
use crate::sound::SoundType;
use crossterm::event::{KeyCode, KeyEvent};

/// Handle profile editing input
pub fn handle_profile_edit_input(key: KeyEvent, app: &mut App) {
    use crate::state::ProfileEditFocus::*;
    
    match key.code {
        KeyCode::Tab | KeyCode::Down => {
            app.profile.profile_edit_focus = match app.profile.profile_edit_focus {
                Bio => Location,
                Location => Url1,
                Url1 => Url2,
                Url2 => Url3,
                Url3 => ProfilePic,
                ProfilePic => ProfilePicDelete,
                ProfilePicDelete => CoverBanner,
                CoverBanner => CoverBannerDelete,
                CoverBannerDelete => Save,
                Save => Cancel,
                Cancel => Bio,
            };
        }
        KeyCode::BackTab | KeyCode::Up => {
            app.profile.profile_edit_focus = match app.profile.profile_edit_focus {
                Bio => Cancel,
                Location => Bio,
                Url1 => Location,
                Url2 => Url1,
                Url3 => Url2,
                ProfilePic => Url3,
                ProfilePicDelete => ProfilePic,
                CoverBanner => ProfilePicDelete,
                CoverBannerDelete => CoverBanner,
                Save => CoverBannerDelete,
                Cancel => Save,
            };
        }
        KeyCode::Enter => {
            match app.profile.profile_edit_focus {
                Save => {
                    if let Err(e) = app.save_profile() {
                        app.profile.profile_edit_error = Some(e.to_string());
                        app.sound_manager.play(SoundType::Error);
                    } else {
                        // Don't change mode here - wait for server response
                        // The server will send a notification when the profile is saved
                    }
                }
                Cancel => {
                    app.ui.set_mode(crate::state::AppMode::Settings);
                }
                Bio => {
                    app.profile.edit_bio.push('\n');
                }
                ProfilePicDelete => {
                    app.profile.edit_profile_pic.clear();
                    app.profile.profile_edit_focus = ProfilePic;
                }
                CoverBannerDelete => {
                    app.profile.edit_cover_banner.clear();
                    app.profile.profile_edit_focus = CoverBanner;
                }
                _ => {}
            }
        }
        KeyCode::Esc => {
            app.ui.set_mode(crate::state::AppMode::Settings);
        }
        KeyCode::Char(c) => {
            match app.profile.profile_edit_focus {
                Bio => app.profile.edit_bio.push(c),
                Url1 => app.profile.edit_url1.push(c),
                Url2 => app.profile.edit_url2.push(c),
                Url3 => app.profile.edit_url3.push(c),
                Location => app.profile.edit_location.push(c),
                ProfilePic => app.profile.edit_profile_pic.push(c),
                CoverBanner => app.profile.edit_cover_banner.push(c),
                _ => {}
            }
        }
        KeyCode::Backspace => {
            match app.profile.profile_edit_focus {
                Bio => { app.profile.edit_bio.pop(); }
                Url1 => { app.profile.edit_url1.pop(); }
                Url2 => { app.profile.edit_url2.pop(); }
                Url3 => { app.profile.edit_url3.pop(); }
                Location => { app.profile.edit_location.pop(); }
                ProfilePic => { app.profile.edit_profile_pic.pop(); }
                CoverBanner => { app.profile.edit_cover_banner.pop(); }
                _ => {}
            }
        }
        _ => {}
    }
}