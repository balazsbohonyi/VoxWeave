use crate::state::AudioSessionState;
use std::time::SystemTime;
#[cfg(not(test))]
use std::{sync::Arc, sync::atomic::AtomicBool};

pub const CAPTURE_SAMPLE_RATE_HZ: u32 = 16_000;
pub const CAPTURE_CHANNELS: u16 = 1;

#[cfg(not(test))]
pub fn new_session(
    active_device: String,
    level_emitter_stop: Arc<AtomicBool>,
    level_emitter_thread: std::thread::JoinHandle<()>,
) -> AudioSessionState {
    AudioSessionState {
        active_device,
        sample_rate_hz: CAPTURE_SAMPLE_RATE_HZ,
        channels: CAPTURE_CHANNELS,
        started_at: SystemTime::now(),
        level_emitter_stop: Some(level_emitter_stop),
        level_emitter_thread: Some(level_emitter_thread),
    }
}

#[cfg(test)]
pub fn new_session(active_device: String) -> AudioSessionState {
    AudioSessionState {
        active_device,
        sample_rate_hz: CAPTURE_SAMPLE_RATE_HZ,
        channels: CAPTURE_CHANNELS,
        started_at: SystemTime::now(),
        level_emitter_stop: None,
        level_emitter_thread: None,
    }
}

