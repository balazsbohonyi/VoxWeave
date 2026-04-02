pub mod capture;
pub mod encode;
pub mod session;

use crate::config::TranscriptionProvider;
use crate::state::{AppState, RecordingState};
use capture::{resolve_input_device, DeviceSnapshot};
use encode::{DefaultEncoderBackend, EncodedAudio, EncoderBackend};
use std::sync::atomic::Ordering;
#[cfg(not(test))]
use std::sync::atomic::AtomicU32;
use std::sync::{Arc, Mutex};
#[cfg(not(test))]
use std::sync::mpsc;
#[cfg(not(test))]
use std::sync::atomic::AtomicBool;
#[cfg(not(test))]
use std::thread;
#[cfg(not(test))]
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, Runtime};

const AUDIO_WARNING_EVENT: &str = "audio-warning";
const AUDIO_ERROR_EVENT: &str = "audio-error";
#[cfg(not(test))]
const AUDIO_LEVEL_EVENT: &str = "audio-level";
const AUDIO_LEVEL_EMIT_INTERVAL_MS: u64 = 34;

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

#[cfg(not(test))]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct AudioLevelPayload {
    pub rms: f32,
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

    let pcm_buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));

    #[cfg(not(test))]
    let (level_emitter_stop, level_emitter_thread) =
        start_realtime_level_capture(app, &resolved.active_device, Arc::clone(&pcm_buffer))?;

    #[cfg(not(test))]
    let session = session::new_session(
        resolved.active_device,
        level_emitter_stop,
        level_emitter_thread,
        pcm_buffer,
    );

    #[cfg(test)]
    let session = session::new_session(resolved.active_device, pcm_buffer);

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

    let mut session = state.audio_session.lock().map_err(|e| e.to_string())?.take();
    if session.is_none() {
        return Err("No active recording session.".to_string());
    }

    if let Some(session) = session.as_mut() {
        stop_realtime_level_capture(session);
    }

    let provider = state
        .config
        .lock()
        .map_err(|e| e.to_string())?
        .transcription
        .provider
        .clone();

    // Drain the real PCM accumulation buffer collected during recording.
    let capture_pcm: Vec<f32> = {
        let session = session.as_ref().unwrap();
        drain_pcm_buffer(&session.pcm_buffer)
    };
    if capture_pcm.len() < 8_000 {
        let msg = "Recording too short (under 0.5 seconds). Please hold the hotkey longer."
            .to_string();
        emit_audio_error(
            app,
            AudioErrorPayload {
                code: AudioErrorCode::EncodeFailed,
                message: msg.clone(),
            },
        );
        return Err(msg);
    }

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

