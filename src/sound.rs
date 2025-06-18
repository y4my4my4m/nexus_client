// client/src/sound.rs
// SoundManager for playing UI sounds
use std::collections::HashMap;
use std::path::PathBuf;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

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
        let notify_path = PathBuf::from(base_path).join("sounds/notify.mp3");
        let login_success_path = PathBuf::from(base_path).join("sounds/login_success_24.mp3");
        let login_failure_path = PathBuf::from(base_path).join("sounds/login_error_3.mp3");
        let received_direct_message_path = PathBuf::from(base_path).join("sounds/voice/new_msg.mp3");
        let sent_direct_message_path = PathBuf::from(base_path).join("sounds/dm.mp3");
        let mention_path = PathBuf::from(base_path).join("sounds/mention.mp3");
        let change_channel_path = PathBuf::from(base_path).join("sounds/change_channel.mp3");
        let send_chat_message_path = PathBuf::from(base_path).join("sounds/send_chat_message.mp3");
        let receive_chat_message_path = PathBuf::from(base_path).join("sounds/receive_chat_message.mp3");
        sounds.insert(SoundType::ChangeChannel, std::fs::read(change_channel_path).unwrap_or_default());
        sounds.insert(SoundType::SendChannelMessage, std::fs::read(send_chat_message_path).unwrap_or_default());
        sounds.insert(SoundType::ReceiveChannelMessage, std::fs::read(receive_chat_message_path).unwrap_or_default());
        sounds.insert(SoundType::LoginSuccess, std::fs::read(login_success_path).unwrap_or_default());
        sounds.insert(SoundType::LoginFailure, std::fs::read(login_failure_path).unwrap_or_default());
        sounds.insert(SoundType::DirectMessage, std::fs::read(received_direct_message_path).unwrap_or_default());
        sounds.insert(SoundType::MessageSent, std::fs::read(sent_direct_message_path).unwrap_or_default());
        sounds.insert(SoundType::Error, std::fs::read(error_path).unwrap_or_default());
        sounds.insert(SoundType::Notify, std::fs::read(notify_path).unwrap_or_default());
        sounds.insert(SoundType::Mention, std::fs::read(mention_path).unwrap_or_default());
        Self { _stream, stream_handle, sounds }
    }

    pub fn play(&self, sound: SoundType) {
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
