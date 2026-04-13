use crate::pipeline::{ollama_status, DiagnosticResult};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn run_diagnostics(state: State<'_, AppState>) -> Result<Vec<DiagnosticResult>, String> {
    let mut results = Vec::new();
    results.push(ollama_status().await);

    let db_ok = state.db.search_history("").is_ok();
    results.push(DiagnosticResult {
        name: "SQLite".to_string(),
        ok: db_ok,
        detail: if db_ok { "voxx.db ready" } else { "query failed" }.to_string(),
    });

    let transcribe_script = std::env::current_dir()
        .map(|cwd| cwd.join("src-tauri").join("sidecar").join("transcribe.py"))
        .map(|path| path.exists())
        .unwrap_or(false);
    results.push(DiagnosticResult {
        name: "Transcription sidecar".to_string(),
        ok: transcribe_script,
        detail: if transcribe_script { "transcribe.py found" } else { "transcribe.py missing" }.to_string(),
    });

    Ok(results)
}

#[tauri::command]
pub async fn get_ollama_status() -> Result<DiagnosticResult, String> {
    Ok(ollama_status().await)
}

