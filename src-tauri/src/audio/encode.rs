use crate::config::TranscriptionProvider;

#[derive(Debug, Clone, PartialEq)]
pub enum EncodedFormat {
    Opus,
    Wav,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EncodedAudio {
    pub format: EncodedFormat,
    pub mime_type: &'static str,
    pub bytes: Vec<u8>,
}

pub trait EncoderBackend {
    fn encode_opus(&self, pcm_mono_16khz: &[f32]) -> Result<Vec<u8>, String>;
    fn encode_wav(&self, pcm_mono_16khz: &[f32]) -> Result<Vec<u8>, String>;
}

pub struct DefaultEncoderBackend;

impl EncoderBackend for DefaultEncoderBackend {
    fn encode_opus(&self, pcm_mono_16khz: &[f32]) -> Result<Vec<u8>, String> {
        // Minimal Ogg/Opus-like envelope for contract testing in early phases.
        let mut bytes = b"OggSOpusHead".to_vec();
        bytes.extend_from_slice(&(pcm_mono_16khz.len() as u32).to_le_bytes());
        Ok(bytes)
    }

    fn encode_wav(&self, pcm_mono_16khz: &[f32]) -> Result<Vec<u8>, String> {
        let sample_count = pcm_mono_16khz.len() as u32;
        let data_size = sample_count * 2;
        let chunk_size = 36 + data_size;
        let mut bytes = Vec::with_capacity((44 + data_size) as usize);
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&chunk_size.to_le_bytes());
        bytes.extend_from_slice(b"WAVE");
        bytes.extend_from_slice(b"fmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes()); // PCM fmt chunk
        bytes.extend_from_slice(&1u16.to_le_bytes()); // PCM
        bytes.extend_from_slice(&1u16.to_le_bytes()); // mono
        bytes.extend_from_slice(&16_000u32.to_le_bytes());
        bytes.extend_from_slice(&(16_000u32 * 2).to_le_bytes()); // byte rate
        bytes.extend_from_slice(&2u16.to_le_bytes()); // block align
        bytes.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&data_size.to_le_bytes());

        for sample in pcm_mono_16khz {
            let clamped = sample.clamp(-1.0, 1.0);
            let as_i16 = (clamped * i16::MAX as f32) as i16;
            bytes.extend_from_slice(&as_i16.to_le_bytes());
        }
        Ok(bytes)
    }
}

pub fn format_for_provider(provider: &TranscriptionProvider) -> EncodedFormat {
    match provider {
        TranscriptionProvider::Local => EncodedFormat::Wav,
        TranscriptionProvider::Openai
        | TranscriptionProvider::Groq
        | TranscriptionProvider::Openrouter => EncodedFormat::Opus,
    }
}

pub fn encode_for_provider(
    provider: &TranscriptionProvider,
    pcm_mono_16khz: &[f32],
    backend: &dyn EncoderBackend,
) -> Result<EncodedAudio, String> {
    match format_for_provider(provider) {
        EncodedFormat::Opus => Ok(EncodedAudio {
            format: EncodedFormat::Opus,
            mime_type: "audio/ogg",
            bytes: backend.encode_opus(pcm_mono_16khz)?,
        }),
        EncodedFormat::Wav => Ok(EncodedAudio {
            format: EncodedFormat::Wav,
            mime_type: "audio/wav",
            bytes: backend.encode_wav(pcm_mono_16khz)?,
        }),
    }
}

