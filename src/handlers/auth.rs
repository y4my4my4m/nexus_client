use crate::app::App;
use crate::sound::SoundType;
use common::ClientMessage;
use crossterm::event::{KeyCode, KeyEvent};

/// Handle authentication input (login/register)
pub fn handle_auth_input(key: KeyEvent, app: &mut App) {
    let is_login = app.ui.mode == crate::state::AppMode::Login;
    
    match key.code {
        KeyCode::Char(c) => {
            if let Some(im) = &app.auth.input_mode {
                match im {
                    crate::state::InputMode::LoginUsername | crate::state::InputMode::RegisterUsername => {
                        app.auth.current_input.push(c);
                    }
                    crate::state::InputMode::LoginPassword | crate::state::InputMode::RegisterPassword => {
                        app.auth.password_input.push(c);
                    }
                    _ => {}
                }
            }
        }
        KeyCode::Backspace => {
            if let Some(im) = &app.auth.input_mode {
                match im {
                    crate::state::InputMode::LoginUsername | crate::state::InputMode::RegisterUsername => {
                        app.auth.current_input.pop();
                    }
                    crate::state::InputMode::LoginPassword | crate::state::InputMode::RegisterPassword => {
                        app.auth.password_input.pop();
                    }
                    _ => {}
                }
            }
        }
        KeyCode::Tab => {
            let focus_order = if is_login {
                [
                    crate::state::InputMode::LoginUsername,
                    crate::state::InputMode::LoginPassword,
                    crate::state::InputMode::AuthSubmit,
                    crate::state::InputMode::AuthSwitch,
                ]
            } else {
                [
                    crate::state::InputMode::RegisterUsername,
                    crate::state::InputMode::RegisterPassword,
                    crate::state::InputMode::AuthSubmit,
                    crate::state::InputMode::AuthSwitch,
                ]
            };
            let current_idx = focus_order.iter().position(|m| Some(m) == app.auth.input_mode.as_ref()).unwrap_or(0);
            let next_idx = (current_idx + 1) % focus_order.len();
            app.auth.input_mode = Some(focus_order[next_idx].clone());
        }
        KeyCode::BackTab => {
            let focus_order = if is_login {
                [
                    crate::state::InputMode::LoginUsername,
                    crate::state::InputMode::LoginPassword,
                    crate::state::InputMode::AuthSubmit,
                    crate::state::InputMode::AuthSwitch,
                ]
            } else {
                [
                    crate::state::InputMode::RegisterUsername,
                    crate::state::InputMode::RegisterPassword,
                    crate::state::InputMode::AuthSubmit,
                    crate::state::InputMode::AuthSwitch,
                ]
            };
            let current_idx = focus_order.iter().position(|m| Some(m) == app.auth.input_mode.as_ref()).unwrap_or(0);
            let next_idx = (current_idx + focus_order.len() - 1) % focus_order.len();
            app.auth.input_mode = Some(focus_order[next_idx].clone());
        }
        KeyCode::Enter => {
            match &app.auth.input_mode {
                Some(crate::state::InputMode::LoginUsername) => {
                    app.auth.input_mode = Some(crate::state::InputMode::LoginPassword);
                }
                Some(crate::state::InputMode::LoginPassword) => {
                    app.auth.input_mode = Some(crate::state::InputMode::AuthSubmit);
                }
                Some(crate::state::InputMode::RegisterUsername) => {
                    app.auth.input_mode = Some(crate::state::InputMode::RegisterPassword);
                }
                Some(crate::state::InputMode::RegisterPassword) => {
                    app.auth.input_mode = Some(crate::state::InputMode::AuthSubmit);
                }
                Some(crate::state::InputMode::AuthSubmit) => {
                    let username = app.auth.current_input.clone();
                    let password = app.auth.password_input.clone();
                    
                    if username.is_empty() || password.is_empty() {
                        app.set_notification("Fields cannot be empty.", None, false);
                        app.sound_manager.play(SoundType::LoginFailure);
                        return;
                    }
                    
                    if is_login {
                        app.send_to_server(ClientMessage::Login { username, password });
                    } else {
                        app.send_to_server(ClientMessage::Register { username, password });
                    }
                    
                    // Clear fields only after sending
                    app.auth.clear_inputs();
                }
                Some(crate::state::InputMode::AuthSwitch) => {
                    app.auth.clear_inputs();
                    if is_login {
                        app.ui.set_mode(crate::state::AppMode::Register);
                        app.auth.input_mode = Some(crate::state::InputMode::RegisterUsername);
                    } else {
                        app.ui.set_mode(crate::state::AppMode::Login);
                        app.auth.input_mode = Some(crate::state::InputMode::LoginUsername);
                    }
                }
                _ => {}
            }
        }
        KeyCode::Esc => {
            app.ui.quit();
        }
        _ => {}
    }
}