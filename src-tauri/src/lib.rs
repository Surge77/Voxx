mod commands;
mod database;
mod focus;
mod modes;
mod pipeline;
mod state;

use crate::pipeline::warm_transcriber;
use commands::{
    audio::get_audio_devices,
    diagnostics::{get_ollama_status, run_diagnostics},
    dictionary::{delete_dictionary_entry, get_dictionary, save_correction},
    history::{copy_entry, delete_history_entry, repaste_entry, search_history, update_history_entry},
    preferences::{get_preferences, set_active_mode, set_preference},
    recording::{cancel_recording, start_recording, stop_recording_and_process},
};
use state::AppState;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    PhysicalPosition, WebviewUrl, WebviewWindowBuilder,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::{Code, Modifiers, ShortcutState};

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_shortcuts(["ctrl+space"])
                .expect("failed to register Voxx hotkey")
                .with_handler(|app, shortcut, event| {
                    if !shortcut.matches(Modifiers::CONTROL, Code::Space) {
                        return;
                    }

                    match event.state {
                        ShortcutState::Pressed => {
                            let state = app.state::<AppState>();
                            if let Err(error) = commands::recording::start_recording_impl(app, &state) {
                                let _ = app.emit("voxx://recording-error", error);
                            }
                        }
                        ShortcutState::Released => {
                            let app = app.clone();
                            tauri::async_runtime::spawn(async move {
                                let state = app.state::<AppState>();
                                match commands::recording::stop_recording_and_process_impl(app.clone(), &state).await {
                                    Ok(result) => {
                                        let _ = app.emit("voxx://pipeline-result", result);
                                    }
                                    Err(error) => {
                                        let _ = app.emit("voxx://recording-error", error);
                                    }
                                }
                            });
                        }
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_sql::Builder::default().build())
        .setup(|app| {
            let state = AppState::new(app.handle())?;
            app.manage(state);
            setup_tray(app)?;
            setup_overlay_window(app)?;
            let _ = app.autolaunch().enable();
            if std::env::args().any(|arg| arg == "--minimized") {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
            warm_transcriber();
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

fn setup_overlay_window(app: &mut tauri::App) -> tauri::Result<()> {
    let width = 88.0;
    let height = 44.0;
    let overlay = WebviewWindowBuilder::new(app, "overlay", WebviewUrl::App("index.html?overlay".into()))
        .title("Voxx Overlay")
        .inner_size(width, height)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .focused(false)
        .visible(false)
        .build()?;

    if let Some(monitor) = overlay.primary_monitor()? {
        let work_area = monitor.work_area();
        let x = work_area.position.x + ((work_area.size.width as f64 - width) / 2.0) as i32;
        let y = work_area.position.y + work_area.size.height as i32 - height as i32 - 28;
        let _ = overlay.set_position(PhysicalPosition::new(x, y));
    }

    Ok(())
}

fn setup_tray(app: &mut tauri::App) -> tauri::Result<()> {
    let open = MenuItemBuilder::with_id("open", "Open Voxx").build(app)?;
    let general = MenuItemBuilder::with_id("mode_general", "Mode: General").build(app)?;
    let code = MenuItemBuilder::with_id("mode_code", "Mode: Code").build(app)?;
    let command = MenuItemBuilder::with_id("mode_command", "Mode: Command").build(app)?;
    let email = MenuItemBuilder::with_id("mode_email", "Mode: Email").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
    let menu = MenuBuilder::new(app)
        .items(&[&open, &general, &code, &command, &email, &quit])
        .build()?;

    let tray = TrayIconBuilder::with_id("main")
        .tooltip("Voxx")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" => show_main_window(app),
            "mode_general" => set_mode(app, modes::DictationMode::General),
            "mode_code" => set_mode(app, modes::DictationMode::Code),
            "mode_command" => set_mode(app, modes::DictationMode::Command),
            "mode_email" => set_mode(app, modes::DictationMode::Email),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        })
        ;
    let tray = if let Some(icon) = app.default_window_icon().cloned() {
        tray.icon(icon)
    } else {
        tray
    };
    tray.build(app)?;

    Ok(())
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn set_mode(app: &tauri::AppHandle, mode: modes::DictationMode) {
    let state = app.state::<AppState>();
    let Ok(mut preferences) = state.preferences.lock() else {
        return;
    };
    preferences.active_mode = mode;
    let _ = state.db.save_preferences(&preferences);
    let _ = app.emit("voxx://preferences-changed", preferences.clone());
}