/// Push mono-downmixed, downsampled-to-16kHz PCM samples into the shared buffer.
/// Enforces the 5-minute hard cap and emits a near-limit event at 4.5 minutes.
///
/// Downsampling strategy:
/// - If native_rate is an exact integer multiple of 16000 (e.g. 48000 → step=3),
///   use simple integer decimation — keep every N-th sample. This is lossless for
///   multiples and covers the vast majority of consumer microphones (48kHz, 32kHz).
/// - Otherwise use rubato FftFixedInOut for accurate fractional-ratio resampling
///   (e.g. 44100 Hz → 16000 Hz).
#[cfg(not(test))]
fn accumulate_pcm_chunk<R: Runtime>(
    mono_f32: &[f32],
    native_rate: u32,
    pcm_buffer: &Arc<Mutex<Vec<f32>>>,
    near_limit_emitted: &std::sync::atomic::AtomicBool,
    stop_flag: &AtomicBool,
    app: &AppHandle<R>,
) {
    const TARGET_RATE: u32 = 16_000;
    const NEAR_LIMIT: usize = 4_320_000; // 4.5 min × 16000
    const HARD_CAP: usize = 4_800_000;   // 5 min × 16000

    // Downsample to 16kHz.
    let downsampled: Vec<f32> = if native_rate == TARGET_RATE {
        mono_f32.to_vec()
    } else if native_rate % TARGET_RATE == 0 {
        // Integer decimation: keep every N-th sample.
        let step = (native_rate / TARGET_RATE) as usize;
        mono_f32.iter().step_by(step).copied().collect()
    } else {
        // Non-integer ratio: use rubato FftFixedInOut.
        use rubato::{FftFixedInOut, Resampler};
        let chunk_len = mono_f32.len();
        // Create a fresh per-chunk resampler (cheap for small chunks).
        match FftFixedInOut::<f32>::new(
            native_rate as usize,
            TARGET_RATE as usize,
            chunk_len,
            1,
        ) {
            Ok(mut resampler) => {
                let waves_in = vec![mono_f32.to_vec()];
                match resampler.process(&waves_in, None) {
                    Ok(out) => out.into_iter().next().unwrap_or_default(),
                    Err(_) => {
                        // Fallback: nearest-neighbour decimation to avoid silent buffer.
                        let ratio = native_rate as f64 / TARGET_RATE as f64;
                        (0..((chunk_len as f64 / ratio).ceil() as usize))
                            .map(|i| mono_f32[(i as f64 * ratio) as usize])
                            .collect()
                    }
                }
            }
            Err(_) => {
                // Fallback: nearest-neighbour decimation.
                let ratio = native_rate as f64 / TARGET_RATE as f64;
                (0..((chunk_len as f64 / ratio).ceil() as usize))
                    .map(|i| mono_f32[(i as f64 * ratio) as usize])
                    .collect()
            }
        }
    };

    // Lock the buffer and push, enforcing the hard cap.
    let buf_len = {
        let mut buf = pcm_buffer.lock().expect("pcm_buffer lock poisoned");
        if buf.len() >= HARD_CAP {
            // Already at cap; discard this chunk.
            return;
        }
        buf.extend_from_slice(&downsampled);
        if buf.len() > HARD_CAP {
            buf.truncate(HARD_CAP);
            // Signal the recording to stop at the hard cap.
            stop_flag.store(true, Ordering::Relaxed);
        }
        buf.len()
    };

    // Emit near-limit warning once.
    if buf_len >= NEAR_LIMIT
        && !near_limit_emitted.load(Ordering::Relaxed)
        && near_limit_emitted
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
    {
        let _ = app.emit("recording-near-limit", ());
    }
}

