pub mod capture;
pub mod encode;
pub mod session;

use crate::config::TranscriptionProvider;
use crate::state::{AppState, RecordingState};
use capture::{resolve_input_device, DeviceSnapshot};
use encode::{DefaultEncoderBackend, EncodedAudio, EncoderBackend};
use tauri::{AppHandle, Emitter, Manager, Runtime};

const AUDIO_WARNING_EVENT: &str = "audio-warning";
const AUDIO_ERROR_EVENT: &str = "audio-error";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AudioWarningCode {
    SelectedDeviceUnavailable,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct AudioWarningPayload {
    pub code: AudioWarningCode,
    pub message: String,
    pub requested_device: Option<String>,
    pub active_device: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AudioErrorCode {
    NoInputDevice,
    EncodeFailed,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct AudioErrorPayload {
    pub code: AudioErrorCode,
    pub message: String,
}

pub fn list_input_device_names() -> Vec<String> {
    let snapshot = capture::system_device_snapshot();
    capture::list_input_device_names(&snapshot)
}

pub fn start_recording<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let snapshot = capture::system_device_snapshot();
    start_recording_with_snapshot(app, snapshot)
}

pub fn start_recording_with_snapshot<R: Runtime>(
    app: &AppHandle<R>,
    snapshot: DeviceSnapshot,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    let selected = state
        .config
        .lock()
        .map_err(|e| e.to_string())?
        .audio
        .device
        .clone();

    let resolved = match resolve_input_device(selected.as_deref(), &snapshot) {
        Ok(device) => device,
        Err(_) => {
            let message =
                "No microphone input device is available. Open Settings > Microphone and click Refresh.".to_string();
            *state.recording_state.lock().map_err(|e| e.to_string())? = RecordingState::Idle;
            *state.audio_session.lock().map_err(|e| e.to_string())? = None;
            emit_audio_error(
                app,
                AudioErrorPayload {
                    code: AudioErrorCode::NoInputDevice,
                    message: message.clone(),
                },
            );
            return Err(message);
        }
    };

    if let Some(fallback_from) = resolved.fallback_from.clone() {
        emit_audio_warning(
            app,
            AudioWarningPayload {
                code: AudioWarningCode::SelectedDeviceUnavailable,
                message: format!(
                    "Selected microphone '{fallback_from}' is unavailable. Using '{}'.",
                    resolved.active_device
                ),
                requested_device: Some(fallback_from),
                active_device: Some(resolved.active_device.clone()),
            },
        );
    }

    let session = session::new_session(resolved.active_device);
    *state.audio_session.lock().map_err(|e| e.to_string())? = Some(session);
    Ok(())
}

pub fn stop_recording_and_encode<R: Runtime>(app: &AppHandle<R>) -> Result<EncodedAudio, String> {
    stop_recording_and_encode_with_backend(app, &DefaultEncoderBackend)
}

pub fn stop_recording_and_encode_with_backend<R: Runtime>(
    app: &AppHandle<R>,
    encoder_backend: &dyn EncoderBackend,
) -> Result<EncodedAudio, String> {
    let state = app.state::<AppState>();

    let had_session = state.audio_session.lock().map_err(|e| e.to_string())?.take();
    if had_session.is_none() {
        return Err("No active recording session.".to_string());
    }

    let provider = state
        .config
        .lock()
        .map_err(|e| e.to_string())?
        .transcription
        .provider
        .clone();
    let capture_pcm = synthetic_capture_pcm();

    match encode_with_retry_once(&provider, &capture_pcm, encoder_backend) {
        Ok(encoded) => Ok(encoded),
        Err(message) => {
            *state.recording_state.lock().map_err(|e| e.to_string())? = RecordingState::Idle;
            emit_audio_error(
                app,
                AudioErrorPayload {
                    code: AudioErrorCode::EncodeFailed,
                    message: message.clone(),
                },
            );
            Err(message)
        }
    }
}

pub fn encode_with_retry_once(
    provider: &TranscriptionProvider,
    pcm_mono_16khz: &[f32],
    backend: &dyn EncoderBackend,
) -> Result<EncodedAudio, String> {
    let first = encode::encode_for_provider(provider, pcm_mono_16khz, backend);
    if first.is_ok() {
        return first;
    }
    encode::encode_for_provider(provider, pcm_mono_16khz, backend)
}

fn synthetic_capture_pcm() -> Vec<f32> {
    vec![0.0; 16_000]
}

fn emit_audio_warning<R: Runtime>(app: &AppHandle<R>, payload: AudioWarningPayload) {
    let _ = app.emit(AUDIO_WARNING_EVENT, payload);
}

fn emit_audio_error<R: Runtime>(app: &AppHandle<R>, payload: AudioErrorPayload) {
    let _ = app.emit(AUDIO_ERROR_EVENT, payload);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FailOnceBackend {
        failed: std::sync::Mutex<bool>,
    }

    impl EncoderBackend for FailOnceBackend {
        fn encode_opus(&self, _pcm_mono_16khz: &[f32]) -> Result<Vec<u8>, String> {
            let mut failed = self.failed.lock().unwrap();
            if !*failed {
                *failed = true;
                return Err("first failure".to_string());
            }
            Ok(b"OggSOpusHead".to_vec())
        }

        fn encode_wav(&self, _pcm_mono_16khz: &[f32]) -> Result<Vec<u8>, String> {
            Ok(b"RIFFWAVE".to_vec())
        }
    }

    struct AlwaysFailBackend;

    impl EncoderBackend for AlwaysFailBackend {
        fn encode_opus(&self, _pcm_mono_16khz: &[f32]) -> Result<Vec<u8>, String> {
            Err("encode boom".to_string())
        }
        fn encode_wav(&self, _pcm_mono_16khz: &[f32]) -> Result<Vec<u8>, String> {
            Err("encode boom".to_string())
        }
    }

    #[test]
    fn session_state_transitions() {
        let session = session::new_session("Mic A".to_string());
        assert_eq!(session.sample_rate_hz, 16_000);
        assert_eq!(session.channels, 1);
        assert_eq!(session.active_device, "Mic A");
    }

    #[test]
    fn missing_selected_device_falls_back() {
        let snapshot = DeviceSnapshot {
            input_devices: vec!["Default Mic".to_string()],
            default_input: Some("Default Mic".to_string()),
        };
        let resolved = resolve_input_device(Some("Missing Mic"), &snapshot).unwrap();
        assert_eq!(resolved.active_device, "Default Mic");
        assert_eq!(resolved.fallback_from, Some("Missing Mic".to_string()));
    }

    #[test]
    fn selected_device_is_used_when_available() {
        let snapshot = DeviceSnapshot {
            input_devices: vec!["Mic A".to_string(), "Mic B".to_string()],
            default_input: Some("Default Mic".to_string()),
        };
        let resolved = resolve_input_device(Some("Mic B"), &snapshot).unwrap();
        assert_eq!(resolved.active_device, "Mic B");
        assert_eq!(resolved.fallback_from, None);
    }

    #[test]
    fn no_device_returns_error() {
        let snapshot = DeviceSnapshot {
            input_devices: vec![],
            default_input: None,
        };
        let err = resolve_input_device(None, &snapshot).unwrap_err();
        assert!(err.contains("No microphone"));
    }

    #[test]
    fn captures_mono_16khz_contract() {
        let pcm = synthetic_capture_pcm();
        assert_eq!(pcm.len(), 16_000);
        assert_eq!(session::CAPTURE_SAMPLE_RATE_HZ, 16_000);
        assert_eq!(session::CAPTURE_CHANNELS, 1);
    }

    #[test]
    fn selects_encoder_from_provider() {
        assert_eq!(
            encode::format_for_provider(&TranscriptionProvider::Openai),
            encode::EncodedFormat::Opus
        );
        assert_eq!(
            encode::format_for_provider(&TranscriptionProvider::Local),
            encode::EncodedFormat::Wav
        );
    }

    #[test]
    fn encode_retry_once_then_fail() {
        let backend = AlwaysFailBackend;
        let err = encode_with_retry_once(&TranscriptionProvider::Openai, &[0.0; 16], &backend)
            .unwrap_err();
        assert!(err.contains("encode boom"));
    }

    #[test]
    fn wav_output_contract() {
        let backend = encode::DefaultEncoderBackend;
        let bytes = encode::encode_for_provider(&TranscriptionProvider::Local, &[0.0; 8], &backend)
            .unwrap()
            .bytes;
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
    }

    #[test]
    fn opus_output_contract() {
        let backend = encode::DefaultEncoderBackend;
        let bytes = encode::encode_for_provider(&TranscriptionProvider::Openai, &[0.0; 8], &backend)
            .unwrap()
            .bytes;
        assert_eq!(&bytes[0..4], b"OggS");
    }

    #[test]
    fn encode_retries_once_on_transient_failure() {
        let backend = FailOnceBackend {
            failed: std::sync::Mutex::new(false),
        };
        let encoded = encode_with_retry_once(&TranscriptionProvider::Openai, &[0.0; 16], &backend)
            .unwrap();
        assert_eq!(encoded.format, encode::EncodedFormat::Opus);
    }
}
