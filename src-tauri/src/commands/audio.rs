use cpal::traits::{DeviceTrait, HostTrait};

#[tauri::command]
pub fn get_audio_devices() -> Result<Vec<String>, String> {
    let host = cpal::default_host();
    let devices = host.input_devices().map_err(|err| err.to_string())?;
    let mut names = Vec::new();

    for device in devices {
        if let Ok(name) = device.name() {
            names.push(name);
        }
    }

    Ok(names)
}

pub fn choose_input_device(selected_name: Option<&str>) -> Result<cpal::Device, String> {
    let host = cpal::default_host();

    if let Some(selected_name) = selected_name {
        for device in host.input_devices().map_err(|err| err.to_string())? {
            if device.name().map(|name| name == selected_name).unwrap_or(false) {
                return Ok(device);
            }
        }
    }

    host.default_input_device()
        .ok_or_else(|| "No default input device is available".to_string())
}

