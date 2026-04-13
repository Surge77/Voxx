mod commands;
mod database;
mod modes;
mod pipeline;
mod state;

use commands::{
    audio::get_audio_devices,
    diagnostics::{get_ollama_status, run_diagnostics},
    dictionary::{delete_dictionary_entry, get_dictionary, save_correction},
    history::{copy_entry, delete_history_entry, repaste_entry, search_history, update_history_entry},
    preferences::{get_preferences, set_active_mode, set_preference},
    recording::{cancel_recording, start_recording, stop_recording_and_process},
};
use state::AppState;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_sql::Builder::default().build())
        .setup(|app| {
            let state = AppState::new(app.handle())?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_recording,
            stop_recording_and_process,
            cancel_recording,
            search_history,
            update_history_entry,
            delete_history_entry,
            copy_entry,
            repaste_entry,
            get_dictionary,
            save_correction,
            delete_dictionary_entry,
            get_preferences,
            set_preference,
            set_active_mode,
            run_diagnostics,
            get_ollama_status,
            get_audio_devices
        ])
        .run(tauri::generate_context!())
        .expect("error while running Voxx");
}

