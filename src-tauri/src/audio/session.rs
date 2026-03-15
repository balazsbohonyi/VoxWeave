use crate::state::AudioSessionState;
use std::time::SystemTime;

pub const CAPTURE_SAMPLE_RATE_HZ: u32 = 16_000;
pub const CAPTURE_CHANNELS: u16 = 1;

pub fn new_session(active_device: String) -> AudioSessionState {
    AudioSessionState {
        active_device,
        sample_rate_hz: CAPTURE_SAMPLE_RATE_HZ,
        channels: CAPTURE_CHANNELS,
        started_at: SystemTime::now(),
    }
}

