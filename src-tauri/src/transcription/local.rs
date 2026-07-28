// Local transcription provider — offline Whisper inference via whisper-rs.

// ---------------------------------------------------------------------------
// WAV helper — always compiled (no native dependency)
// ---------------------------------------------------------------------------

/// Convert 16-bit PCM WAV bytes (44-byte header, mono, 16 kHz) to f32 samples
/// normalized to [-1.0, 1.0].
///
/// Returns an error if `wav_bytes` is shorter than the 44-byte header.
pub fn wav_bytes_to_f32(wav_bytes: &[u8]) -> Result<Vec<f32>, String> {
    const HEADER_LEN: usize = 44;
    if wav_bytes.len() < HEADER_LEN {
        return Err(format!(
            "WAV data too short: {} bytes (minimum is {} for the header)",
            wav_bytes.len(),
            HEADER_LEN
        ));
    }

    let pcm_bytes = &wav_bytes[HEADER_LEN..];
    // Each sample is 2 bytes (i16 little-endian)
    let sample_count = pcm_bytes.len() / 2;
    let mut samples = Vec::with_capacity(sample_count);

    for chunk in pcm_bytes.chunks_exact(2) {
        let sample_i16 = i16::from_le_bytes([chunk[0], chunk[1]]);
        samples.push(sample_i16 as f32 / 32768.0_f32);
    }

    Ok(samples)
}

/// Remove known Whisper non-speech annotations from a local transcript.
///
/// Only complete, allowlisted square-bracket annotations are removed. Other
/// bracketed content is copied verbatim, while whitespace outside those spans
/// is collapsed so a removed annotation remains a word boundary.
pub(crate) fn normalize_local_transcript(transcript: &str) -> String {
    fn is_non_speech_annotation(label: &str) -> bool {
        let normalized = label
            .split(|c: char| c.is_whitespace() || matches!(c, '_' | '-'))
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();

        matches!(
            normalized.as_str(),
            "blank audio"
                | "pause"
                | "silence"
                | "no speech"
                | "music"
                | "applause"
                | "laughter"
                | "noise"
                | "inaudible"
        )
    }

    fn push_pending_space(output: &mut String, pending_space: &mut bool) {
        if *pending_space && !output.is_empty() {
            output.push(' ');
        }
        *pending_space = false;
    }

    let mut output = String::with_capacity(transcript.len());
    let mut pending_space = false;
    let mut offset = 0;

    while offset < transcript.len() {
        let remaining = &transcript[offset..];
        if remaining.starts_with('[') {
            let Some(relative_end) = remaining.find(']') else {
                push_pending_space(&mut output, &mut pending_space);
                output.push_str(remaining);
                break;
            };

            let span_end = relative_end + 1;
            let label = &remaining[1..relative_end];
            if is_non_speech_annotation(label) {
                pending_space = true;
            } else {
                push_pending_space(&mut output, &mut pending_space);
                output.push_str(&remaining[..span_end]);
            }
            offset += span_end;
            continue;
        }

        let ch = remaining
            .chars()
            .next()
            .expect("offset always points inside transcript");
        if ch.is_whitespace() {
            pending_space = true;
        } else {
            push_pending_space(&mut output, &mut pending_space);
            output.push(ch);
        }
        offset += ch.len_utf8();
    }

    output
}

// ---------------------------------------------------------------------------
// LocalProvider — requires whisper-rs native library (CMake + MSVC)
// ---------------------------------------------------------------------------

mod provider_impl {
    use super::{normalize_local_transcript, wav_bytes_to_f32};
    use crate::audio::encode::{EncodedAudio, EncodedFormat};
    use crate::config::TranscriptionConfig;
    use crate::transcription::provider::{TranscriptionError, TranscriptionProviderTrait};

    /// Transcription provider that runs Whisper inference locally via whisper-rs.
    ///
    /// Inference runs on a blocking thread (via `tokio::task::spawn_blocking`) so
    /// that CPU-bound work never starves the Tokio async executor.
    pub struct LocalProvider {
        pub(super) model_path: String,
    }

    impl LocalProvider {
        pub fn new(model_path: String) -> Self {
            Self { model_path }
        }
    }

