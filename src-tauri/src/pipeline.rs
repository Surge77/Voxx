use crate::database::Database;
use crate::modes::{mode_prompt, DictationMode};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineResult {
    pub ignored: bool,
    pub raw_text: String,
    pub processed_text: String,
    pub mode: DictationMode,
    pub duration_ms: i64,
    pub history_id: Option<i64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticResult {
    pub name: String,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Deserialize)]
struct TranscribeOutput {
    #[serde(rename = "rawText")]
    raw_text: String,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
}

pub async fn transcribe_audio(audio_path: &Path) -> anyhow::Result<String> {
    if let Ok(fixture) = std::env::var("VOXX_FIXTURE_TRANSCRIPT") {
        return Ok(fixture);
    }

    let script_path = std::env::current_dir()?.join("src-tauri").join("sidecar").join("transcribe.py");
    let output = Command::new("python")
        .arg(script_path)
        .arg("--audio")
        .arg(audio_path)
        .output()?;

    if !output.status.success() {
        anyhow::bail!(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let parsed: TranscribeOutput = serde_json::from_slice(&output.stdout)?;
    Ok(parsed.raw_text)
}

pub async fn post_process_with_ollama(db: &Database, mode: DictationMode, raw_text: &str) -> anyhow::Result<String> {
    let dictionary = db.dictionary_prompt_lines()?.join("\n");
    let prompt = format!(
        "{}\n\nCustom dictionary:\n{}\n\nRaw transcript:\n{}\n\nReturn only the final corrected text.",
        mode_prompt(mode),
        dictionary,
        raw_text
    );

    let client = Client::new();
    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&serde_json::json!({
            "model": "phi4-mini",
            "prompt": prompt,
            "stream": false
        }))
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!("Ollama returned {}", response.status());
    }

    let parsed = response.json::<OllamaResponse>().await?;
    Ok(parsed.response.trim().to_string())
}

pub async fn ollama_status() -> DiagnosticResult {
    let client = Client::new();
    match client.get("http://localhost:11434/api/tags").send().await {
        Ok(response) if response.status().is_success() => DiagnosticResult {
            name: "Ollama".to_string(),
            ok: true,
            detail: "localhost:11434 reachable".to_string(),
        },
        Ok(response) => DiagnosticResult {
            name: "Ollama".to_string(),
            ok: false,
            detail: format!("HTTP {}", response.status()),
        },
        Err(error) => DiagnosticResult {
            name: "Ollama".to_string(),
            ok: false,
            detail: error.to_string(),
        },
    }
}

