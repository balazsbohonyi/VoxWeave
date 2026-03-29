// Local transcription provider — offline Whisper inference via whisper-rs.
//
// wav_bytes_to_f32 is a pure audio helper compiled unconditionally.
// LocalProvider (which links whisper-rs) is gated behind `local-transcription`.

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

// ---------------------------------------------------------------------------
// LocalProvider — requires whisper-rs native library (CMake + MSVC)
// ---------------------------------------------------------------------------

#[cfg(feature = "local-transcription")]
mod provider_impl {
    use super::wav_bytes_to_f32;
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
                let n_segments =
                    state
                        .full_n_segments()
                        .map_err(|e| TranscriptionError::Network {
                            message: format!("Failed to get segment count: {:?}", e),
                        })?;

                let mut text = String::new();
                for i in 0..n_segments {
                    if let Ok(segment) = state.full_get_segment_text(i) {
                        text.push_str(&segment);
                    }
                }

                Ok::<String, TranscriptionError>(text.trim().to_string())
            })
            .await
            .map_err(|e| TranscriptionError::Network {
                message: format!("Blocking task panicked: {e}"),
            })?;

            result
        }
    }
}

#[cfg(feature = "local-transcription")]
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
}
