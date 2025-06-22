// client/src/global_prefs.rs
// Global preferences for the app (not user-specific)
use std::path::PathBuf;
use std::fs;
use serde::{Serialize, Deserialize};
use once_cell::sync::OnceCell;
use std::sync::RwLock;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GlobalPrefs {
    pub sound_effects_enabled: bool,
    pub minimal_banner_glitch_enabled: bool,
}

impl Default for GlobalPrefs {
    fn default() -> Self {
        Self {
            sound_effects_enabled: true,
            minimal_banner_glitch_enabled: true,
        }
    }
}

impl GlobalPrefs {
    pub fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".cyberpunk_bbs_prefs.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(prefs) = serde_json::from_str(&data) {
                return prefs;
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Ok(data) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, data);
        }
    }
}

static GLOBAL_PREFS: OnceCell<RwLock<GlobalPrefs>> = OnceCell::new();

pub fn init_global_prefs() {
    let prefs = GlobalPrefs::load();
    GLOBAL_PREFS.set(RwLock::new(prefs)).ok();
}

pub fn global_prefs() -> std::sync::RwLockReadGuard<'static, GlobalPrefs> {
    GLOBAL_PREFS.get().expect("GlobalPrefs not initialized").read().expect("RwLock poisoned")
}

pub fn global_prefs_mut() -> std::sync::RwLockWriteGuard<'static, GlobalPrefs> {
    GLOBAL_PREFS.get().expect("GlobalPrefs not initialized").write().expect("RwLock poisoned")
}
