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

fn make_opus_head() -> Vec<u8> {
    let mut h = Vec::with_capacity(19);
    h.extend_from_slice(b"OpusHead");
    h.push(1);                                     // version
    h.push(1);                                     // channels (mono)
    h.extend_from_slice(&312u16.to_le_bytes());    // pre-skip
    h.extend_from_slice(&16_000u32.to_le_bytes()); // input sample rate
    h.extend_from_slice(&0i16.to_le_bytes());      // output gain
    h.push(0);                                     // channel mapping family
    h
}

fn make_opus_tags() -> Vec<u8> {
    let vendor = b"VoxFlow";
    let mut t = Vec::new();
    t.extend_from_slice(b"OpusTags");
    t.extend_from_slice(&(vendor.len() as u32).to_le_bytes());
    t.extend_from_slice(vendor);
    t.extend_from_slice(&0u32.to_le_bytes()); // 0 user comments
    t
}

impl EncoderBackend for DefaultEncoderBackend {
    fn encode_opus(&self, pcm_mono_16khz: &[f32]) -> Result<Vec<u8>, String> {
        use audiopus::coder::Encoder;
        use audiopus::{Application, Bitrate, Channels, SampleRate};
        use ogg::writing::{PacketWriteEndInfo, PacketWriter};
        use std::io::Cursor;

        const FRAME_SIZE: usize = 320; // 20ms at 16kHz mono
        const SERIAL: u32 = 1;

        let mut encoder = Encoder::new(SampleRate::Hz16000, Channels::Mono, Application::Voip)
            .map_err(|e| format!("{e:?}"))?;
        encoder
            .set_bitrate(Bitrate::BitsPerSecond(24_000))
            .map_err(|e| format!("{e:?}"))?;

        let mut output = Cursor::new(Vec::new());
        let mut pw = PacketWriter::new(&mut output);

        // Identification headers — MUST precede audio data (RFC 7845)
        pw.write_packet(make_opus_head(), SERIAL, PacketWriteEndInfo::EndPage, 0)
            .map_err(|e| e.to_string())?;
        pw.write_packet(make_opus_tags(), SERIAL, PacketWriteEndInfo::EndPage, 0)
            .map_err(|e| e.to_string())?;

        // Encode audio frames. If pcm is empty, produce no audio packets (headers-only is valid).
        let chunks: Vec<&[f32]> = pcm_mono_16khz.chunks(FRAME_SIZE).collect();
        let num_chunks = chunks.len();
        let mut granule: u64 = 0;
        let mut out_buf = vec![0u8; 4000];

        for (i, chunk) in chunks.iter().enumerate() {
            let frame: Vec<f32> = if chunk.len() == FRAME_SIZE {
                chunk.to_vec()
            } else {
                let mut padded = chunk.to_vec();
                padded.resize(FRAME_SIZE, 0.0);
                padded
            };
            let n = encoder
                .encode_float(&frame, &mut out_buf)
                .map_err(|e| format!("{e:?}"))?;
            granule += FRAME_SIZE as u64;
            let is_last = i == num_chunks - 1;
            let end_info = if is_last {
                PacketWriteEndInfo::EndStream
            } else {
                PacketWriteEndInfo::NormalPacket
            };
            pw.write_packet(out_buf[..n].to_vec(), SERIAL, end_info, granule)
                .map_err(|e| e.to_string())?;
        }

        Ok(output.into_inner())
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
        TranscriptionProvider::Local | TranscriptionProvider::Openai => EncodedFormat::Wav,
        TranscriptionProvider::Groq => EncodedFormat::Opus,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TranscriptionProvider;

    // --- Opus encoding tests (Task 1: real OggS output) ---

    #[test]
    fn opus_output_is_valid_ogg() {
        // 2 seconds at 16kHz = 32000 samples
        let pcm = vec![0.1f32; 32_000];
        let backend = DefaultEncoderBackend;
        let bytes = backend.encode_opus(&pcm).expect("encode_opus should succeed");
        assert_eq!(
            &bytes[0..4],
            b"OggS",
            "Output must start with OggS magic bytes"
        );
    }

    #[test]
    fn ogg_contains_opushead() {
        let pcm = vec![0.1f32; 32_000];
        let backend = DefaultEncoderBackend;
        let bytes = backend.encode_opus(&pcm).expect("encode_opus should succeed");
        let window = b"OpusHead";
        let found = bytes.windows(window.len()).any(|w| w == window);
        assert!(found, "Output must contain OpusHead identification packet");
    }

    #[test]
    fn opus_encodes_short_pcm() {
        // Less than one 20ms frame (320 samples at 16kHz): must zero-pad and succeed
        let pcm = vec![0.05f32; 100];
        let backend = DefaultEncoderBackend;
        let result = backend.encode_opus(&pcm);
        assert!(result.is_ok(), "encode_opus must succeed on short input");
        let bytes = result.unwrap();
        assert_eq!(
            &bytes[0..4],
            b"OggS",
            "Short input must still produce valid OggS output"
        );
    }

    #[test]
    fn opus_encodes_empty_pcm() {
        // Empty slice: must produce headers-only OggS output without panic
        let pcm: &[f32] = &[];
        let backend = DefaultEncoderBackend;
        let result = backend.encode_opus(pcm);
        assert!(result.is_ok(), "encode_opus must succeed on empty input");
        let bytes = result.unwrap();
        assert_eq!(
            &bytes[0..4],
            b"OggS",
            "Empty input must still produce valid OggS output"
        );
    }

    #[test]
    fn test_format_for_provider_openai_returns_wav() {
        assert_eq!(
            format_for_provider(&TranscriptionProvider::Openai),
            EncodedFormat::Wav,
            "OpenAI requires WAV format (not Opus/Ogg)"
        );
    }

    #[test]
    fn test_format_for_provider_groq_returns_opus() {
        assert_eq!(
            format_for_provider(&TranscriptionProvider::Groq),
            EncodedFormat::Opus,
            "Groq accepts Opus/Ogg format"
        );
    }

    #[test]
    fn test_format_for_provider_local_returns_wav() {
        assert_eq!(
            format_for_provider(&TranscriptionProvider::Local),
            EncodedFormat::Wav,
            "Local whisper.cpp requires WAV format"
        );
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

