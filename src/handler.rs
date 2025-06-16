use crate::app::{App, AppMode};
use common::ClientMessage;
use crossterm::event::{self, KeyCode, KeyEvent};
use ratatui::widgets::ListState;

// A helper trait to make list navigation wrap around.
pub trait ListStateExt {
    fn select_next(&mut self, list_len: usize);
    fn select_previous(&mut self, list_len: usize);
}

impl ListStateExt for ListState {
    fn select_next(&mut self, list_len: usize) {
        if list_len == 0 { return; }
        let i = match self.selected() {
            Some(i) => {
                if i >= list_len - 1 { 0 } else { i + 1 }
            }
            None => 0,
        };
        self.select(Some(i));
    }

    fn select_previous(&mut self, list_len: usize) {
        if list_len == 0 { return; }
        let i = match self.selected() {
            Some(i) => {
                if i == 0 { list_len - 1 } else { i - 1 }
            }
            None => 0,
        };
        self.select(Some(i));
    }
}

pub fn handle_key_event(key: KeyEvent, app: &mut App) {
    if key.kind != event::KeyEventKind::Press {
        return;
    }

    match app.mode {
        AppMode::Chat => match key.code {
            KeyCode::Char(c) => app.chat_input.push(c),
            KeyCode::Backspace => {
                app.chat_input.pop();
            }
            KeyCode::Enter => {
                if !app.chat_input.is_empty() {
                    let msg_content = app.chat_input.drain(..).collect();
                    app.send_to_server(ClientMessage::SendChatMessage(msg_content));
                }
            }
            KeyCode::Esc => app.mode = AppMode::MainMenu,
            _ => {}
        },
        _ => match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => app.should_quit = true,
            KeyCode::Up => match app.mode {
                AppMode::MainMenu => app.main_menu_state.select_previous(3),
                AppMode::ForumList => app.forum_list_state.select_previous(app.forums.len()),
                AppMode::ThreadList => {
                    if let Some(forum_idx) = app.current_forum_index {
                        let thread_count = app.forums[forum_idx].threads.len();
                        app.thread_list_state.select_previous(thread_count);
                    }
                }
                _ => {}
            },
            KeyCode::Down => match app.mode {
                AppMode::MainMenu => app.main_menu_state.select_next(3),
                AppMode::ForumList => app.forum_list_state.select_next(app.forums.len()),
                AppMode::ThreadList => {
                    if let Some(forum_idx) = app.current_forum_index {
                        let thread_count = app.forums[forum_idx].threads.len();
                        app.thread_list_state.select_next(thread_count);
                    }
                }
                _ => {}
            },
            KeyCode::Enter => match app.mode {
                AppMode::MainMenu => {
                    if let Some(selected) = app.main_menu_state.selected() {
                        match selected {
                            0 => {
                                // Request forums when entering the list
                                app.send_to_server(ClientMessage::GetForums);
                                app.mode = AppMode::ForumList;
                            },
                            1 => app.mode = AppMode::Chat,
                            2 => app.should_quit = true,
                            _ => {}
                        }
                    }
                }
                AppMode::ForumList => {
                    if let Some(selected) = app.forum_list_state.selected() {
                        app.current_forum_index = Some(selected);
                        app.thread_list_state.select(Some(0));
                        app.mode = AppMode::ThreadList;
                    }
                }
                AppMode::ThreadList => {
                    if let Some(selected) = app.thread_list_state.selected() {
                        app.current_thread_index = Some(selected);
                        app.mode = AppMode::PostView;
                    }
                }
                _ => {}
            },
            KeyCode::Esc => match app.mode {
                AppMode::ForumList | AppMode::Chat => app.mode = AppMode::MainMenu,
                AppMode::ThreadList => app.mode = AppMode::ForumList,
                AppMode::PostView => app.mode = AppMode::ThreadList,
                _ => {}
            },
            _ => {}
        },
    }
}