    #[async_trait::async_trait]
    impl TranscriptionProviderTrait for LocalProvider {
        async fn transcribe(
            &self,
            audio: &EncodedAudio,
            config: &TranscriptionConfig,
        ) -> Result<String, TranscriptionError> {
            // Validate: WAV format only
            if !matches!(audio.format, EncodedFormat::Wav) {
                return Err(TranscriptionError::Network {
                    message: "LocalProvider requires WAV audio format".to_string(),
                });
            }

            // Check model file exists on disk
            let model_path = self.model_path.clone();
            if model_path.is_empty() || !std::path::Path::new(&model_path).exists() {
                let msg = if model_path.is_empty() {
                    "No local model path configured. Please select a model in Settings."
                        .to_string()
                } else {
                    format!("Model file not found: {}", model_path)
                };
                return Err(TranscriptionError::ModelMissing { message: msg });
            }

            // Convert WAV bytes to f32 samples
            let pcm_f32 =
                wav_bytes_to_f32(&audio.bytes).map_err(|e| TranscriptionError::Network {
                    message: format!("WAV decode error: {e}"),
                })?;

            // Extract language hint before moving into blocking closure
            let language = if config.language.is_empty() {
                None
            } else {
                Some(config.language.clone())
            };

            // Run inference on a blocking thread — never block the Tokio executor
            // with CPU-bound whisper.cpp work.
            let result = tokio::task::spawn_blocking(move || {
                use whisper_rs::{
                    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters,
                };

                // Load the model
                let ctx = WhisperContext::new_with_params(
                    &model_path,
                    WhisperContextParameters::default(),
                )
                .map_err(|e| TranscriptionError::ModelLoadFailed {
                    message: format!("Failed to load model '{}': {:?}", model_path, e),
                })?;

                // Create inference state
                let mut state = ctx.create_state().map_err(|e| TranscriptionError::Network {
                    message: format!("Failed to create whisper state: {:?}", e),
                })?;

                // Configure inference parameters
                let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
                if let Some(ref lang) = language {
                    params.set_language(Some(lang));
                }
                params.set_print_progress(false);
                params.set_print_realtime(false);
                params.set_print_timestamps(false);

                // Run transcription
                state.full(params, &pcm_f32).map_err(|e| TranscriptionError::Network {
                    message: format!("Whisper inference failed: {:?}", e),
                })?;

                // Collect segment text
                let n_segments = state.full_n_segments();

                let mut text = String::new();
                for i in 0..n_segments {
                    if let Some(segment) = state.get_segment(i) {
                        if let Ok(s) = segment.to_str() {
                            text.push_str(s);
                        }
                    }
                }

                Ok::<String, TranscriptionError>(normalize_local_transcript(&text))
            })
            .await
            .map_err(|e| TranscriptionError::Network {
                message: format!("Blocking task panicked: {e}"),
            })?;

            result
        }
    }
}

pub use provider_impl::LocalProvider;

