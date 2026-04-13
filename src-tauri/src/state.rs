use crate::database::Database;
use crate::modes::DictationMode;
use cpal::Stream;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{AppHandle, Manager};

pub struct RecordingSession {
    pub started_at: Instant,
    pub audio_path: PathBuf,
    pub sample_rate: u32,
    pub channels: u16,
    pub samples: Arc<Mutex<Vec<f32>>>,
    pub stream: Option<Stream>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preferences {
    pub active_mode: DictationMode,
    pub wake_word_enabled: bool,
    pub sound_enabled: bool,
    pub sound_volume: f32,
    pub theme: String,
    pub restore_clipboard: bool,
    pub selected_audio_device: Option<String>,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            active_mode: DictationMode::General,
            wake_word_enabled: false,
            sound_enabled: false,
            sound_volume: 0.4,
            theme: "dark".to_string(),
            restore_clipboard: true,
            selected_audio_device: None,
        }
    }
}

pub struct AppState {
    pub db: Database,
    pub recording: Mutex<Option<RecordingSession>>,
    pub preferences: Mutex<Preferences>,
}

impl AppState {
    pub fn new(app: &AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let data_dir = app.path().app_data_dir()?;
        std::fs::create_dir_all(&data_dir)?;
        let db = Database::open(data_dir.join("voxx.db"))?;
        let preferences = db.load_preferences().unwrap_or_default();

        Ok(Self {
            db,
            recording: Mutex::new(None),
            preferences: Mutex::new(preferences),
        })
    }
}
