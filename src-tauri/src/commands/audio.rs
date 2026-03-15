use crate::audio;

#[tauri::command]
pub fn list_audio_input_devices() -> Result<Vec<String>, String> {
    Ok(audio::list_input_device_names())
}

#[cfg(test)]
mod tests {
    use super::list_audio_input_devices;

    #[test]
    fn lists_input_devices() {
        let devices = list_audio_input_devices().expect("command failed");
        assert!(!devices.is_empty());
    }
}

