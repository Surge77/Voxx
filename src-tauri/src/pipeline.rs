use crate::database::Database;
use crate::modes::{mode_prompt, DictationMode};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Mutex, OnceLock};

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
    error: Option<String>,
}

struct TranscriberProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl Drop for TranscriberProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

static TRANSCRIBER: OnceLock<Mutex<Option<TranscriberProcess>>> = OnceLock::new();

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
}

pub async fn transcribe_audio(audio_path: &Path) -> anyhow::Result<String> {
    if let Ok(fixture) = std::env::var("VOXX_FIXTURE_TRANSCRIPT") {
        return Ok(fixture);
    }

    if let Ok(text) = transcribe_audio_with_server(audio_path) {
        return Ok(text);
    }

    // Fallback keeps the app usable if the warm sidecar died.
    let _ = TRANSCRIBER.get_or_init(|| Mutex::new(None)).lock().map(|mut process| {
        *process = None;
    });

    let script_path = resolve_transcribe_script_path_from(&std::env::current_dir()?);
    let output = Command::new("python")
        .arg(script_path)
        .arg("--audio")
        .arg(audio_path)
        .output()?;

    if !output.status.success() {
        anyhow::bail!(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let parsed: TranscribeOutput = serde_json::from_slice(&output.stdout)?;
    if let Some(error) = parsed.error {
        anyhow::bail!(error);
    }
    Ok(parsed.raw_text)
}

pub fn warm_transcriber() {
    if std::env::var("VOXX_FIXTURE_TRANSCRIPT").is_ok() {
        return;
    }

    std::thread::spawn(|| {
        let mutex = TRANSCRIBER.get_or_init(|| Mutex::new(None));
        let Ok(mut guard) = mutex.lock() else {
            eprintln!("Voxx transcriber warmup failed: lock poisoned");
            return;
        };

        if guard.is_none() {
            match start_transcriber_process() {
                Ok(process) => {
                    *guard = Some(process);
                }
                Err(error) => {
                    eprintln!("Voxx transcriber warmup failed: {error}");
                }
            }
        }
    });
}

fn transcribe_audio_with_server(audio_path: &Path) -> anyhow::Result<String> {
    let mutex = TRANSCRIBER.get_or_init(|| Mutex::new(None));
    let mut guard = mutex.lock().map_err(|_| anyhow::anyhow!("Transcriber lock poisoned"))?;
    if guard.is_none() {
        *guard = Some(start_transcriber_process()?);
    }

    let transcriber = guard
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("Transcriber process missing"))?;
    writeln!(
        transcriber.stdin,
        "{}",
        serde_json::json!({ "audio": audio_path }).to_string()
    )?;
    transcriber.stdin.flush()?;

    let mut line = String::new();
    let bytes_read = transcriber.stdout.read_line(&mut line)?;
    if bytes_read == 0 {
        anyhow::bail!("Transcriber process closed");
    }

    let parsed: TranscribeOutput = serde_json::from_str(&line)?;
    if let Some(error) = parsed.error {
        anyhow::bail!(error);
    }
    Ok(parsed.raw_text)
}

fn start_transcriber_process() -> anyhow::Result<TranscriberProcess> {
    let script_path = resolve_transcribe_script_path_from(&std::env::current_dir()?);
    let mut child = Command::new("python")
        .arg(script_path)
        .arg("--serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let stdin = child.stdin.take().ok_or_else(|| anyhow::anyhow!("Transcriber stdin unavailable"))?;
    let stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("Transcriber stdout unavailable"))?;
    let mut stdout = BufReader::new(stdout);
    let mut ready = String::new();
    stdout.read_line(&mut ready)?;

    let ready_value: serde_json::Value = serde_json::from_str(&ready)?;
    if ready_value.get("ready").and_then(|value| value.as_bool()) != Some(true) {
        anyhow::bail!("Transcriber did not report ready");
    }

    Ok(TranscriberProcess { child, stdin, stdout })
}

pub fn resolve_transcribe_script_path_from(cwd: &Path) -> PathBuf {
    let root_relative = cwd.join("src-tauri").join("sidecar").join("transcribe.py");
    if root_relative.exists() {
        return root_relative;
    }

    cwd.join("sidecar").join("transcribe.py")
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
            "stream": false,
            "options": {
                "temperature": 0,
                "num_predict": 256
            },
            "keep_alive": "10m"
        }))
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!("Ollama returned {}", response.status());
    }

    let parsed = response.json::<OllamaResponse>().await?;
    Ok(parsed.response.trim().to_string())
}

pub async fn warm_ollama() {
    let client = Client::new();
    let result = client
        .post("http://localhost:11434/api/generate")
        .json(&serde_json::json!({
            "model": "phi4-mini",
            "prompt": "ok",
            "stream": false,
            "options": {
                "temperature": 0,
                "num_predict": 1
            },
            "keep_alive": "10m"
        }))
        .send()
        .await;

    if let Err(error) = result {
        eprintln!("Voxx Ollama warmup failed: {error}");
    }
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

#[cfg(test)]
mod tests {
    use super::resolve_transcribe_script_path_from;
    use std::path::PathBuf;

    #[test]
    fn resolves_transcribe_script_from_project_root_or_src_tauri() {
        let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("project root")
            .to_path_buf();
        let src_tauri = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        assert!(resolve_transcribe_script_path_from(&project_root).ends_with("src-tauri/sidecar/transcribe.py"));
        assert!(resolve_transcribe_script_path_from(&src_tauri).ends_with("sidecar/transcribe.py"));
    }
}