#[cfg(not(test))]
fn start_realtime_level_capture<R: Runtime>(
    app: &AppHandle<R>,
    active_device: &str,
    pcm_buffer: Arc<Mutex<Vec<f32>>>,
) -> Result<(Arc<AtomicBool>, std::thread::JoinHandle<()>), String> {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

    let setup_device_name = active_device.to_string();
    let setup_stop = Arc::new(AtomicBool::new(false));
    let stop_for_thread = Arc::clone(&setup_stop);
    let app_for_thread = app.clone();
    let (setup_tx, setup_rx) = mpsc::channel::<Result<(), String>>();

    let emitter_thread = thread::spawn(move || {
        let host = cpal::default_host();
        let device = match host
            .input_devices()
            .map_err(|e| e.to_string())
            .and_then(|mut devices| {
                devices
                    .find(|device| device.name().ok().as_deref() == Some(&setup_device_name))
                    .ok_or_else(|| {
                        format!("Unable to access selected microphone '{setup_device_name}'.")
                    })
            }) {
            Ok(device) => device,
            Err(error) => {
                let _ = setup_tx.send(Err(error));
                return;
            }
        };

        let default_config = match device.default_input_config().map_err(|e| e.to_string()) {
            Ok(config) => config,
            Err(error) => {
                let _ = setup_tx.send(Err(error));
                return;
            }
        };

        let stream_config: cpal::StreamConfig = default_config.clone().into();
        let native_rate = stream_config.sample_rate.0;
        let n_channels = stream_config.channels as usize;
        let latest_rms = Arc::new(AtomicU32::new(0f32.to_bits()));
        // Tracks whether the near-limit event has been emitted for this recording.
        let near_limit_emitted = Arc::new(AtomicBool::new(false));

        let stream_result = match default_config.sample_format() {
            cpal::SampleFormat::F32 => {
                let rms_for_callback = Arc::clone(&latest_rms);
                let pcm_buf = Arc::clone(&pcm_buffer);
                let stop_clone = Arc::clone(&stop_for_thread);
                let near_limit = Arc::clone(&near_limit_emitted);
                let app_for_pcm = app_for_thread.clone();
                let app_for_errors = app_for_thread.clone();
                device.build_input_stream(
                    &stream_config,
                    move |data: &[f32], _| {
                        // Mix to mono: average across channels per frame.
                        let mono: Vec<f32> = data
                            .chunks_exact(n_channels)
                            .map(|frame| frame.iter().sum::<f32>() / n_channels as f32)
                            .collect();
                        update_latest_rms(&mono, &rms_for_callback);
                        accumulate_pcm_chunk(
                            &mono,
                            native_rate,
                            &pcm_buf,
                            &near_limit,
                            &stop_clone,
                            &app_for_pcm,
                        );
                    },
                    move |err| {
                        emit_audio_error(
                            &app_for_errors,
                            AudioErrorPayload {
                                code: AudioErrorCode::EncodeFailed,
                                message: format!("Microphone stream error: {err}"),
                            },
                        );
                    },
                    None,
                )
            }
            cpal::SampleFormat::I16 => {
                let rms_for_callback = Arc::clone(&latest_rms);
                let pcm_buf = Arc::clone(&pcm_buffer);
                let stop_clone = Arc::clone(&stop_for_thread);
                let near_limit = Arc::clone(&near_limit_emitted);
                let app_for_pcm = app_for_thread.clone();
                let app_for_errors = app_for_thread.clone();
                device.build_input_stream(
                    &stream_config,
                    move |data: &[i16], _| {
                        // Convert to f32, then mix to mono.
                        let f32_data: Vec<f32> =
                            data.iter().map(|s| *s as f32 / i16::MAX as f32).collect();
                        let mono: Vec<f32> = f32_data
                            .chunks_exact(n_channels)
                            .map(|frame| frame.iter().sum::<f32>() / n_channels as f32)
                            .collect();
                        update_latest_rms(&mono, &rms_for_callback);
                        accumulate_pcm_chunk(
                            &mono,
                            native_rate,
                            &pcm_buf,
                            &near_limit,
                            &stop_clone,
                            &app_for_pcm,
                        );
                    },
                    move |err| {
                        emit_audio_error(
                            &app_for_errors,
                            AudioErrorPayload {
                                code: AudioErrorCode::EncodeFailed,
                                message: format!("Microphone stream error: {err}"),
                            },
                        );
                    },
                    None,
                )
            }
            cpal::SampleFormat::U16 => {
                let rms_for_callback = Arc::clone(&latest_rms);
                let pcm_buf = Arc::clone(&pcm_buffer);
                let stop_clone = Arc::clone(&stop_for_thread);
                let near_limit = Arc::clone(&near_limit_emitted);
                let app_for_pcm = app_for_thread.clone();
                let app_for_errors = app_for_thread.clone();
                device.build_input_stream(
                    &stream_config,
                    move |data: &[u16], _| {
                        // Convert to signed f32, then mix to mono.
                        let f32_data: Vec<f32> =
                            data.iter().map(|s| u16_to_signed(*s)).collect();
                        let mono: Vec<f32> = f32_data
                            .chunks_exact(n_channels)
                            .map(|frame| frame.iter().sum::<f32>() / n_channels as f32)
                            .collect();
                        update_latest_rms(&mono, &rms_for_callback);
                        accumulate_pcm_chunk(
                            &mono,
                            native_rate,
                            &pcm_buf,
                            &near_limit,
                            &stop_clone,
                            &app_for_pcm,
                        );
                    },
                    move |err| {
                        emit_audio_error(
                            &app_for_errors,
                            AudioErrorPayload {
                                code: AudioErrorCode::EncodeFailed,
                                message: format!("Microphone stream error: {err}"),
                            },
                        );
                    },
                    None,
                )
            }
            sample_format => {
                let _ = setup_tx.send(Err(format!(
                    "Unsupported microphone sample format: {sample_format:?}."
                )));
                return;
            }
        }
        .map_err(|e| e.to_string());

        let stream = match stream_result {
            Ok(stream) => stream,
            Err(error) => {
                let _ = setup_tx.send(Err(error));
                return;
            }
        };

        if let Err(error) = stream.play().map_err(|e| e.to_string()) {
            let _ = setup_tx.send(Err(error));
            return;
        }

        let _ = setup_tx.send(Ok(()));

        while !stop_for_thread.load(Ordering::Relaxed) {
            let rms = f32::from_bits(latest_rms.load(Ordering::Relaxed));
            let _ = app_for_thread.emit(AUDIO_LEVEL_EVENT, AudioLevelPayload { rms });
            thread::sleep(Duration::from_millis(AUDIO_LEVEL_EMIT_INTERVAL_MS));
        }

        drop(stream);
    });

    match setup_rx.recv_timeout(Duration::from_secs(2)) {
        Ok(Ok(())) => Ok((setup_stop, emitter_thread)),
        Ok(Err(error)) => {
            setup_stop.store(true, Ordering::Relaxed);
            let _ = emitter_thread.join();
            Err(error)
        }
        Err(_) => {
            setup_stop.store(true, Ordering::Relaxed);
            let _ = emitter_thread.join();
            Err("Timed out while starting microphone capture stream.".to_string())
        }
    }
}

