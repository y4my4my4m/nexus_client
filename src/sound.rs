// client/src/sound.rs
// SoundManager for playing UI sounds
use crate::global_prefs::global_prefs;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum SoundType {
    ChangeChannel,
    SendChannelMessage,
    ReceiveChannelMessage,
    DirectMessage,
    Error,
    Notify,
    LoginSuccess,
    LoginFailure,
    MessageSent,
    Mention,
    PopupOpen,
    PopupClose,
    Select,
    Scroll,
    Save,
}

pub struct SoundManager {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sounds: HashMap<SoundType, Vec<u8>>, // Store sound data in memory
}

impl SoundManager {
    pub fn new() -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().expect("Failed to open audio output");
        let mut sounds = HashMap::new();
        let base_path = env!("CARGO_MANIFEST_DIR");
        let error_path = PathBuf::from(base_path).join("sounds/error.mp3");
        let notify_path = PathBuf::from(base_path).join("assets/sounds/notify.mp3");
        let login_success_path = PathBuf::from(base_path).join("assets/sounds/login_success.mp3");
        let login_failure_path = PathBuf::from(base_path).join("assets/sounds/login_failure.mp3");
        let received_direct_message_path = PathBuf::from(base_path).join("assets/sounds/received_direct_message.mp3");
        let sent_direct_message_path = PathBuf::from(base_path).join("assets/sounds/sent_direct_message.mp3");
        let mention_path = PathBuf::from(base_path).join("assets/sounds/mention.mp3");
        let change_channel_path = PathBuf::from(base_path).join("assets/sounds/change_channel.mp3");
        let send_channel_message_path = PathBuf::from(base_path).join("assets/sounds/send_channel_message.mp3");
        let receive_channel_message_path = PathBuf::from(base_path).join("assets/sounds/receive_channel_message.mp3");
        let popup_open_path = PathBuf::from(base_path).join("assets/sounds/popup_open.mp3");
        let popup_close_path = PathBuf::from(base_path).join("assets/sounds/popup_close.mp3");
        let scroll_path = PathBuf::from(base_path).join("assets/sounds/scroll.mp3");
        let save_path = PathBuf::from(base_path).join("assets/sounds/save.mp3");
        let select_path = PathBuf::from(base_path).join("assets/sounds/select.mp3");
        sounds.insert(SoundType::Select, std::fs::read(select_path).unwrap_or_default());
        sounds.insert(SoundType::Save, std::fs::read(save_path).unwrap_or_default());
        sounds.insert(SoundType::Scroll, std::fs::read(scroll_path).unwrap_or_default());
        sounds.insert(SoundType::ChangeChannel, std::fs::read(change_channel_path).unwrap_or_default());
        sounds.insert(SoundType::SendChannelMessage, std::fs::read(send_channel_message_path).unwrap_or_default());
        sounds.insert(SoundType::ReceiveChannelMessage, std::fs::read(receive_channel_message_path).unwrap_or_default());
        sounds.insert(SoundType::PopupOpen, std::fs::read(popup_open_path).unwrap_or_default());
        sounds.insert(SoundType::LoginSuccess, std::fs::read(login_success_path).unwrap_or_default());
        sounds.insert(SoundType::LoginFailure, std::fs::read(login_failure_path).unwrap_or_default());
        sounds.insert(SoundType::DirectMessage, std::fs::read(received_direct_message_path).unwrap_or_default());
        sounds.insert(SoundType::MessageSent, std::fs::read(sent_direct_message_path).unwrap_or_default());
        sounds.insert(SoundType::Error, std::fs::read(error_path).unwrap_or_default());
        sounds.insert(SoundType::PopupClose, std::fs::read(popup_close_path).unwrap_or_default());
        sounds.insert(SoundType::Notify, std::fs::read(notify_path).unwrap_or_default());
        sounds.insert(SoundType::Mention, std::fs::read(mention_path).unwrap_or_default());
        Self { _stream, stream_handle, sounds }
    }

    pub fn play(&self, sound: SoundType) {
        if !global_prefs().sound_effects_enabled {
            return;
        }
        if let Some(data) = self.sounds.get(&sound) {
            if !data.is_empty() {
                let cursor = std::io::Cursor::new(data.clone());
                if let Ok(decoder) = Decoder::new(cursor) {
                    if let Ok(sink) = Sink::try_new(&self.stream_handle) {
                        sink.append(decoder);
                        sink.detach(); // Play in background
                    }
                }
            }
        }
    }
}
