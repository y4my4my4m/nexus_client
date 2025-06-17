// client/src/sound.rs
// SoundManager for playing UI sounds
use std::collections::HashMap;
use std::path::PathBuf;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum SoundType {
    // Click,
    Error,
    Notify,
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
        // let click_path = PathBuf::from(base_path).join("sounds/click.mp3");
        let error_path = PathBuf::from(base_path).join("sounds/error.mp3");
        let notify_path = PathBuf::from(base_path).join("sounds/notify.mp3");
        // sounds.insert(SoundType::Click, std::fs::read(click_path).unwrap_or_default());
        sounds.insert(SoundType::Error, std::fs::read(error_path).unwrap_or_default());
        sounds.insert(SoundType::Notify, std::fs::read(notify_path).unwrap_or_default());
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