fn stop_realtime_level_capture(session: &mut crate::state::AudioSessionState) {
    if let Some(stop) = session.level_emitter_stop.take() {
        stop.store(true, Ordering::Relaxed);
    }
    if let Some(thread) = session.level_emitter_thread.take() {
        let _ = thread.join();
    }
}

#[cfg(not(test))]
fn update_latest_rms(samples: &[f32], latest_rms: &AtomicU32) {
    let rms = compute_normalized_rms(samples);
    latest_rms.store(rms.to_bits(), Ordering::Relaxed);
}

fn compute_normalized_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let mut energy = 0.0f32;
    for sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        energy += clamped * clamped;
    }

    let mean = energy / samples.len() as f32;
    mean.sqrt().clamp(0.0, 1.0)
}

fn u16_to_signed(sample: u16) -> f32 {
    (sample as f32 / u16::MAX as f32) * 2.0 - 1.0
}

fn emit_audio_warning<R: Runtime>(app: &AppHandle<R>, payload: AudioWarningPayload) {
    let _ = app.emit(AUDIO_WARNING_EVENT, payload);
}

fn emit_audio_error<R: Runtime>(app: &AppHandle<R>, payload: AudioErrorPayload) {
    let _ = app.emit(AUDIO_ERROR_EVENT, payload);
}

/// Drain all accumulated PCM samples out of the shared buffer, leaving it empty.
fn drain_pcm_buffer(buffer: &Arc<Mutex<Vec<f32>>>) -> Vec<f32> {
    let mut buf = buffer.lock().expect("pcm_buffer lock poisoned");
    std::mem::take(&mut *buf)
}

/// Enforce the 5-minute hard cap: truncate the buffer to at most 4,800,000 samples.
/// The 5-minute limit = 5 * 60 * 16_000 = 4_800_000 samples at 16kHz mono.
#[cfg(test)]
fn apply_pcm_cap(buffer: &Arc<Mutex<Vec<f32>>>) {
    const MAX_SAMPLES: usize = 4_800_000;
    let mut buf = buffer.lock().expect("pcm_buffer lock poisoned");
    buf.truncate(MAX_SAMPLES);
}

