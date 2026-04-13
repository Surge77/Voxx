use crate::database::DictionaryEntry;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn get_dictionary(state: State<'_, AppState>) -> Result<Vec<DictionaryEntry>, String> {
    state.db.list_dictionary().map_err(|err| err.to_string())
}

#[tauri::command]
pub fn save_correction(
    state: State<'_, AppState>,
    _history_id: i64,
    wrong: String,
    right: String,
) -> Result<DictionaryEntry, String> {
    if wrong.trim().is_empty() || right.trim().is_empty() {
        return Err("Correction terms cannot be empty".to_string());
    }
    state.db.insert_dictionary(&wrong, &right).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn delete_dictionary_entry(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    state.db.delete_dictionary_entry(id).map_err(|err| err.to_string())
}

