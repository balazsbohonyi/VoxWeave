use crate::state::AudioSessionState;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;

pub const CAPTURE_SAMPLE_RATE_HZ: u32 = 16_000;
pub const CAPTURE_CHANNELS: u16 = 1;

pub fn new_session(
    active_device: String,
    level_emitter_stop: Option<Arc<AtomicBool>>,
    level_emitter_thread: Option<std::thread::JoinHandle<()>>,
    pcm_buffer: Arc<Mutex<Vec<f32>>>,
) -> AudioSessionState {
    AudioSessionState {
        active_device,
        sample_rate_hz: CAPTURE_SAMPLE_RATE_HZ,
        channels: CAPTURE_CHANNELS,
        level_emitter_stop,
        level_emitter_thread,
        pcm_buffer,
    }
}
