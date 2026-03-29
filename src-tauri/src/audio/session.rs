use crate::state::AudioSessionState;
use std::sync::{Arc, Mutex};
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
    AudioSessionState {
        active_device,
        sample_rate_hz: CAPTURE_SAMPLE_RATE_HZ,
        channels: CAPTURE_CHANNELS,
        level_emitter_stop: Some(level_emitter_stop),
        level_emitter_thread: Some(level_emitter_thread),
        pcm_buffer,
    }
}

#[cfg(test)]
pub fn new_session(active_device: String, pcm_buffer: Arc<Mutex<Vec<f32>>>) -> AudioSessionState {
    AudioSessionState {
        active_device,
        sample_rate_hz: CAPTURE_SAMPLE_RATE_HZ,
        channels: CAPTURE_CHANNELS,
        level_emitter_stop: None,
        level_emitter_thread: None,
        pcm_buffer,
    }
}
