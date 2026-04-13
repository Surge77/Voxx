use crate::commands::audio::choose_input_device;
use crate::commands::history::paste_from_clipboard;
use crate::pipeline::{post_process_with_ollama, transcribe_audio, PipelineResult};
use crate::state::{AppState, RecordingSession};
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::SampleFormat;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_clipboard_manager::ClipboardExt;

const MIN_HOLD_MS: i64 = 300;

#[tauri::command]
pub fn start_recording(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let mut recording = state.recording.lock().map_err(|_| "Recording lock poisoned".to_string())?;
    if recording.is_some() {
        return Ok(());
    }

    let preferences = state
        .preferences
        .lock()
        .map_err(|_| "Preferences lock poisoned".to_string())?
        .clone();

    let data_dir = app.path().app_data_dir().map_err(|err| err.to_string())?;
    std::fs::create_dir_all(&data_dir).map_err(|err| err.to_string())?;
    let audio_path = data_dir.join("last-recording.wav");
    let device = choose_input_device(preferences.selected_audio_device.as_deref())?;
    let config = device.default_input_config().map_err(|err| err.to_string())?;
    let sample_rate = config.sample_rate().0;
    let channels = config.channels();
    let samples = Arc::new(Mutex::new(Vec::<f32>::new()));
    let stream = build_stream(&device, &config.into(), samples.clone())?;
    stream.play().map_err(|err| err.to_string())?;

    *recording = Some(RecordingSession {
        started_at: Instant::now(),
        audio_path,
        sample_rate,
        channels,
        samples,
        stream: Some(stream),
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_recording_and_process(app: AppHandle, state: State<'_, AppState>) -> Result<PipelineResult, String> {
    let session = {
        let mut recording = state.recording.lock().map_err(|_| "Recording lock poisoned".to_string())?;
        recording.take()
    };

    let mut session = session.ok_or_else(|| "No active recording".to_string())?;
    let duration_ms = session.started_at.elapsed().as_millis() as i64;
    drop(session.stream.take());

    let mode = state
        .preferences
        .lock()
        .map_err(|_| "Preferences lock poisoned".to_string())?
        .active_mode;

    if duration_ms < MIN_HOLD_MS {
        return Ok(PipelineResult {
            ignored: true,
            raw_text: String::new(),
            processed_text: String::new(),
            mode,
            duration_ms,
            history_id: None,
            error: None,
        });
    }

    write_wav(&session).map_err(|err| err.to_string())?;
    let raw_text = match transcribe_audio(&session.audio_path).await {
        Ok(text) => text,
        Err(error) => {
            return Ok(PipelineResult {
                ignored: false,
                raw_text: String::new(),
                processed_text: String::new(),
                mode,
                duration_ms,
                history_id: None,
                error: Some(format!("Transcription failed: {error}")),
            });
        }
    };

    let processed_text = match post_process_with_ollama(&state.db, mode, &raw_text).await {
        Ok(text) => text,
        Err(error) => {
            return Ok(PipelineResult {
                ignored: false,
                raw_text,
                processed_text: String::new(),
                mode,
                duration_ms,
                history_id: None,
                error: Some(format!("Ollama post-processing failed: {error}")),
            });
        }
    };

    app.clipboard()
        .write_text(processed_text.clone())
        .map_err(|err| err.to_string())?;
    paste_from_clipboard();
    let entry = state
        .db
        .insert_history(&raw_text, &processed_text, mode, duration_ms)
        .map_err(|err| err.to_string())?;

    Ok(PipelineResult {
        ignored: false,
        raw_text,
        processed_text,
        mode,
        duration_ms,
        history_id: Some(entry.id),
        error: None,
    })
}

#[tauri::command]
pub fn cancel_recording(state: State<'_, AppState>) -> Result<(), String> {
    let mut recording = state.recording.lock().map_err(|_| "Recording lock poisoned".to_string())?;
    *recording = None;
    Ok(())
}

fn build_stream(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    samples: Arc<Mutex<Vec<f32>>>,
) -> Result<cpal::Stream, String> {
    let supported = device.default_input_config().map_err(|err| err.to_string())?;
    let error_handler = |err| eprintln!("Voxx audio stream error: {err}");

    match supported.sample_format() {
        SampleFormat::F32 => device
            .build_input_stream(
                config,
                move |data: &[f32], _| push_samples(data.iter().copied(), &samples),
                error_handler,
                None,
            )
            .map_err(|err| err.to_string()),
        SampleFormat::I16 => device
            .build_input_stream(
                config,
                move |data: &[i16], _| push_samples(data.iter().map(|sample| *sample as f32 / i16::MAX as f32), &samples),
                error_handler,
                None,
            )
            .map_err(|err| err.to_string()),
        SampleFormat::U16 => device
            .build_input_stream(
                config,
                move |data: &[u16], _| push_samples(data.iter().map(|sample| (*sample as f32 / u16::MAX as f32) * 2.0 - 1.0), &samples),
                error_handler,
                None,
            )
            .map_err(|err| err.to_string()),
        _ => Err("Unsupported audio sample format".to_string()),
    }
}

fn push_samples<I>(incoming: I, samples: &Arc<Mutex<Vec<f32>>>)
where
    I: Iterator<Item = f32>,
{
    if let Ok(mut buffer) = samples.lock() {
        buffer.extend(incoming);
    }
}

fn write_wav(session: &RecordingSession) -> hound::Result<()> {
    let samples = session.samples.lock().map(|samples| samples.clone()).unwrap_or_default();
    let spec = hound::WavSpec {
        channels: session.channels,
        sample_rate: session.sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(&session.audio_path, spec)?;
    for sample in samples {
        let scaled = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        writer.write_sample(scaled)?;
    }
    writer.finalize()
}

#[cfg(test)]
mod tests {
    use super::MIN_HOLD_MS;

    #[test]
    fn minimum_hold_threshold_matches_product_spec() {
        assert_eq!(MIN_HOLD_MS, 300);
    }
}
