use crate::state::AudioSessionState;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
#[cfg(not(test))]
use std::sync::atomic::AtomicBool;

pub const CAPTURE_SAMPLE_RATE_HZ: u32 = 16_000;
pub const CAPTURE_CHANNELS: u16 = 1;

#[cfg(not(test))]
pub fn new_session(
    active_device: String,
    level_emitter_stop: Arc<AtomicBool>,
    level_emitter_thread: std::thread::JoinHandle<()>,
    pcm_buffer: Arc<Mutex<Vec<f32>>>,
) -> AudioSessionState {
    // Clone level_emitter_stop so both level emitter and PCM accumulation
    // share the same AtomicBool — one stop flag signals both paths.
    let pcm_emitter_stop = Arc::clone(&level_emitter_stop);
    AudioSessionState {
        active_device,
        sample_rate_hz: CAPTURE_SAMPLE_RATE_HZ,
        channels: CAPTURE_CHANNELS,
        started_at: SystemTime::now(),
        level_emitter_stop: Some(level_emitter_stop),
        level_emitter_thread: Some(level_emitter_thread),
        pcm_buffer,
        pcm_emitter_stop: Some(pcm_emitter_stop),
    }
}

#[cfg(test)]
pub fn new_session(active_device: String, pcm_buffer: Arc<Mutex<Vec<f32>>>) -> AudioSessionState {
    AudioSessionState {
        active_device,
        sample_rate_hz: CAPTURE_SAMPLE_RATE_HZ,
        channels: CAPTURE_CHANNELS,
        started_at: SystemTime::now(),
        level_emitter_stop: None,
        level_emitter_thread: None,
        pcm_buffer,
        pcm_emitter_stop: None,
    }
}