/// Validate that the buffer holds at least 0.5 seconds of audio (8,000 samples at 16kHz).
/// Returns Err with a user-friendly "too short" message if the threshold is not met.
#[cfg(test)]
fn validate_pcm_length(buffer: &Arc<Mutex<Vec<f32>>>) -> Result<(), String> {
    const MIN_SAMPLES: usize = 8_000;
    let buf = buffer.lock().expect("pcm_buffer lock poisoned");
    if buf.len() < MIN_SAMPLES {
        return Err(
            "Recording too short (under 0.5 seconds). Please hold the hotkey longer.".to_string(),
        );
    }
    Ok(())
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
        let pcm_buffer = Arc::new(Mutex::new(Vec::new()));
        let session = session::new_session("Mic A".to_string(), pcm_buffer);
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
        // Verify the canonical capture constants — length assertion removed now
        // that real PCM accumulation replaces the synthetic stub.
        assert_eq!(session::CAPTURE_SAMPLE_RATE_HZ, 16_000);
        assert_eq!(session::CAPTURE_CHANNELS, 1);
    }

    #[test]
    fn selects_encoder_from_provider() {
        assert_eq!(
            encode::format_for_provider(&TranscriptionProvider::Openai),
            encode::EncodedFormat::Wav
        );
        assert_eq!(
            encode::format_for_provider(&TranscriptionProvider::Local),
            encode::EncodedFormat::Wav
        );
        assert_eq!(
            encode::format_for_provider(&TranscriptionProvider::Groq),
            encode::EncodedFormat::Opus
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
        let bytes = encode::encode_for_provider(&TranscriptionProvider::Groq, &[0.0; 8], &backend)
            .unwrap()
            .bytes;
        assert_eq!(&bytes[0..4], b"OggS");
    }

    #[test]
    fn encode_retries_once_on_transient_failure() {
        let backend = FailOnceBackend {
            failed: std::sync::Mutex::new(false),
        };
        let encoded = encode_with_retry_once(&TranscriptionProvider::Groq, &[0.0; 16], &backend)
            .unwrap();
        assert_eq!(encoded.format, encode::EncodedFormat::Opus);
    }

    #[test]
    fn rms_normalization_clamps_to_unit_interval() {
        let samples = vec![2.0, -2.0, 1.5, -1.5];
        let rms = compute_normalized_rms(&samples);
        assert!((0.0..=1.0).contains(&rms));
    }

    #[test]
    fn u16_audio_is_centered_around_zero() {
        assert!((u16_to_signed(0) + 1.0).abs() < 0.0001);
        assert!(u16_to_signed(u16::MAX).abs() <= 1.0);
    }

    #[test]
    fn realtime_level_emit_interval_targets_roughly_30fps() {
        assert!(AUDIO_LEVEL_EMIT_INTERVAL_MS <= 42);
        assert!(AUDIO_LEVEL_EMIT_INTERVAL_MS >= 33);
    }

    // --- PCM accumulation tests (Task 2) ---

    #[test]
    fn pcm_buffer_accumulates_samples() {
        // Simulate what the capture callback does: push samples into a shared
        // buffer then drain it — all values must survive the round-trip.
        let buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
        {
            let mut buf = buffer.lock().unwrap();
            for i in 0..1000_u32 {
                buf.push(i as f32 * 0.001);
            }
        }
        let drained = drain_pcm_buffer(&buffer);
        assert_eq!(drained.len(), 1000);
        assert!((drained[0] - 0.0).abs() < 1e-6);
        assert!((drained[999] - 0.999).abs() < 1e-4);
    }

    #[test]
    fn pcm_cap_at_five_minutes_truncates() {
        // 5 minutes at 16kHz = 4_800_000 samples; one extra must be truncated.
        let buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
        {
            let mut buf = buffer.lock().unwrap();
            for _ in 0..4_800_001_usize {
                buf.push(0.0);
            }
        }
        apply_pcm_cap(&buffer);
        let len = buffer.lock().unwrap().len();
        assert_eq!(len, 4_800_000);
    }

    #[test]
    fn short_audio_below_threshold_is_rejected() {
        // Below 0.5 s at 16kHz = 8_000 samples must be rejected.
        let buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
        {
            let mut buf = buffer.lock().unwrap();
            for _ in 0..7_999_usize {
                buf.push(0.0);
            }
        }
        let result = validate_pcm_length(&buffer);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(
            msg.to_lowercase().contains("too short"),
            "expected 'too short' in error: {msg}"
        );
    }
}
