use crate::database::HistoryEntry;
use crate::state::AppState;
use tauri::State;
use tauri_plugin_clipboard_manager::ClipboardExt;

#[tauri::command]
pub fn search_history(state: State<'_, AppState>, query: String) -> Result<Vec<HistoryEntry>, String> {
    state.db.search_history(&query).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn update_history_entry(state: State<'_, AppState>, id: i64, processed_text: String) -> Result<HistoryEntry, String> {
    state
        .db
        .update_history_entry(id, &processed_text)
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "History entry not found".to_string())
}

#[tauri::command]
pub fn delete_history_entry(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    state.db.delete_history_entry(id).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn copy_entry(app: tauri::AppHandle, state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let entry = state
        .db
        .get_history_entry(id)
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "History entry not found".to_string())?;
    app.clipboard().write_text(entry.processed_text).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn repaste_entry(app: tauri::AppHandle, state: State<'_, AppState>, id: i64) -> Result<(), String> {
    copy_entry(app, state, id)?;
    paste_from_clipboard();
    Ok(())
}

pub fn paste_from_clipboard() {
    #[cfg(windows)]
    {
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VK_CONTROL,
            VK_V,
        };

        fn keyboard_input(vk: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY(vk),
                        wScan: 0,
                        dwFlags: flags,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            }
        }

        let inputs = [
            keyboard_input(VK_CONTROL.0, KEYBD_EVENT_FLAGS(0)),
            keyboard_input(VK_V.0, KEYBD_EVENT_FLAGS(0)),
            keyboard_input(VK_V.0, KEYEVENTF_KEYUP),
            keyboard_input(VK_CONTROL.0, KEYEVENTF_KEYUP),
        ];

        unsafe {
            let _ = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        }
    }
}
