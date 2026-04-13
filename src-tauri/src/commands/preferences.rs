use crate::modes::DictationMode;
use crate::state::{AppState, Preferences};
use serde_json::Value;
use tauri::State;

#[tauri::command]
pub fn get_preferences(state: State<'_, AppState>) -> Result<Preferences, String> {
    state
        .preferences
        .lock()
        .map(|preferences| preferences.clone())
        .map_err(|_| "Preferences lock poisoned".to_string())
}

#[tauri::command]
pub fn set_active_mode(state: State<'_, AppState>, mode: DictationMode) -> Result<Preferences, String> {
    let mut preferences = state.preferences.lock().map_err(|_| "Preferences lock poisoned".to_string())?;
    preferences.active_mode = mode;
    state.db.save_preferences(&preferences).map_err(|err| err.to_string())?;
    Ok(preferences.clone())
}

#[tauri::command]
pub fn set_preference(state: State<'_, AppState>, key: String, value: Value) -> Result<Preferences, String> {
    let mut preferences = state.preferences.lock().map_err(|_| "Preferences lock poisoned".to_string())?;
    match key.as_str() {
        "activeMode" => {
            preferences.active_mode = serde_json::from_value(value).map_err(|err| err.to_string())?;
        }
        "wakeWordEnabled" => preferences.wake_word_enabled = value.as_bool().unwrap_or(false),
        "soundEnabled" => preferences.sound_enabled = value.as_bool().unwrap_or(false),
        "soundVolume" => preferences.sound_volume = value.as_f64().unwrap_or(0.4) as f32,
        "theme" => preferences.theme = value.as_str().unwrap_or("dark").to_string(),
        "restoreClipboard" => preferences.restore_clipboard = value.as_bool().unwrap_or(true),
        "selectedAudioDevice" => preferences.selected_audio_device = value.as_str().map(ToString::to_string),
        _ => return Err(format!("Unknown preference key: {key}")),
    }
    state.db.save_preferences(&preferences).map_err(|err| err.to_string())?;
    Ok(preferences.clone())
}