// ---------------------------------------------------------------------------
// Tests — wav helper tests run unconditionally (no native dependency)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transcription::provider::TranscriptionError;

    /// Build a minimal valid WAV buffer: 44-byte header + PCM samples.
    fn make_wav_bytes(samples: &[i16]) -> Vec<u8> {
        let pcm_bytes: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
        let data_chunk_size = pcm_bytes.len() as u32;
        let riff_chunk_size = 36 + data_chunk_size;
        let sample_rate: u32 = 16_000;
        let num_channels: u16 = 1;
        let bits_per_sample: u16 = 16;
        let byte_rate = sample_rate * num_channels as u32 * bits_per_sample as u32 / 8;
        let block_align = num_channels * bits_per_sample / 8;

        let mut wav = Vec::with_capacity(44 + pcm_bytes.len());
        // RIFF chunk
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&riff_chunk_size.to_le_bytes());
        wav.extend_from_slice(b"WAVE");
        // fmt sub-chunk
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes()); // sub-chunk size
        wav.extend_from_slice(&1u16.to_le_bytes()); // PCM format
        wav.extend_from_slice(&num_channels.to_le_bytes());
        wav.extend_from_slice(&sample_rate.to_le_bytes());
        wav.extend_from_slice(&byte_rate.to_le_bytes());
        wav.extend_from_slice(&block_align.to_le_bytes());
        wav.extend_from_slice(&bits_per_sample.to_le_bytes());
        // data sub-chunk
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&data_chunk_size.to_le_bytes());
        wav.extend_from_slice(&pcm_bytes);
        wav
    }

    #[test]
    fn test_wav_bytes_to_f32_converts_correctly() {
        // i16::MAX = 32767 → ~1.0, 0 → 0.0, i16::MIN = -32768 → -1.0
        let samples = [0i16, 16384, -16384, 32767, -32768];
        let wav = make_wav_bytes(&samples);
        let result = wav_bytes_to_f32(&wav).expect("should succeed");

        assert_eq!(result.len(), samples.len());
        assert!((result[0] - 0.0).abs() < 1e-4, "0 → ~0.0");
        assert!((result[1] - 0.5).abs() < 1e-3, "16384 → ~0.5");
        assert!((result[2] - (-0.5)).abs() < 1e-3, "-16384 → ~-0.5");
        assert!((result[3] - 1.0).abs() < 1e-3, "32767 → ~1.0");
        // i16::MIN / 32768 = exactly -1.0
        assert!((result[4] - (-1.0)).abs() < 1e-6, "-32768 → -1.0");
    }

    #[test]
    fn test_wav_bytes_to_f32_short_input_returns_error() {
        let short_bytes = vec![0u8; 10]; // less than 44 bytes
        let result = wav_bytes_to_f32(&short_bytes);
        assert!(result.is_err(), "should fail with short input");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("too short"),
            "error should mention 'too short', got: {msg}"
        );
    }

    #[test]
    fn test_wav_bytes_to_f32_exact_header_returns_empty_samples() {
        // Exactly 44 bytes = header only, no PCM data → empty sample vec
        let wav = make_wav_bytes(&[]);
        assert_eq!(wav.len(), 44);
        let result = wav_bytes_to_f32(&wav).expect("44-byte header should succeed");
        assert!(result.is_empty(), "no PCM data means empty sample vec");
    }

    #[test]
    fn test_model_missing_variant_exists() {
        let missing = TranscriptionError::ModelMissing {
            message: "test missing".to_string(),
        };
        assert!(
            matches!(missing, TranscriptionError::ModelMissing { .. }),
            "ModelMissing variant should match"
        );
    }

    #[test]
    fn test_model_load_failed_variant_exists() {
        let failed = TranscriptionError::ModelLoadFailed {
            message: "test failed".to_string(),
        };
        assert!(
            matches!(failed, TranscriptionError::ModelLoadFailed { .. }),
            "ModelLoadFailed variant should match"
        );
    }

    #[test]
    fn test_normalize_local_transcript_removes_blank_audio_variants() {
        for transcript in [
            "[BLANK_AUDIO]",
            "[blank audio]",
            "[Blank-Audio]",
            "[ blank_audio ]",
        ] {
            assert_eq!(normalize_local_transcript(transcript), "");
        }
    }

    #[test]
    fn test_normalize_local_transcript_removes_repeated_pauses() {
        assert_eq!(normalize_local_transcript("[Pause][Pause][Pause]"), "");
        assert_eq!(
            normalize_local_transcript(" [pause] \n [PAUSE]\t[pause] "),
            ""
        );
    }

    #[test]
    fn test_normalize_local_transcript_keeps_words_separated() {
        assert_eq!(
            normalize_local_transcript("Hello[Pause]world"),
            "Hello world"
        );
    }

    #[test]
    fn test_normalize_local_transcript_removes_allowlisted_annotations() {
        for label in [
            "blank audio",
            "pause",
            "silence",
            "no speech",
            "music",
            "applause",
            "laughter",
            "noise",
            "inaudible",
        ] {
            let transcript = format!("before [{label}] after");
            assert_eq!(
                normalize_local_transcript(&transcript),
                "before after",
                "label should be removed: {label}"
            );
        }

        assert_eq!(
            normalize_local_transcript("before [NO_SPEECH] [blank-audio] after"),
            "before after"
        );
    }

    #[test]
    fn test_normalize_local_transcript_preserves_arbitrary_brackets() {
        assert_eq!(
            normalize_local_transcript("Add [TODO] [1] [important] after [Noise]"),
            "Add [TODO] [1] [important] after"
        );
        assert_eq!(
            normalize_local_transcript("Keep [two  words] exactly"),
            "Keep [two  words] exactly"
        );
    }

    #[test]
    fn test_normalize_local_transcript_preserves_malformed_brackets() {
        assert_eq!(
            normalize_local_transcript("Keep [Pause and everything after"),
            "Keep [Pause and everything after"
        );
        assert_eq!(
            normalize_local_transcript("Keep ] stray [TODO]"),
            "Keep ] stray [TODO]"
        );
    }

    #[test]
    fn test_normalize_local_transcript_collapses_whitespace_and_keeps_unicode() {
        assert_eq!(
            normalize_local_transcript("  Bună\t[Silence]\n  lume!  "),
            "Bună lume!"
        );
    }

    #[test]
    fn test_normalize_local_transcript_normalizes_plain_whitespace() {
        assert_eq!(
            normalize_local_transcript("  Hello \n\tworld!  "),
            "Hello world!"
        );
    }
}
